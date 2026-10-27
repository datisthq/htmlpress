use std::cell::RefCell;
use std::sync::OnceLock;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, element, rewrite_str, text};
use math_core::{LatexToMathML, MathCoreConfig, MathDisplay};

static CONVERTER: OnceLock<Option<LatexToMathML>> = OnceLock::new();

/// Render every `<htmlpress-math>` (written in a page, or made from `$…$` by the prose
/// pass) as MathML, which browsers draw natively; `display` makes it a block. The LaTeX
/// stays in `data-latex` for the Markdown copy. Invalid LaTeX is an error.
pub(crate) fn render_math(html: &str) -> Result<String, String> {
    if !html.contains("<htmlpress-math") {
        return Ok(html.to_owned());
    }
    let sources = list_sources(html)?;
    let Some(converter) = CONVERTER
        .get_or_init(|| LatexToMathML::new(MathCoreConfig::default(), vec![]).ok())
        .as_ref()
    else {
        return Ok(html.to_owned());
    };
    let index = RefCell::new(0usize);
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "htmlpress-math",
        |el| {
            let latex = sources.get(*index.borrow()).cloned().unwrap_or_default();
            *index.borrow_mut() += 1;
            let display = match el.has_attribute("display") {
                true => MathDisplay::Block,
                false => MathDisplay::Inline,
            };
            let mathml = converter
                .convert_with_local_state(latex.trim(), display)
                .map_err(|error| format!("math \"{}\": {error}", latex.trim()))?;
            el.set_attribute("data-latex", latex.trim())?;
            el.set_inner_content(&mathml.mathml, ContentType::Html);
            Ok(())
        }
    ));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

// Each element's LaTeX, decoded, in document order: the text is only whole once the
// element ends, after its start tag is already written
fn list_sources(html: &str) -> Result<Vec<String>, String> {
    let sources: RefCell<Vec<String>> = RefCell::new(vec![]);
    let settings = RewriteStrSettings::new()
        .append_element_content_handler(element!("htmlpress-math", |_| {
            sources.borrow_mut().push(String::new());
            Ok(())
        }))
        .append_element_content_handler(text!("htmlpress-math", |chunk| {
            if let Some(source) = sources.borrow_mut().last_mut() {
                source.push_str(chunk.as_str());
            }
            Ok(())
        }));
    rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(sources
        .into_inner()
        .into_iter()
        .map(|source| {
            source
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&amp;", "&")
        })
        .collect())
}

#[cfg(test)]
#[path = "math_unit.rs"]
mod unit;
