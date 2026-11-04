use serde_json::{Value, json};
use textlane::html::Highlight;
use textlane::{Processor, html};

/// Whether Shiki's built-in set has a theme by this name, for checking the config.
pub(crate) fn has_theme(name: &str) -> bool {
    let processor = Processor::new()
        .with(html::Parse {
            fragment: true,
            ..html::Parse::default()
        })
        .with(Highlight {
            theme: Some(name.to_owned()),
            ..Highlight::default()
        })
        .with(html::Stringify::default())
        .freeze();
    processor
        .and_then(|processor| {
            processor.process("<pre><code class=\"language-js\">a</code></pre>")
        })
        .is_ok()
}

/// The Shiki options every code block is colored with: both themes, and a block's
/// `data-lines`, `data-add`, `data-remove`, `data-error`, `data-warning`, `data-focus`,
/// `data-words` and `data-line-numbers` marking its lines; a block holding markup of its
/// own stays as it is, and inline code naming a language is colored too.
pub(crate) fn create_highlight(light: &str, dark: &str) -> Value {
    json!({
        "themes": { "light": light, "dark": dark },
        "defaultLanguage": "text",
        "fallbackLanguage": "text",
        // Latte's lighter accents fail WCAG AA on the code frame; each is darkened, hue kept
        "colorReplacements": { "catppuccin-latte": {
            "#40a02b": "#2a7e16",
            "#df8e1d": "#9e6414",
            "#fe640b": "#c64901",
            "#179299": "#147d83",
            "#04a5e5": "#037aa9",
            "#209fb5": "#197d8e",
            "#7287fd": "#4560fc",
            "#7c7f93": "#6e7185",
            "#ea76cb": "#ce209f",
            "#dc8a78": "#c14c32",
            "#dd7878": "#ce3d3d",
            "#e64553": "#df1e2f",
        } },
        "inline": "class",
        "skipElements": true,
        "keepProperties": true,
        "lineAttributes": {
            "data-lines": "highlighted",
            "data-add": "diff add",
            "data-remove": "diff remove",
            "data-error": "severity error",
            "data-warning": "severity warning",
            "data-focus": { "line": "focused", "pre": "has-focused" },
        },
        "wordAttribute": "data-words",
        "lineNumberAttribute": "data-line-numbers",
    })
}

#[cfg(test)]
#[path = "highlight_unit.rs"]
mod unit;
