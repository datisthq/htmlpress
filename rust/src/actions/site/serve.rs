use std::path::{Component, Path, PathBuf};
use std::sync::mpsc::{Receiver, RecvTimeoutError, channel};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use tiny_http::{Header, Request, Response, Server};

use crate::actions::site::build::build_site;
use crate::models::build::BuildOptions;
use crate::models::error::Error;
use crate::settings::{PROJECT_DIR, RELOAD_PATH};

/// Build the project, serve the output on `127.0.0.1:port` and rebuild on every change,
/// reloading open pages. Calls `on_ready` with the URL once listening; runs until killed.
pub fn serve_site(
    options: &BuildOptions,
    port: u16,
    on_ready: impl FnOnce(&str),
) -> crate::Result<()> {
    let options = BuildOptions {
        reload: true,
        ..options.clone()
    };
    build_site(&options)?;
    let server = Server::http(("127.0.0.1", port)).map_err(|error| {
        Error::Usage(format!("cannot listen on port {port}: {error}"))
    })?;
    let state = Arc::new(Reload::default());

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

    on_ready(&format!("http://127.0.0.1:{}", read_port(&server, port)));
    for request in server.incoming_requests() {
        let output = options.output.clone();
        let state = state.clone();
        std::thread::spawn(move || answer_request(request, &output, &state));
    }
    Ok(())
}

#[derive(Default)]
struct Reload {
    version: Mutex<u64>,
    changed: Condvar,
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
        if !is_source_event(&event, &root, &output) {
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
        match build_site(options) {
            Ok(report) => eprintln!("rebuilt {} pages", report.pages),
            Err(error) => eprintln!("error: {error}"),
        }
        if let Ok(mut version) = state.version.lock() {
            *version += 1;
            state.changed.notify_all();
        }
    }
}

// A build reads every source and rewrites the output, so access events and anything under
// the output, or the project folder itself, must not count: each would rebuild forever.
fn is_source_event(
    event: &notify::Result<notify::Event>,
    root: &Path,
    output: &Path,
) -> bool {
    let Ok(event) = event else { return false };
    if matches!(event.kind, notify::EventKind::Access(_)) {
        return false;
    }
    event.paths.iter().any(|path| {
        if path.starts_with(output) {
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
        match first == PROJECT_DIR {
            true => parts.next().is_some(),
            false => !first.starts_with('.') && first != "node_modules",
        }
    })
}

fn answer_request(request: Request, output: &Path, state: &Reload) {
    let url = request.url().to_owned();
    let (path, query) = url.split_once('?').unwrap_or((&url, ""));
    let result = match path == RELOAD_PATH {
        true => {
            let version = wait_for_change(state, read_since(query));
            request.respond(Response::from_string(version.to_string()))
        }
        false => answer_file(request, output, path),
    };
    if let Err(error) = result {
        eprintln!("error: {url}: {error}");
    }
}

// A long poll: answer at once when the client is behind, else when the next build lands,
// else after a timeout so idle connections never pile up.
fn wait_for_change(state: &Reload, since: Option<u64>) -> u64 {
    let Ok(version) = state.version.lock() else {
        return 0;
    };
    let Some(since) = since else { return *version };
    let timeout = Duration::from_secs(25);
    match state
        .changed
        .wait_timeout_while(version, timeout, |version| *version == since)
    {
        Ok((version, _)) => *version,
        Err(_) => since,
    }
}

fn read_since(query: &str) -> Option<u64> {
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix("since="))
        .and_then(|value| value.parse().ok())
}

fn answer_file(request: Request, output: &Path, path: &str) -> std::io::Result<()> {
    let Some(file) = resolve_file(output, path) else {
        let missing = output.join("404.html");
        let body = std::fs::read(&missing).unwrap_or_else(|_| b"Not found".to_vec());
        let response = Response::from_data(body).with_status_code(404);
        return request.respond(add_headers(response, "404.html"));
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

fn read_port(server: &Server, fallback: u16) -> u16 {
    server
        .server_addr()
        .to_ip()
        .map(|address| address.port())
        .unwrap_or(fallback)
}

#[cfg(test)]
#[path = "serve_unit.rs"]
mod unit;
