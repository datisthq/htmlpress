use textlane::ArenaVec;
use textlane::hast::{Element, Node};

/// What [`walk_nodes`] does after visiting an element.
pub(crate) enum Step<'a> {
    /// Go on into its children.
    Continue,
    /// Leave its children unvisited, such as markup the visit just put there.
    Skip,
    /// Take the element, children and all, out of the tree.
    Remove,
    /// Put these nodes, unvisited, where the element was.
    Replace(ArenaVec<'a, Node<'a>>),
}

/// Visit every element under `nodes` in document order, as `visit_elements` does, but
/// able to remove one; a `template`'s content is not visited.
pub(crate) fn walk_nodes<'a, E>(
    nodes: &mut ArenaVec<'a, Node<'a>>,
    visit: &mut impl FnMut(&mut Element<'a>) -> Result<Step<'a>, E>,
) -> Result<(), E> {
    let mut index = 0;
    while index < nodes.len() {
        let step = match nodes.get_mut(index) {
            Some(Node::Element(element)) => visit(element)?,
            _ => Step::Skip,
        };
        match step {
            Step::Remove => {
                nodes.remove(index);
                continue;
            }
            Step::Continue => {
                if let Some(Node::Element(element)) = nodes.get_mut(index) {
                    walk_nodes(&mut element.children, visit)?;
                }
            }
            Step::Skip => {}
            Step::Replace(replacement) => {
                nodes.remove(index);
                let count = replacement.len();
                for (offset, node) in replacement.into_iter().enumerate() {
                    nodes.insert(index + offset, node);
                }
                index += count;
                continue;
            }
        }
        index += 1;
    }
    Ok(())
}
