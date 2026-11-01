use std::sync::{Arc, Mutex};

use serde_json::{Value, json};
use textlane::{Pipeline, TreeKind};

use crate::models::config::PluginEntry;
use crate::models::plugin::RunPlugins;

/// The project's textlane plugins at one hook as one stage over `kind`, or `None` without
/// any. Each must change that tree in place: a parser, a compiler, a bridge to another
/// tree or a syntax extension, which adds no step once the text is parsed, is an error.
pub(crate) fn load_plugins(
    kind: TreeKind,
    entries: &[PluginEntry],
) -> Result<Option<RunPlugins>, String> {
    if entries.is_empty() {
        return Ok(None);
    }
    let hook = match kind {
        TreeKind::Mdast => "md",
        _ => "html",
    };
    for entry in entries {
        check_plugin(kind, hook, entry)?;
    }
    let mut pipeline = Pipeline::new();
    let list = Value::Array(entries.iter().map(write_entry).collect());
    let fail = |error: textlane::Error| format!("plugins.{hook}: {}", error.message);
    pipeline.use_value(&list, None).map_err(fail)?;
    pipeline.freeze().map_err(fail)?;
    Ok(Some(RunPlugins {
        kind,
        pipeline: Arc::new(Mutex::new(pipeline)),
    }))
}

fn check_plugin(kind: TreeKind, hook: &str, entry: &PluginEntry) -> Result<(), String> {
    let name = match entry {
        PluginEntry::Name(name) => name,
        PluginEntry::Options(options) => &options.name,
    };
    let fail = |reason: &str| format!("plugins.{hook}: \"{name}\" {reason}");
    let mut alone = Pipeline::new();
    alone
        .use_value(&write_entry(entry), None)
        .map_err(|error| format!("plugins.{hook}: {}", error.message))?;
    let frozen = alone
        .freeze()
        .map_err(|error| format!("plugins.{hook}: \"{name}\": {}", error.message))?;
    if frozen.has_parser() || frozen.has_compiler() {
        return Err(fail(
            "reads or writes text; a hook takes a plugin that changes the tree",
        ));
    }
    if frozen.input_kind() != Some(kind) {
        return Err(fail(&format!("adds no step on the {hook} tree")));
    }
    // Only a compiler after it checks what the plugin hands on: a bridge fails to freeze
    let mut checked = Pipeline::new();
    let list = json!([write_entry(entry), format!("{hook}.stringify")]);
    checked
        .use_value(&list, None)
        .and_then(|_| checked.freeze().map(|_| ()))
        .map_err(|_| fail(&format!("turns the {hook} tree into another kind")))
}

fn write_entry(entry: &PluginEntry) -> Value {
    match entry {
        PluginEntry::Name(name) => Value::String(name.clone()),
        PluginEntry::Options(options) => match &options.options {
            Some(value) => json!({ "name": options.name, "options": value }),
            None => json!({ "name": options.name }),
        },
    }
}

#[cfg(test)]
#[path = "load_unit.rs"]
mod unit;
