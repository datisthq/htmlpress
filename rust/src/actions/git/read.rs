use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// When each file under `root` last changed, by its path relative to `root`, from one
/// `git log`. Empty when git is missing or `root` is not in a repository.
pub(crate) fn read_dates(root: &Path) -> HashMap<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "-c",
            "core.quotepath=off",
            "log",
            "--format=%x00%cs",
            "--name-only",
            "--relative",
            "--",
            ".",
        ])
        .output();
    let Some(output) = output.ok().filter(|output| output.status.success()) else {
        return HashMap::new();
    };
    let mut dates = HashMap::new();
    let mut date = "";
    for line in std::str::from_utf8(&output.stdout)
        .unwrap_or_default()
        .lines()
    {
        match line.strip_prefix('\0') {
            Some(next) => date = next,
            // The log runs newest first, so a file's first date is its latest
            None if !line.is_empty() && !date.is_empty() => {
                dates
                    .entry(line.to_owned())
                    .or_insert_with(|| date.to_owned());
            }
            None => {}
        }
    }
    dates
}
