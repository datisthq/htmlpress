import { mkdir, writeFile } from "node:fs/promises"
import { join } from "node:path"
import { temporaryDirectoryTask } from "tempy"
import { describe, expect, it } from "vite-plus/test"
import { listPages } from "../generate/site.ts"
import { countBuiltPages, summarize } from "../harness/measure.ts"

describe("countBuiltPages", () => {
  it("counts the pages whose token some built HTML file holds", () =>
    temporaryDirectoryTask(async directory => {
      const pages = listPages(3, 10)
      const site = { profile: "docs", size: 3, seed: 1, directory, pages, bytes: 0 }
      await mkdir(join(directory, "out", "a"), { recursive: true })
      await writeFile(join(directory, "out", "index.html"), `<p>${pages[0]?.token}</p>`)
      await writeFile(
        join(directory, "out", "a", "index.html"),
        `<p>${pages[2]?.token}</p>`,
      )
      // A search index names every page, but it is not a built page
      await writeFile(join(directory, "out", "search.json"), JSON.stringify(pages))
      expect(await countBuiltPages(join(directory, "out"), site)).toBe(2)
    }))
})

describe("summarize", () => {
  it("takes the median of an even series as the mean of the middle two", () => {
    expect(summarize([4, 1, 3, 2])).toEqual({ median: 2.5, min: 1, max: 4 })
  })
})
