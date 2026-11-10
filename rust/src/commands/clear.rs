use crate::actions::site::clear::clear_site;
use crate::models::build::BuildOptions;

#[fireargs::input]
pub(crate) struct ClearInput {
    /// The project folder holding htmlpress.yaml and the HTML pages.
    #[fireargs(argument, default = ".")]
    root: String,
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
        .description("Remove the project's .htmlpress/build output.")
        .handler(async |input: ClearInput, _| {
            let options = BuildOptions::new(&input.root);
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
