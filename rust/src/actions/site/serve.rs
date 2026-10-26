use std::net::{IpAddr, UdpSocket};
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use tiny_http::{Header, Request, Response, Server};

use crate::actions::config::load::load_config;
use crate::actions::path::select::read_bases;
use crate::actions::site::build::build_site;
use crate::models::build::BuildOptions;
use crate::models::error::Error;
use crate::settings::{
    CACHE_DIR, CONFIG_FILE, PORTS, PROJECT_DIR, RELOAD, RELOAD_PATH, RELOAD_SCRIPT,
};

/// Build the project, serve the output and rebuild on every change, reloading open pages;
/// a failed build is shown in the browser until the next one succeeds. `host` defaults to localhost and `port` to the first free one of [`PORTS`]. Calls
/// `on_ready` with the URLs once listening, the local one first; runs until killed.
pub fn serve_site(
    options: &BuildOptions,
    host: Option<&str>,
    port: Option<u16>,
    on_ready: impl FnOnce(&[String]),
) -> crate::Result<()> {
    let options = BuildOptions {
        reload: true,
        ..options.clone()
    };
    let (server, urls) = open_server(host, port)?;
    let base =
        (load_config(&options.root).map(|config| config.read_base())).unwrap_or_default();
    let state = Arc::new(Reload::default());
    report_build(&options, &state, false);

    let (sender, events) = channel();
    let mut watcher = notify::recommended_watcher(sender)
        .map_err(|error| Error::Usage(format!("cannot watch files: {error}")))?;
    watcher
        .watch(&options.root, RecursiveMode::Recursive)
        .map_err(|error| Error::Usage(format!("cannot watch files: {error}")))?;
    let rebuild_state = state.clone();
    let rebuild_options = options.clone();
    std::thread::spawn(move || {
        rebuild_on_change(&rebuild_options, &events, &rebuild_state)
    });

    on_ready(&add_base(urls, &base));
    answer_requests(&server, &options.output, &base, &state);
    Ok(())
}

/// Listen on `host`, localhost by default, and `port`, else the first free one of
/// [`PORTS`]; returns the server and the URLs it answers at, the local one first.
pub(crate) fn open_server(
    host: Option<&str>,
    port: Option<u16>,
) -> crate::Result<(Server, Vec<String>)> {
    let address = host.unwrap_or("127.0.0.1");
    let ports: Vec<u16> = match port {
        Some(port) => vec![port],
        None => PORTS.collect(),
    };
    let server = bind_server(address, &ports)?;
    let urls = create_urls(address, read_port(&server));
    Ok((server, urls))
}

/// Answer every request from the files in `output`, served under `base` (`/repo`, or
/// empty for the root), until the server stops.
pub(crate) fn answer_requests(
    server: &Server,
    output: &Path,
    base: &str,
    state: &Arc<Reload>,
) {
    for request in server.incoming_requests() {
        let output = output.to_owned();
        let base = base.to_owned();
        let state = state.clone();
        std::thread::spawn(move || answer_request(request, &output, &base, &state));
    }
}

/// The URLs to print, each followed by the base the site is served under.
pub(crate) fn add_base(urls: Vec<String>, base: &str) -> Vec<String> {
    match base.is_empty() {
        true => urls,
        false => urls
            .into_iter()
            .map(|url| format!("{url}{base}/"))
            .collect(),
    }
}

#[derive(Default)]
pub(crate) struct Reload {
    status: Mutex<Status>,
    changed: Condvar,
}

// The error of the last build, if it failed: open pages show it over the page, and new
// requests for a page get it instead, since a failed build leaves the output half written.
#[derive(Default, Clone)]
struct Status {
    version: u64,
    error: Option<String>,
}

fn report_build(options: &BuildOptions, state: &Reload, rebuilt: bool) {
    let error = match build_site(options) {
        Ok(report) => {
            let verb = if rebuilt { "rebuilt" } else { "built" };
            eprintln!("{verb} {} pages", report.pages);
            None
        }
        Err(error) => {
            eprintln!("error: {error}");
            Some(error.to_string())
        }
    };
    if let Ok(mut status) = state.status.lock() {
        status.version += 1;
        status.error = error;
        state.changed.notify_all();
    }
}

type Events = Receiver<notify::Result<notify::Event>>;

