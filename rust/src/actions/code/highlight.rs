use serde_json::json;
use textlane::html::{Highlight, InlineCode, LineMark};
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
pub(crate) fn create_highlight(light: &str, dark: &str) -> Highlight {
    let lines = |line: &str| LineMark::Line(html::Classes::One(line.to_owned()));
    let line_attributes = [
        ("data-lines", lines("highlighted")),
        ("data-add", lines("diff add")),
        ("data-remove", lines("diff remove")),
        ("data-error", lines("severity error")),
        ("data-warning", lines("severity warning")),
        (
            "data-focus",
            LineMark::Both {
                line: html::Classes::One("focused".to_owned()),
                pre: Some(html::Classes::One("has-focused".to_owned())),
            },
        ),
    ]
    .into_iter()
    .map(|(name, mark)| (name.to_owned(), mark))
    .collect();
    let themes = json!({ "light": light, "dark": dark });
    Highlight {
        themes: themes.as_object().cloned(),
        default_language: Some("text".to_owned()),
        fallback_language: Some("text".to_owned()),
        // Livemark's fix: Latte's green strings fail WCAG AA on white, at 2.6:1
        color_replacements: serde_json::from_value(
            json!({ "catppuccin-latte": { "#40a02b": "#2a7e16" } }),
        )
        .unwrap_or_default(),
        inline: Some(InlineCode::Class),
        skip_elements: true,
        keep_properties: true,
        line_attributes,
        word_attribute: Some("data-words".to_owned()),
        line_number_attribute: Some("data-line-numbers".to_owned()),
        ..Highlight::default()
    }
}

#[cfg(test)]
#[path = "highlight_unit.rs"]
mod unit;
