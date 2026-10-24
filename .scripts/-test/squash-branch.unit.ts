import { execFileSync, spawnSync } from "node:child_process"
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs"
import { tmpdir } from "node:os"
import { join, resolve } from "node:path"
import { afterEach, beforeEach, describe, expect, it } from "vite-plus/test"

const SCRIPT = resolve(import.meta.dirname, "..", "squash-branch.ts")

let root = ""
let tree = ""

const runGit = (cwd: string, ...args: string[]): string =>
  execFileSync("git", ["-C", cwd, ...args], { encoding: "utf8" }).trim()

const runSquash = (cwd: string, ...args: string[]) =>
  spawnSync("node", [SCRIPT, ...args], { cwd, encoding: "utf8" })

const writeFile = (dir: string, path: string, text: string) => {
  writeFileSync(join(dir, path), text)
}

beforeEach(() => {
  root = mkdtempSync(join(tmpdir(), "squash-branch-"))
  tree = join(root, ".worktrees", "topic")
  runGit(root, "init", "-q", "-b", "main")
  runGit(root, "config", "user.email", "test@example.com")
  runGit(root, "config", "user.name", "test")
  // Windows runners default to autocrlf, which checks the worktree out with CRLF.
  runGit(root, "config", "core.autocrlf", "false")
  writeFile(root, "shared.txt", "base\n")
  writeFile(root, "gone.txt", "base\n")
  writeFile(root, "other.txt", "base\n")
  writeFile(root, ".gitignore", ".worktrees/\n")
  runGit(root, "add", ".")
  runGit(root, "commit", "-q", "-m", "chore: start")
  runGit(root, "worktree", "add", "-q", tree, "-b", "topic")
  writeFile(tree, "shared.txt", "topic\n")
  writeFile(tree, "added.txt", "topic\n")
  runGit(tree, "rm", "-q", "gone.txt")
  runGit(tree, "add", ".")
  runGit(tree, "commit", "-q", "-m", "wip one")
  writeFile(tree, "added.txt", "topic two\n")
  runGit(tree, "commit", "-q", "-am", "wip two")
})

afterEach(() => {
  rmSync(root, { recursive: true, force: true })
})

describe("pnpm squash", () => {
  it("squashes the branch onto main without touching another session's staging", () => {
    writeFile(root, "other.txt", "another session\n")
    runGit(root, "add", "other.txt")

    const result = runSquash(tree, "-m", "feat(topic): land the topic")

    expect(result.stderr).toBe("")
    expect(runGit(root, "log", "--format=%s", "main")).toBe(
      "feat(topic): land the topic\nchore: start",
    )
    expect(
      runGit(root, "show", "--name-only", "--format=", "HEAD").split("\n").sort(),
    ).toEqual(["added.txt", "gone.txt", "shared.txt"])
    expect(runGit(root, "status", "--porcelain")).toBe("M  other.txt")
    expect(readFileSync(join(root, "added.txt"), "utf8")).toBe("topic two\n")
  })

  it("refuses a branch that does not contain main", () => {
    writeFile(root, "other.txt", "moved on\n")
    runGit(root, "commit", "-q", "-am", "fix: move main")

    const result = runSquash(root, "topic", "-m", "feat: land")

    expect(result.status).toBe(1)
    expect(result.stderr).toContain("does not contain main")
  })

  it("refuses when the root has uncommitted edits to a landed path", () => {
    writeFile(root, "shared.txt", "someone else\n")

    const result = runSquash(root, "topic", "-m", "feat: land")

    expect(result.status).toBe(1)
    expect(result.stderr).toContain("shared.txt")
    expect(runGit(root, "log", "--format=%s", "main")).toBe("chore: start")
  })

  it("refuses a breaking or unconventional message", () => {
    expect(runSquash(tree, "-m", "feat!: land").status).toBe(1)
    expect(runSquash(tree, "-m", "feat: land", "-m", "BREAKING CHANGE: no").status).toBe(
      1,
    )
    expect(runSquash(tree, "-m", "land it").status).toBe(1)
  })
})
