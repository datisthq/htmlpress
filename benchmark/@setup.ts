import { execFile } from "node:child_process"
import { createHash } from "node:crypto"
import { access, mkdir, readFile, rm, writeFile } from "node:fs/promises"
import { join } from "node:path"
import { promisify } from "node:util"
import { BINARIES, RELEASES, THEME_RELEASES, THEMES } from "./tools/binaries.ts"
import { BENCHMARK, NODE_RIVALS, PYTHON_RIVALS } from "./tools/project.ts"

const run = promisify(execFile)

// The binary measured is this checkout's, built as it ships
console.log("htmlpress: cargo build --release")
await run("cargo", [
  "build",
  "--release",
  "--manifest-path",
  join(BENCHMARK, "..", "rust", "Cargo.toml"),
])

await mkdir(BINARIES, { recursive: true })
for (const [name, release] of Object.entries(RELEASES)) {
  const folder = join(BINARIES, name)
  const stamp = join(folder, ".version")
  const isFresh = await readFile(stamp, "utf8")
    .then(version => version === release.version)
    .catch(() => false)
  if (isFresh) continue
  console.log(`${name} ${release.version}: ${release.url}`)
  const archive = join(BINARIES, `${name}.tar.gz`)
  await writeFile(archive, await download(release))
  await rm(folder, { recursive: true, force: true })
  await mkdir(folder)
  await run("tar", ["-xzf", archive, "-C", folder])
  await writeFile(stamp, release.version)
}

await mkdir(THEMES, { recursive: true })
for (const [name, release] of Object.entries(THEME_RELEASES)) {
  const folder = join(THEMES, name)
  const stamp = join(THEMES, `${name}.version`)
  const isFresh = await access(folder)
    .then(() => run("cat", [stamp]))
    .then(result => result.stdout.trim() === release.version)
    .catch(() => false)
  if (isFresh) continue
  console.log(`${name} ${release.version}: ${release.url}`)
  const bytes = await download(release)
  const archive = join(THEMES, `${name}.tar.gz`)
  await writeFile(archive, bytes)
  await rm(folder, { recursive: true, force: true })
  await mkdir(folder)
  await run("tar", ["-xzf", archive, "-C", folder, "--strip-components=1"])
  await writeFile(stamp, release.version)
}

console.log("rivals/node: pnpm install --frozen-lockfile")
await run("pnpm", ["install", "--frozen-lockfile"], { cwd: NODE_RIVALS })
console.log("rivals/python: uv sync --frozen")
await run("uv", ["sync", "--frozen"], { cwd: PYTHON_RIVALS })

// An archive's bytes, refused unless they hash to the pinned SHA-256
async function download(release: { url: string; sha256: string }) {
  const response = await fetch(release.url)
  if (!response.ok) throw new Error(`${release.url}: ${response.status}`)
  const bytes = Buffer.from(await response.arrayBuffer())
  const digest = createHash("sha256").update(bytes).digest("hex")
  if (digest !== release.sha256) {
    throw new Error(`${release.url}: SHA-256 ${digest}, expected ${release.sha256}`)
  }
  return bytes
}
