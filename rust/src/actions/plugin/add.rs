use serde_json::{Value, json};
use textlane::Pipeline;

use crate::models::config::PluginEntry;

/// Add the project's textlane plugins at one hook of a page pipeline, as Livemark's
/// `plugins` does: one the pipeline already has, or adds after the hook, is that plugin
/// reconfigured, its options merged, not a second run.
pub(crate) fn add_plugins(
    pipeline: &mut Pipeline,
    hook: &str,
    entries: &[PluginEntry],
) -> Result<(), String> {
    for entry in entries {
        (pipeline.use_value(&write_entry(entry), None))
            .map_err(|error| format!("plugins.{hook}: {}", error.message))?;
    }
    Ok(())
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
