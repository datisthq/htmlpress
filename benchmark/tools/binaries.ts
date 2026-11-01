import { symlink } from "node:fs/promises"
import { join } from "node:path"
import type { SitePage } from "../generate/site.ts"
import { CACHE, writeProject, writeYaml } from "./project.ts"
import type { Tool } from "./types.ts"

/** Where `setup:tools` unpacks each binary rival. */
export const BINARIES = join(CACHE, "bin")

/** Each binary rival's release for Linux x86-64, with the SHA-256 its project publishes;
 * `setup:tools` unpacks each into a folder of its own, as two Zola releases share a name. */
export const RELEASES = {
  hugo: {
    version: "0.168.0",
    url: "https://github.com/gohugoio/hugo/releases/download/v0.168.0/hugo_0.168.0_linux-amd64.tar.gz",
    sha256: "fada83ad56bb27ed4cb7dd81590303bdac16140f944ac4e66fc248117e2d5594",
  },
  zola: {
    version: "0.23.6",
    url: "https://github.com/getzola/zola/releases/download/v0.23.6/zola-v0.23.6-x86_64-unknown-linux-gnu.tar.gz",
    sha256: "8f5132b3522412d04e395e0b25f6d68613ad272a873e54a2b3ebf664873024a4",
  },
  // The newest Zola before Tera 2, whose template syntax neither theme tried speaks yet
  "zola-tera1": {
    version: "0.22.1",
    url: "https://github.com/getzola/zola/releases/download/v0.22.1/zola-v0.22.1-x86_64-unknown-linux-gnu.tar.gz",
    sha256: "0ca09aa40376aaa9ddfb512ff9ad963262ef95edb0d0f2d5ec6961b6f5cf22ef",
  },
  mdbook: {
    version: "0.5.4",
    url: "https://github.com/rust-lang/mdBook/releases/download/v0.5.4/mdbook-v0.5.4-x86_64-unknown-linux-gnu.tar.gz",
    sha256: "3f28de05dafca9d0f2eab99c662116b0e37b89b1d96a08f8f430b9eeae958cd7",
  },
}

/** Hugo themes `setup:tools` unpacks under `THEMES`, pinned by the SHA-256 of their tag's
 * source archive, which their project does not publish: computed when pinned. */
export const THEMES = join(CACHE, "themes")
export const THEME_RELEASES = {
  book: {
    version: "4d4c44d",
    url: "https://github.com/getzola/book/archive/4d4c44d8fe36b6ed66844981b774a65260acb960.tar.gz",
    sha256: "1b29640d897eac441683799dc3025809bede4ee480dfcb62fb7b5b0e40cdbc16",
  },
  hextra: {
    version: "0.13.0",
    url: "https://github.com/imfing/hextra/archive/refs/tags/v0.13.0.tar.gz",
    sha256: "603dc5576aa1af84531e303ff376b0bff2213ee0391727a514e0d3bdd26ad006",
  },
}

/** A binary rival's executable, in the folder `setup:tools` unpacked its release into. */
export function findBinary(release: keyof typeof RELEASES, name: string) {
  return join(BINARIES, release, name)
}

const BARE = "minimal templates: no theme, navigation, table of contents or search"

// A section's index page is the folder's `_index.md` in both Hugo and Zola: an `index.md`
// would make Hugo treat the folder's other pages as resources of one page
function writeSectionPath(page: SitePage) {
  return page.path.endsWith("index") ? page.path.replace(/index$/, "_index") : page.path
}

const HUGO_LAYOUT = `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>{{ .Title }}</title></head>
<body><main><h1>{{ .Title }}</h1>{{ .Content }}{{ block "main" . }}{{ end }}</main></body></html>
`

export const hugo: Tool = {
  name: "hugo",
  title: "Hugo",
  isThemed: false,
  caveats: [BARE],
  getVersion: async () => RELEASES.hugo.version,
  createProject: site =>
    writeProject({
      tool: "hugo",
      site,
      files: {
        "hugo.toml": `baseURL = "https://example.com/"\ntitle = "Benchmark"\ndisableKinds = ["taxonomy", "term", "rss", "sitemap"]\n`,
        "layouts/baseof.html": HUGO_LAYOUT,
        "layouts/page.html": `{{ define "main" }}{{ end }}\n`,
        "layouts/section.html": `{{ define "main" }}<ul>{{ range .Pages }}<li><a href="{{ .RelPermalink }}">{{ .Title }}</a></li>{{ end }}</ul>{{ end }}\n`,
        "layouts/home.html": `{{ define "main" }}<ul>{{ range .Sections }}<li><a href="{{ .RelPermalink }}">{{ .Title }}</a></li>{{ end }}</ul>{{ end }}\n`,
      },
      writePage: (page, body) => ({
        path: `content/${writeSectionPath(page)}.md`,
        content: `${writeYaml({ title: page.title, weight: page.order + 1 })}${body}`,
      }),
    }),
  createCommand: directory => ({
    file: findBinary("hugo", "hugo"),
    args: [
      "--source",
      directory,
      "--cacheDir",
      join(directory, ".hugo-cache"),
      "--quiet",
    ],
  }),
  listOutputs: directory =>
    ["public", "resources", ".hugo-cache", ".hugo_build.lock"].map(path =>
      join(directory, path),
    ),
  readOutput: directory => join(directory, "public"),
}

