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
    let all = [String::new()];
    let docs = ["docs/".to_owned()];
    let event = |path: &str| {
        Ok(notify::Event::new(notify::EventKind::Any).add_path(PathBuf::from(path)))
    };
    let watched =
        |path: &str, bases: &[String]| is_source_event(&event(path), root, output, bases);
    assert!(watched("/site/index.html", &all));
    assert!(watched("/site/.htmlpress/layout.html", &all));
    assert!(!watched("/site/.htmlpress/build/index.html", &all));
    assert!(!watched("/site/.htmlpress", &all));
    assert!(!watched("/site/.git/index", &all));
    assert!(!watched("/site/node_modules/x.html", &all));
    let access = notify::EventKind::Access(notify::event::AccessKind::Any);
    let read = Ok(notify::Event::new(access).add_path(PathBuf::from("/site/index.html")));
    assert!(!is_source_event(&read, root, output, &all));
    assert!(watched("/site/docs/index.html", &docs));
    assert!(watched("/site/htmlpress.yaml", &docs));
    assert!(watched("/site/.htmlpress/layouts/landing.html", &docs));
    assert!(!watched("/site/rust/target/debug/htmlpress", &docs));
}

#[test]
fn takes_the_first_free_port() -> crate::Result<()> {
    let bind =
        || std::net::TcpListener::bind(("127.0.0.1", 0)).map_err(Error::io("bind"));
    let read = |listener: &std::net::TcpListener| {
        listener
            .local_addr()
            .map(|address| address.port())
            .map_err(Error::io("port"))
    };
    let taken = bind()?;
    let free = bind()?;
    let ports = [read(&taken)?, read(&free)?];
    drop(free);
    let server = bind_server("127.0.0.1", &ports)?;
    assert_eq!(read_port(&server), ports[1]);
    let error = bind_server("127.0.0.1", &ports[..1])
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains(&format!("port {}", ports[0]))));
    Ok(())
}

#[test]
fn shows_loopback_as_localhost() {
    assert_eq!(create_urls("127.0.0.1", 8000), ["http://localhost:8000"]);
    assert_eq!(
        create_urls("192.168.1.5", 8001),
        ["http://192.168.1.5:8001"]
    );
    let urls = create_urls("0.0.0.0", 8000);
    assert_eq!(urls[0], "http://localhost:8000");
    assert!(urls.iter().skip(1).all(|url| !url.contains("localhost")));
}

#[test]
fn strips_the_base_from_request_paths() {
    assert_eq!(strip_base("/guide.html", ""), Some("/guide.html"));
    assert_eq!(strip_base("/repo", "/repo"), Some("/"));
    assert_eq!(strip_base("/repo/", "/repo"), Some("/"));
    assert_eq!(strip_base("/repo/a/b.html", "/repo"), Some("/a/b.html"));
    assert_eq!(strip_base("/repository/", "/repo"), None);
    assert_eq!(strip_base("/", "/repo"), None);
}
