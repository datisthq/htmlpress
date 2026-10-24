use std::path::Path;

use crate::actions::page::render::render_page;
use crate::actions::site::scan::scan_site;
use crate::models::build::{BuildOptions, BuildReport};
use crate::models::error::Error;
use crate::settings::{ASSETS_DIR, COMPONENTS, RELOAD, THEME};

/// Build the project at `options.root` into `options.output`, replacing what is there.
pub fn build_site(options: &BuildOptions) -> crate::Result<BuildReport> {
    check_output(&options.root, &options.output)?;
    let site = scan_site(&options.root)?;

    if options.output.exists() {
        std::fs::remove_dir_all(&options.output).map_err(Error::io(&options.output))?;
    }
    for page in &site.pages {
        let html = render_page(&site, page, options.reload)?;
        write_file(&options.output.join(&page.path), html.as_bytes())?;
    }
    for file in &site.files {
        let source = options.root.join(file);
        let target = options.output.join(file);
        create_parent(&target)?;
        std::fs::copy(&source, &target).map_err(Error::io(&source))?;
    }
    let assets = options.output.join(ASSETS_DIR);
    write_file(&assets.join("theme.css"), THEME.as_bytes())?;
    write_file(&assets.join("components.js"), COMPONENTS.as_bytes())?;
    if options.reload {
        write_file(&assets.join("reload.js"), RELOAD.as_bytes())?;
    }

    Ok(BuildReport {
        output: options.output.clone(),
        pages: site.pages.len(),
        files: site.files.len(),
    })
}

// The output is wiped before each build, so one that holds the project would delete it.
fn check_output(root: &Path, output: &Path) -> crate::Result<()> {
    let root = std::path::absolute(root).map_err(Error::io(root))?;
    let output = std::path::absolute(output).map_err(Error::io(output))?;
    if root.starts_with(&output) {
        return Err(Error::Usage(format!(
            "output {} would hold the project {}; pick a folder inside or beside it",
            output.display(),
            root.display(),
        )));
    }
    let inside = output.strip_prefix(&root).ok();
    let hidden = inside
        .and_then(|path| path.components().next())
        .is_some_and(|part| {
            part.as_os_str().to_string_lossy().starts_with('.')
                || part.as_os_str() == "node_modules"
        });
    if inside.is_some() && !hidden {
        return Err(Error::Usage(format!(
            "output {} is inside the project, so its pages would be read back as sources; \
             use a dot-folder such as .htmlpress/build",
            output.display(),
        )));
    }
    Ok(())
}

fn write_file(path: &Path, bytes: &[u8]) -> crate::Result<()> {
    create_parent(path)?;
    std::fs::write(path, bytes).map_err(Error::io(path))
}

fn create_parent(path: &Path) -> crate::Result<()> {
    match path.parent() {
        Some(parent) => std::fs::create_dir_all(parent).map_err(Error::io(parent)),
        None => Ok(()),
    }
}

#[cfg(test)]
#[path = "build_unit.rs"]
mod unit;
