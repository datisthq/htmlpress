use super::*;

fn convert_all(command: &str) -> Vec<String> {
    MANAGERS
        .iter()
        .map(|manager| convert_command(command, manager))
        .collect()
}

#[test]
fn converts_installs_and_dev_dependencies() {
    assert_eq!(
        convert_all("npm install htmlpress"),
        [
            "npm install htmlpress",
            "pnpm add htmlpress",
            "yarn add htmlpress",
            "bun add htmlpress"
        ]
    );
    assert_eq!(
        convert_all("npm i --save-dev vite"),
        [
            "npm i --save-dev vite",
            "pnpm add -D vite",
            "yarn add -D vite",
            "bun add -D vite"
        ]
    );
    assert_eq!(
        convert_all("npm install"),
        ["npm install", "pnpm install", "yarn install", "bun install"]
    );
}

#[test]
fn converts_global_installs_exec_and_scripts() {
    assert_eq!(
        convert_all("npm install -g htmlpress")[2],
        "yarn global add htmlpress"
    );
    assert_eq!(
        convert_all("npx gethtmlpress start docs"),
        [
            "npx gethtmlpress start docs",
            "pnpm dlx gethtmlpress start docs",
            "yarn dlx gethtmlpress start docs",
            "bunx gethtmlpress start docs"
        ]
    );
    assert_eq!(
        convert_all("npm run build"),
        [
            "npm run build",
            "pnpm run build",
            "yarn build",
            "bun run build"
        ]
    );
    assert_eq!(convert_all("npm test")[3], "bun run test");
}

#[test]
fn keeps_other_lines_and_indentation() {
    assert_eq!(
        convert_command("cd docs\n  npm ci", "pnpm"),
        "cd docs\n  pnpm install --frozen-lockfile"
    );
}
