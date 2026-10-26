use lol_html::{RewriteStrSettings, element, rewrite_str};

const URL_ATTRIBUTES: &[&str] = &[
    "href",
    "src",
    "poster",
    "action",
    "data-endpoint",
    "data-index",
];

/// Put `base` (`/repo`) before every root-relative URL in a rendered page's attributes,
/// `srcset` included. Relative, absolute and protocol-relative URLs are left alone.
pub(crate) fn prefix_urls(html: &str, base: &str) -> Result<String, String> {
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "[href], [src], [srcset], [poster], [action], [data-endpoint], [data-index]",
        |el| {
            for name in URL_ATTRIBUTES {
                if let Some(value) = el.get_attribute(name) {
                    el.set_attribute(name, &prefix_url(&value, base))?;
                }
            }
            if let Some(srcset) = el.get_attribute("srcset") {
                let candidates: Vec<String> = srcset
                    .split(',')
                    .map(|candidate| {
                        let trimmed = candidate.trim_start();
                        let lead = &candidate[..candidate.len() - trimmed.len()];
                        format!("{lead}{}", prefix_url(trimmed, base))
                    })
                    .collect();
                el.set_attribute("srcset", &candidates.join(","))?;
            }
            Ok(())
        }
    ));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

fn prefix_url(url: &str, base: &str) -> String {
    match url.starts_with('/') && !url.starts_with("//") {
        true => format!("{base}{url}"),
        false => url.to_owned(),
    }
}

#[cfg(test)]
#[path = "prefix_unit.rs"]
mod unit;
