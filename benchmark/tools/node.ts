import { readFile } from "node:fs/promises"
import { join } from "node:path"
import { NODE_RIVALS, writeProject, writeYaml } from "./project.ts"
import type { Tool } from "./types.ts"

// Under the rivals' install, so each project resolves its pinned packages
const PROJECTS = join(NODE_RIVALS, ".cache", "projects")
const MODULES = join(NODE_RIVALS, "node_modules")

function readVersion(name: string) {
  return async () => {
    const text = await readFile(join(MODULES, name, "package.json"), "utf8")
    const version: unknown = JSON.parse(text).version
    return typeof version === "string" ? version : "unknown"
  }
}

function runNode(script: string, args: string[]) {
  return {
    file: process.execPath,
    args: [join(MODULES, script), ...args],
    env: { NODE_ENV: "production" },
  }
}

export const vitepress: Tool = {
  name: "vitepress",
  title: "VitePress",
  isThemed: true,
  caveats: [],
  getVersion: readVersion("vitepress"),
  createProject: site => {
    const sections = site.pages.filter(page => page.section && page.order === 0)
    const sidebar = sections.map(section => ({
      text: section.title,
      link: `/${section.path.replace(/index$/, "")}`,
      items: site.pages
        .filter(page => page.section === section.section && page.order > 0)
        .map(page => ({ text: page.title, link: `/${page.path}` })),
    }))
    const config = {
      title: "Benchmark",
      themeConfig: { sidebar, search: { provider: "local" } },
    }
    return writeProject({
      tool: "vitepress",
      site,
      root: PROJECTS,
      files: { ".vitepress/config.mjs": `export default ${JSON.stringify(config)}\n` },
      writePage: (page, body) => ({
        path: `${page.path}.md`,
        content: `${writeYaml({ title: page.title })}# ${page.title}\n\n${body}`,
      }),
    })
  },
  createCommand: directory => runNode("vitepress/bin/vitepress.js", ["build", directory]),
  listOutputs: directory => [
    join(directory, ".vitepress", "dist"),
    join(directory, ".vitepress", "cache"),
  ],
  readOutput: directory => join(directory, ".vitepress", "dist"),
}

export const starlight: Tool = {
  name: "starlight",
  title: "Astro Starlight",
  isThemed: true,
  caveats: [],
  getVersion: readVersion("@astrojs/starlight"),
  createProject: site =>
    writeProject({
      tool: "starlight",
      site,
      root: PROJECTS,
      files: {
        "astro.config.mjs": `import starlight from "@astrojs/starlight"\nimport { defineConfig } from "astro/config"\n\nexport default defineConfig({ integrations: [starlight({ title: "Benchmark" })] })\n`,
        "src/content.config.ts": `import { defineCollection } from "astro:content"\nimport { docsLoader } from "@astrojs/starlight/loaders"\nimport { docsSchema } from "@astrojs/starlight/schema"\n\nexport const collections = { docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }) }\n`,
      },
      writePage: (page, body) => ({
        path: `src/content/docs/${page.path}.md`,
        content: `${writeYaml({ title: page.title })}${body}`,
      }),
    }),
  createCommand: directory =>
    runNode("astro/bin/astro.mjs", ["build", "--root", directory]),
  listOutputs: directory =>
    ["dist", ".astro", "node_modules"].map(path => join(directory, path)),
  readOutput: directory => join(directory, "dist"),
}

export const docusaurus: Tool = {
  name: "docusaurus",
  title: "Docusaurus",
  isThemed: true,
  caveats: ["no search: the classic preset has none built in"],
  getVersion: readVersion("@docusaurus/core"),
  createProject: site =>
    writeProject({
      tool: "docusaurus",
      site,
      root: PROJECTS,
      files: {
        "docusaurus.config.mjs": `export default {
  title: "Benchmark",
  url: "https://example.com",
  baseUrl: "/",
  onBrokenLinks: "warn",
  presets: [["classic", { docs: { routeBasePath: "/" }, blog: false, pages: false }]],
}
`,
      },
      writePage: (page, body) => ({
        path: `docs/${page.path}.md`,
        content: `${writeYaml({ title: page.title, sidebar_position: page.order + 1 })}${body}`,
      }),
    }),
  createCommand: directory =>
    runNode("@docusaurus/core/bin/docusaurus.mjs", [
      "build",
      directory,
      "--out-dir",
      join(directory, "build"),
    ]),
  // Its webpack cache sits in the rivals' shared `node_modules/.cache`, out of the project:
  // left there, a build is warm, and a site of another size reads a stale one and crashes
  listOutputs: directory => [
    ...["build", ".docusaurus", "node_modules"].map(path => join(directory, path)),
    join(MODULES, ".cache"),
  ],
  readOutput: directory => join(directory, "build"),
}

export const eleventy: Tool = {
  name: "eleventy",
  title: "Eleventy",
  isThemed: false,
  caveats: [
    "minimal templates: no theme, navigation, table of contents or search",
    "code blocks left plain: highlighting is a plugin",
  ],
  getVersion: readVersion("@11ty/eleventy"),
  createProject: site =>
    writeProject({
      tool: "eleventy",
      site,
      root: PROJECTS,
      files: {
        "_includes/page.njk": `<!doctype html>\n<html lang="en"><head><meta charset="utf-8"><title>{{ title }}</title></head>\n<body><main><h1>{{ title }}</h1>{{ content | safe }}</main></body></html>\n`,
      },
      writePage: (page, body) => ({
        path: `${page.path}.md`,
        content: `${writeYaml({ title: page.title, layout: "page.njk" })}${body}`,
      }),
    }),
  createCommand: directory =>
    runNode("@11ty/eleventy/cmd.cjs", [
      `--input=${directory}`,
      `--output=${join(directory, "_site")}`,
      "--quiet",
    ]),
  listOutputs: directory => [join(directory, "_site")],
  readOutput: directory => join(directory, "_site"),
}

export const livemark: Tool = {
  name: "livemark",
  title: "Livemark",
  isThemed: true,
  caveats: [
    "the newest release past the workspace's seven-day quarantine",
    "each page's URL set by its `path` frontmatter: this release routes by file name alone",
  ],
  getVersion: readVersion("livemark"),
  createProject: site =>
    writeProject({
      tool: "livemark",
      site,
      root: PROJECTS,
      files: {
        "livemark.config.ts": `import { defineConfig } from "livemark"\n\nexport default defineConfig({ title: "Benchmark", include: "**/*.md" })\n`,
        // Livemark serves its prerender from ESM; the rivals' own package.json is not
        "package.json": `{ "private": true, "type": "module" }\n`,
      },
      writePage: (page, body) => ({
        path: `${page.path}.md`,
        content: `${writeYaml({ title: page.title, order: page.order + 1, path: writeRoute(page.path) })}${body}`,
      }),
    }),
  createCommand: () => runNode("livemark/build/entrypoints/main.js", ["build"]),
  listOutputs: directory => [join(directory, ".livemark"), join(directory, ".tanstack")],
  readOutput: directory => join(directory, ".livemark", "build", "client"),
}

// `section-001/page-01` → `/section-001/page-01/`, `index` → `/`
function writeRoute(path: string) {
  const folder = path.replace(/(^|\/)index$/, "")
  return folder ? `/${folder}/` : "/"
}
