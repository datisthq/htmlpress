import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs"
import { join, resolve } from "node:path"
import { parseArgs } from "node:util"
import { PACKAGE_NAME, PLATFORMS } from "../settings.ts"

// Lays out every package `pnpm release` publishes under compile/: one per platform with
// its binary, and the main package whose optionalDependencies pin them at this version.
// `--binaries` holds `<platform>/<binary>`, e.g. `linux-x64/htmlpress`.

const { values } = parseArgs({ options: { binaries: { type: "string" } } })
if (!values.binaries) throw new Error('usage: pnpm pack:platforms --binaries "<dir>"')

const ROOT = resolve(import.meta.dirname, "..")
const COMPILE = join(ROOT, "compile")
const BINARIES = resolve(values.binaries)
const manifest = JSON.parse(readFileSync(join(ROOT, "package.json"), "utf8"))
const version: string = manifest.version

rmSync(COMPILE, { recursive: true, force: true })

const missing = Object.entries(PLATFORMS)
  .map(([key, platform]) => join(BINARIES, key, platform.binary))
  .filter(path => !existsSync(path))
if (missing.length > 0) throw new Error(`missing binaries:\n  ${missing.join("\n  ")}`)

for (const [key, platform] of Object.entries(PLATFORMS)) {
  const [os, cpu] = key.split("-")
  const target = join(COMPILE, platform.packageName)
  mkdirSync(join(target, "bin"), { recursive: true })
  const binary = join(target, "bin", platform.binary)
  cpSync(join(BINARIES, key, platform.binary), binary)
  chmodSync(binary, 0o755)
  cpSync(join(ROOT, "LICENSE.md"), join(target, "LICENSE.md"))
  writeJson(join(target, "package.json"), {
    name: platform.packageName,
    version,
    description: `The ${key} binary of ${PACKAGE_NAME}`,
    license: manifest.license,
    repository: manifest.repository,
    os: [os],
    cpu: [cpu],
    files: ["bin"],
  })
}

const main = join(COMPILE, PACKAGE_NAME)
mkdirSync(main, { recursive: true })
cpSync(join(ROOT, "build"), join(main, "build"), { recursive: true })
for (const file of ["README.md", "LICENSE.md"]) cpSync(join(ROOT, file), join(main, file))
const { scripts, devDependencies, ...published } = manifest
writeJson(join(main, "package.json"), {
  ...published,
  optionalDependencies: Object.fromEntries(
    Object.values(PLATFORMS).map(platform => [platform.packageName, version]),
  ),
})

console.log(
  `packed ${Object.keys(PLATFORMS).length + 1} packages at ${version} into ${COMPILE}`,
)

function writeJson(path: string, value: object) {
  writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`)
}
