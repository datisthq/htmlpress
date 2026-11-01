use std::cell::RefCell;
use std::collections::HashMap;

use serde_json::{Map, Value, json};
use textlane::{Api, File, ProcessJob};

use crate::actions::pipeline::create::{use_native, use_plugins};
use crate::actions::plugin::add::add_plugins;
use crate::models::pipeline::MarkdownOptions;
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
    static PROCESSORS: RefCell<HashMap<(Vec<String>, String), ProcessJob>> =
        RefCell::new(HashMap::new());
}

/// Turn Markdown into HTML: Livemark's directives, alerts and fences as htmlpress'
/// elements, any other `:::name` as `<htmlpress-name>` of one of the project's components,
/// the project's `md` plugins after the directives, and a fence's meta as its block's
/// `data-*` attributes. A `page` takes its
/// frontmatter as the head's `<title>` and metas; a partial's is dropped, unread.
pub(crate) fn compile_markdown(
    markdown: &str,
    options: &MarkdownOptions,
    page: bool,
) -> Result<String, String> {
    PROCESSORS.with(|processors| {
        let mut processors = processors.borrow_mut();
        let plugins =
            serde_json::to_string(&options.plugins).map_err(|e| e.to_string())?;
        let key = (options.components.clone(), plugins);
        let processor = match processors.get(&key) {
            Some(processor) => processor,
            None => {
                let processor = create_processor(options)?;
                processors.entry(key).or_insert(processor)
            }
        };
        let mut file = File::new();
        let output =
            (processor.process(markdown, &mut file)).map_err(|error| error.message)?;
        let html = mark_inline_code(&output);
        if !page {
            return Ok(html);
        }
        let matter = match file.data.get("matter") {
            Some(Value::Object(matter)) => matter.clone(),
            _ => Map::new(),
        };
        Ok(format!(
            "<!doctype html>\n<html>\n<head>\n{}</head>\n<body>\n{}</body>\n</html>\n",
            create_head(&matter)?,
            html
        ))
    })
}

/// The Markdown pipeline as one job; also what the config's `md` plugins are checked
/// against before any page compiles.
pub(crate) fn create_processor(options: &MarkdownOptions) -> Result<ProcessJob, String> {
    let heading_id = json!({
        "attributes": true,
        "markers": {
            "[step]": { "data-step": true },
            "[!toc]": { "data-toc": "false" },
            "[toc]": { "data-toc": "only" },
        },
    });
    let mut pipeline = textlane::Pipeline::new();
    use_plugins(
        &mut pipeline,
        json!([
            "md.parse",
            ["md.frontmatter", { "data": true, "strip": true }],
            "md.gfm",
            "md.definitionList",
            "md.math",
            "md.directive",
        ]),
    )?;
    // Before md.component, which renders an unknown directive as its source text
    add_plugins(&mut pipeline, "md", &options.plugins)?;
    use_plugins(
        &mut pipeline,
        json!([
            ["md.headingId", heading_id],
            ["md.component", create_rules(&options.components)],
            ["md.toHtml", { "allowDangerousHtml": true, "jsxElements": true }],
        ]),
    )?;
    // Before the reparse, which keeps no node's `data`, where the fence's meta waits
    use_native(&mut pipeline, "htmlpress.annotateCode", AnnotateCode)?;
    use_plugins(
        &mut pipeline,
        json!([
            "html.raw",
            ["html.stringify", { "allowDangerousHtml": true, "closeEmptyElements": true }],
        ]),
    )?;
    let fail = |error: textlane::Error| match options.plugins.is_empty() {
        true => error.message,
        false => format!("plugins.md: {}", error.message),
    };
    pipeline.freeze().map_err(fail)?;
    pipeline.prepare_process(Api::Sync).map_err(fail)
}

