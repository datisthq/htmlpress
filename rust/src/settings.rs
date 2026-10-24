/// The project's config file, read from the project root.
pub const CONFIG_FILE: &str = "htmlpress.toml";

/// The project's own folder: build output and escapes. Never scanned for pages.
pub const PROJECT_DIR: &str = ".htmlpress";

/// The build output, relative to the project root, unless `--out` names another.
pub const OUTPUT_DIR: &str = ".htmlpress/build";

/// A layout under this path replaces the built-in one.
pub const LAYOUT_ESCAPE: &str = ".htmlpress/layout.html";

/// Where the built-in theme and components are written in the output.
pub const ASSETS_DIR: &str = "_htmlpress";

/// The dev server's long-poll endpoint the reload script waits on.
pub const RELOAD_PATH: &str = "/_htmlpress/reload";

/// The built-in layout, theme, components and reload script.
pub const LAYOUT: &str = include_str!("assets/layout.html");
pub const THEME: &str = include_str!("assets/theme.css");
pub const COMPONENTS: &str = include_str!("assets/components.js");
pub const RELOAD: &str = include_str!("assets/reload.js");
