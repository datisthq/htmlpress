use std::path::PathBuf;

use crate::actions::site::serve::serve_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct StartInput {
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
pub(crate) struct StartOutput {}

pub(crate) fn create_start_command() -> fireargs::Command<StartInput, StartOutput> {
    fireargs::command()
        .description("Serve the project with live reload, rebuilding on every change.")
        .handler(async |input: StartInput, _| {
            let mut options = BuildOptions::new(&input.root);
            if let Some(out) = input.out {
                options.output = PathBuf::from(out);
            }
            // stderr, not stdout: under --json stdout carries only the result
            serve_site(&options, input.port, |url| eprintln!("serving {url}"))?;
            Ok(StartOutput {})
        })
}
