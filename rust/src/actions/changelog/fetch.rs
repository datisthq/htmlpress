use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::actions::changelog::split::{Release, create_slug, raise_headings};
use crate::actions::markdown::render::render_untrusted_markdown;
use crate::settings::CACHE_DIR;

/// Whether a changelog's `source` names a GitHub repository rather than a file.
pub(crate) fn is_github_source(source: &str) -> bool {
    read_repository(source).is_some()
}

/// A repository's GitHub releases, newest first, as Livemark reads them: the API's first
/// 100, with `GITHUB_TOKEN` when set. The answer is cached under `.htmlpress/cache/` and
/// revalidated by its ETag; a failed request falls back to the cache, else fails.
pub(crate) fn fetch_releases(root: &Path, source: &str) -> Result<Vec<Release>, String> {
    // Every language builds the changelog, and the dev server rebuilds on each save: ask
    // GitHub at most once a minute per repository
    let key = format!("{}\n{source}", root.display());
    let fresh =
        |(when, _): &(Instant, Vec<Release>)| when.elapsed() < Duration::from_secs(60);
    if let Ok(memo) = MEMO.lock()
        && let Some(entry) = memo.get(&key).filter(|entry| fresh(entry))
    {
        return Ok(entry.1.clone());
    }
    let releases = request_releases(root, source)?;
    if let Ok(mut memo) = MEMO.lock() {
        memo.insert(key, (Instant::now(), releases.clone()));
    }
    Ok(releases)
}

type Memo = HashMap<String, (Instant, Vec<Release>)>;

static MEMO: LazyLock<Mutex<Memo>> = LazyLock::new(Mutex::default);

fn request_releases(root: &Path, source: &str) -> Result<Vec<Release>, String> {
    let (owner, repository) = read_repository(source)
        .ok_or_else(|| format!("\"{source}\" is not a GitHub repository URL"))?;
    let cache = root
        .join(CACHE_DIR)
        .join(format!("changelog-{owner}-{repository}.json"));
    let cached: Option<Value> = (std::fs::read_to_string(&cache).ok())
        .and_then(|text| serde_json::from_str(&text).ok());
    let url = format!(
        "https://api.github.com/repos/{owner}/{repository}/releases?per_page=100"
    );
    let mut request = ureq::get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "htmlpress");
    if let Ok(token) = std::env::var("GITHUB_TOKEN")
        && !token.is_empty()
    {
        request = request.header("Authorization", &format!("Bearer {token}"));
    }
    if let Some(etag) = cached.as_ref().and_then(|cache| cache["etag"].as_str()) {
        request = request.header("If-None-Match", etag);
    }
    let fetched = match request.config().http_status_as_error(false).build().call() {
        Ok(mut response) if response.status() == 200 => {
            let etag = (response.headers().get("etag"))
                .and_then(|value| value.to_str().ok())
                .map(str::to_owned);
            let body = (response.body_mut().read_to_string()).map_err(|error| {
                format!("GitHub releases of {owner}/{repository}: {error}")
            })?;
            let releases: Value = serde_json::from_str(&body).map_err(|error| {
                format!("GitHub releases of {owner}/{repository}: {error}")
            })?;
            let entry = json!({ "etag": etag, "releases": releases });
            // A cache that cannot be written only costs the next build a full download
            if let Some(folder) = cache.parent()
                && std::fs::create_dir_all(folder).is_ok()
            {
                let _ = std::fs::write(&cache, entry.to_string());
            }
            Ok(entry)
        }
        Ok(response) if response.status() == 304 => {
            cached.clone().ok_or_else(String::new)
        }
        Ok(response) => Err(format!("GitHub answered {}", response.status())),
        Err(error) => Err(error.to_string()),
    };
    let entry = match (fetched, cached) {
        (Ok(entry), _) => entry,
        (Err(message), Some(cached)) => {
            // stderr: under --json stdout carries only the result
            eprintln!(
                "warning: GitHub releases of {owner}/{repository}: {message}; using the cache"
            );
            cached
        }
        (Err(message), None) => {
            return Err(format!(
                "GitHub releases of {owner}/{repository}: {message}"
            ));
        }
    };
    Ok(read_releases(&entry["releases"]))
}

fn read_releases(releases: &Value) -> Vec<Release> {
    let empty = vec![];
    (releases.as_array().unwrap_or(&empty).iter())
        .filter(|release| !release["draft"].as_bool().unwrap_or(false))
        .filter_map(|release| {
            let version = release["tag_name"].as_str()?.to_owned();
            let date = (release["published_at"].as_str())
                .and_then(|date| date.get(..10))
                .map(str::to_owned);
            let body = release["body"].as_str().unwrap_or_default().trim();
            let html = raise_headings(&render_untrusted_markdown(body));
            // A body opening with its own title repeats the page's <h1>
            let html = match html.trim_start().starts_with("<h1") {
                true => html
                    .split_once("</h1>")
                    .map_or(html.clone(), |(_, rest)| rest.to_owned()),
                false => html,
            };
            Some(Release {
                slug: create_slug(&version),
                version,
                date,
                html,
                url: release["html_url"].as_str().map(str::to_owned),
            })
        })
        .collect()
}

// `https://github.com/acme/tool`, `github.com/acme/tool.git#readme`: the owner and the name
fn read_repository(source: &str) -> Option<(String, String)> {
    let rest = source
        .strip_prefix("https://")
        .or(source.strip_prefix("http://"))
        .unwrap_or(source);
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    let rest = rest.strip_prefix("github.com/")?;
    let mut parts = rest.split(['/', '#', '?']);
    let owner = parts.next().filter(|part| !part.is_empty())?;
    let name = parts.next().map(|name| name.trim_end_matches(".git"))?;
    let valid = |text: &str| {
        !text.is_empty()
            && text
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c))
    };
    (valid(owner) && valid(name) && name != "." && name != "..")
        .then(|| (owner.to_owned(), name.to_owned()))
}

#[cfg(test)]
#[path = "fetch_unit.rs"]
mod unit;
