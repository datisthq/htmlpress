use crate::commands::build::create_build_command;
use crate::commands::escape::create_escape_command;
use crate::commands::start::create_start_command;

/// The `htmlpress` CLI: `build`, `start` and `escape`, with `--json` and `--llms` on every command.
pub fn create_program() -> fireargs::Program {
    fireargs::program()
        .name("htmlpress")
        .description("Turn a folder of HTML pages into a site.")
        .version(env!("CARGO_PKG_VERSION"))
        .command("build", create_build_command())
        .command("start", create_start_command())
        .command("escape", create_escape_command())
}
