use std::path::Path;

use textlane::hast::{Element, Node};
use textlane::html::read_attribute;
use textlane::{Allocator, ArenaVec};

use crate::actions::node::create::{create_element, create_text};
use crate::actions::page::parse::{create_partial_nodes, load_source};
use crate::actions::path::contain::{contain_path, read_contained};
use crate::models::source::Sources;

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

/// The attributes a code block's `<pre>` takes: an included file's or a Markdown fence's.
pub(crate) const PRE_ATTRIBUTES: &[&str] = &["data-title", "data-collapse"];
/// The attributes a code block's `<code>` takes, which mark its lines and words.
pub(crate) const CODE_ATTRIBUTES: &[&str] = &[
    "data-lines",
    "data-line-numbers",
    "data-add",
    "data-remove",
    "data-focus",
    "data-error",
    "data-warning",
    "data-words",
];

/// Replace every `<htmlpress-include src="…">` in a page's tree with the file it names,
/// relative to the page's `source` (or the project root with a leading `/`): an `.html`
/// file's body, a `.md` file compiled, or any other file as a titled, highlighted code
/// block. Includes nest up to 5 levels; a missing file, a cycle or a path leaving the
/// project is an error. Each file is parsed once a build, and copied into every page.
pub(crate) fn resolve_includes<'a>(
    root: &Path,
    source: &str,
    node: &mut Node<'a>,
    sources: &mut Sources<'a>,
) -> Result<(), String> {
    let children = match node {
        Node::Root(root) => &mut root.children,
        Node::Element(element) => &mut element.children,
        _ => return Ok(()),
    };
    let mut chain = vec![];
    resolve_nested(
        root,
        source,
        children,
        sources,
        &mut chain,
        &mut Budget::default(),
    )
}

// `chain` holds the real paths being included, outermost first, to catch a cycle
fn resolve_nested<'a>(
    root: &Path,
    source: &str,
    nodes: &mut ArenaVec<'a, Node<'a>>,
    sources: &mut Sources<'a>,
    chain: &mut Vec<String>,
    budget: &mut Budget,
) -> Result<(), String> {
    let mut index = 0;
    while index < nodes.len() {
        let Some(Node::Element(element)) = nodes.get_mut(index) else {
            index += 1;
            continue;
        };
        if element.tag_name != "htmlpress-include" {
            resolve_nested(root, source, &mut element.children, sources, chain, budget)?;
            index += 1;
            continue;
        }
        let mut included = expand_include(root, source, element, sources, chain, budget)?;
        // Unwrapped rather than replaced: an include left unclosed holds the page that
        // follows as its children, which must not vanish with it
        let mut rest =
            std::mem::replace(&mut element.children, ArenaVec::new_in(sources.alloc));
        resolve_nested(root, source, &mut rest, sources, chain, budget)?;
        nodes.remove(index);
        let count = included.len() + rest.len();
        let tail: Vec<Node<'a>> = nodes.drain(index..).collect();
        nodes.extend(included.drain(..));
        nodes.extend(rest.drain(..));
        nodes.extend(tail);
        index += count;
    }
    Ok(())
}

fn expand_include<'a>(
    root: &Path,
    source: &str,
    element: &Element<'a>,
    sources: &mut Sources<'a>,
    chain: &mut Vec<String>,
    budget: &mut Budget,
) -> Result<ArenaVec<'a, Node<'a>>, String> {
    let src = read_attribute(element, "src")
        .unwrap_or_default()
        .into_owned();
    if src.trim().is_empty() {
        return Err("<htmlpress-include> needs a src".to_owned());
    }
    if chain.len() >= MAX_DEPTH {
        return Err(format!(
            "includes nest deeper than {MAX_DEPTH}: {}",
            chain.join(" → ")
        ));
    }
    let folder = source.rsplit_once('/').map_or("", |(folder, _)| folder);
    let path = match (src.strip_prefix('/'), folder.is_empty()) {
        (Some(rooted), _) => rooted.to_owned(),
        (None, true) => src.clone(),
        (None, false) => format!("{folder}/{src}"),
    };
    let key = contain_path(root, &path, false)?;
    let name = key.to_string_lossy().into_owned();
    if chain.contains(&name) {
        return Err(format!("include cycle: {} → {src}", chain.join(" → ")));
    }
    let extension = (path.rsplit_once('.'))
        .map_or("", |(_, extension)| extension)
        .to_lowercase();
    let alloc = sources.alloc;
    let (mut nodes, bytes) = match extension.as_str() {
        "html" | "htm" | "md" | "markdown" => {
            let markdown = matches!(extension.as_str(), "md" | "markdown");
            let read = || read_contained(root, &path, false);
            let nodes = create_partial_nodes(
                load_source(sources, key.clone(), &path, markdown, read)?,
                alloc,
            );
            (nodes, sources.sizes.get(&key).copied().unwrap_or_default())
        }
        _ => {
            let text = read_contained(root, &path, false)?;
            let mut nodes = ArenaVec::new_in(alloc);
            nodes.push(create_code_block(&path, &extension, &text, element, alloc));
            (nodes, text.len())
        }
    };
    budget.expansions += 1;
    budget.bytes += bytes;
    if budget.expansions > MAX_EXPANSIONS || budget.bytes > MAX_BYTES {
        return Err(format!(
            "includes expand past {MAX_EXPANSIONS} files or 16 MB in one page, at {src}"
        ));
    }
    chain.push(name);
    let nested = resolve_nested(root, &path, &mut nodes, sources, chain, budget);
    chain.pop();
    nested?;
    Ok(nodes)
}

fn borrow_pairs(list: &[(String, String)]) -> Vec<(&str, &str)> {
    list.iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect()
}

fn create_code_block<'a>(
    path: &str,
    extension: &str,
    text: &str,
    include: &Element<'a>,
    alloc: &'a Allocator,
) -> Node<'a> {
    let pick = |names: &[&str]| -> Vec<(String, String)> {
        (names.iter())
            .filter_map(|name| {
                read_attribute(include, name)
                    .map(|value| ((*name).to_owned(), value.into_owned()))
            })
            .collect()
    };
    let mut pre_attributes = pick(PRE_ATTRIBUTES);
    if !pre_attributes.iter().any(|(name, _)| name == "data-title") {
        let name = path.rsplit('/').next().unwrap_or(path);
        pre_attributes.insert(0, ("data-title".to_owned(), name.to_owned()));
    }
    let mut code_attributes = vec![];
    if let Some(language) = read_language(extension) {
        code_attributes.push(("class".to_owned(), format!("language-{language}")));
    }
    code_attributes.extend(pick(CODE_ATTRIBUTES));
    let mut code = create_element("code", &borrow_pairs(&code_attributes), alloc);
    code.children.push(create_text(text.trim_end(), alloc));
    let mut pre = create_element("pre", &borrow_pairs(&pre_attributes), alloc);
    pre.children.push(Node::Element(code));
    Node::Element(pre)
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

#[cfg(test)]
#[path = "include_unit.rs"]
mod unit;
