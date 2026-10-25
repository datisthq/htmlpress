use std::sync::Arc;

use crate::actions::site::build::build_site;
use crate::actions::site::serve::{Reload, answer_requests, open_server, read_port};
use crate::models::build::BuildOptions;

/// Build the project for production and serve the output on `127.0.0.1:port`, with no
/// watching or reloading. Calls `on_ready` with the URL once listening; runs until killed.
pub fn preview_site(
    options: &BuildOptions,
    port: u16,
    on_ready: impl FnOnce(&str),
) -> crate::Result<()> {
    let options = BuildOptions {
        reload: false,
        ..options.clone()
    };
    build_site(&options)?;
    let server = open_server(port)?;
    on_ready(&format!("http://127.0.0.1:{}", read_port(&server, port)));
    answer_requests(&server, &options.output, &Arc::new(Reload::default()));
    Ok(())
}
