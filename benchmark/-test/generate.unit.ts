import { readFile } from "node:fs/promises"
import { join } from "node:path"
import { temporaryDirectoryTask } from "tempy"
import { describe, expect, it } from "vite-plus/test"
import { generateSite, listPages } from "../generate/site.ts"
import { docs } from "../profiles/docs.ts"

describe("listPages", () => {
  it("puts the home page first, then sections led by their index", () => {
    const pages = listPages(23, 10)
    expect(pages).toHaveLength(23)
    expect(pages[0]?.path).toBe("index")
    expect(pages[1]?.path).toBe("section-001/index")
    expect(pages[10]?.path).toBe("section-001/page-09")
    expect(pages[11]?.path).toBe("section-002/index")
    expect(pages.at(-1)?.path).toBe("section-003/page-01")
    expect(new Set(pages.map(page => page.token)).size).toBe(23)
  })
})

describe("generateSite", () => {
  it("writes the same pages for the same seed, each holding its own token", () =>
    temporaryDirectoryTask(async directory => {
      const first = await generateSite({
        profile: docs,
        size: 12,
        directory: join(directory, "a"),
      })
      const second = await generateSite({
        profile: docs,
        size: 12,
        directory: join(directory, "b"),
      })
      expect(first.bytes).toBe(second.bytes)
      for (const page of first.pages) {
        const body = await readFile(
          join(first.directory, "pages", `${page.path}.md`),
          "utf8",
        )
        expect(body).toContain(page.token)
        expect(body.match(/hpbench\d{5}/g)).toHaveLength(1)
      }
    }))

  it("reuses a cached site of the same size", () =>
    temporaryDirectoryTask(async directory => {
      const first = await generateSite({ profile: docs, size: 5, directory })
      const again = await generateSite({ profile: docs, size: 5, directory })
      expect(again).toEqual(first)
    }))
})
