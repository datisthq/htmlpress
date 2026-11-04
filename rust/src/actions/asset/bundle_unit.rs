use std::collections::BTreeMap;

use super::{bundle_assets, join_bundle};

fn create_assets() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("a.js".to_owned(), "const name = 1".to_owned()),
        ("b.js".to_owned(), "const name = 2".to_owned()),
        ("a.css".to_owned(), "a { color: red }".to_owned()),
    ])
}

#[test]
fn bundle_assets_skips_missing_sources_and_names_by_content() {
    let assets = create_assets();
    let sources = vec!["a.js".to_owned(), "gone.js".to_owned(), "b.js".to_owned()];
    let bundle = bundle_assets(&assets, sources, "js");
    let Some(bundle) = bundle else {
        panic!("no bundle");
    };
    assert_eq!(bundle.sources, ["a.js", "b.js"]);
    assert!(bundle.path.starts_with("bundles/") && bundle.path.ends_with(".js"));
    let mut changed = assets.clone();
    changed.insert("b.js".to_owned(), "const name = 3".to_owned());
    let other = bundle_assets(&changed, bundle.sources.clone(), "js");
    assert_ne!(other.map(|other| other.path), Some(bundle.path));
}

#[test]
fn bundle_assets_is_none_without_sources() {
    assert_eq!(
        bundle_assets(&create_assets(), vec!["gone.css".to_owned()], "css"),
        None
    );
}

#[test]
fn join_bundle_joins_stylesheets_in_order() {
    let assets = create_assets();
    let sources = vec!["a.css".to_owned(), "a.js".to_owned()];
    let Some(bundle) = bundle_assets(&assets, sources, "css") else {
        panic!("no bundle");
    };
    assert_eq!(
        join_bundle(&assets, &bundle),
        "a { color: red }\nconst name = 1\n"
    );
}
