use super::*;
use crate::actions::page::read::read_page;

fn read_pages(paths: &[&str]) -> crate::Result<Vec<Page>> {
    (paths.iter())
        .map(|path| read_page(path, &format!("<title>{path}</title>")))
        .collect()
}

fn list_paths(nodes: &[NavNode]) -> Vec<String> {
    let mut paths = vec![];
    for node in nodes {
        let path = node.page.path.as_str();
        let children = list_paths(&node.children);
        paths.push(match children.is_empty() {
            true => path.to_owned(),
            false => format!("{path}[{}]", children.join(" ")),
        });
    }
    paths
}

#[test]
fn nests_pages_under_their_folders_index_at_any_depth() -> crate::Result<()> {
    let pages = read_pages(&[
        "index.html",
        "guide/index.html",
        "guide/install.html",
        "guide/cli/index.html",
        "guide/cli/run.html",
        "about.html",
    ])?;
    let pages: Vec<&Page> = pages.iter().collect();
    let nav = build_nav(&pages, "", false);
    assert_eq!(
        list_paths(&nav),
        [
            "index.html",
            "guide/index.html[guide/install.html guide/cli/index.html[guide/cli/run.html]]",
            "about.html",
        ]
    );
    let order: Vec<&str> = (NavNode::flatten(&nav).iter())
        .map(|page| page.path.as_str())
        .collect();
    assert_eq!(order[4], "guide/cli/run.html");
    Ok(())
}

#[test]
fn keeps_the_pages_of_a_folder_without_an_index_at_the_top() -> crate::Result<()> {
    let pages = read_pages(&[
        "index.html",
        "plain/a.html",
        "guide/index.html",
        "guide/deep/b.html",
        "plain/c.html",
    ])?;
    let pages: Vec<&Page> = pages.iter().collect();
    assert_eq!(
        list_paths(&build_nav(&pages, "", false)),
        [
            "index.html",
            "plain/a.html",
            "guide/index.html",
            "guide/deep/b.html",
            "plain/c.html"
        ]
    );
    Ok(())
}

#[test]
fn nests_under_an_index_that_sorts_after_its_pages() -> crate::Result<()> {
    let pages = read_pages(&[
        "api/auth/login.html",
        "api/index.html",
        "api/auth/index.html",
    ])?;
    let pages: Vec<&Page> = pages.iter().collect();
    let nav = build_nav(&pages, "/api/", false);
    assert_eq!(
        list_paths(&nav),
        ["api/index.html", "api/auth/index.html[api/auth/login.html]"]
    );
    Ok(())
}

#[test]
fn keeps_every_page_at_the_top_when_flat() -> crate::Result<()> {
    let pages = read_pages(&["blog/index.html", "blog/2026/post.html"])?;
    let pages: Vec<&Page> = pages.iter().collect();
    assert_eq!(
        list_paths(&build_nav(&pages, "/blog/", true)),
        ["blog/index.html", "blog/2026/post.html"]
    );
    Ok(())
}

#[test]
fn places_a_folder_by_its_index_order_over_its_pages() -> crate::Result<()> {
    let page = |path: &str, order: i64| {
        let html =
            format!("<title>{path}</title><meta name=\"order\" content=\"{order}\">");
        read_page(path, &html)
    };
    // As `sort_pages` leaves them: the folder's own page sorts after the other order 1
    let pages = [
        read_page("index.html", "<title>Home</title>")?,
        page("guides/build.html", 1)?,
        page("zeta.html", 1)?,
        page("guides/index.html", 2)?,
    ];
    let pages: Vec<&Page> = pages.iter().collect();
    let nav = build_nav(&pages, "", false);
    assert_eq!(
        list_paths(&nav),
        [
            "index.html",
            "zeta.html",
            "guides/index.html[guides/build.html]"
        ]
    );
    Ok(())
}
