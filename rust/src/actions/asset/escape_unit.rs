use super::*;
use crate::actions::asset::list::list_assets;

#[test]
fn copies_an_asset_and_every_file_of_a_component() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let names = ["layout.html".to_owned(), "htmlpress-tabs".to_owned()];
    let written = escape_assets(root.path(), &names)?;
    assert_eq!(
        written,
        [
            ".htmlpress/layout.html",
            ".htmlpress/components/htmlpress-tabs.css",
            ".htmlpress/components/htmlpress-tabs.js",
        ]
    );
    let copied = std::fs::read_to_string(root.path().join(&written[0]))
        .map_err(Error::io("layout"))?;
    assert!(copied.contains("<htmlpress-content>"));
    let escaped: Vec<_> = list_assets(root.path())
        .into_iter()
        .filter(|status| status.escaped)
        .map(|status| status.path)
        .collect();
    assert_eq!(
        escaped,
        [
            "layout.html",
            "components/htmlpress-tabs.css",
            "components/htmlpress-tabs.js"
        ]
    );
    Ok(())
}

#[test]
fn refuses_an_unknown_name_or_an_escaped_file() -> crate::Result<()> {
    let root = tempfile::tempdir().map_err(Error::io("tempdir"))?;
    let error = escape_assets(root.path(), &["htmlpress-tbas".to_owned()]).err();
    assert!(error.is_some_and(|error| error.to_string().contains("htmlpress-tbas")));
    escape_assets(root.path(), &["styles/tokens.css".to_owned()])?;
    let names = ["htmlpress-badge".to_owned(), "styles/tokens.css".to_owned()];
    let error = escape_assets(root.path(), &names).err();
    assert!(error.is_some_and(|error| error.to_string().contains("already escaped")));
    assert!(
        !root
            .path()
            .join(".htmlpress/components/htmlpress-badge.css")
            .exists()
    );
    Ok(())
}
