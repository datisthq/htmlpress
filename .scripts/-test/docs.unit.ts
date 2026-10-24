import { existsSync, readFileSync, readdirSync, realpathSync } from "node:fs"
import { join, resolve } from "node:path"
import { describe, expect, it } from "vite-plus/test"

const ROOT = resolve(import.meta.dirname, "..", "..")
const REPO_SKILLS = join(ROOT, ".agents", "skills")
const SECTIONS = ["typescript", "python", "rust"]
const SHIPPED_SKILLS = [join(ROOT, ".skills")]

describe("documentation stays true", () => {
  it("wires .claude/skills to .agents/skills", () => {
    const linked = realpathSync(join(ROOT, ".claude", "skills"))
    expect(linked).toBe(realpathSync(REPO_SKILLS))
  })

  it("keeps exactly one AGENTS.md, at the root", () => {
    const stray: string[] = []
    const walk = (directory: string) => {
      for (const entry of readdirSync(directory, { withFileTypes: true })) {
        if (["node_modules", "target"].includes(entry.name)) continue
        if (entry.name.startsWith(".")) continue
        const path = join(directory, entry.name)
        if (entry.isDirectory()) walk(path)
        else if (entry.name === "AGENTS.md" && directory !== ROOT) {
          stray.push(path.slice(ROOT.length + 1))
        }
      }
    }
    walk(ROOT)
    expect(stray).toEqual([])
  })

  it("keeps the package READMEs' usage in step", () => {
    const usage = (name: string) => {
      const text = readFileSync(join(ROOT, name, "README.md"), "utf8")
      return text.slice(text.indexOf("## Usage"))
    }
    const drifted = SECTIONS.filter(name => usage(name) !== usage("rust"))
    expect(drifted).toEqual([])
  })

  it("lists the same platforms in the npm settings and the release matrix", () => {
    const settings = readFileSync(join(ROOT, "typescript", "settings.ts"), "utf8")
    const release = readFileSync(
      join(ROOT, ".github", "workflows", "release.yaml"),
      "utf8",
    )
    const pairs = (text: string, pattern: RegExp) =>
      [...text.matchAll(pattern)].map(match => `${match[1]} ${match[2]}`).toSorted()
    const declared = pairs(settings, /createPlatform\("([a-z0-9-]+)", "([a-z0-9_-]+)"\)/g)
    const built = pairs(release, /platform: ([a-z0-9-]+)\s+target: ([a-z0-9_-]+)/g)
    expect(declared.length).toBeGreaterThan(0)
    expect(built).toEqual(declared)
  })

  it("keeps code snippets within 100 columns", () => {
    const long: string[] = []
    for (const path of readSnippetDocs()) {
      const text = readFileSync(join(ROOT, path), "utf8")
      for (const [, code = ""] of text.matchAll(SNIPPET)) {
        for (const line of code.split("\n")) {
          if (line.length > 100) long.push(`${path}: ${line.trim().slice(0, 60)}`)
        }
      }
    }
    expect(long).toEqual([])
  })

  it("names only paths that exist", () => {
    const missing: string[] = []
    for (const doc of readDocs()) {
      for (const [, span = ""] of doc.text.matchAll(PATH_SPAN)) {
        if (!looksLikeRepoPath(span)) continue
        if (!existsSync(join(ROOT, span))) missing.push(`${doc.path}: ${span}`)
      }
    }
    expect(missing).toEqual([])
  })

  it("names only pnpm scripts that exist", () => {
    const scripts = readScripts(ROOT)
    const missing: string[] = []
    for (const doc of readDocs()) {
      const commands = [
        ...doc.text.matchAll(/`([^`\n]*pnpm [^`\n]*)`/g),
        ...doc.text.matchAll(/^\s*(pnpm .+)$/gm),
      ].map(match => match[1] ?? "")
      for (const [, section, script = ""] of commands
        .join("\n")
        .matchAll(/pnpm (?:-C ([a-z]+) )?([a-z][a-z0-9:-]*)/g)) {
        if (PNPM_BUILTINS.includes(script)) continue
        const known = section ? readScripts(join(ROOT, section)) : scripts
        if (!known.includes(script)) {
          missing.push(`${doc.path}: pnpm ${section ? `-C ${section} ` : ""}${script}`)
        }
      }
    }
    expect(missing).toEqual([])
  })

  it("gives every skill a name and a short description", () => {
    const problems: string[] = []
    for (const directory of [REPO_SKILLS, ...SHIPPED_SKILLS]) {
      for (const name of readdirSync(directory)) {
        const path = join(directory, name, "SKILL.md")
        if (!existsSync(path)) {
          problems.push(`${name}: no SKILL.md`)
          continue
        }
        const [, frontmatter = ""] = readFileSync(path, "utf8").split("---", 2)
        if (!new RegExp(`^name: ${name}$`, "m").test(frontmatter)) {
          problems.push(`${name}: frontmatter name must match the directory`)
        }
        // A description over the listing budget is cut to a bare name, and the skill is
        // then found only through the AGENTS.md table.
        const description = /^description: (.*)$/m.exec(frontmatter)?.[1] ?? ""
        if (!description) problems.push(`${name}: no description`)
        if (description.length > 250) {
          problems.push(`${name}: description is ${description.length} chars, keep ≤ 250`)
        }
      }
    }
    expect(problems).toEqual([])
  })

  it("lists every skill in the AGENTS.md tables, and only those", () => {
    const agents = readFileSync(join(ROOT, "AGENTS.md"), "utf8")
    const listed = [...agents.matchAll(/^\| `([a-z-]+)` +\|/gm)].map(match => match[1])
    const skills = [REPO_SKILLS, ...SHIPPED_SKILLS].flatMap(directory =>
      readdirSync(directory),
    )
    expect(listed.toSorted()).toEqual(skills.toSorted())
  })
})

