use std::path::PathBuf;

use crate::actions::site::preview::preview_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct PreviewInput {
    /// The project folder holding htmlpress.config.yaml and the HTML pages.
    #[fireargs(argument, default = ".")]
    root: String,
    /// The port to serve on; 0 picks a free one. Default: the first free of 8000-8009.
    #[fireargs(short = 'p')]
    port: Option<u16>,
    /// The address to serve on; alone, every one, to reach it from the local network.
    /// Default: localhost only.
    #[fireargs(preset = "0.0.0.0")]
    host: Option<String>,
    /// The output folder; defaults to .htmlpress/build inside the project.
    #[fireargs(short = 'o')]
    out: Option<String>,
}

#[fireargs::output]
pub(crate) struct PreviewOutput {}

pub(crate) fn create_preview_command() -> fireargs::Command<PreviewInput, PreviewOutput> {
    fireargs::command()
        .description(
            "Build the project for production and serve it, without live reload.",
        )
        .handler(async |input: PreviewInput, _| {
            let mut options = BuildOptions::new(&input.root);
            if let Some(out) = input.out {
                options.output = PathBuf::from(out);
            }
            // stderr, not stdout: under --json stdout carries only the result
            let host = input.host.as_deref();
            preview_site(&options, host, input.port, |urls| {
                for (index, url) in urls.iter().enumerate() {
                    let label = if index == 0 { "serving" } else { "network" };
                    eprintln!("{label} {url}");
                }
            })?;
            Ok(PreviewOutput {})
        })
}
