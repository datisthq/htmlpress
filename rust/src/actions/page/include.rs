use std::cell::RefCell;
use std::path::Path;

use lol_html::html_content::ContentType;
use lol_html::{RewriteStrSettings, doctype, element, rewrite_str};

use crate::actions::markdown::render::render_markdown;
use crate::actions::path::contain::{contain_path, read_contained};

// Livemark's limit; a deeper chain is a mistake, and a cycle is caught before it
const MAX_DEPTH: usize = 5;

// Depth alone does not bound a page: ten includes of a file that itself includes ten,
// five deep, expand 100,000 times, so each page also gets a budget
const MAX_EXPANSIONS: usize = 1000;
const MAX_BYTES: usize = 16 * 1024 * 1024;

#[derive(Default)]
struct Budget {
    expansions: usize,
    bytes: usize,
}

// The attributes an include of a code file passes to the block it builds
const PRE_ATTRIBUTES: &[&str] = &["data-title", "data-collapse"];
const CODE_ATTRIBUTES: &[&str] = &[
    "data-lines",
    "data-line-numbers",
    "data-add",
    "data-remove",
    "data-focus",
    "data-error",
    "data-warning",
    "data-words",
];

/// Replace every `<htmlpress-include src="…">` in a page with the file it names, relative to
/// the page's `source` (or the project root with a leading `/`): an `.html` file's body, a
/// `.md` file rendered, or any other file as a titled, highlighted code block. Includes nest
/// up to 5 levels; a missing file, a cycle or a path leaving the project is an error.
pub(crate) fn resolve_includes(
    root: &Path,
    source: &str,
    html: &str,
) -> Result<String, String> {
    resolve_nested(
        root,
        source,
        html,
        &RefCell::new(vec![]),
        &RefCell::new(Budget::default()),
    )
}

// `chain` holds the real paths being included, outermost first, to catch a cycle
fn resolve_nested(
    root: &Path,
    source: &str,
    html: &str,
    chain: &RefCell<Vec<String>>,
    budget: &RefCell<Budget>,
) -> Result<String, String> {
    if !html.contains("<htmlpress-include") {
        return Ok(html.to_owned());
    }
    if chain.borrow().len() >= MAX_DEPTH {
        return Err(format!(
            "includes nest deeper than {MAX_DEPTH}: {}",
            chain.borrow().join(" → ")
        ));
    }
    let folder = source.rsplit_once('/').map_or("", |(folder, _)| folder);
    let settings = RewriteStrSettings::new().append_element_content_handler(element!(
        "htmlpress-include",
        |el| {
            let src = el.get_attribute("src").unwrap_or_default();
            if src.trim().is_empty() {
                return Err("<htmlpress-include> needs a src".into());
            }
            let path = match (src.strip_prefix('/'), folder.is_empty()) {
                (Some(rooted), _) => rooted.to_owned(),
                (None, true) => src.clone(),
                (None, false) => format!("{folder}/{src}"),
            };
            let key = contain_path(root, &path, false)?
                .to_string_lossy()
                .into_owned();
            if chain.borrow().contains(&key) {
                return Err(format!(
                    "include cycle: {} → {src}",
                    chain.borrow().join(" → ")
                )
                .into());
            }
            let text = read_contained(root, &path, false)?;
            let attributes: Vec<(String, String)> = (el.attributes().iter())
                .map(|attribute| (attribute.name(), attribute.value()))
                .collect();
            let content = create_content(&path, &text, &attributes);
            {
                let mut budget = budget.borrow_mut();
                budget.expansions += 1;
                budget.bytes += content.len();
                if budget.expansions > MAX_EXPANSIONS || budget.bytes > MAX_BYTES {
                    return Err(format!(
                        "includes expand past {MAX_EXPANSIONS} files or 16 MB in one page, at {src}"
                    )
                    .into());
                }
            }
            chain.borrow_mut().push(key);
            let nested = resolve_nested(root, &path, &content, chain, budget);
            chain.borrow_mut().pop();
            // Before, then unwrapped rather than replaced: an include left unclosed holds
            // the page that follows as its children, which must not vanish with it
            el.before(&nested?, ContentType::Html);
            el.remove_and_keep_content();
            Ok(())
        }
    ));
    rewrite_str(html, settings).map_err(|error| error.to_string())
}

fn create_content(path: &str, text: &str, attributes: &[(String, String)]) -> String {
    let extension = path
        .rsplit_once('.')
        .map_or("", |(_, extension)| extension)
        .to_lowercase();
    match extension.as_str() {
        "html" | "htm" => read_body(text).unwrap_or_else(|_| text.to_owned()),
        "md" | "markdown" => render_markdown(strip_frontmatter(text)),
        _ => create_code_block(path, &extension, text, attributes),
    }
}

fn create_code_block(
    path: &str,
    extension: &str,
    text: &str,
    attributes: &[(String, String)],
) -> String {
    let name = path.rsplit('/').next().unwrap_or(path);
    let pick = |names: &[&str]| -> String {
        (attributes.iter())
            .filter(|(name, _)| names.contains(&name.as_str()))
            .map(|(name, value)| format!(" {name}=\"{}\"", value.replace('"', "&quot;")))
            .collect()
    };
    let mut pre = pick(PRE_ATTRIBUTES);
    if !pre.contains("data-title=") {
        pre = format!(" data-title=\"{}\"{pre}", escape_text(name));
    }
    let class = read_language(extension).map_or(String::new(), |language| {
        format!(" class=\"language-{language}\"")
    });
    format!(
        "<pre{pre}><code{class}{}>{}</code></pre>",
        pick(CODE_ATTRIBUTES),
        escape_text(text.trim_end())
    )
}

// Livemark's map from a code file's extension to its highlighting language
fn read_language(extension: &str) -> Option<&'static str> {
    Some(match extension {
        "ts" => "typescript",
        "tsx" => "tsx",
        "js" | "mjs" | "cjs" => "javascript",
        "jsx" => "jsx",
        "py" => "python",
        "rs" => "rust",
        "go" => "go",
        "rb" => "ruby",
        "java" => "java",
        "kt" => "kotlin",
        "swift" => "swift",
        "c" | "h" => "c",
        "cpp" => "cpp",
        "cs" => "csharp",
        "css" => "css",
        "scss" => "scss",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "xml" => "xml",
        "sql" => "sql",
        "sh" | "bash" => "bash",
        "zsh" => "zsh",
        "fish" => "fish",
        "ps1" => "powershell",
        "dockerfile" => "dockerfile",
        "graphql" => "graphql",
        "proto" => "protobuf",
        "lua" => "lua",
        "php" => "php",
        "r" => "r",
        "zig" => "zig",
        "ex" | "exs" => "elixir",
        _ => return None,
    })
}

fn read_body(html: &str) -> Result<String, String> {
    let settings = RewriteStrSettings::new()
        .append_document_content_handler(doctype!(|doctype| {
            doctype.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("head", |el| {
            el.remove();
            Ok(())
        }))
        .append_element_content_handler(element!("html, body", |el| {
            el.remove_and_keep_content();
            Ok(())
        }));
    let body = rewrite_str(html, settings).map_err(|error| error.to_string())?;
    Ok(body.trim().to_owned())
}

fn strip_frontmatter(text: &str) -> &str {
    text.strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---").map(|(_, after)| after))
        .map_or(text, |after| {
            after.split_once('\n').map_or("", |(_, body)| body)
        })
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
#[path = "include_unit.rs"]
mod unit;