const PATH_SPAN = /`(\.?[a-z0-9][a-z0-9._@/-]*\/[a-z0-9._@/-]+)`/gi

const PNPM_BUILTINS = ["install", "exec", "add", "remove", "dlx", "run", "publish"]

// Only spans rooted at a real top-level directory are checked: the docs also hold
// project-relative examples that are correct and resolve nowhere.
const WORKSPACE = new Set([
  ...readdirSync(ROOT, { withFileTypes: true })
    .filter(entry => entry.isDirectory() && !entry.name.startsWith("."))
    .map(entry => entry.name),
  ".agents",
  ".github",
  ".release",
  ".scripts",
  ".skills",
])

// Code only: text and shell blocks hold real CLI output, which is as wide as it is
const SNIPPET = /```(?:ts|typescript|js|rust|rs|html)\b[^\n]*\n([\s\S]*?)```/g

function readSnippetDocs() {
  const paths = ["README.md", ...SECTIONS.map(section => join(section, "README.md"))]
  for (const entry of readdirSync(join(ROOT, ".skills"), { recursive: true })) {
    if (String(entry).endsWith(".md")) paths.push(join(".skills", String(entry)))
  }
  return paths
}

function readScripts(directory: string) {
  const path = join(directory, "package.json")
  if (!existsSync(path)) return []
  return Object.keys(JSON.parse(readFileSync(path, "utf8")).scripts ?? {})
}

function readDocs() {
  const docs = [
    join(ROOT, "AGENTS.md"),
    join(ROOT, "README.md"),
    join(ROOT, "CONTRIBUTING.md"),
    ...readdirSync(REPO_SKILLS).map(name => join(REPO_SKILLS, name, "SKILL.md")),
  ]
  return docs
    .filter(path => existsSync(path))
    .map(path => ({
      path: path.slice(ROOT.length + 1),
      text: readFileSync(path, "utf8"),
    }))
}

function looksLikeRepoPath(span: string) {
  if (span.includes("*") || span.includes("<") || span.includes("://")) return false
  if (span.startsWith("@")) return false
  if (!WORKSPACE.has(span.split("/")[0] ?? "")) return false
  // Build output is gitignored, so a fresh checkout or CI would fail on it.
  const generated = ["build", "target", "compile"]
  if (span.split("/").some(part => generated.includes(part))) return false
  return (
    /\.(ts|json|md|css|js|html|yaml|yml|svg|png|rs|toml)$/.test(span) ||
    span.endsWith("/")
  )
}