export const hextra: Tool = {
  name: "hugo-hextra",
  title: "Hugo + Hextra",
  isThemed: true,
  caveats: ["Hextra's documentation theme, its search index built with the site"],
  getVersion: async () => `${RELEASES.hugo.version} + ${THEME_RELEASES.hextra.version}`,
  createProject: site =>
    writeProject({
      tool: "hugo-hextra",
      site,
      files: {
        "hugo.toml": `baseURL = "https://example.com/"
title = "Benchmark"
theme = "hextra"
themesDir = ${JSON.stringify(THEMES)}
disableKinds = ["taxonomy", "term", "rss", "sitemap"]
`,
      },
      // The home page's cascade makes every page a docs page, with Hextra's sidebar
      writePage: (page, body) => {
        const cascade = page.path === "index" ? "cascade:\n  type: docs\n" : ""
        const header = writeYaml({ title: page.title, weight: page.order + 1 })
        return {
          path: `content/${writeSectionPath(page)}.md`,
          content: header.replace(/---\n\n$/, `${cascade}---\n\n`) + body,
        }
      },
    }),
  createCommand: hugo.createCommand,
  listOutputs: hugo.listOutputs,
  readOutput: hugo.readOutput,
}

const ZOLA_LAYOUT = (main: string) => `<!doctype html>
<html lang="en"><head><meta charset="utf-8"><title>{{ ${main}.title }}</title></head>
<body><main><h1>{{ ${main}.title }}</h1>{{ ${main}.content | safe }}</main></body></html>
`

export const zola: Tool = {
  name: "zola",
  title: "Zola",
  isThemed: false,
  caveats: [BARE],
  getVersion: async () => RELEASES.zola.version,
  createProject: site =>
    writeProject({
      tool: "zola",
      site,
      files: {
        "config.toml": `base_url = "https://example.com"\ntitle = "Benchmark"\n\n[markdown.highlighting]\ntheme = "github-light"\n`,
        "templates/index.html": ZOLA_LAYOUT("section"),
        "templates/section.html": ZOLA_LAYOUT("section"),
        "templates/page.html": ZOLA_LAYOUT("page"),
      },
      writePage: (page, body) => ({
        path: `content/${writeSectionPath(page)}.md`,
        content: `+++\ntitle = ${JSON.stringify(page.title)}\nweight = ${page.order + 1}\n+++\n\n${body}`,
      }),
    }),
  createCommand: directory => ({
    file: findBinary("zola", "zola"),
    args: [
      "--root",
      directory,
      "build",
      "--output-dir",
      join(directory, "public"),
      "--force",
    ],
  }),
  listOutputs: directory => [join(directory, "public")],
  readOutput: directory => join(directory, "public"),
}

export const zolaBook: Tool = {
  name: "zola-book",
  title: "Zola + book",
  isThemed: true,
  caveats: [
    "Zola 0.22.1, the newest before Tera 2: neither theme tried (book, AdiDoks) builds on 0.23",
    "the book theme of Zola's own organization, its search index built with the site",
    "its home template overridden to show the home page's text, which the theme leaves out",
  ],
  getVersion: async () =>
    `${RELEASES["zola-tera1"].version} + book ${THEME_RELEASES.book.version}`,
  // Zola reads themes only from the project's own `themes/`, so it links the downloaded ones
  createProject: async site => {
    const directory = await writeProject({
      tool: "zola-book",
      site,
      files: {
        "config.toml": `base_url = "https://example.com"\ntitle = "Benchmark"\ntheme = "book"\nbuild_search_index = true\n\n[markdown.highlighting]\ntheme = "github-light"\n`,
        "templates/index.html": `{% extends "book/templates/index.html" %}\n{% block content %}{{ section.content | safe }}{% endblock content %}\n`,
      },
      writePage: (page, body) => ({
        path: `content/${writeSectionPath(page)}.md`,
        content: `+++\ntitle = ${JSON.stringify(page.title)}\nweight = ${page.order + 1}\n+++\n\n${body}`,
      }),
    })
    await symlink(THEMES, join(directory, "themes")).catch(() => undefined)
    return directory
  },
  createCommand: directory => ({
    file: findBinary("zola-tera1", "zola"),
    args: [
      "--root",
      directory,
      "build",
      "--output-dir",
      join(directory, "public"),
      "--force",
    ],
  }),
  listOutputs: directory => [join(directory, "public")],
  readOutput: directory => join(directory, "public"),
}

export const mdbook: Tool = {
  name: "mdbook",
  title: "mdBook",
  isThemed: true,
  caveats: ["code colored in the browser by highlight.js, not at build time"],
  getVersion: async () => RELEASES.mdbook.version,
  createProject: site => {
    const summary = site.pages.map(page => {
      const indent = !page.section ? "" : page.order === 0 ? "- " : "  - "
      const line = `[${page.title}](${page.path}.md)`
      return indent ? `${indent}${line}` : line
    })
    return writeProject({
      tool: "mdbook",
      site,
      files: {
        "book.toml": `[book]\ntitle = "Benchmark"\n`,
        "src/SUMMARY.md": `# Summary\n\n${summary.join("\n")}\n`,
      },
      writePage: (page, body) => ({
        path: `src/${page.path}.md`,
        content: `# ${page.title}\n\n${body}`,
      }),
    })
  },
  createCommand: directory => ({
    file: findBinary("mdbook", "mdbook"),
    args: ["build", directory, "--dest-dir", join(directory, "book")],
  }),
  listOutputs: directory => [join(directory, "book")],
  readOutput: directory => join(directory, "book"),
}
