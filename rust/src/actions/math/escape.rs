use textlane::mdast::Node;
use textlane::{Allocator, Position};

/// Write each dollar a Markdown page escaped, `\$`, escaped again in its text, since the
/// HTML math step runs after Markdown and would pair two plain dollars into a formula.
pub(crate) fn escape_dollars<'a>(
    node: &mut Node<'a>,
    source: &str,
    alloc: &'a Allocator,
) {
    let mut stack: Vec<&mut Node<'a>> = vec![node];
    while let Some(node) = stack.pop() {
        if let Node::Text(text) = node {
            let Some(Position::Span { start, end }) = text.position else {
                continue;
            };
            let Some(span) = source.get(start.offset()..end.offset()) else {
                continue;
            };
            if let Some(value) = escape_text(text.value, span) {
                text.value = alloc.alloc_str(&value);
            }
            continue;
        }
        if let Some(children) = node.children_mut() {
            stack.extend(children.iter_mut());
        }
    }
}

// Each dollar of the text stands for one of its source, in order; None when nothing was
// escaped, or when the two disagree (a `&#36;` reference), so the text stays as parsed
fn escape_text(value: &str, span: &str) -> Option<String> {
    let mut escaped = vec![];
    let mut slashes = 0;
    for char in span.chars() {
        if char == '$' {
            escaped.push(slashes % 2 == 1);
        }
        slashes = if char == '\\' { slashes + 1 } else { 0 };
    }
    if !escaped.contains(&true) || escaped.len() != value.matches('$').count() {
        return None;
    }
    let mut flags = escaped.into_iter();
    let mut result = String::with_capacity(value.len() + 2);
    for char in value.chars() {
        if char == '$' && flags.next() == Some(true) {
            result.push('\\');
        }
        result.push(char);
    }
    Some(result)
}

#[cfg(test)]
#[path = "escape_unit.rs"]
mod unit;
