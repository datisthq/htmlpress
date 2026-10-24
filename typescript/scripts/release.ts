import { execFileSync } from "node:child_process"
import { readFileSync } from "node:fs"
import { join, resolve } from "node:path"
import { PACKAGE_NAME, PLATFORMS } from "../settings.ts"

// Publishes what `pnpm pack:platforms` laid out. Platform packages go first: the main
// package's optionalDependencies must resolve the moment it is public. A version already
// on the registry is skipped, so a rerun after a partial failure finishes the set.

const COMPILE = resolve(import.meta.dirname, "..", "compile")
const names = [
  ...Object.values(PLATFORMS).map(platform => platform.packageName),
  PACKAGE_NAME,
]

for (const name of names) {
  const folder = join(COMPILE, name)
  const { version } = JSON.parse(readFileSync(join(folder, "package.json"), "utf8"))
  if (isPublished(name, version)) {
    console.log(`skip ${name}@${version}: already published`)
    continue
  }
  execFileSync("npm", ["publish", folder, "--access", "public", "--provenance"], {
    stdio: "inherit",
  })
}

function isPublished(name: string, version: string) {
  try {
    execFileSync("npm", ["view", `${name}@${version}`, "version"], { stdio: "pipe" })
    return true
  } catch {
    return false
  }
}
