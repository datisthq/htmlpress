use super::*;

#[test]
fn resolves_files_folders_and_rejects_escapes() -> crate::Result<()> {
    let output = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let output = output.path();
    std::fs::create_dir_all(output.join("guides")).map_err(Error::io("mkdir"))?;
    for path in ["index.html", "guides/index.html", "my page.html"] {
        std::fs::write(output.join(path), "x").map_err(Error::io(path))?;
    }
    assert_eq!(resolve_file(output, "/"), Some(output.join("index.html")));
    assert_eq!(
        resolve_file(output, "/guides/"),
        Some(output.join("guides/index.html"))
    );
    assert_eq!(
        resolve_file(output, "/guides"),
        Some(output.join("guides/index.html"))
    );
    assert_eq!(
        resolve_file(output, "/my%20page.html"),
        Some(output.join("my page.html"))
    );
    assert_eq!(resolve_file(output, "/missing.html"), None);
    assert_eq!(resolve_file(output, "/../etc/passwd"), None);
    assert_eq!(resolve_file(output, "/%2e%2e/etc/passwd"), None);
    Ok(())
}

#[test]
fn reads_the_since_query() {
    assert_eq!(read_since("since=3"), Some(3));
    assert_eq!(read_since("a=1&since=12"), Some(12));
    assert_eq!(read_since(""), None);
}

#[test]
fn answers_at_once_when_the_client_is_behind() {
    let state = Reload::default();
    if let Ok(mut version) = state.version.lock() {
        *version = 2;
    }
    assert_eq!(wait_for_change(&state, None), 2);
    assert_eq!(wait_for_change(&state, Some(1)), 2);
}

#[test]
fn keeps_watching_sources_and_the_project_folder_only() {
    let root = Path::new("/site");
    let output = Path::new("/site/.htmlpress/build");
    let event = |path: &str| {
        Ok(notify::Event::new(notify::EventKind::Any).add_path(PathBuf::from(path)))
    };
    assert!(is_source_event(&event("/site/index.html"), root, output));
    assert!(is_source_event(
        &event("/site/.htmlpress/layout.html"),
        root,
        output
    ));
    assert!(!is_source_event(
        &event("/site/.htmlpress/build/index.html"),
        root,
        output
    ));
    assert!(!is_source_event(&event("/site/.htmlpress"), root, output));
    assert!(!is_source_event(&event("/site/.git/index"), root, output));
    let access = notify::EventKind::Access(notify::event::AccessKind::Any);
    let read = Ok(notify::Event::new(access).add_path(PathBuf::from("/site/index.html")));
    assert!(!is_source_event(&read, root, output));
    assert!(!is_source_event(
        &event("/site/node_modules/x.html"),
        root,
        output
    ));
}
