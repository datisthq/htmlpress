use lol_html::{RewriteStrSettings, element, rewrite_str};

use crate::models::page::Page;

/// Make a page's relative media URLs (`src`, `srcset`, `poster`) and its `image` root-relative,
/// resolved from the folder its source file is copied to: a page served elsewhere, through
/// `<meta name="path">` or a language prefix, would otherwise point beside its new URL.
pub(crate) fn rebase_page(page: &mut Page, bases: &[String]) -> Result<(), String> {
    if page.source.is_empty() {
        return Ok(());
    }
    let folder = read_source_folder(&page.source, bases);
    if let Some(image) = &page.image {
        page.image = Some(resolve_url(image, &folder).unwrap_or_else(|| image.clone()));
    }
    if folder != read_url_folder(&page.url) {
        page.content = rebase_urls(&page.content, &folder)?;
    }
    Ok(())
}

fn rebase_urls(html: &str, folder: &str) -> Result<String, String> {
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "[src], [srcset], [poster]",
        |el| {
            for name in ["src", "poster"] {
                if let Some(value) = el.get_attribute(name)
                    && let Some(url) = resolve_url(&value, folder)
                {
                    el.set_attribute(name, &url)?;
                }
            }
            if let Some(srcset) = el.get_attribute("srcset") {
                let candidates: Vec<String> = (srcset.split(','))
                    .map(|candidate| {
                        let trimmed = candidate.trim_start();
                        let lead = &candidate[..candidate.len() - trimmed.len()];
                        let (url, rest) = trimmed.split_at(
                            trimmed.find(char::is_whitespace).unwrap_or(trimmed.len()),
                        );
                        let url =
                            resolve_url(url, folder).unwrap_or_else(|| url.to_owned());
                        format!("{lead}{url}{rest}")
                    })
                    .collect();
                el.set_attribute("srcset", &candidates.join(","))?;
            }
            Ok(())
        }
    ));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

/// `/guides/` for `docs/guides/setup.html` when `docs/` is an include base.
fn read_source_folder(source: &str, bases: &[String]) -> String {
    let folder = source.rfind('/').map_or("", |end| &source[..=end]);
    let folder = (bases.iter())
        .filter(|base| !base.is_empty())
        .filter_map(|base| folder.strip_prefix(base.as_str()))
        .min_by_key(|rest| rest.len())
        .unwrap_or(folder);
    format!("/{folder}")
}

fn read_url_folder(url: &str) -> &str {
    url.rfind('/').map_or("/", |end| &url[..=end])
}

// None for a URL that is not relative: root-relative, absolute, `data:`, a fragment or a query
fn resolve_url(url: &str, folder: &str) -> Option<String> {
    let scheme = url.find(':').is_some_and(|colon| {
        let scheme = &url[..colon];
        !scheme.is_empty()
            && !scheme.contains(['/', '?', '#'])
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+.-".contains(c))
    });
    if url.is_empty() || scheme || url.starts_with(['/', '#', '?']) {
        return None;
    }
    let end = url.find(['?', '#']).unwrap_or(url.len());
    let mut segments: Vec<&str> = folder.split('/').filter(|s| !s.is_empty()).collect();
    for segment in url[..end].split('/') {
        match segment {
            "." => {}
            ".." => {
                segments.pop();
            }
            segment => segments.push(segment),
        }
    }
    Some(format!("/{}{}", segments.join("/"), &url[end..]))
}

#[cfg(test)]
#[path = "rebase_unit.rs"]
mod unit;
