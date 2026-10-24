import { mkdirSync, rmSync, symlinkSync } from "node:fs"
import { join, resolve } from "node:path"

// Not `ln -s`: Git Bash on Windows copies the folder instead, so edits to a repo skill
// never reach Claude Code. A junction needs no admin rights; other platforms ignore it.

const ROOT = resolve(import.meta.dirname, "..")
const LINK = join(ROOT, ".claude", "skills")

mkdirSync(join(ROOT, ".claude"), { recursive: true })
rmSync(LINK, { recursive: true, force: true })
symlinkSync(join("..", ".agents", "skills"), LINK, "junction")