// Livemark's directives, alerts and fences, as its `create.ts` declares them, onto
// htmlpress' elements; any other directive naming a component becomes that component
fn create_rules(components: &[String]) -> Value {
    json!({
        "directives": [
            {
                "name": ["note", "tip", "info", "warning", "danger"],
                "kind": "container",
                "component": "htmlpress-callout",
                "nameAs": "type",
                "labelAs": "title",
                "attributes": [],
            },
            {
                "name": "card",
                "kind": "container",
                "component": "htmlpress-card",
                "attributes": [{ "name": "title", "default": "" }, "href", "icon"],
                "group": { "component": "htmlpress-cards" },
            },
            {
                "name": "column",
                "kind": "container",
                "component": "htmlpress-column",
                "attributes": [],
                "group": { "component": "htmlpress-columns" },
            },
            {
                "name": "tab",
                "kind": "container",
                "component": "htmlpress-tab",
                "attributes": [{ "name": "title", "default": "" }, "sync"],
                "group": { "component": "htmlpress-tabs", "min": 1, "inherit": ["sync"] },
            },
            {
                "name": "details",
                "kind": "container",
                "component": "details",
                "labelAs": "summary",
                "labelDefault": "Details",
                "attributes": [],
            },
            {
                "name": "filetree",
                "kind": "container",
                "component": "htmlpress-filetree",
                "attributes": [],
            },
            {
                "name": "badge",
                "kind": "text",
                "component": "htmlpress-badge",
                "attributes": ["variant"],
            },
            {
                "name": "button",
                "kind": "leaf",
                "component": "htmlpress-button",
                "labelAs": "label",
                "attributes": ["href", "label", "variant", "size"],
                "required": ["href", "label"],
            },
            {
                "name": "icon",
                "kind": "text",
                "component": "htmlpress-icon",
                "attributes": ["name"],
                "required": ["name"],
            },
            {
                "name": "abbr",
                "kind": "text",
                "component": "abbr",
                "attributes": [{ "name": "title", "default": "" }],
            },
            {
                "name": "toc",
                "kind": "leaf",
                "component": "htmlpress-toc",
                "attributes": [{ "name": "maxLevel", "rename": "max-level" }],
            },
            {
                "name": "video",
                "kind": "leaf",
                "where": { "type": "youtube" },
                "component": "htmlpress-video",
                "attributes": ["type", "url"],
                "required": ["url"],
            },
            {
                "name": "audio",
                "kind": "leaf",
                "where": { "type": "soundcloud" },
                "component": "htmlpress-audio",
                "attributes": ["type", "url"],
                "required": ["url"],
            },
            {
                "name": "include",
                "kind": "leaf",
                "component": "htmlpress-include",
                "attributes": [{ "name": "file", "rename": "src" }],
            },
            { "name": components, "component": "htmlpress-{name}" },
        ],
        "alerts": {
            "component": "htmlpress-callout",
            "typeAs": "type",
            "types": {
                "NOTE": "note",
                "TIP": "tip",
                "WARNING": "warning",
                "CAUTION": "danger",
                "IMPORTANT": "info",
            },
        },
        "codes": [
            { "lang": "npm", "component": "htmlpress-package-tabs" },
            {
                "meta": "tab",
                "group": { "component": "htmlpress-tabs", "collect": "tabs", "inherit": ["sync"] },
            },
        ],
        "unknownDirectives": "error",
    })
}

// Livemark's inline `code{:lang}`, as the `language-*` class the highlighter reads; only
// a bare `<code>`, since a block's carries its own class or sits in a `<pre>`
fn mark_inline_code(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("<code>") {
        let body = &rest[start + "<code>".len()..];
        let Some(end) = body.find("</code>") else {
            break;
        };
        let code = &body[..end];
        out.push_str(&rest[..start]);
        let lang = code
            .strip_suffix('}')
            .and_then(|code| code.rsplit_once("{:"))
            .filter(|(_, lang)| {
                !lang.is_empty()
                    && lang
                        .chars()
                        .all(|char| char.is_ascii_alphanumeric() || "+#-".contains(char))
            });
        match lang {
            Some((code, lang)) if !out.ends_with("<pre>") => {
                out.push_str(&format!("<code class=\"language-{lang}\">{code}</code>"))
            }
            _ => out.push_str(&format!("<code>{code}</code>")),
        }
        rest = &body[end + "</code>".len()..];
    }
    out.push_str(rest);
    out
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
