use serde_json::json;
use textlane::TreeKind;

use super::load_plugins;
use crate::models::config::{PluginEntry, PluginOptions};

fn load_error(kind: TreeKind, name: &str) -> String {
    match load_plugins(kind, &[PluginEntry::Name(name.to_owned())]) {
        Ok(_) => format!("{name} loaded"),
        Err(message) => message,
    }
}

#[test]
fn loads_nothing_without_entries() -> Result<(), String> {
    assert!(load_plugins(TreeKind::Mdast, &[])?.is_none());
    Ok(())
}

#[test]
fn loads_a_transform_of_the_hook_tree_with_its_options() -> Result<(), String> {
    let entry = PluginEntry::Options(PluginOptions {
        name: "html.externalLinks".to_owned(),
        options: Some(json!({ "target": "_blank" })),
    });
    assert!(load_plugins(TreeKind::Hast, &[entry])?.is_some());
    Ok(())
}

#[test]
fn rejects_a_plugin_that_cannot_change_the_hook_tree() {
    let cases = [
        (TreeKind::Mdast, "md.nope", "no plugin named `md.nope`"),
        (TreeKind::Mdast, "md.parse", "reads or writes text"),
        (TreeKind::Hast, "html.stringify", "reads or writes text"),
        (TreeKind::Mdast, "md.gfm", "adds no step on the md tree"),
        (
            TreeKind::Mdast,
            "html.externalLinks",
            "adds no step on the md tree",
        ),
        (
            TreeKind::Mdast,
            "md.toHtml",
            "turns the md tree into another kind",
        ),
    ];
    for (kind, name, reason) in cases {
        let error = load_error(kind, name);
        assert!(error.contains(reason), "{name}: {error}");
    }
}
