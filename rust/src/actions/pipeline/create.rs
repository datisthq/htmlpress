use textlane::{FrozenProcessor, Processor, TreeKind, html, md};

use crate::actions::code::highlight::create_highlight;
use crate::actions::plugin::load::load_plugins;
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

/// The processors a page goes through once: bare URLs, emails and `:emoji:` shortcodes in
/// its prose, `$…$` math as MathML in `<htmlpress-math>`, an id on every heading from `<h2>`
/// down that has none, never one the page already uses, then what `options` asks for, and
/// the Markdown copy of the same tree. The project's `html` plugins run before the ids.
pub(crate) fn create_pipeline(options: &PipelineOptions) -> Result<Pipeline, String> {
    Ok(Pipeline {
        document: create_html(options, false)?,
        fragment: create_html(options, true)?,
        markdown: create_markdown().map_err(|error| error.message)?,
    })
}

fn create_html(
    options: &PipelineOptions,
    fragment: bool,
) -> Result<FrozenProcessor, String> {
    let ignore = Some(
        LITERAL
            .iter()
            .map(|tag| (*tag).to_owned())
            .collect::<Vec<_>>(),
    );
    let ignore_attributes = vec!["data-literal".to_owned()];
    let mut processor = Processor::new()
        .with(html::Parse {
            fragment,
            ..html::Parse::default()
        })
        .with(html::Gemoji {
            ignore: ignore.clone(),
            ignore_attributes: ignore_attributes.clone(),
            ..html::Gemoji::default()
        })
        .with(html::Linkify {
            ignore: ignore.clone(),
            ignore_attributes: ignore_attributes.clone(),
        })
        .with(html::Math {
            ignore,
            ignore_attributes,
            ..html::Math::default()
        })
        .with(WrapMath)
        .with(html::Katex {
            output: Some(html::KatexOutput::Mathml),
            ..html::Katex::default()
        });
    if let Some(plugins) = load_plugins(TreeKind::Hast, &options.plugins)? {
        processor = processor.with(plugins);
    }
    processor = processor.with(html::Slug {
        reserve_existing: true,
        ranks: Some(vec![2, 3, 4, 5, 6]),
        ..html::Slug::default()
    });
    if let Some(root) = &options.rebase {
        processor = processor.with(html::PathRewrite {
            root: (!root.is_empty()).then(|| root.clone()),
            pages: (options.links.iter())
                .map(|(file, url)| (file.trim_start_matches('/').to_owned(), url.clone()))
                .collect(),
            ..html::PathRewrite::default()
        });
    }
    if let Some((light, dark)) = &options.themes {
        processor = processor.with(create_highlight(light, dark));
    }
    processor
        .with(html::Stringify {
            close_empty_elements: true,
            ..html::Stringify::default()
        })
        .freeze()
        .map_err(|error| error.message)
}

// KaTeX's MathML carries its LaTeX, which comes back as `$…$`, what agents read
fn create_markdown() -> Result<FrozenProcessor, textlane::Error> {
    Processor::new()
        .with(SimplifyPage)
        .with(html::ToMarkdown {
            math: true,
            ..html::ToMarkdown::default()
        })
        .with(md::Gfm::default())
        .with(md::Math::default())
        .with(md::Stringify {
            bullet: Some("-".to_owned()),
            ..md::Stringify::default()
        })
        .freeze()
}

#[cfg(test)]
#[path = "create_unit.rs"]
mod unit;
