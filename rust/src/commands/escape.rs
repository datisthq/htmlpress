use std::path::Path;

use crate::actions::asset::escape::escape_assets;
use crate::actions::asset::list::list_assets;

#[fireargs::input]
pub(crate) struct EscapeInput {
    /// Assets to copy into .htmlpress/: a path such as components/tabs/tabs.js, or a
    /// component's tag or name (htmlpress-tabs, tabs) for all its files. Without any, only
    /// list them.
    #[fireargs(argument)]
    #[serde(default)]
    names: Vec<String>,
    /// The project folder.
    #[fireargs(short = 'r', default = ".")]
    root: String,
}

#[fireargs::output]
pub(crate) struct EscapeOutput {
    /// The files just written, relative to the project folder.
    escaped: Vec<String>,
    /// Every built-in asset and whether the project replaces it.
    assets: Vec<AssetEntry>,
}

#[fireargs::output]
pub(crate) struct AssetEntry {
    /// The asset's path under .htmlpress/.
    path: String,
    /// Whether the project has its own copy.
    escaped: bool,
}

pub(crate) fn create_escape_command() -> fireargs::Command<EscapeInput, EscapeOutput> {
    fireargs::command()
        .description(
            "Copy built-in layout, styles and components into .htmlpress/ to edit them.",
        )
        .handler(async |input: EscapeInput, _| {
            let root = Path::new(&input.root);
            let escaped = escape_assets(root, &input.names)?;
            let assets = list_assets(root)
                .into_iter()
                .map(|status| AssetEntry {
                    path: status.path.to_owned(),
                    escaped: status.escaped,
                })
                .collect();
            Ok(EscapeOutput { escaped, assets })
        })
        .render(|output, _| match output.escaped.is_empty() {
            false => output
                .escaped
                .iter()
                .map(|path| format!("escaped {path}"))
                .collect::<Vec<_>>()
                .join("\n"),
            true => output
                .assets
                .iter()
                .map(|asset| match asset.escaped {
                    true => format!("{}  (escaped)", asset.path),
                    false => asset.path.clone(),
                })
                .collect::<Vec<_>>()
                .join("\n"),
        })
}