fn rebuild_on_change(options: &BuildOptions, events: &Events, state: &Reload) {
    let root =
        std::path::absolute(&options.root).unwrap_or_else(|_| options.root.clone());
    let output =
        std::path::absolute(&options.output).unwrap_or_else(|_| options.output.clone());
    let output = output.canonicalize().unwrap_or(output);
    let root = root.canonicalize().unwrap_or(root);
    while let Ok(event) = events.recv() {
        let bases = match load_config(&root) {
            Ok(config) => read_bases(&config),
            Err(_) => vec![String::new()],
        };
        if !is_source_event(&event, &root, &output, &bases) {
            continue;
        }
        // Editors save in bursts (temp file, rename, chmod): settle before building once.
        loop {
            match events.recv_timeout(Duration::from_millis(100)) {
                Ok(_) => continue,
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        report_build(options, state, true);
    }
}

// A build reads every source and rewrites the output, so access events and anything under
// the output, or the project folder itself, must not count: each would rebuild forever.
// Outside the include bases nothing is read, so a `cargo build` beside docs/ is no change.
fn is_source_event(
    event: &notify::Result<notify::Event>,
    root: &Path,
    output: &Path,
    bases: &[String],
) -> bool {
    let Ok(event) = event else { return false };
    if matches!(event.kind, notify::EventKind::Access(_)) {
        return false;
    }
    event.paths.iter().any(|path| {
        if path.starts_with(output) || path.starts_with(root.join(CACHE_DIR)) {
            return false;
        }
        let Ok(inside) = path.strip_prefix(root) else {
            return false;
        };
        let mut parts = inside.components();
        let Some(Component::Normal(first)) = parts.next() else {
            return false;
        };
        let first = first.to_string_lossy();
        if first == PROJECT_DIR {
            return parts.next().is_some();
        }
        let inside = inside.to_string_lossy().replace('\\', "/");
        let in_base = bases.iter().any(|base| inside.starts_with(base.as_str()));
        !first.starts_with('.')
            && first != "node_modules"
            && (inside == CONFIG_FILE || in_base)
    })
}

fn answer_request(request: Request, output: &Path, base: &str, state: &Reload) {
    let url = request.url().to_owned();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let Some(path) = strip_base(path, base) else {
        // The root of a sub-path site has nothing of its own, so it leads to the site
        let result = match path {
            "/" => answer_redirect(request, &format!("{base}/")),
            _ => answer_missing(request, output),
        };
        if let Err(error) = result {
            eprintln!("error: {url}: {error}");
        }
        return;
    };
    let error = (state.status.lock().ok()).and_then(|status| status.error.clone());
    let result = match path {
        RELOAD_PATH => {
            let status = wait_for_change(state, read_since(query));
            let body =
                format!("{}\n{}", status.version, status.error.unwrap_or_default());
            request.respond(Response::from_string(body))
        }
        // A failed first build may not have written it
        RELOAD_SCRIPT => {
            request.respond(add_headers(Response::from_string(RELOAD), RELOAD_SCRIPT))
        }
        _ => match error {
            Some(error) if is_page_path(path) => answer_error(request, base, &error),
            _ => answer_file(request, output, path),
        },
    };
    if let Err(error) = result {
        eprintln!("error: {url}: {error}");
    }
}

// A long poll: answer at once when the client is behind, else when the next build lands,
// else after a timeout so idle connections never pile up.
fn wait_for_change(state: &Reload, since: Option<u64>) -> Status {
    let Ok(status) = state.status.lock() else {
        return Status::default();
    };
    let Some(since) = since else {
        return status.clone();
    };
    let timeout = Duration::from_secs(25);
    match (state.changed)
        .wait_timeout_while(status, timeout, |status| status.version == since)
    {
        Ok((status, _)) => status.clone(),
        Err(_) => Status {
            version: since,
            error: None,
        },
    }
}

fn is_page_path(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or_default();
    name.is_empty() || name.ends_with(".html") || !name.contains('.')
}

fn answer_error(request: Request, base: &str, error: &str) -> std::io::Result<()> {
    let error = error.replace('&', "&amp;").replace('<', "&lt;");
    let body = format!(
        "<!doctype html>\n<html lang=\"en\">\n<meta charset=\"utf-8\">\n\
         <title>Build failed</title>\n<h1>Build failed</h1>\n<pre>{error}</pre>\n\
         <p>Fix the error and save: the page reloads once the build succeeds.</p>\n\
         <script type=\"module\" src=\"{base}{RELOAD_SCRIPT}\" \
         data-endpoint=\"{base}{RELOAD_PATH}\"></script>\n</html>\n"
    );
    let response = Response::from_string(body).with_status_code(500);
    request.respond(add_headers(response, "error.html"))
}

fn read_since(query: &str) -> Option<u64> {
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("since="))
        .and_then(|value| value.parse().ok())
}

pub(crate) fn strip_base<'a>(path: &'a str, base: &str) -> Option<&'a str> {
    if base.is_empty() {
        return Some(path);
    }
    match path.strip_prefix(base)? {
        "" => Some("/"),
        rest if rest.starts_with('/') => Some(rest),
        _ => None,
    }
}

