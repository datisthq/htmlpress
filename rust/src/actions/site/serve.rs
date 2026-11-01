use std::collections::BTreeSet;
use std::net::{IpAddr, UdpSocket};
use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use notify::{RecursiveMode, Watcher};
use tiny_http::{Header, Request, Response, Server};

use crate::actions::config::load::load_config;
use crate::actions::path::select::read_bases;
use crate::actions::site::build::{
    Cache, build_cached, create_build_paths, rebuild_pages,
};
use crate::models::build::{BuildOptions, format_count};
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
    // Counted from the start time, not from 0: a tab left open across a restart would
    // otherwise hold the new server's version already and never reload
    let start = (SystemTime::now().duration_since(UNIX_EPOCH))
        .map_or(0, |elapsed| u64::try_from(elapsed.as_millis()).unwrap_or(0));
    let state = Arc::new(Reload::new(start, String::new()));
    let mut cache = None;
    let inputs = report_build(&options, &state, &mut cache, None).unwrap_or_default();

    let (sender, events) = channel();
    let mut watcher = create_watcher(sender)?;
    // The real path: events name paths under the folder watched, and the filter compares
    // them with real ones, so a symlinked root would match nothing
    watcher
        .watch(&read_real_path(&options.root), RecursiveMode::Recursive)
        .map_err(|error| Error::Usage(format!("cannot watch files: {error}")))?;
    let rebuild_state = state.clone();
    let rebuild_options = options.clone();
    std::thread::spawn(move || {
        rebuild_on_change(&rebuild_options, &events, &rebuild_state, inputs, cache)
    });

    on_ready(&add_base(urls, &state.read_base()));
    answer_requests(&server, &options.output, &state);
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

