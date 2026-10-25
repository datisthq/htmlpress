use std::path::PathBuf;

use crate::actions::site::preview::preview_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct PreviewInput {
    /// The project folder holding htmlpress.config.yaml and the HTML pages.
    #[fireargs(argument, default = ".")]
    root: String,
    /// The port to serve on; 0 picks a free one.
    #[fireargs(short = 'p', default = 8080)]
    port: u16,
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
            preview_site(&options, input.port, |url| eprintln!("serving {url}"))?;
            Ok(PreviewOutput {})
        })
}
