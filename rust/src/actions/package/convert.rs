/// The package managers a `htmlpress-package-tabs` block shows, in tab order.
pub(crate) const MANAGERS: &[&str] = &["npm", "pnpm", "yarn", "bun"];

/// Rewrite each `npm` or `npx` line of `commands` for `manager`; other lines are kept.
pub(crate) fn convert_command(commands: &str, manager: &str) -> String {
    commands
        .lines()
        .map(|line| convert_line(line, manager))
        .collect::<Vec<_>>()
        .join("\n")
}

fn convert_line(line: &str, manager: &str) -> String {
    let indent = &line[..line.len() - line.trim_start().len()];
    let words: Vec<&str> = line.split_whitespace().collect();
    let converted = match words.as_slice() {
        ["npx", rest @ ..] => convert_exec(rest, manager),
        ["npm", "exec", rest @ ..] => convert_exec(rest, manager),
        ["npm", verb, rest @ ..] => convert_npm(verb, rest, manager),
        _ => return line.to_owned(),
    };
    format!("{indent}{converted}")
}

fn convert_exec(rest: &[&str], manager: &str) -> String {
    let rest = rest.join(" ");
    match manager {
        "pnpm" => format!("pnpm dlx {rest}"),
        "yarn" => format!("yarn dlx {rest}"),
        "bun" => format!("bunx {rest}"),
        _ => format!("npx {rest}"),
    }
}

fn convert_npm(verb: &str, rest: &[&str], manager: &str) -> String {
    if manager == "npm" {
        return join_words("npm", verb, rest);
    }
    let rest: Vec<&str> = rest
        .iter()
        .map(|word| convert_flag(word, manager))
        .collect();
    match verb {
        "install" | "i" | "add" if rest.iter().all(|word| word.starts_with('-')) => {
            join_words(manager, "install", &rest)
        }
        "install" | "i" | "add" => {
            let global = rest.contains(&"-g");
            match (manager, global) {
                ("yarn", true) => {
                    let rest: Vec<&str> =
                        rest.into_iter().filter(|word| *word != "-g").collect();
                    join_words("yarn global", "add", &rest)
                }
                _ => join_words(manager, "add", &rest),
            }
        }
        "uninstall" | "un" | "remove" | "rm" => join_words(manager, "remove", &rest),
        "ci" => join_words(manager, "install", &["--frozen-lockfile"]),
        "create" | "init" if !rest.is_empty() => join_words(manager, "create", &rest),
        // bun test is bun's own runner, not the package's test script
        "run" | "test" | "start" if manager == "bun" => match verb {
            "run" => join_words("bun", "run", &rest),
            _ => join_words("bun run", verb, &rest),
        },
        "run" if manager == "yarn" => join_words("yarn", "", &rest),
        _ => join_words(manager, verb, &rest),
    }
}

fn convert_flag<'a>(word: &'a str, manager: &str) -> &'a str {
    match (word, manager) {
        ("--save-dev", _) => "-D",
        ("--global", _) => "-g",
        ("--save" | "-S", _) => "",
        _ => word,
    }
}

fn join_words(manager: &str, verb: &str, rest: &[&str]) -> String {
    [manager, verb]
        .into_iter()
        .chain(rest.iter().copied())
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
#[path = "convert_unit.rs"]
mod unit;
