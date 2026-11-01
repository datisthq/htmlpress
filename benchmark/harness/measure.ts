import { readFile, readdir, rm } from "node:fs/promises"
import { join } from "node:path"
import type { Site } from "../generate/site.ts"
import type { Tool } from "../tools/types.ts"
import { type Outcome, runCommand } from "./run.ts"

export type Result = {
  case: string
  tool: string
  title: string
  version: string
  isThemed: boolean
  profile: string
  pages: number
  bytes: number
  runs: number
  wallMs?: { median: number; min: number; max: number }
  /** Peak anonymous memory of the build's process tree, sampled. */
  peakMemoryMb?: number
  outcome: Outcome
  /** How the failed run ended, for example `code 1` or `signal 9`. */
  exit?: string
  /** How long the failed run lasted before it timed out, ran out of memory or crashed. */
  failedAfterMs?: number
  /** How many of the site's pages the output holds, by their tokens. */
  builtPages?: number
  /** False when the output misses a page: a build that skipped work is not a fast one. */
  correct: boolean
  caveats: string[]
  stderr?: string
}

/** Builds one site with one tool `warmup + runs` times, cold; the first failed run ends the series. */
export async function measureTool(props: {
  caseName: string
  tool: Tool
  site: Site
  runs: number
  warmup: number
  timeoutMs: number
  memoryMax?: string
}) {
  const tool = props.tool
  const directory = await tool.createProject(props.site)
  const timings: number[] = []
  const memories: number[] = []
  let outcome: Outcome = "ok"
  let builtPages: number | undefined
  let stderr: string | undefined
  let failedAfterMs: number | undefined
  let exit: string | undefined

  for (let index = 0; index < props.warmup + props.runs; index++) {
    for (const path of tool.listOutputs(directory))
      await rm(path, { recursive: true, force: true })
    const run = await runCommand(tool.createCommand(directory), {
      cwd: directory,
      timeoutMs: props.timeoutMs,
      memoryMax: props.memoryMax,
    })
    if (run.outcome !== "ok") {
      outcome = run.outcome
      failedAfterMs = run.wallMs
      exit = run.exit
      stderr = `${run.stdout}\n${run.stderr}`.slice(-2000)
      break
    }
    builtPages = await countBuiltPages(tool.readOutput(directory), props.site)
    if (index < props.warmup) continue
    timings.push(run.wallMs)
    if (run.peakMemoryMb !== undefined) memories.push(run.peakMemoryMb)
  }

  const result: Result = {
    case: props.caseName,
    tool: tool.name,
    title: tool.title,
    version: await tool.getVersion().catch(() => "unknown"),
    isThemed: tool.isThemed,
    profile: props.site.profile,
    pages: props.site.size,
    bytes: props.site.bytes,
    runs: timings.length,
    wallMs: timings.length ? summarize(timings) : undefined,
    peakMemoryMb: memories.length ? Math.max(...memories) : undefined,
    outcome,
    exit,
    failedAfterMs,
    builtPages,
    correct: outcome === "ok" && builtPages === props.site.size,
    caveats: tool.caveats,
    stderr,
  }
  return result
}

/** How many of the site's pages have their token in some built HTML file. */
export async function countBuiltPages(output: string, site: Site) {
  const found = new Set<string>()
  for (const path of await listHtmlFiles(output)) {
    const html = await readFile(path, "utf8")
    for (const match of html.matchAll(/hpbench\d{5}/g)) found.add(match[0])
  }
  return site.pages.filter(page => found.has(page.token)).length
}

async function listHtmlFiles(folder: string): Promise<string[]> {
  const entries = await readdir(folder, { withFileTypes: true }).catch(() => [])
  const files: string[] = []
  for (const entry of entries) {
    const path = join(folder, entry.name)
    if (entry.isDirectory()) files.push(...(await listHtmlFiles(path)))
    else if (entry.name.endsWith(".html")) files.push(path)
  }
  return files
}

export function summarize(values: number[]) {
  const sorted = values.toSorted((left, right) => left - right)
  const middle = Math.floor(sorted.length / 2)
  const median =
    sorted.length % 2
      ? (sorted[middle] ?? 0)
      : ((sorted[middle - 1] ?? 0) + (sorted[middle] ?? 0)) / 2
  return { median, min: sorted[0] ?? 0, max: sorted.at(-1) ?? 0 }
}
