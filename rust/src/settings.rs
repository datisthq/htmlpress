use crate::models::asset::Asset;

/// The project's config file, read from the project root.
pub const CONFIG_FILE: &str = "htmlpress.yaml";

/// The project's own folder: build output and escapes. Never scanned for pages.
pub const PROJECT_DIR: &str = ".htmlpress";

/// The build output, relative to the project root, unless `--out` names another.
pub const OUTPUT_DIR: &str = ".htmlpress/build";

/// Where the built-in theme and components are written in the output.
pub const ASSETS_DIR: &str = "_htmlpress";

/// The dev server's ports, tried in order when `--port` names none.
pub const PORTS: std::ops::RangeInclusive<u16> = 8000..=8009;

/// The dev server's long-poll endpoint the reload script waits on.
pub const RELOAD_PATH: &str = "/_htmlpress/reload";

/// Where a host looks for the page it serves on a missing URL; never listed in the
/// navigation or the sitemap.
pub const NOT_FOUND_PAGE: &str = "404.html";

/// The page a project without its own `404.html` gets; never escaped, since a project
/// replaces it with a page of the same name.
pub const NOT_FOUND: &str = include_str!("404.html");

/// The dev server's reload script; never escaped.
pub const RELOAD: &str = include_str!("reload.js");

macro_rules! asset {
    ($path:literal) => {
        Asset {
            path: $path,
            content: include_str!($path),
        }
    };
}

/// Every built-in asset, by its path under `.htmlpress/`: a project file at the same path
/// replaces it.
pub const ASSETS: &[Asset] = &[
    asset!("layout.html"),
    asset!("styles/tokens.css"),
    asset!("styles/layout.css"),
    asset!("styles/content.css"),
    asset!("scripts/code.js"),
    asset!("scripts/hints.js"),
    asset!("components/htmlpress-aside.css"),
    asset!("components/htmlpress-back-to-top.css"),
    asset!("components/htmlpress-back-to-top.js"),
    asset!("components/htmlpress-badge.css"),
    asset!("components/htmlpress-banner.css"),
    asset!("components/htmlpress-brand.css"),
    asset!("components/htmlpress-breadcrumbs.css"),
    asset!("components/htmlpress-button.css"),
    asset!("components/htmlpress-callout.css"),
    asset!("components/htmlpress-card.css"),
    asset!("components/htmlpress-edit.css"),
    asset!("components/htmlpress-file-tree.css"),
    asset!("components/htmlpress-footer.css"),
    asset!("components/htmlpress-grid.css"),
    asset!("components/htmlpress-header.css"),
    asset!("components/htmlpress-icon.css"),
    asset!("components/htmlpress-links.css"),
    asset!("components/htmlpress-main.css"),
    asset!("components/htmlpress-mermaid.css"),
    asset!("components/htmlpress-mermaid.js"),
    asset!("components/htmlpress-nav-toggle.css"),
    asset!("components/htmlpress-nav-toggle.js"),
    asset!("components/htmlpress-nav.css"),
    asset!("components/htmlpress-page.css"),
    asset!("components/htmlpress-pager.css"),
    asset!("components/htmlpress-shell.css"),
    asset!("components/htmlpress-sidebar.css"),
    asset!("components/htmlpress-skip.css"),
    asset!("components/htmlpress-soundcloud.css"),
    asset!("components/htmlpress-soundcloud.js"),
    asset!("components/htmlpress-steps.css"),
    asset!("components/htmlpress-table.css"),
    asset!("components/htmlpress-table.js"),
    asset!("components/htmlpress-tabs.css"),
    asset!("components/htmlpress-tabs.js"),
    asset!("components/htmlpress-terminal.css"),
    asset!("components/htmlpress-theme-toggle.css"),
    asset!("components/htmlpress-theme-toggle.js"),
    asset!("components/htmlpress-toc.css"),
    asset!("components/htmlpress-toc.js"),
    asset!("components/htmlpress-toolbar.css"),
    asset!("components/htmlpress-toolbar.js"),
    asset!("components/htmlpress-youtube.css"),
    asset!("components/htmlpress-youtube.js"),
    asset!("components/htmlpress-zoom.css"),
    asset!("components/htmlpress-zoom.js"),
];

/// The stylesheets joined, in this order, into the global `theme.css`.
pub const THEME_STYLES: &[&str] = &[
    "styles/tokens.css",
    "styles/layout.css",
    "styles/content.css",
];

/// The layout elements the build fills; a project component may replace one by name.
pub const LAYOUT_ELEMENTS: &[&str] = &[
    "htmlpress-banner",
    "htmlpress-brand",
    "htmlpress-breadcrumbs",
    "htmlpress-content",
    "htmlpress-edit",
    "htmlpress-footer",
    "htmlpress-links",
    "htmlpress-nav",
    "htmlpress-pager",
    "htmlpress-toc",
    "htmlpress-toolbar",
];
