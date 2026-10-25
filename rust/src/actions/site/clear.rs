use crate::actions::site::build::check_output;
use crate::models::build::BuildOptions;
use crate::models::error::Error;

/// Remove the build output at `options.output`; returns whether there was one.
pub fn clear_site(options: &BuildOptions) -> crate::Result<bool> {
    check_output(&options.root, &options.output)?;
    if !options.output.exists() {
        return Ok(false);
    }
    std::fs::remove_dir_all(&options.output).map_err(Error::io(&options.output))?;
    Ok(true)
}

#[cfg(test)]
#[path = "clear_unit.rs"]
mod unit;
