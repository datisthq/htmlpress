use std::sync::Arc;

use crate::actions::config::load::load_config;
use crate::actions::site::build::build_site;
use crate::actions::site::serve::{Reload, add_base, answer_requests, open_server};
use crate::models::build::BuildOptions;

/// Build the project for production and serve the output, with no watching or reloading.
/// `host` and `port` default as for `serve_site`. Calls `on_ready` with the URLs once
/// listening, the local one first; runs until killed.
pub fn preview_site(
    options: &BuildOptions,
    host: Option<&str>,
    port: Option<u16>,
    on_ready: impl FnOnce(&[String]),
) -> crate::Result<()> {
    let options = BuildOptions {
        reload: false,
        ..options.clone()
    };
    build_site(&options)?;
    let (server, urls) = open_server(host, port)?;
    let base = load_config(&options.root)?.read_base();
    on_ready(&add_base(urls, &base));
    let state = Arc::new(Reload::default());
    answer_requests(&server, &options.output, &base, &state);
    Ok(())
}
