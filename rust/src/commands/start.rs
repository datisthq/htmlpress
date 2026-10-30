use std::path::PathBuf;

use crate::actions::browser::open::open_browser;
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
    /// Open the site in the browser once it listens, at the given path. Default: /.
    #[fireargs(preset = "/")]
    open: Option<String>,
    /// The output folder; defaults to .htmlpress/build inside the project.
    #[fireargs(short = 'o')]
    out: Option<String>,
}

#[fireargs::output]
pub(crate) struct StartOutput {
    /// The local URL, printed as one JSON line under --json once the server listens.
    url: String,
    /// The network URLs, with --host.
    network: Vec<String>,
}

pub(crate) fn create_start_command() -> fireargs::Command<StartInput, StartOutput> {
    fireargs::command()
        .description("Serve the project with live reload, rebuilding on every change.")
        .handler(async |input: StartInput, context| {
            let mut options = BuildOptions::new(&input.root);
            if let Some(out) = input.out {
                options.output = PathBuf::from(out);
            }
            let host = input.host.as_deref();
            serve_site(&options, host, input.port, |urls| {
                // The server never returns, so --json gets its result as soon as it listens
                if context.json {
                    let output = StartOutput {
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
                if let (Some(path), Some(url)) = (&input.open, urls.first()) {
                    open_browser(url, path);
                }
            })?;
            Ok(StartOutput {
                url: String::new(),
                network: vec![],
            })
        })
}
