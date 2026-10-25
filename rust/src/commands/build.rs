use std::path::PathBuf;

use crate::actions::site::build::build_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct BuildInput {
    /// The project folder holding htmlpress.yaml and the HTML pages.
    #[fireargs(argument, default = ".")]
    root: String,
    /// The output folder; defaults to .htmlpress/build inside the project.
    #[fireargs(short = 'o')]
    out: Option<String>,
}

#[fireargs::output]
pub(crate) struct BuildOutput {
    /// The folder the site was written to.
    output: String,
    /// How many HTML pages were rendered.
    pages: u64,
    /// How many other files were copied.
    files: u64,
}

pub(crate) fn create_build_command() -> fireargs::Command<BuildInput, BuildOutput> {
    fireargs::command()
        .description("Build the project's pages into a static site.")
        .handler(async |input: BuildInput, _| {
            let mut options = BuildOptions::new(&input.root);
            if let Some(out) = input.out {
                options.output = PathBuf::from(out);
            }
            let report = build_site(&options)?;
            Ok(BuildOutput {
                output: report.output.display().to_string(),
                pages: report.pages as u64,
                files: report.files as u64,
            })
        })
        .render(|output, _| {
            format!(
                "built {} pages and {} files into {}",
                output.pages, output.files, output.output
            )
        })
}
