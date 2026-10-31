use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::json;
use textlane::html::{Highlight, InlineCode, LineMark};
use textlane::{FrozenProcessor, Processor, html};

use crate::actions::page::transform::name_references;

thread_local! {
    static PROCESSORS: RefCell<HashMap<(String, String), FrozenProcessor>> =
        RefCell::new(HashMap::new());
}

/// Color every `<pre><code class="language-…">`, and inline code naming a language, with
/// the light and dark themes as Shiki does; a block's `data-lines`, `data-add`,
/// `data-remove`, `data-error`, `data-warning`, `data-focus`, `data-words` and
/// `data-line-numbers` mark its lines. A block holding markup of its own stays as it is.
pub(crate) fn highlight_code(
    html: &str,
    light: &str,
    dark: &str,
) -> Result<String, String> {
    if !html.contains("<code") {
        return Ok(html.to_owned());
    }
    PROCESSORS.with(|processors| {
        let mut processors = processors.borrow_mut();
        let key = (light.to_owned(), dark.to_owned());
        let processor = match processors.get(&key) {
            Some(processor) => processor,
            None => {
                let processor =
                    create_processor(light, dark).map_err(|error| error.message)?;
                processors.entry(key).or_insert(processor)
            }
        };
        let output = processor.process(html).map_err(|error| error.message)?;
        Ok(name_references(&output.value))
    })
}

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

fn create_processor(light: &str, dark: &str) -> Result<FrozenProcessor, textlane::Error> {
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
    Processor::new()
        .with(html::Parse {
            fragment: true,
            ..html::Parse::default()
        })
        .with(Highlight {
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
        })
        .with(html::Stringify {
            close_empty_elements: true,
            ..html::Stringify::default()
        })
        .freeze()
}

#[cfg(test)]
#[path = "highlight_unit.rs"]
mod unit;
