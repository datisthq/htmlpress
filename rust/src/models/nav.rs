use crate::models::page::Page;

/// One sidebar entry: a page, holding the pages of its folder when it is that folder's
/// `index.html`.
#[derive(Debug)]
pub(crate) struct NavNode<'a> {
    pub page: &'a Page,
    pub children: Vec<NavNode<'a>>,
}

impl<'a> NavNode<'a> {
    /// The pages under these entries, each before its children, as the pager walks them.
    pub(crate) fn flatten(nodes: &[NavNode<'a>]) -> Vec<&'a Page> {
        let mut pages = vec![];
        for node in nodes {
            pages.push(node.page);
            pages.extend(Self::flatten(&node.children));
        }
        pages
    }
}