fn answer_redirect(request: Request, location: &str) -> std::io::Result<()> {
    let response = Response::empty(302);
    match Header::from_bytes(&b"Location"[..], location.as_bytes()) {
        Ok(header) => request.respond(response.with_header(header)),
        Err(()) => request.respond(response),
    }
}

fn answer_missing(request: Request, output: &Path) -> std::io::Result<()> {
    let missing = output.join("404.html");
    let body = std::fs::read(&missing).unwrap_or_else(|_| b"Not found".to_vec());
    let response = Response::from_data(body).with_status_code(404);
    request.respond(add_headers(response, "404.html"))
}

fn answer_file(request: Request, output: &Path, path: &str) -> std::io::Result<()> {
    let Some(file) = resolve_file(output, path) else {
        return answer_missing(request, output);
    };
    match std::fs::File::open(&file) {
        Ok(handle) => {
            let name = file.to_string_lossy().into_owned();
            request.respond(add_headers(Response::from_file(handle), &name))
        }
        Err(_) => {
            request.respond(Response::from_string("Not found").with_status_code(404))
        }
    }
}

pub(crate) fn resolve_file(output: &Path, url_path: &str) -> Option<PathBuf> {
    let decoded = decode_percent(url_path)?;
    let mut file = output.to_owned();
    for part in decoded.split('/').filter(|part| !part.is_empty()) {
        if part == ".." || part.contains('\\') {
            return None;
        }
        file.push(part);
    }
    if file.is_dir() {
        file.push("index.html");
    }
    file.is_file().then_some(file)
}

fn decode_percent(text: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut rest = text.as_bytes();
    while let Some((&byte, tail)) = rest.split_first() {
        if byte == b'%' && tail.len() >= 2 {
            let hex = std::str::from_utf8(&tail[..2]).ok()?;
            bytes.push(u8::from_str_radix(hex, 16).ok()?);
            rest = &tail[2..];
        } else {
            bytes.push(byte);
            rest = tail;
        }
    }
    String::from_utf8(bytes).ok()
}

fn add_headers<R: std::io::Read>(response: Response<R>, name: &str) -> Response<R> {
    let extension = name
        .rsplit('.')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let kind = match extension.as_str() {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "json" => "application/json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        "txt" | "md" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    };
    let headers = [("Content-Type", kind), ("Cache-Control", "no-store")];
    headers.into_iter().fold(
        response,
        |response, (name, value)| match Header::from_bytes(
            name.as_bytes(),
            value.as_bytes(),
        ) {
            Ok(header) => response.with_header(header),
            Err(()) => response,
        },
    )
}

// The first port that binds wins, so a second server, in a worktree say, takes the next
fn bind_server(address: &str, ports: &[u16]) -> crate::Result<Server> {
    let mut failure = String::new();
    for port in ports {
        match Server::http((address, *port)) {
            Ok(server) => return Ok(server),
            Err(error) => failure = error.to_string(),
        }
    }
    let ports = match ports {
        [port] => format!("port {port}"),
        [first, .., last] => format!("any port from {first} to {last}"),
        [] => "no port".to_owned(),
    };
    Err(Error::Usage(format!(
        "cannot listen on {address}, {ports}: {failure}; pick one with --port"
    )))
}

fn read_port(server: &Server) -> u16 {
    server
        .server_addr()
        .to_ip()
        .map_or(0, |address| address.port())
}

// Loopback shows as localhost; every interface shows as localhost plus the address the
// local network reaches this machine at, when it has one.
fn create_urls(address: &str, port: u16) -> Vec<String> {
    let ip = address.parse::<IpAddr>().ok();
    let host = match ip {
        Some(ip) if ip.is_loopback() || ip.is_unspecified() => "localhost".to_owned(),
        Some(IpAddr::V6(ip)) => format!("[{ip}]"),
        _ => address.to_owned(),
    };
    let mut urls = vec![format!("http://{host}:{port}")];
    if ip.is_some_and(|ip| ip.is_unspecified())
        && let Some(network) = read_network_ip()
    {
        urls.push(format!("http://{network}:{port}"));
    }
    urls
}

// Connecting a UDP socket sends nothing; it only makes the system pick the interface that
// would route out, whose address the network sees.
fn read_network_ip() -> Option<IpAddr> {
    let socket = UdpSocket::bind(("0.0.0.0", 0)).ok()?;
    socket.connect(("192.0.2.1", 80)).ok()?;
    let ip = socket.local_addr().ok()?.ip();
    (!ip.is_loopback() && !ip.is_unspecified()).then_some(ip)
}

#[cfg(test)]
#[path = "serve_unit.rs"]
mod unit;