/// Answer every request from the files in `output`, served under the state's base
/// (`/repo`, or empty for the root), until the server stops.
pub(crate) fn answer_requests(server: &Server, output: &Path, state: &Arc<Reload>) {
    for request in server.incoming_requests() {
        let output = output.to_owned();
        let state = state.clone();
        std::thread::spawn(move || answer_request(request, &output, &state));
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

impl Reload {
    /// A build state starting at `version`, serving under `base`.
    pub(crate) fn new(version: u64, base: String) -> Self {
        Self {
            status: Mutex::new(Status {
                version,
                error: None,
                base,
            }),
            changed: Condvar::new(),
        }
    }

    fn read_base(&self) -> String {
        (self.status.lock().ok()).map_or_else(String::new, |status| status.base.clone())
    }
}

// The error of the last build, if it failed: open pages show it over the page, and new
// requests for a page get it instead, since a failed build leaves the output half written.
#[derive(Default, Clone)]
struct Status {
    version: u64,
    error: Option<String>,
    // Read again after every build, so a changed `base` needs no restart
    base: String,
}

// Returns the files the build read beside its pages, or None when it failed. With the
// paths that `changed`, it rebuilds only their pages when it can
fn report_build(
    options: &BuildOptions,
    state: &Reload,
    cache: &mut Option<Cache>,
    changed: Option<&[String]>,
) -> Option<Vec<String>> {
    let partial = match (changed, cache.as_mut()) {
        (Some(changed), Some(cache)) => rebuild_pages(options, cache, changed),
        _ => Ok(None),
    };
    let built = match partial {
        Ok(Some(report)) => Ok(report),
        // A failed partial rebuild leaves the cache half updated: build it all again
        Ok(None) | Err(_) => build_cached(options).map(|(report, built)| {
            *cache = built;
            report
        }),
    };
    let (error, inputs) = match built {
        Ok(report) => {
            let verb = if changed.is_some() {
                "rebuilt"
            } else {
                "built"
            };
            eprintln!("{verb} {}", format_count(report.pages, "page"));
            (None, Some(report.inputs))
        }
        Err(error) => {
            // The output still holds the last good build, but the error may lie anywhere
            *cache = None;
            eprintln!("error: {error}");
            (Some(error.to_string()), None)
        }
    };
    let base = load_config(&options.root).map(|config| config.read_base());
    if let Ok(mut status) = state.status.lock() {
        status.version += 1;
        status.error = error;
        if let Ok(base) = base {
            status.base = base;
        }
        state.changed.notify_all();
    }
    inputs
}

type Events = Receiver<notify::Result<notify::Event>>;

// Symlinks are not followed: inotify gives a folder one watch however many paths reach it,
// so pnpm's `node_modules/.pnpm/node_modules/<package>` link to a workspace folder renamed
// every event under it into `node_modules/`, which the filter drops, and nothing rebuilt
fn create_watcher(
    sender: std::sync::mpsc::Sender<notify::Result<notify::Event>>,
) -> crate::Result<notify::RecommendedWatcher> {
    let config = notify::Config::default().with_follow_symlinks(false);
    notify::RecommendedWatcher::new(sender, config)
        .map_err(|error| Error::Usage(format!("cannot watch files: {error}")))
}

const SETTLE: Duration = Duration::from_millis(100);

fn rebuild_on_change(
    options: &BuildOptions,
    events: &Events,
    state: &Reload,
    inputs: Vec<String>,
    mut cache: Option<Cache>,
) {
    let output = read_real_path(&options.output);
    let (staging, previous) = create_build_paths(&output);
    let root = read_real_path(&options.root);
    let config = root.join(CONFIG_FILE);
    let mut sources = Sources {
        bases: vec![],
        root,
        outputs: vec![output, staging, previous],
        inputs,
    };
    let mut loaded = false;
    while let Ok(event) = events.recv() {
        // On the first change and when it changes only: a build writes thousands of files,
        // and parsing the config for each of their events held the next edit back by
        // seconds. Reading it makes access events of its own, which must never count
        let touched = event
            .as_ref()
            .is_ok_and(|event| event.paths.contains(&config));
        if is_change_event(&event) && (touched || !loaded) {
            sources.bases = read_watched_bases(&sources.root);
            loaded = true;
        }
        if !sources.contains(&event) {
            continue;
        }
        let mut changed = BTreeSet::new();
        sources.add_changed(&event, &mut changed);
        // Editors save in bursts (temp file, rename, chmod): settle before building once.
        // Only a source event extends the wait, or a `cargo build` beside the docs, which
        // writes all the time, would hold the build back until it ends
        let mut deadline = Instant::now() + SETTLE;
        while let Some(wait) = deadline.checked_duration_since(Instant::now()) {
            match events.recv_timeout(wait) {
                Ok(event) if sources.contains(&event) => {
                    sources.add_changed(&event, &mut changed);
                    deadline = Instant::now() + SETTLE
                }
                Ok(_) => {}
                Err(RecvTimeoutError::Timeout) => break,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        }
        // A failed build read only part of the project: keep the last good build's list
        let changed: Vec<String> = changed.into_iter().collect();
        if let Some(inputs) = report_build(options, state, &mut cache, Some(&changed)) {
            sources.inputs = inputs;
        }
    }
}

// Every folder, when the config cannot be read, so fixing it is a change
fn read_watched_bases(root: &Path) -> Vec<String> {
    match load_config(root) {
        Ok(config) => read_bases(&config),
        Err(_) => vec![String::new()],
    }
}

// The real path, or, for one not created yet (an output before the first build), its
// real parent's joined with its name
fn read_real_path(path: &Path) -> PathBuf {
    if let Ok(real) = path.canonicalize() {
        return real;
    }
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_owned());
    match (path.parent().map(Path::canonicalize), path.file_name()) {
        (Some(Ok(parent)), Some(name)) => parent.join(name),
        _ => path,
    }
}

fn is_change_event(event: &notify::Result<notify::Event>) -> bool {
    event
        .as_ref()
        .is_ok_and(|event| !matches!(event.kind, notify::EventKind::Access(_)))
}

// What the watcher counts as a change, every path real
struct Sources {
    root: PathBuf,
    // The output and the folders a build writes beside it
    outputs: Vec<PathBuf>,
    bases: Vec<String>,
    // The files the last good build read beside its pages, relative to the root
    inputs: Vec<String>,
}

impl Sources {
    // A build reads every source and rewrites the output, so access events and anything
    // under the output, or the project folder itself, must not count: each would rebuild
    // forever. Outside the include bases and the inputs nothing is read, so a `cargo
    // build` beside docs/ is no change.
    fn contains(&self, event: &notify::Result<notify::Event>) -> bool {
        if !is_change_event(event) {
            return false;
        }
        let Ok(event) = event else { return false };
        event.paths.iter().any(|path| self.contains_path(path))
    }

    // Each path of the event that counts, relative to the root with `/` separators
    fn add_changed(
        &self,
        event: &notify::Result<notify::Event>,
        changed: &mut BTreeSet<String>,
    ) {
        let Ok(event) = event else { return };
        for path in event.paths.iter().filter(|path| self.contains_path(path)) {
            if let Ok(inside) = path.strip_prefix(&self.root) {
                changed.insert(inside.to_string_lossy().replace('\\', "/"));
            }
        }
    }

    fn contains_path(&self, path: &Path) -> bool {
        let skipped = self.outputs.iter().any(|output| path.starts_with(output));
        if skipped || path.starts_with(self.root.join(CACHE_DIR)) {
            return false;
        }
        let Ok(inside) = path.strip_prefix(&self.root) else {
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
        // Before the dot rule: skills are read from `.skills/`
        if self.inputs.contains(&inside) {
            return true;
        }
        let in_base = (self.bases.iter()).any(|base| inside.starts_with(base.as_str()));
        !first.starts_with('.')
            && first != "node_modules"
            && (inside == CONFIG_FILE || in_base)
    }
}

fn answer_request(request: Request, output: &Path, state: &Reload) {
    let base = state.read_base();
    let base = base.as_str();
    let url = request.url().to_owned();
    let (full, query) = url.split_once('?').unwrap_or((&url, ""));
    let Some(path) = strip_base(full, base) else {
        // The root of a sub-path site has nothing of its own, so it leads to the site
        let result = match full {
            "/" => answer_redirect(request, &format!("{base}/"), 302),
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
        _ => match (error, find_redirect(output, full)) {
            (Some(error), _) if is_page_path(path) => answer_error(request, base, &error),
            (_, Some((location, status))) => {
                let query = match query {
                    "" => String::new(),
                    query => format!("?{query}"),
                };
                answer_redirect(request, &format!("{location}{query}"), status)
            }
            _ => answer_file(request, output, &url, path),
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
            ..Status::default()
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
         data-endpoint=\"{base}{RELOAD_PATH}\" data-error-page></script>\n</html>\n"
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

// As Cloudflare and Netlify do, so `start` never shows the stub the build leaves for other
// hosts; only exact paths, the only rules the build writes
pub(crate) fn find_redirect(output: &Path, path: &str) -> Option<(String, u16)> {
    let rules = std::fs::read_to_string(output.join("_redirects")).ok()?;
    rules.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let (from, to) = (fields.next()?, fields.next()?);
        let status = fields
            .next()
            .map_or(Some(301), |status| status.parse().ok())?;
        let exact = !from.starts_with('#') && !from.contains(['*', ':']);
        (exact && from == path && (300..400).contains(&status))
            .then(|| (to.to_owned(), status))
    })
}

fn answer_redirect(request: Request, location: &str, status: u16) -> std::io::Result<()> {
    let response = Response::empty(status);
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

fn answer_file(
    request: Request,
    output: &Path,
    url: &str,
    path: &str,
) -> std::io::Result<()> {
    let Some(file) = resolve_file(output, path) else {
        return answer_missing(request, output);
    };
    // As static hosts do: a page served at `/guide` would resolve its relative URLs a level up
    if !path.ends_with('/')
        && file.ends_with("index.html")
        && !path.ends_with("index.html")
    {
        let (location, query) = url
            .split_once('?')
            .map_or((url, None), |(a, b)| (a, Some(b)));
        let query = query.map(|query| format!("?{query}")).unwrap_or_default();
        return answer_redirect(request, &format!("{location}/{query}"), 302);
    }
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
