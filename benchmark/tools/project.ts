import { createHash } from "node:crypto"
import { mkdir, readFile, rm, writeFile } from "node:fs/promises"
import { basename, dirname, join } from "node:path"
import { type Site, readBody } from "../generate/site.ts"

export const BENCHMARK = join(import.meta.dirname, "..")
export const CACHE = join(BENCHMARK, ".cache")
/** Where the npm rivals are installed; their projects sit under it to resolve them. */
export const NODE_RIVALS = join(BENCHMARK, "rivals", "node")
export const PYTHON_RIVALS = join(BENCHMARK, "rivals", "python")

/**
 * Writes a tool's project for `site` into `<root>/<tool>/<site>`, once: `files` names its
 * own files, `writePage` each page's file and content.
 */
export async function writeProject(props: {
  tool: string
  site: Site
  root?: string
  files: Record<string, string>
  writePage: (
    page: Site["pages"][number],
    body: string,
  ) => { path: string; content: string }
}) {
  const root = props.root ?? join(CACHE, "projects")
  const directory = join(root, props.tool, basename(props.site.directory))
  const stamp = join(directory, ".benchmark")
  const pages = []
  for (const page of props.site.pages) {
    pages.push(props.writePage(page, await readBody(props.site, page)))
  }
  // Every byte the project holds, so a changed adapter writes its projects again
  const hash = createHash("sha256")
  for (const file of [
    ...Object.entries(props.files),
    ...pages.map(page => [page.path, page.content]),
  ]) {
    hash.update(`${file[0]}\0${file[1]}\0`)
  }
  const key = hash.digest("hex")
  if ((await readFile(stamp, "utf8").catch(() => "")) === key) return directory
  await rm(directory, { recursive: true, force: true })
  for (const [path, content] of Object.entries(props.files)) {
    await writeFileIn(join(directory, path), content)
  }
  for (const page of pages) await writeFileIn(join(directory, page.path), page.content)
  await writeFile(stamp, key)
  return directory
}

/** YAML frontmatter with each field a JSON string or number, valid YAML either way. */
export function writeYaml(fields: Record<string, string | number>) {
  const lines = Object.entries(fields).map(
    ([key, value]) => `${key}: ${JSON.stringify(value)}`,
  )
  return `---\n${lines.join("\n")}\n---\n\n`
}

async function writeFileIn(path: string, content: string) {
  await mkdir(dirname(path), { recursive: true })
  await writeFile(path, content)
}
