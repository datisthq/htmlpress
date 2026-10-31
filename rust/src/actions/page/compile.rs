use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::{Map, Value, json};
use textlane::{FrozenProcessor, Processor, html, md};

use crate::models::plugin::AnnotateCode;

/// The frontmatter keys a Markdown page may set: `title`, `head`, and every
/// `<meta name>` an HTML page's head may carry, by the same names.
pub(crate) const FRONTMATTER_KEYS: &[&str] = &[
    "title",
    "head",
    "author",
    "breadcrumbs",
    "date",
    "description",
    "icon",
    "image",
    "keywords",
    "label",
    "language",
    "layout",
    "order",
    "path",
    "sidebar",
    "source-action",
    "source-url",
    "toc",
];

thread_local! {
    static PROCESSORS: RefCell<HashMap<Vec<String>, FrozenProcessor>> =
        RefCell::new(HashMap::new());
}

/// Turn Markdown into HTML: `:::name` directives as `<htmlpress-name>` of one of
/// `components`, a fence's meta as its block's `data-*` attributes. A `page` takes its
/// frontmatter as the head's `<title>` and metas; a partial's is dropped, unread.
pub(crate) fn compile_markdown(
    markdown: &str,
    components: &[String],
    page: bool,
) -> Result<String, String> {
    PROCESSORS.with(|processors| {
        let mut processors = processors.borrow_mut();
        let processor = match processors.get(components) {
            Some(processor) => processor,
            None => {
                let processor = create_processor(components)?;
                processors.entry(components.to_vec()).or_insert(processor)
            }
        };
        let output = processor.process(markdown).map_err(|error| error.message)?;
        if !page {
            return Ok(output.value);
        }
        let matter = match output.file.data.get("matter") {
            Some(Value::Object(matter)) => matter.clone(),
            _ => Map::new(),
        };
        Ok(format!(
            "<!doctype html>\n<html>\n<head>\n{}</head>\n<body>\n{}</body>\n</html>\n",
            create_head(&matter)?,
            output.value
        ))
    })
}

fn create_processor(components: &[String]) -> Result<FrozenProcessor, String> {
    let component: md::Component = serde_json::from_value(json!({
        "directives": [{ "name": components, "component": "htmlpress-{name}" }],
        "unknownDirectives": "error",
    }))
    .map_err(|error| error.to_string())?;
    Processor::new()
        .with(md::Parse::default())
        .with(md::Frontmatter {
            data: true,
            strip: true,
            ..md::Frontmatter::default()
        })
        .with(md::Gfm::default())
        .with(md::Math::default())
        .with(md::Directive::default())
        .with(component)
        .with(md::ToHtml {
            allow_dangerous_html: true,
            jsx_elements: true,
            ..md::ToHtml::default()
        })
        // Before the reparse, which keeps no node's `data`, where the fence's meta waits
        .with(AnnotateCode)
        .with(html::Raw::default())
        .with(html::Stringify {
            allow_dangerous_html: true,
            close_empty_elements: true,
            ..html::Stringify::default()
        })
        .freeze()
        .map_err(|error| error.message)
}

fn create_head(matter: &Map<String, Value>) -> Result<String, String> {
    let mut head = String::new();
    let mut raw = String::new();
    for (key, value) in matter {
        if !FRONTMATTER_KEYS.contains(&key.as_str()) {
            return Err(format!(
                "frontmatter: unknown key \"{key}\"; the keys are {}",
                FRONTMATTER_KEYS.join(", ")
            ));
        }
        let texts: Vec<String> = match value {
            Value::Array(items) if key == "author" || key == "keywords" => {
                let items: Vec<String> = items.iter().map(read_text).collect();
                match key.as_str() {
                    "keywords" => vec![items.join(", ")],
                    _ => items,
                }
            }
            Value::Null => vec![],
            value => vec![read_text(value)],
        };
        for text in texts {
            match key.as_str() {
                "title" => {
                    head.push_str(&format!("<title>{}</title>\n", escape_html(&text)))
                }
                "head" => raw.push_str(&format!("{text}\n")),
                key => head.push_str(&format!(
                    "<meta name=\"{key}\" content=\"{}\">\n",
                    escape_html(&text)
                )),
            }
        }
    }
    Ok(head + &raw)
}

fn read_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        value => value.to_string(),
    }
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
#[path = "compile_unit.rs"]
mod unit;
