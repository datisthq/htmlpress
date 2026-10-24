import { execFileSync } from "node:child_process"
import { dirname } from "node:path"
import { parseArgs } from "node:util"

// The root checkout's index is shared by every session working there, so a land must
// never go through it: `merge --squash` + `commit` commits whatever anyone else staged.

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { message: { type: "string", short: "m", multiple: true } },
})

const runGit = (cwd: string, args: string[], input?: string): string =>
  execFileSync("git", ["-C", cwd, ...args], {
    encoding: "utf8",
    env: { ...process.env, GIT_LITERAL_PATHSPECS: "1" },
    input,
    stdio: ["pipe", "pipe", "pipe"],
  }).trim()

const failSquash = (reason: string): never => {
  console.error(`squash: ${reason}`)
  process.exit(1)
}

const listPaths = (output: string): string[] => output.split("\0").filter(Boolean)

const cwd = process.cwd()
const root = dirname(
  runGit(cwd, ["rev-parse", "--path-format=absolute", "--git-common-dir"]),
)
const branch = positionals[0] ?? runGit(cwd, ["branch", "--show-current"])
const message = (values.message ?? []).join("\n\n")

if (positionals.length > 1)
  failSquash('usage: pnpm squash [<branch>] -m "<type(scope): subject>"')
if (!branch || branch === "main")
  failSquash("name the branch to land, or run inside its worktree")
if (!/^[a-z]+(\([a-z0-9,-]+\))?: \S/.test(message)) {
  failSquash(
    `"${message.split("\n")[0]}" is not a Conventional Commit subject (type(scope): subject)`,
  )
}
if (/BREAKING[ -]CHANGE:/.test(message)) failSquash("never mark a commit breaking")
if (runGit(root, ["symbolic-ref", "-q", "HEAD"]) !== "refs/heads/main") {
  failSquash(`${root} is not on main`)
}

const base = runGit(root, ["rev-parse", "refs/heads/main"])
const tip = runGit(root, ["rev-parse", "--verify", `refs/heads/${branch}^{commit}`])
try {
  runGit(root, ["merge-base", "--is-ancestor", base, tip])
} catch {
  failSquash(
    `${branch} does not contain main — run \`git merge main\` in it, gate it, and rerun`,
  )
}

const landed = listPaths(
  runGit(root, ["diff", "-z", "--no-renames", "--name-only", base, tip]),
)
if (landed.length === 0) failSquash(`${branch} changes nothing on main`)

const dirty = new Set([
  ...listPaths(runGit(root, ["diff", "-z", "--no-renames", "--name-only", "HEAD"])),
  ...listPaths(runGit(root, ["ls-files", "-z", "--others", "--exclude-standard"])),
])
const clashes = landed.filter(path => dirty.has(path))
if (clashes.length > 0) {
  failSquash(
    `uncommitted edits in ${root} touch landed paths:\n  ${clashes.join("\n  ")}`,
  )
}

const commit = runGit(
  root,
  ["commit-tree", `${tip}^{tree}`, "-p", base, "-F", "-"],
  message,
)
try {
  runGit(root, ["update-ref", "-m", `squash: ${branch}`, "refs/heads/main", commit, base])
} catch {
  failSquash("main moved while landing — merge main into the branch again and rerun")
}

// HEAD now has the landed paths; refresh only those, leaving every other session's work alone.
runGit(
  root,
  [
    "restore",
    "--source=HEAD",
    "--staged",
    "--worktree",
    "--pathspec-from-file=-",
    "--pathspec-file-nul",
  ],
  landed.join("\0"),
)

console.log(
  `landed ${branch} on main as ${commit.slice(0, 9)} (${landed.length} paths, not pushed)`,
)
