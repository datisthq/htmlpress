use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use serde_json::{Map, Value, json};
use textlane::hast::{Doctype, Node, Root};
use textlane::html::{Visit, visit_elements, write_attribute};
use textlane::{Allocator, ArenaVec, File, Tree};

use crate::actions::node::collect::collect_text;
use crate::actions::node::create::{create_element, create_text};
use crate::actions::node::parse::parse_fragment;
use crate::actions::pipeline::create::{use_native, use_plugins};
use crate::actions::plugin::add::add_plugins;
use crate::models::pipeline::MarkdownOptions;
use crate::models::plugin::{AnnotateCode, EscapeDollars, MarkInlineCode};
use crate::models::source::Compiled;

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
    static PIPELINES: RefCell<HashMap<(Vec<String>, String), textlane::Pipeline>> =
        RefCell::new(HashMap::new());
}

/// Turn Markdown into hast: Livemark's directives, alerts and fences as htmlpress'
/// elements, any other `:::name` as `<htmlpress-name>` of one of the project's components,
/// the project's `md` plugins after the directives, and a fence's meta as its block's
/// `data-*` attributes. A page's frontmatter becomes its head with [`create_document`];
/// a partial's is dropped, unread.
pub(crate) fn compile_markdown<'a>(
    markdown: &'a str,
    options: &MarkdownOptions,
    alloc: &'a Allocator,
) -> Result<Compiled<'a>, String> {
    PIPELINES.with(|pipelines| {
        let mut pipelines = pipelines.borrow_mut();
        let plugins =
            serde_json::to_string(&options.plugins).map_err(|e| e.to_string())?;
        let pipeline = match pipelines.entry((options.components.clone(), plugins)) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(create_processor(options)?),
        };
        let processor = pipeline.freeze().map_err(|error| error.message)?;
        let mut file = File::new();
        let tree = (processor.parse(markdown, &mut file, alloc))
            .and_then(|tree| processor.run_sync(tree, &mut file, alloc))
            .map_err(|error| error.message)?;
        let body = match tree {
            Tree::Hast(Node::Root(root)) => root.children,
            Tree::Hast(node) => {
                let mut nodes = ArenaVec::new_in(alloc);
                nodes.push(node);
                nodes
            }
            _ => return Err("the Markdown pipeline did not end in HTML".to_owned()),
        };
        let matter = match file.data.get("matter") {
            Some(Value::Object(matter)) => matter.clone(),
            _ => Map::new(),
        };
        Ok(Compiled { body, matter })
    })
}

/// A compiled page as the document an HTML page would be: its frontmatter as the head's
/// `<title>` and metas, then the frontmatter's raw `head`, its body as the body.
pub(crate) fn create_document<'a>(
    compiled: Compiled<'a>,
    alloc: &'a Allocator,
) -> Result<Node<'a>, String> {
    let mut head = create_element("head", &[], alloc);
    head.children = create_head(&compiled.matter, alloc)?;
    let mut body = create_element("body", &[], alloc);
    body.children = compiled.body;
    let mut html = create_element("html", &[], alloc);
    html.children.push(Node::Element(head));
    html.children.push(Node::Element(body));
    let mut children = ArenaVec::new_in(alloc);
    children.push(Node::Doctype(Doctype {
        position: None,
        data: None,
    }));
    children.push(Node::Element(html));
    Ok(Node::Root(Root {
        children,
        position: None,
        data: None,
        quirks_mode: None,
    }))
}

/// The Markdown pipeline, frozen; also what the config's `md` plugins are checked
/// against before any page compiles.
pub(crate) fn create_processor(
    options: &MarkdownOptions,
) -> Result<textlane::Pipeline, String> {
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
            // Pandoc's dollar rule, as the HTML math step reads `$`: `$5 and $10` is no formula
            ["md.math", { "pandocTextMath": true }],
            "md.directive",
        ]),
    )?;
    use_native(&mut pipeline, "htmlpress.escapeDollars", EscapeDollars)?;
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
    use_plugins(&mut pipeline, json!(["html.raw"]))?;
    use_native(&mut pipeline, "htmlpress.markInlineCode", MarkInlineCode)?;
    let fail = |error: textlane::Error| match options.plugins.is_empty() {
        true => error.message,
        false => format!("plugins.md: {}", error.message),
    };
    pipeline.freeze().map_err(fail)?;
    Ok(pipeline)
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

/// Livemark's inline `code{:lang}`, as the `language-*` class the highlighter reads; only
/// a bare `<code>`, since a block's carries its own class or sits in a `<pre>`.
pub(crate) fn mark_inline_code<'a>(node: &mut Node<'a>, alloc: &'a Allocator) {
    visit_elements(node, |element, ancestors| {
        let bare = element.tag_name == "code"
            && element.properties.is_empty()
            && ancestors.last() != Some(&"pre")
            && (element.children.iter()).all(|child| matches!(child, Node::Text(_)));
        if !bare {
            return Visit::Continue;
        }
        let text = collect_text(&element.children);
        let lang = text
            .strip_suffix('}')
            .and_then(|code| code.rsplit_once("{:"))
            .filter(|(_, lang)| {
                !lang.is_empty()
                    && lang
                        .chars()
                        .all(|char| char.is_ascii_alphanumeric() || "+#-".contains(char))
            });
        if let Some((code, lang)) = lang {
            write_attribute(element, "class", &format!("language-{lang}"), alloc);
            element.children.clear();
            element.children.push(create_text(code, alloc));
        }
        Visit::Skip
    });
}

fn create_head<'a>(
    matter: &Map<String, Value>,
    alloc: &'a Allocator,
) -> Result<ArenaVec<'a, Node<'a>>, String> {
    let mut head = ArenaVec::new_in(alloc);
    let mut raw = vec![];
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
                    let mut title = create_element("title", &[], alloc);
                    title.children.push(create_text(&text, alloc));
                    head.push(Node::Element(title));
                }
                // Raw HTML the author wrote, parsed here for the first time
                "head" => raw.push(text),
                key => head.push(Node::Element(create_element(
                    "meta",
                    &[("name", key), ("content", &text)],
                    alloc,
                ))),
            }
        }
    }
    for text in raw {
        head.extend(parse_fragment(&text, alloc));
    }
    Ok(head)
}

fn read_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        value => value.to_string(),
    }
}

#[cfg(test)]
#[path = "compile_unit.rs"]
mod unit;
