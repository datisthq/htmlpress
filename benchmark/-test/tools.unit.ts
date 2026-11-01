import { readFile, rm } from "node:fs/promises"
import { join } from "node:path"
import { temporaryDirectoryTask } from "tempy"
import { describe, expect, it } from "vite-plus/test"
import { generateSite } from "../generate/site.ts"
import { docs } from "../profiles/docs.ts"
import { TOOLS } from "../tools/index.ts"

describe("TOOLS", () => {
  it("names every tool once", () => {
    const names = TOOLS.map(tool => tool.name)
    expect(new Set(names).size).toBe(names.length)
  })

  it("writes each page where its tool reads it, with its title", () =>
    temporaryDirectoryTask(async directory => {
      const site = await generateSite({ profile: docs, size: 3, directory })
      const read = (tool: string, path: string) =>
        readFile(join(tool, path), "utf8").catch(() => "")
      const written: string[] = []
      const project = async (name: string) => {
        const tool = TOOLS.find(item => item.name === name)
        if (!tool) throw new Error(name)
        const folder = await tool.createProject(site)
        written.push(folder)
        return folder
      }
      const zola = await project("zola")
      expect(await read(zola, "content/section-001/_index.md")).toContain(
        'title = "Section 1"',
      )
      const hugo = await project("hugo")
      expect(await read(hugo, "content/_index.md")).toContain('title: "Home"')
      const mdbook = await project("mdbook")
      expect(await read(mdbook, "src/SUMMARY.md")).toContain(
        "  - [Page 1.1](section-001/page-01.md)",
      )
      expect(await read(mdbook, "src/index.md")).toMatch(/^# Home\n/)
      for (const folder of written) await rm(folder, { recursive: true, force: true })
    }))
})
