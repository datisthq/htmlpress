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
    if let Ok(mut status) = state.status.lock() {
        status.version = 2;
        status.error = Some("broken".to_owned());
    }
    assert_eq!(wait_for_change(&state, None).version, 2);
    let status = wait_for_change(&state, Some(1));
    assert_eq!(
        (status.version, status.error.as_deref()),
        (2, Some("broken"))
    );
}

#[test]
fn tells_pages_from_other_files() {
    assert!(is_page_path("/"));
    assert!(is_page_path("/guides/"));
    assert!(is_page_path("/guides/setup.html"));
    assert!(is_page_path("/guides"));
    assert!(!is_page_path("/_htmlpress/theme.css"));
    assert!(!is_page_path("/llms.txt"));
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
    assert!(!watched("/site/.htmlpress/cache/changelog-a-b.json", &all));
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
    // Port 0 asks for any free port: a freed one could be taken by a parallel test first
    let ports = [read(&taken)?, 0];
    let server = bind_server("127.0.0.1", &ports)?;
    assert!(![0, ports[0]].contains(&read_port(&server)));
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

#[test]
fn reading_the_config_feeds_no_events_back() -> crate::Result<()> {
    let project = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let root = project
        .path()
        .canonicalize()
        .map_err(Error::io("canonicalize"))?;
    std::fs::write(root.join(CONFIG_FILE), "title: a\n").map_err(Error::io("config"))?;
    let options = BuildOptions::new(&root);
    let (watched, raw) = channel();
    let mut watcher = notify::recommended_watcher(watched)
        .map_err(|error| Error::Usage(error.to_string()))?;
    watcher
        .watch(&root, RecursiveMode::Recursive)
        .map_err(|error| Error::Usage(error.to_string()))?;
    // FSEvents can deliver the config's own creation after watching starts: let it pass
    while raw.recv_timeout(Duration::from_millis(500)).is_ok() {}
    let (sender, events) = channel();
    let access = notify::EventKind::Access(notify::event::AccessKind::Any);
    let read = notify::Event::new(access).add_path(root.join(CONFIG_FILE));
    let _ = sender.send(Ok(read));
    let state = Reload::default();
    let fed_back = std::thread::scope(|scope| {
        let (options, state) = (&options, &state);
        scope.spawn(move || rebuild_on_change(options, &events, state));
        let mut fed_back = 0;
        while let Ok(event) = raw.recv_timeout(Duration::from_millis(500)) {
            fed_back += 1;
            if fed_back > 100 || sender.send(event).is_err() {
                break;
            }
        }
        drop(sender);
        fed_back
    });
    assert_eq!(
        fed_back, 0,
        "reading the config fed events back to the watcher"
    );
    Ok(())
}
