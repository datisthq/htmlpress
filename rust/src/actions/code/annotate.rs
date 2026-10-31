use textlane::hast::{Element, Node};
use textlane::html::write_attribute;
use textlane::{Allocator, Data};

use crate::actions::page::include::{CODE_ATTRIBUTES, PRE_ATTRIBUTES};

/// Give each Markdown fence the attributes an HTML page writes on its code block: the
/// meta's `title="a.js"` and `collapse="10"` onto the `<pre>`, `lines`, `line-numbers`,
/// `add`, `remove`, `focus`, `error`, `warning` and `words` onto the `<code>`.
pub(crate) fn annotate_code<'a>(node: &mut Node<'a>, alloc: &'a Allocator) {
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        let children = match node {
            Node::Root(root) => &mut root.children,
            Node::Element(element) => &mut element.children,
            _ => continue,
        };
        for child in children.iter_mut() {
            match child {
                Node::Element(pre) if pre.tag_name == "pre" => annotate_block(pre, alloc),
                child => stack.push(child),
            }
        }
    }
}

fn annotate_block<'a>(pre: &mut Element<'a>, alloc: &'a Allocator) {
    let Some(Node::Element(code)) = pre.children.first_mut() else {
        return;
    };
    let Some(meta) = read_meta(code) else {
        return;
    };
    for (key, value) in parse_meta(meta) {
        let name = format!("data-{key}");
        if PRE_ATTRIBUTES.contains(&name.as_str()) {
            write_attribute(pre, &name, &value, alloc);
        } else if CODE_ATTRIBUTES.contains(&name.as_str())
            && let Some(Node::Element(code)) = pre.children.first_mut()
        {
            write_attribute(code, &name, &value, alloc);
        }
    }
}

// What follows a fence's language, which `md.toHtml` keeps in the `<code>`'s data
fn read_meta<'a>(code: &Element<'a>) -> Option<&'a str> {
    let Some(Data::Object(entries)) = code.data else {
        return None;
    };
    entries.iter().find_map(|(key, value)| match (*key, value) {
        ("meta", Data::String(meta)) => Some(*meta),
        _ => None,
    })
}

// `key="a b"`, `key='a'`, `key=a` and a bare `key`, which is an empty value
fn parse_meta(meta: &str) -> Vec<(String, String)> {
    let mut pairs = vec![];
    let mut rest = meta.trim();
    while !rest.is_empty() {
        let end = rest.find(|char: char| char == '=' || char.is_whitespace());
        let (key, after) = rest.split_at(end.unwrap_or(rest.len()));
        let (value, after) = match after.strip_prefix('=') {
            Some(value) => match value.chars().next() {
                Some(quote @ ('"' | '\'')) => {
                    let inner = &value[1..];
                    let close = inner.find(quote).unwrap_or(inner.len());
                    (&inner[..close], inner.get(close + 1..).unwrap_or(""))
                }
                _ => {
                    value.split_at(value.find(char::is_whitespace).unwrap_or(value.len()))
                }
            },
            None => ("", after),
        };
        if !key.is_empty() {
            pairs.push((key.to_owned(), value.to_owned()));
        }
        rest = after.trim_start();
    }
    pairs
}

#[cfg(test)]
#[path = "annotate_unit.rs"]
mod unit;
