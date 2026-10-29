use std::path::PathBuf;

use crate::actions::site::preview::preview_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct PreviewInput {
    /// The project folder holding htmlpress.yaml and the HTML pages.
    #[fireargs(argument, default = ".")]
    root: String,
    /// The port to serve on; 0 picks a free one. Default: the first free of 8000-8009.
    #[fireargs(short = 'p')]
    port: Option<u16>,
    /// The address to serve on. Default: every one, to reach it from the local network.
    #[fireargs(preset = "0.0.0.0", conflicts = no_host)]
    host: Option<String>,
    /// Serve on localhost only, out of the local network's reach.
    #[fireargs(default)]
    no_host: bool,
    /// The output folder; defaults to .htmlpress/build inside the project.
    #[fireargs(short = 'o')]
    out: Option<String>,
}

#[fireargs::output]
pub(crate) struct PreviewOutput {
    /// The local URL, printed as one JSON line under --json once the server listens.
    url: String,
    /// The network URLs; none with --no-host.
    network: Vec<String>,
}

pub(crate) fn create_preview_command() -> fireargs::Command<PreviewInput, PreviewOutput> {
    fireargs::command()
        .description(
            "Build the project for production and serve it, without live reload.",
        )
        .handler(async |input: PreviewInput, context| {
            let mut options = BuildOptions::new(&input.root);
            if let Some(out) = input.out {
                options.output = PathBuf::from(out);
            }
            let host = if input.no_host {
                Some("127.0.0.1")
            } else {
                input.host.as_deref()
            };
            preview_site(&options, host, input.port, |urls| {
                // The server never returns, so --json gets its result as soon as it listens
                if context.json {
                    let output = PreviewOutput {
                        url: urls.first().cloned().unwrap_or_default(),
                        network: urls.iter().skip(1).cloned().collect(),
                    };
                    if let Ok(line) = serde_json::to_string(&output) {
                        println!("{line}");
                    }
                }
                for (index, url) in urls.iter().enumerate() {
                    let label = if index == 0 { "serving" } else { "network" };
                    eprintln!("{label} {url}");
                }
            })?;
            Ok(PreviewOutput {
                url: String::new(),
                network: vec![],
            })
        })
}
