use std::path::Path;

use crate::models::asset::AssetStatus;
use crate::settings::{ASSETS, PROJECT_DIR};

/// List every built-in asset and whether the project at `root` has escaped it.
pub fn list_assets(root: &Path) -> Vec<AssetStatus> {
    let project = root.join(PROJECT_DIR);
    ASSETS
        .iter()
        .map(|asset| AssetStatus {
            path: asset.path,
            escaped: project.join(asset.path).is_file(),
        })
        .collect()
}

#[cfg(test)]
#[path = "list_unit.rs"]
mod unit;
