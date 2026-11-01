import { readFile, readdir, writeFile } from "node:fs/promises"
import { join, resolve } from "node:path"
import { parseArgs } from "node:util"
import type { Result } from "./harness/measure.ts"
import { type Report, renderReport } from "./report/render.ts"
import { TOOLS } from "./tools/index.ts"

const { values, positionals } = parseArgs({
  allowPositionals: true,
  options: { out: { type: "string" } },
})

const RESULTS = join(import.meta.dirname, ".cache", "results")
const paths = positionals.length
  ? positionals.map(path => resolve(path))
  : (await readdir(RESULTS))
      .filter(file => file.endsWith(".json"))
      .sort()
      .map(file => join(RESULTS, file))

// later files win, so a rerun of one tool replaces its earlier result
const latest = new Map<string, Result>()
let report: Report | undefined
for (const path of paths) {
  const file: Report & { startedAt: string } = JSON.parse(await readFile(path, "utf8"))
  report = { measuredAt: file.startedAt, machine: file.machine, results: [] }
  for (const result of file.results)
    latest.set(`${result.case} ${result.pages} ${result.tool}`, result)
}
if (!report)
  throw new Error(`No results in ${RESULTS}; run pnpm -C benchmark measure first`)

const out = resolve(values.out ?? join(import.meta.dirname, ".cache", "report.html"))
// caveats describe the adapters, so the current ones win over those a result was stored with
const results = [...latest.values()].map(result => {
  const tool = TOOLS.find(item => item.name === result.tool)
  return tool
    ? {
        ...result,
        caveats: tool.caveats,
        isThemed: tool.isThemed,
      }
    : result
})
await writeFile(out, renderReport({ ...report, results }))
console.log(out)
