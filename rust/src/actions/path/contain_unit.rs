use super::*;

fn create_project() -> Result<tempfile::TempDir, String> {
    let dir = tempfile::tempdir().map_err(|error| error.to_string())?;
    let write = |path: &str, bytes: &[u8]| -> Result<(), String> {
        let file = dir.path().join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(file, bytes).map_err(|error| error.to_string())
    };
    write("project/docs/notes.md", b"# Notes")?;
    write("project/.env", b"SECRET=1")?;
    write("project/.git/config", b"[core]")?;
    write("project/node_modules/x/README.md", b"x")?;
    write("project/.htmlpress/build/index.html", b"<h1>A</h1>")?;
    write("project/.skills/a/SKILL.md", b"---")?;
    write("project/binary.dat", &[0xff, 0xfe, 0x00])?;
    write("outside.md", b"secret")?;
    Ok(dir)
}

#[test]
fn reads_a_file_inside_the_project() -> Result<(), String> {
    let dir = create_project()?;
    let root = dir.path().join("project");
    assert_eq!(read_contained(&root, "docs/notes.md", false)?, "# Notes");
    assert_eq!(read_contained(&root, "/docs/notes.md", false)?, "# Notes");
    assert_eq!(
        read_contained(&root, "docs/../docs/notes.md", false)?,
        "# Notes"
    );
    assert!(read_contained(&root, ".skills/a/SKILL.md", true).is_ok());
    Ok(())
}

#[test]
fn refuses_every_way_out_and_every_secret() -> Result<(), String> {
    let dir = create_project()?;
    let root = dir.path().join("project");
    for path in [
        "../outside.md",
        "docs/../../outside.md",
        ".env",
        ".git/config",
        "node_modules/x/README.md",
        ".htmlpress/build/index.html",
        ".skills/a/SKILL.md",
        "missing.md",
        "docs",
        "binary.dat",
    ] {
        assert!(read_contained(&root, path, false).is_err(), "{path}");
    }
    let absolute = dir.path().join("outside.md").to_string_lossy().into_owned();
    assert!(read_contained(&root, &absolute, false).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn refuses_a_symlink_that_leads_outside() -> Result<(), String> {
    let dir = create_project()?;
    let root = dir.path().join("project");
    std::os::unix::fs::symlink(dir.path().join("outside.md"), root.join("docs/link.md"))
        .map_err(|error| error.to_string())?;
    assert!(read_contained(&root, "docs/link.md", false).is_err());
    Ok(())
}
