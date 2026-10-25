use std::path::Path;

type Error = Box<dyn std::error::Error + Send + Sync>;
type TestResult = Result<(), Error>;

fn run_cli(args: &[&str]) -> Result<(u8, String), Error> {
    let mut out = vec![];
    let mut err = vec![];
    let argv = std::iter::once("htmlpress").chain(args.iter().copied());
    let program = htmlpress::create_program();
    let code = pollster::block_on(program.try_run_from(argv, &mut out, &mut err))?;
    Ok((code, String::from_utf8(out)?))
}

fn write_page(root: &Path, path: &str, html: &str) -> TestResult {
    let file = root.join(path);
    if let Some(parent) = file.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(file, html)?;
    Ok(())
}

#[test]
fn builds_a_project_as_json() -> TestResult {
    let root = tempfile::tempdir()?;
    write_page(root.path(), "index.html", "<h1>Home</h1>")?;
    write_page(
        root.path(),
        "guides/build.html",
        "<title>Build</title><p>Run it.</p>",
    )?;
    let path = root.path().to_string_lossy().into_owned();

    let (code, out) = run_cli(&["build", &path, "--json"])?;
    assert_eq!(code, 0);
    let report: serde_json::Value = serde_json::from_str(&out)?;
    assert_eq!(report["pages"], 2);
    assert_eq!(report["files"], 0);
    let page =
        std::fs::read_to_string(root.path().join(".htmlpress/build/guides/build.html"))?;
    assert!(page.contains("<p>Run it.</p>"));
    Ok(())
}

#[test]
fn lists_every_command_for_agents() -> TestResult {
    let (code, out) = run_cli(&["--llms"])?;
    assert_eq!(code, 0);
    let manifest: serde_json::Value = serde_json::from_str(&out)?;
    let names: Vec<&str> = manifest["tools"]
        .as_array()
        .map(|tools| {
            tools
                .iter()
                .filter_map(|tool| tool["name"].as_str())
                .collect()
        })
        .unwrap_or_default();
    assert!(
        ["build", "start", "preview", "clear", "escape"]
            .iter()
            .all(|name| names.contains(name))
    );
    Ok(())
}

#[test]
fn escapes_a_component_and_builds_with_it() -> TestResult {
    let root = tempfile::tempdir()?;
    write_page(
        root.path(),
        "index.html",
        "<htmlpress-badge>New</htmlpress-badge>",
    )?;
    let path = root.path().to_string_lossy().into_owned();

    let (code, out) = run_cli(&["escape", "htmlpress-badge", "--root", &path, "--json"])?;
    assert_eq!(code, 0);
    let report: serde_json::Value = serde_json::from_str(&out)?;
    assert_eq!(
        report["escaped"][0],
        ".htmlpress/components/htmlpress-badge.css"
    );
    let escaped = root
        .path()
        .join(".htmlpress/components/htmlpress-badge.css");
    std::fs::write(&escaped, "/* mine */")?;

    let (code, _) = run_cli(&["build", &path])?;
    assert_eq!(code, 0);
    let output = root.path().join(".htmlpress/build/_htmlpress/components");
    assert_eq!(
        std::fs::read_to_string(output.join("htmlpress-badge.css"))?,
        "/* mine */"
    );
    let again = run_cli(&["escape", "htmlpress-badge", "--root", &path]);
    assert!(again.is_err_and(|error| error.to_string().contains("already escaped")));
    Ok(())
}

#[test]
fn clears_the_build_as_json() -> TestResult {
    let root = tempfile::tempdir()?;
    write_page(root.path(), "index.html", "<h1>Home</h1>")?;
    let path = root.path().to_string_lossy().into_owned();
    run_cli(&["build", &path])?;
    let (code, out) = run_cli(&["clear", &path, "--json"])?;
    assert_eq!(code, 0);
    let clear: serde_json::Value = serde_json::from_str(&out)?;
    assert_eq!(clear["removed"], true);
    assert!(!root.path().join(".htmlpress/build").exists());
    Ok(())
}
