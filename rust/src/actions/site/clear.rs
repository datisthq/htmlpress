use crate::actions::site::build::{check_output, stays_inside};
use crate::actions::site::claim::claim_site;
use crate::models::build::BuildOptions;
use crate::models::error::Error;
use crate::settings::CACHE_DIR;

/// Remove the build output at `options.output` and the project's cache, refusing while
/// `start` serves the project; returns whether there was either.
pub fn clear_site(options: &BuildOptions) -> crate::Result<bool> {
    let _claim = claim_site(&options.root)?;
    check_output(&options.root, &options.output)?;
    let cache = options.root.join(CACHE_DIR);
    if !stays_inside(&options.root, &cache) {
        return Err(Error::Usage(format!(
            "{} leads out of the project, so it is not removed",
            cache.display()
        )));
    }
    let mut removed = false;
    for folder in [options.output.clone(), cache] {
        if folder.exists() {
            std::fs::remove_dir_all(&folder).map_err(Error::io(&folder))?;
            removed = true;
        }
    }
    Ok(removed)
}

#[cfg(test)]
#[path = "clear_unit.rs"]
mod unit;
