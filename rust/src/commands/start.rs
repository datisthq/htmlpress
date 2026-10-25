use std::path::PathBuf;

use crate::actions::site::serve::serve_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct StartInput {
    /// The project folder holding htmlpress.yaml and the HTML pages.
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
            let host = input.host.as_deref();
            serve_site(&options, host, input.port, |urls| {
                for (index, url) in urls.iter().enumerate() {
                    let label = if index == 0 { "serving" } else { "network" };
                    eprintln!("{label} {url}");
                }
            })?;
            Ok(StartOutput {})
        })
}
