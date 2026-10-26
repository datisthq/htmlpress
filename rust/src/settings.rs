use crate::models::asset::Asset;

/// The project's config file, read from the project root.
pub const CONFIG_FILE: &str = "htmlpress.yaml";

/// The project's own folder: build output and escapes. Never scanned for pages.
pub const PROJECT_DIR: &str = ".htmlpress";

/// The build output, relative to the project root, unless `--out` names another.
pub const OUTPUT_DIR: &str = ".htmlpress/build";

/// Where builds keep what they fetched, such as GitHub releases; never published.
pub const CACHE_DIR: &str = ".htmlpress/cache";

/// Where the built-in theme and components are written in the output.
pub const ASSETS_DIR: &str = "_htmlpress";

/// The dev server's ports, tried in order when `--port` names none.
pub const PORTS: std::ops::RangeInclusive<u16> = 8000..=8009;

/// The dev server's long-poll endpoint the reload script waits on.
pub const RELOAD_PATH: &str = "/_htmlpress/reload";
/// The reload script, which the dev server answers even before a build wrote it.
pub const RELOAD_SCRIPT: &str = "/_htmlpress/reload.js";

/// Where a host looks for the page it serves on a missing URL; never listed in the
/// navigation or the sitemap.
pub const NOT_FOUND_PAGE: &str = "404.html";

/// The page a project without its own `404.html` gets; never escaped, since a project
/// replaces it with a page of the same name.
pub const NOT_FOUND: &str = include_str!("404.html");

/// The bundled UI strings, by language code; English holds every key.
pub const MESSAGES: [(&str, &str); 7] = [
    ("en", include_str!("messages/en.yaml")),
    ("de", include_str!("messages/de.yaml")),
    ("es", include_str!("messages/es.yaml")),
    ("fr", include_str!("messages/fr.yaml")),
    ("ja", include_str!("messages/ja.yaml")),
    ("ru", include_str!("messages/ru.yaml")),
    ("zh", include_str!("messages/zh.yaml")),
];

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
    asset!("favicon.svg"),
    asset!("styles/tokens.css"),
    asset!("styles/layout.css"),
    asset!("styles/content.css"),
    asset!("scripts/code.js"),
    asset!("scripts/hints.js"),
    asset!("scripts/keys.js"),
    asset!("components/aside/aside.css"),
    asset!("components/back-to-top/back-to-top.css"),
    asset!("components/back-to-top/back-to-top.js"),
    asset!("components/badge/badge.css"),
    asset!("components/banner/banner.css"),
    asset!("components/brand/brand.css"),
    asset!("components/breadcrumbs/breadcrumbs.css"),
    asset!("components/byline/byline.css"),
    asset!("components/button/button.css"),
    asset!("components/callout/callout.css"),
    asset!("components/card/card.css"),
    asset!("components/edit/edit.css"),
    asset!("components/file-tree/file-tree.css"),
    asset!("components/footer/footer.css"),
    asset!("components/grid/grid.css"),
    asset!("components/header/header.css"),
    asset!("components/icon/icon.css"),
    asset!("components/language/language.css"),
    asset!("components/language/language.js"),
    asset!("components/links/links.css"),
    asset!("components/links/links.js"),
    asset!("components/main/main.css"),
    asset!("components/math/math.css"),
    asset!("components/mermaid/mermaid.css"),
    asset!("components/mermaid/mermaid.js"),
    asset!("components/nav-toggle/nav-toggle.css"),
    asset!("components/nav-toggle/nav-toggle.js"),
    asset!("components/nav/nav.css"),
    asset!("components/page/page.css"),
    asset!("components/pager/pager.css"),
    asset!("components/shell/shell.css"),
    asset!("components/search/search.css"),
    asset!("components/search/search.js"),
    asset!("components/sidebar/sidebar.css"),
    asset!("components/sidebar/sidebar.js"),
    asset!("components/skip/skip.css"),
    asset!("components/soundcloud/soundcloud.css"),
    asset!("components/soundcloud/soundcloud.js"),
    asset!("components/steps/steps.css"),
    asset!("components/table/table.css"),
    asset!("components/table/table.js"),
    asset!("components/tabs/tabs.css"),
    asset!("components/tabs/tabs.js"),
    asset!("components/terminal/terminal.css"),
    asset!("components/theme-toggle/theme-toggle.css"),
    asset!("components/theme-toggle/theme-toggle.js"),
    asset!("components/toc/toc.css"),
    asset!("components/toc/toc.js"),
    asset!("components/toc-bar/toc-bar.css"),
    asset!("components/toc-bar/toc-bar.js"),
    asset!("components/toolbar/toolbar.css"),
    asset!("components/toolbar/toolbar.js"),
    asset!("components/youtube/youtube.css"),
    asset!("components/youtube/youtube.js"),
    asset!("components/zoom/zoom.css"),
    asset!("components/zoom/zoom.js"),
];

/// The stylesheets joined, in this order, into the global `theme.css`.
pub const THEME_STYLES: &[&str] = &[
    "styles/tokens.css",
    "styles/layout.css",
    "styles/content.css",
];
