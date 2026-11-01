import { mkdir, readFile, rm, writeFile } from "node:fs/promises"
import { dirname, join } from "node:path"
import type { Profile } from "../profiles/types.ts"
import { createRandom } from "./random.ts"

/** A generated site: every page's body under `directory/pages/`, as each tool reads it. */
export type Site = {
  profile: string
  size: number
  seed: number
  directory: string
  pages: SitePage[]
  /** The Markdown bodies' total size in bytes. */
  bytes: number
}

export type SitePage = {
  /** The page's path without `.md`: `index`, `section-03/index`, `section-03/page-07`. */
  path: string
  title: string
  /** A word in the page's first paragraph alone, which the built page must hold. */
  token: string
  /** The section's folder, or none for the home page. */
  section?: string
  /** Its place in its section, the index first. */
  order: number
}

// Bumped when the shape of a generated page changes, so a cached site is written again
const GENERATION = 1

/** Writes the site of `size` pages for `profile`, or reuses the one cached under `directory`. */
export async function generateSite(props: {
  profile: Profile
  size: number
  directory: string
  seed?: number
}) {
  const seed = props.seed ?? 1
  const directory = join(props.directory, `${props.profile.name}-${props.size}-${seed}`)
  const manifestPath = join(directory, "manifest.json")
  const cached = await readFile(manifestPath, "utf8")
    .then(text => JSON.parse(text))
    .catch(() => undefined)
  if (cached?.size === props.size && cached?.generation === GENERATION) {
    const { generation: _, ...site }: Site & { generation: number } = cached
    return site
  }

  await rm(directory, { recursive: true, force: true })
  const random = createRandom(seed)
  const pages = listPages(props.size, props.profile.sectionSize)
  let bytes = 0
  for (const page of pages) {
    const body = props.profile.writeBody({ random, token: page.token })
    bytes += Buffer.byteLength(body)
    const path = join(directory, "pages", `${page.path}.md`)
    await mkdir(dirname(path), { recursive: true })
    await writeFile(path, body)
  }
  const site: Site = {
    profile: props.profile.name,
    size: props.size,
    seed,
    directory,
    pages,
    bytes,
  }
  const manifest = { ...site, generation: GENERATION }
  await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)
  return site
}

/** The body of one page, as `generateSite` wrote it. */
export function readBody(site: Site, page: SitePage) {
  return readFile(join(site.directory, "pages", `${page.path}.md`), "utf8")
}

export function describeSite(site: Site) {
  const sections = new Set(site.pages.map(page => page.section).filter(Boolean)).size
  const kilobytes = Math.round(site.bytes / 1024)
  return `${site.size} pages in ${sections} sections, ${kilobytes} KB of Markdown`
}

/** The home page, then sections of `sectionSize` pages, each led by its index. */
export function listPages(size: number, sectionSize: number) {
  const pages: SitePage[] = [
    { path: "index", title: "Home", token: createToken(0), order: 0 },
  ]
  for (let index = 1; index < size; index++) {
    const number = Math.floor((index - 1) / sectionSize) + 1
    const order = (index - 1) % sectionSize
    const section = `section-${String(number).padStart(3, "0")}`
    const isIndex = order === 0
    pages.push({
      path: isIndex
        ? `${section}/index`
        : `${section}/page-${String(order).padStart(2, "0")}`,
      title: isIndex ? `Section ${number}` : `Page ${number}.${order}`,
      token: createToken(index),
      section,
      order,
    })
  }
  return pages
}

// A word no generated prose holds, so finding it proves its page was built
function createToken(index: number) {
  return `hpbench${String(index).padStart(5, "0")}`
}
