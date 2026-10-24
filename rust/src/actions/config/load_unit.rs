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
