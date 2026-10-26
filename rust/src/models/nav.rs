use crate::models::page::Page;

/// One sidebar entry: a page, a folder, or a folder with its index page, holding the
/// entries nested under it.
#[derive(Debug)]
pub(crate) struct NavNode<'a> {
    pub page: Option<&'a Page>,
    /// The `index.html` path a folder's entry stands for; None for a plain page.
    pub folder: Option<String>,
    pub children: Vec<NavNode<'a>>,
}

impl<'a> NavNode<'a> {
    /// The pages under these entries, each before its children, as the pager walks them.
    pub(crate) fn flatten(nodes: &[NavNode<'a>]) -> Vec<&'a Page> {
        let mut pages = vec![];
        for node in nodes {
            pages.extend(node.page);
            pages.extend(Self::flatten(&node.children));
        }
        pages
    }
}
