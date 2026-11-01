use serde_json::{Value, json};

use crate::actions::code::highlight::create_highlight;
use crate::actions::plugin::add::add_plugins;
use crate::models::pipeline::{Pipeline, PipelineOptions};
use crate::models::plugin::{SimplifyPage, WrapMath};

// Where text is code, already a link, or not prose a reader sees as such
const LITERAL: &[&str] = &[
    "a",
    "code",
    "pre",
    "kbd",
    "samp",
    "script",
    "style",
    "textarea",
    "title",
    "math",
    "svg",
    "htmlpress-math",
    "htmlpress-terminal",
    "htmlpress-mermaid",
];

/// The pipelines a page goes through once: bare URLs, emails and `:emoji:` shortcodes in
/// its prose, `$…$` math as MathML in `<htmlpress-math>`, the project's `html` plugins, an
/// id on every heading from `<h2>` down that has none, never one the page already uses,
/// then what `options` asks for, and the Markdown copy of the same tree.
pub(crate) fn create_pipeline(options: &PipelineOptions) -> Result<Pipeline, String> {
    let mut pipeline = Pipeline {
        document: create_html(options, false)?,
        fragment: create_html(options, true)?,
        markdown: create_markdown()?,
    };
    // Frozen here, so a plugin that does not fit is the config's error, not a page's
    let plugins = match options.plugins.is_empty() {
        true => "",
        false => "plugins.html: ",
    };
    for part in [&mut pipeline.document, &mut pipeline.fragment] {
        part.freeze()
            .map_err(|error| format!("{plugins}{}", error.message))?;
    }
    pipeline.markdown.freeze().map_err(|error| error.message)?;
    Ok(pipeline)
}

fn create_html(
    options: &PipelineOptions,
    fragment: bool,
) -> Result<textlane::Pipeline, String> {
    let ignore = json!({ "ignore": LITERAL, "ignoreAttributes": ["data-literal"] });
    let mut pipeline = textlane::Pipeline::new();
    use_plugins(
        &mut pipeline,
        json!([
            ["html.parse", { "fragment": fragment }],
            ["html.gemoji", ignore],
            ["html.linkify", ignore],
            ["html.math", ignore],
        ]),
    )?;
    use_native(&mut pipeline, "htmlpress.wrapMath", WrapMath)?;
    use_plugins(
        &mut pipeline,
        json!([["html.katex", { "output": "mathml" }]]),
    )?;
    add_plugins(&mut pipeline, "html", &options.plugins)?;
    let slug = json!({ "reserveExisting": true, "ranks": [2, 3, 4, 5, 6] });
    use_plugins(&mut pipeline, json!([["html.slug", slug]]))?;
    if let Some(root) = &options.rebase {
        let pages: serde_json::Map<String, Value> = (options.links.iter())
            .map(|(file, url)| (file.trim_start_matches('/').to_owned(), json!(url)))
            .collect();
        let root = (!root.is_empty()).then(|| root.clone());
        let rewrite = json!({ "root": root, "pages": pages });
        use_plugins(&mut pipeline, json!([["html.pathRewrite", rewrite]]))?;
    }
    if let Some((light, dark)) = &options.themes {
        let highlight = create_highlight(light, dark);
        use_plugins(&mut pipeline, json!([["html.highlight", highlight]]))?;
    }
    let stringify = json!({ "closeEmptyElements": true });
    use_plugins(&mut pipeline, json!([["html.stringify", stringify]]))?;
    Ok(pipeline)
}

// KaTeX's MathML carries its LaTeX, which comes back as `$…$`, what agents read
fn create_markdown() -> Result<textlane::Pipeline, String> {
    let mut pipeline = textlane::Pipeline::new();
    use_native(&mut pipeline, "htmlpress.simplifyPage", SimplifyPage)?;
    use_plugins(
        &mut pipeline,
        json!([
            ["html.toMarkdown", { "math": true }],
            "md.gfm",
            "md.math",
            ["md.stringify", { "bullet": "-" }],
        ]),
    )?;
    Ok(pipeline)
}

/// Add plugins by name to a pipeline, as `use_value` takes them.
pub(crate) fn use_plugins(
    pipeline: &mut textlane::Pipeline,
    list: Value,
) -> Result<(), String> {
    pipeline
        .use_value(&list, None)
        .map_err(|error| error.message)
}

/// Add one of htmlpress' own plugins to a pipeline.
pub(crate) fn use_native(
    pipeline: &mut textlane::Pipeline,
    id: &str,
    plugin: impl textlane::Plugin + 'static,
) -> Result<(), String> {
    pipeline
        .use_native(id, plugin)
        .map_err(|error| error.message)
}

#[cfg(test)]
#[path = "create_unit.rs"]
mod unit;
