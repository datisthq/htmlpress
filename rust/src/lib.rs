//! Htmlpress turns a folder of HTML pages into a site: every page gets one layout, one
//! theme and a navigation, with a live-reloading dev server. The binary is `htmlpress`;
//! this library is what it runs.

mod actions {
    pub(crate) mod asset {
        pub(crate) mod escape;
        pub(crate) mod list;
    }
    pub(crate) mod config {
        pub(crate) mod load;
    }
    pub(crate) mod page {
        pub(crate) mod read;
        pub(crate) mod render;
    }
    pub(crate) mod site {
        pub(crate) mod build;
        pub(crate) mod scan;
        pub(crate) mod serve;
    }
}
mod commands {
    pub(crate) mod build;
    pub(crate) mod escape;
    pub(crate) mod start;
}
mod models {
    pub(crate) mod asset;
    pub(crate) mod build;
    pub(crate) mod config;
    pub(crate) mod error;
    pub(crate) mod page;
    pub(crate) mod site;
}
mod program;
mod settings;

pub use actions::asset::escape::escape_assets;
pub use actions::asset::list::list_assets;
pub use actions::config::load::load_config;
pub use actions::page::read::read_page;
pub use actions::site::build::build_site;
pub use actions::site::scan::scan_site;
pub use actions::site::serve::serve_site;
pub use models::asset::{Asset, AssetStatus};
pub use models::build::{BuildOptions, BuildReport};
pub use models::config::{Config, Link};
pub use models::error::Error;
pub use models::page::{Heading, Page};
pub use models::site::Site;
pub use program::create_program;

/// A result whose error is [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
