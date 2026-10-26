use super::*;
use crate::actions::page::read::read_page;
use crate::models::page::SourceAction;

#[test]
fn replaces_what_the_patch_sets_and_keeps_the_rest() -> Result<(), String> {
    let page = read_page("README.html", "<h1>Readme</h1><p>Hi.</p><h2>Setup</h2>")
        .map_err(|error| error.to_string())?;
    let article = PatchArticle {
        title: Some("Introduction".to_owned()),
        path: Some("/introduction/".to_owned()),
        order: Some(1),
        toc: Some(false),
        author: vec!["Ann".to_owned()],
        source_action: Some(SourceAction::View),
        ..PatchArticle::default()
    };
    let page = patch_page(page, &article)?;
    assert_eq!(
        (page.title.as_str(), page.url.as_str()),
        ("Introduction", "/introduction/")
    );
    assert_eq!(page.path, "introduction/index.html");
    assert_eq!(page.order, Some(1));
    assert!(page.toc.is_empty() && page.sidebar);
    assert_eq!(page.authors, ["Ann"]);
    assert_eq!(page.source_action, SourceAction::View);
    let bad = PatchArticle {
        path: Some("intro".to_owned()),
        ..PatchArticle::default()
    };
    let page = read_page("a.html", "<h1>A</h1>").map_err(|error| error.to_string())?;
    assert!(patch_page(page, &bad).is_err());
    Ok(())
}
