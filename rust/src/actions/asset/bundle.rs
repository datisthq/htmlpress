use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::models::asset::Bundle;

/// The bundle joining `sources`, the asset paths that exist, into one file under
/// `bundles/` with `extension`, named by a hash of what it holds; None when none exists.
pub(crate) fn bundle_assets(
    assets: &BTreeMap<String, String>,
    sources: Vec<String>,
    extension: &str,
) -> Option<Bundle> {
    let sources: Vec<String> = (sources.into_iter())
        .filter(|path| assets.contains_key(path))
        .collect();
    if sources.is_empty() {
        return None;
    }
    let mut hash = Sha256::new();
    for path in &sources {
        hash.update(path.as_bytes());
        hash.update([0]);
        hash.update(assets.get(path).map_or("", String::as_str).as_bytes());
        hash.update([0]);
    }
    let digest: String = (hash.finalize().iter().take(8))
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Some(Bundle {
        path: format!("bundles/{digest}.{extension}"),
        sources,
    })
}

/// The text of a bundle: its sources in order, each on its own lines.
pub(crate) fn join_bundle(assets: &BTreeMap<String, String>, bundle: &Bundle) -> String {
    let mut text = String::new();
    for path in &bundle.sources {
        text.push_str(assets.get(path).map_or("", String::as_str));
        text.push('\n');
    }
    text
}

#[cfg(test)]
#[path = "bundle_unit.rs"]
mod unit;
