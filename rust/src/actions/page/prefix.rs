use textlane::hast::Node;
use textlane::html::{read_attribute, write_attribute};
use textlane::{Allocator, ArenaVec};

use crate::actions::node::walk::{Step, walk_nodes};

const URL_ATTRIBUTES: &[&str] = &[
    "href",
    "src",
    "poster",
    "action",
    "data-endpoint",
    "data-index",
];

/// Put `base` (`/repo`) before every root-relative URL in these nodes' attributes,
/// `srcset` included. Relative, absolute and protocol-relative URLs are left alone, and
/// so is raw markup, which was prefixed as its own tree.
pub(crate) fn prefix_urls<'a>(
    nodes: &mut ArenaVec<'a, Node<'a>>,
    base: &str,
    alloc: &'a Allocator,
) {
    let _ = walk_nodes(nodes, &mut |element| {
        for name in URL_ATTRIBUTES {
            if let Some(value) = read_attribute(element, name) {
                let prefixed = prefix_url(&value, base);
                write_attribute(element, name, &prefixed, alloc);
            }
        }
        if let Some(srcset) = read_attribute(element, "srcset") {
            let candidates: Vec<String> = srcset
                .split(',')
                .map(|candidate| {
                    let trimmed = candidate.trim_start();
                    let lead = &candidate[..candidate.len() - trimmed.len()];
                    format!("{lead}{}", prefix_url(trimmed, base))
                })
                .collect();
            write_attribute(element, "srcset", &candidates.join(","), alloc);
        }
        Ok::<_, ()>(Step::Continue)
    });
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
