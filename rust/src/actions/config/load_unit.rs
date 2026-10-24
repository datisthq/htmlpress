use super::*;

#[test]
fn defaults_without_a_config_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    assert_eq!(load_config(root.path())?, Config::default());
    Ok(())
}

#[test]
fn reads_the_config_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "title = \"Docs\"\nsite = \"https://example.com\"\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(config.title.as_deref(), Some("Docs"));
    assert_eq!(config.site.as_deref(), Some("https://example.com"));
    Ok(())
}

#[test]
fn reads_the_links_and_the_edit_url() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let text = "edit = \"https://example.com/edit/\"\n\n[[links]]\ntitle = \"GitHub\"\nurl = \"https://github.com\"\n";
    std::fs::write(root.path().join(CONFIG_FILE), text).map_err(Error::io("write"))?;
    let config = load_config(root.path())?;
    assert_eq!(config.edit.as_deref(), Some("https://example.com/edit/"));
    assert_eq!(config.links.len(), 1);
    assert_eq!(config.links[0].title, "GitHub");
    assert_eq!(config.links[0].url, "https://github.com");
    Ok(())
}

#[test]
fn rejects_an_unknown_key() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    std::fs::write(root.path().join(CONFIG_FILE), "titel = \"Docs\"\n")
        .map_err(Error::io("write"))?;
    let error = load_config(root.path())
        .err()
        .map(|error| error.to_string());
    assert!(error.is_some_and(|message| message.contains("titel")));
    Ok(())
}
