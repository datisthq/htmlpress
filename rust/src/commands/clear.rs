use std::path::PathBuf;

use crate::actions::site::clear::clear_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct ClearInput {
    /// The project folder holding htmlpress.config.yaml and the HTML pages.
    #[fireargs(argument, default = ".")]
    root: String,
    /// The output folder; defaults to .htmlpress/build inside the project.
    #[fireargs(short = 'o')]
    out: Option<String>,
}

#[fireargs::output]
pub(crate) struct ClearOutput {
    /// The output folder.
    output: String,
    /// Whether there was an output to remove.
    removed: bool,
}

pub(crate) fn create_clear_command() -> fireargs::Command<ClearInput, ClearOutput> {
    fireargs::command()
        .description("Remove the project's build output.")
        .handler(async |input: ClearInput, _| {
            let mut options = BuildOptions::new(&input.root);
            if let Some(out) = input.out {
                options.output = PathBuf::from(out);
            }
            let removed = clear_site(&options)?;
            Ok(ClearOutput {
                output: options.output.display().to_string(),
                removed,
            })
        })
        .render(|output, _| match output.removed {
            true => format!("removed {}", output.output),
            false => format!("nothing to remove at {}", output.output),
        })
}
