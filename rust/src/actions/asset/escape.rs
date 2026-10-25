use std::path::Path;

use crate::models::asset::Asset;
use crate::models::error::Error;
use crate::settings::{ASSETS, PROJECT_DIR};

/// Copy built-in assets into the project's `.htmlpress/` to be edited there. A name is an
/// asset's path (`components/htmlpress-tabs.js`) or a component's tag, for all its files.
/// Nothing is written unless every name resolves and none is escaped yet.
pub fn escape_assets(root: &Path, names: &[String]) -> crate::Result<Vec<String>> {
    let mut assets: Vec<&Asset> = vec![];
    for name in names {
        let prefix = format!("components/{name}.");
        let found: Vec<&Asset> = ASSETS
            .iter()
            .filter(|asset| asset.path == name || asset.path.starts_with(&prefix))
            .collect();
        if found.is_empty() {
            return Err(Error::Usage(format!(
                "no built-in asset or component \"{name}\"; `htmlpress escape` lists them"
            )));
        }
        for asset in found {
            if !assets.contains(&asset) {
                assets.push(asset);
            }
        }
    }
    let project = root.join(PROJECT_DIR);
    if let Some(asset) = assets
        .iter()
        .find(|asset| project.join(asset.path).exists())
    {
        return Err(Error::Usage(format!(
            "{PROJECT_DIR}/{} is already escaped; delete it to start over",
            asset.path
        )));
    }
    let mut written = vec![];
    for asset in assets {
        let target = project.join(asset.path);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(Error::io(parent))?;
        }
        std::fs::write(&target, asset.content).map_err(Error::io(&target))?;
        written.push(format!("{PROJECT_DIR}/{}", asset.path));
    }
    Ok(written)
}

#[cfg(test)]
#[path = "escape_unit.rs"]
mod unit;
