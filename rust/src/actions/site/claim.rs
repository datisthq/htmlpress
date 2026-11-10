use std::fs::{File, OpenOptions, TryLockError};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::models::error::Error;

/// Claim the project at `root` for as long as the returned file lives: `start` holds it
/// while serving, so a second `start` or a `clear` of the same project is refused.
pub(crate) fn claim_site(root: &Path) -> crate::Result<File> {
    let root = std::path::absolute(root).map_err(Error::io(root))?;
    let digest = Sha256::digest(root.to_string_lossy().as_bytes());
    let name: String = digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    // An OS lock, not a pid file, so a crash never leaves the project claimed; in the
    // per-user runtime folder where there is one, as Linux' /tmp lets another user hold it
    let folder = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|folder| folder.is_dir())
        .unwrap_or_else(std::env::temp_dir);
    let path = folder.join(format!("htmlpress-{name}.lock"));
    let file = OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(Error::io(&path))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(Error::Usage(format!(
            "htmlpress start is running for {}; stop it first",
            root.display()
        ))),
        Err(TryLockError::Error(error)) => Err(Error::io(&path)(error)),
    }
}

#[cfg(test)]
#[path = "claim_unit.rs"]
mod unit;
