use std::process::Command;

/// Open `path` under the site's local `url` in the default browser, warning on failure
/// rather than stopping the server it was asked of.
pub(crate) fn open_browser(url: &str, path: &str) {
    let target = join_url(url, path);
    let opener = match cfg!(target_os = "macos") {
        true => Command::new("open").arg(&target).spawn(),
        false if cfg!(windows) => Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(&target)
            .spawn(),
        false => Command::new("xdg-open").arg(&target).spawn(),
    };
    if let Err(error) = opener {
        eprintln!("could not open {target}: {error}");
    }
}

fn join_url(url: &str, path: &str) -> String {
    format!(
        "{}/{}",
        url.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

#[cfg(test)]
#[path = "open_unit.rs"]
mod tests;
