use super::*;
use crate::models::config::Position;

#[test]
fn defaults_without_a_config_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    assert_eq!(load_config(root.path())?, Config::default());
    Ok(())
}

#[test]
fn reads_the_config_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "title: Docs\nsite: https://example.com\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(config.title.as_deref(), Some("Docs"));
    assert_eq!(config.site.as_deref(), Some("https://example.com"));
    Ok(())
}

#[test]
fn reads_the_links_and_the_edit_url() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "edit: https://example.com/edit/\nlinks:\n  - title: GitHub\n    url: https://github.com\n  - title: Chat\n    url: https://chat.example.com\n    position: sidebar\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(config.edit.as_deref(), Some("https://example.com/edit/"));
    assert_eq!(config.links.len(), 2);
    assert_eq!(config.links[0].title, "GitHub");
    assert_eq!(config.links[0].url, "https://github.com");
    assert_eq!(config.links[0].position, Position::Header);
    assert_eq!(config.links[1].position, Position::Sidebar);
    Ok(())
}

#[test]
fn rejects_an_unknown_key() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    std::fs::write(root.path().join(CONFIG_FILE), "titel: Docs\n")
        .map_err(Error::io("write"))?;
    let error = load_config(root.path())
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("titel")));
    Ok(())
}

#[test]
fn reads_head_tags_and_an_empty_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "head:\n  meta:\n    - name: author\n      content: Ann\n  scripts:\n    - src: /stats.js\n      defer: \"\"\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(
        config.head.meta[0].get("content").map(String::as_str),
        Some("Ann")
    );
    assert_eq!(
        config.head.scripts[0].get("src").map(String::as_str),
        Some("/stats.js")
    );
    std::fs::write(root.path().join(CONFIG_FILE), "\n").map_err(Error::io("write"))?;
    assert_eq!(load_config(root.path())?, Config::default());
    Ok(())
}
