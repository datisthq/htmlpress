use std::path::Path;

use crate::models::config::Config;
use crate::models::error::Error;
use crate::settings::CONFIG_FILE;

/// Read `htmlpress.config.yaml` from the project root; a project without one gets the defaults.
pub fn load_config(root: &Path) -> crate::Result<Config> {
    let path = root.join(CONFIG_FILE);
    if !path.is_file() {
        return Ok(Config::default());
    }
    let text = std::fs::read_to_string(&path).map_err(Error::io(&path))?;
    // An empty file is a project with every default, not a parse error
    if text.trim().is_empty() {
        return Ok(Config::default());
    }
    serde_saphyr::from_str(&text).map_err(|error| {
        let message = error.to_string();
        let message = message
            .strip_prefix("error: ")
            .unwrap_or(&message)
            .to_owned();
        Error::Config { path, message }
    })
}

#[cfg(test)]
#[path = "load_unit.rs"]
mod unit;
