import type { Result } from "../harness/measure.ts"

export type Machine = {
  cpu: string
  cpus: number
  memory: string
  os: string
  memoryMax?: string
}

export type Report = {
  measuredAt: string
  machine: Machine
  results: Result[]
}

/** The page `show:report` writes: per case and size, time and memory bars, then each tool. */
export function renderReport(report: Report) {
  const groups = groupResults(report.results)
  const rivals = new Set(report.results.map(result => result.tool)).size - 1
  return `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Site Build Benchmark</title>
<style>${STYLE}</style>
</head>
<body>
<main>
<header>
  <p class="eyebrow">htmlpress benchmark</p>
  <h1>How fast does a documentation site build?</h1>
  <p class="lede">htmlpress against ${rivals} other site generators, each building the same
  generated Markdown site from cold. Measured ${escapeHtml(report.measuredAt.slice(0, 10))} on
  ${escapeHtml(describeMachine(report.machine))}.</p>
</header>
${groups.map(renderGroup).join("\n")}
${renderTools(report.results)}
${renderMethod()}
</main>
</body>
</html>
`
}

type Group = {
  caseName: string
  pages: number
  bytes: number
  results: Result[]
}

function groupResults(results: Result[]) {
  const groups = new Map<string, Group>()
  for (const result of results) {
    const key = `${result.case} ${result.pages}`
    const group = groups.get(key) ?? {
      caseName: result.case,
      pages: result.pages,
      bytes: result.bytes,
      results: [],
    }
    group.results.push(result)
    groups.set(key, group)
  }
  return [...groups.values()].sort(
    (left, right) =>
      left.caseName.localeCompare(right.caseName) || left.pages - right.pages,
  )
}

function renderGroup(group: Group) {
  const results = group.results.toSorted(compareResults)
  const maxTime = Math.max(...results.map(result => result.wallMs?.median ?? 0), 1)
  const maxMemory = Math.max(...results.map(result => result.peakMemoryMb ?? 0), 1)
  return `<section>
  <h2>${escapeHtml(group.caseName)} · ${group.pages} pages</h2>
  <p class="caption">${Math.round(group.bytes / 1024)} KB of Markdown · median of ${results[0]?.runs || 1} cold build(s) · shorter is better</p>
  <div class="legend"><span><i class="swatch htmlpress"></i>htmlpress</span><span><i class="swatch rival"></i>Documentation sites</span><span><i class="swatch bare"></i>Bare templates</span></div>
  <div class="charts">
    ${renderBars("Build time", results, result => result.wallMs?.median, maxTime, formatTime)}
    ${renderBars("Peak memory", results, result => result.peakMemoryMb, maxMemory, formatMemoryValue)}
  </div>
  <div class="table-wrap"><table>
    <thead><tr><th>Tool</th><th>Status</th><th class="num">Time</th><th class="num">Range</th><th class="num">Memory</th><th class="num">Pages</th></tr></thead>
    <tbody>${results
      .map(
        result =>
          `<tr><td>${escapeHtml(result.title)}</td><td>${renderStatus(result)}</td><td class="num">${escapeHtml(formatOutcome(result))}</td><td class="num">${escapeHtml(formatRange(result))}</td><td class="num">${escapeHtml(formatMemory(result))}</td><td class="num">${result.builtPages ?? "–"} of ${result.pages}</td></tr>`,
      )
      .join("")}</tbody>
  </table></div>
</section>`
}

function renderBars(
  title: string,
  results: Result[],
  measure: (result: Result) => number | undefined,
  max: number,
  format: (value: number) => string,
) {
  const rows = results.map(result => {
    const value = measure(result)
    const kind = readKind(result)
    const bar =
      value === undefined
        ? `<span class="missing">${escapeHtml(formatOutcome(result))}</span>`
        : `<span class="bar ${kind}" style="width:${Math.max((value / max) * 100, 0.5).toFixed(1)}%"></span><span class="value">${escapeHtml(format(value))}</span>`
    return `<div class="row" data-tip="${escapeHtml(`${result.title} ${result.version}: ${value === undefined ? formatOutcome(result) : format(value)}`)}"><span class="name">${escapeHtml(result.title)}</span><span class="track">${bar}</span></div>`
  })
  return `<figure><figcaption>${title}</figcaption>${rows.join("")}</figure>`
}

function renderStatus(result: Result) {
  if (result.outcome === "oom")
    return `<span class="chip critical">✕ out of memory</span>`
  if (result.outcome === "timeout")
    return `<span class="chip critical">✕ timed out</span>`
  if (result.outcome === "crash") return `<span class="chip critical">✕ crashed</span>`
  if (!result.correct) return `<span class="chip serious">≠ pages missing</span>`
  return `<span class="chip good">✓ every page</span>`
}

function renderTools(results: Result[]) {
  const tools = new Map(results.map(result => [result.tool, result]))
  return `<section>
  <h2>What each tool builds</h2>
  <p class="caption">Every tool builds the same pages with its own defaults. Documentation sites
  add navigation, a table of contents, search and a styled theme; bare templates only wrap each
  page. Differences from htmlpress:</p>
  <div class="table-wrap"><table>
    <thead><tr><th>Tool</th><th>Version</th><th>Kind</th><th>Caveats</th></tr></thead>
    <tbody>${[...tools.values()]
      .toSorted((left, right) => Number(right.isThemed) - Number(left.isThemed))
      .map(
        result =>
          `<tr><td>${escapeHtml(result.title)}</td><td>${escapeHtml(result.version)}</td><td>${result.isThemed ? "documentation site" : "bare templates"}</td><td>${result.caveats.map(escapeHtml).join("<br>") || "–"}</td></tr>`,
      )
      .join("")}</tbody>
  </table></div>
</section>`
}

function renderMethod() {
  return `<section>
  <h2>Method</h2>
  <ul>
    <li>The site is synthetic and seeded, so every tool reads the same Markdown: a home page
    and sections of ten pages, each with headings, prose, lists, tables and code blocks in six
    languages. Each tool's adapter writes it into the layout and frontmatter the tool expects.</li>
    <li>Every build is cold: the tool's output and caches are removed before each run, and the
    first run is a warm-up left out of the median. Time is the whole process, start-up
    included, as a build in CI or on save would take it.</li>
    <li>A build counts only when every page reaches the output: each page carries a word no
    other page holds, and the built HTML must hold every one.</li>
    <li>Memory is the peak anonymous memory of the build's processes, sampled every 50 ms.</li>
    <li>Reproduce: <code>pnpm -C benchmark setup:tools</code>, then
    <code>pnpm -C benchmark measure</code> and <code>pnpm -C benchmark show:report</code>.</li>
  </ul>
</section>`
}

function compareResults(left: Result, right: Result) {
  const leftTime = left.wallMs?.median ?? Number.POSITIVE_INFINITY
  const rightTime = right.wallMs?.median ?? Number.POSITIVE_INFINITY
  return leftTime - rightTime || left.title.localeCompare(right.title)
}

function readKind(result: Result) {
  if (result.tool === "htmlpress") return "htmlpress"
  return result.isThemed ? "rival" : "bare"
}

function formatRange(result: Result) {
  if (!result.wallMs) return "–"
  return `${formatTime(result.wallMs.min)} – ${formatTime(result.wallMs.max)}`
}

function formatOutcome(result: Result) {
  if (result.wallMs) return formatTime(result.wallMs.median)
  const after = result.failedAfterMs ? ` after ${formatTime(result.failedAfterMs)}` : ""
  if (result.outcome === "oom") return `out of memory${after}`
  if (result.outcome === "timeout") return `timed out${after}`
  return `crashed${after}`
}

function formatMemory(result: Result) {
  return result.peakMemoryMb === undefined ? "–" : formatMemoryValue(result.peakMemoryMb)
}

export function formatMemoryValue(megabytes: number) {
  return megabytes >= 1024
    ? `${(megabytes / 1024).toFixed(1)} GB`
    : `${Math.round(megabytes)} MB`
}

export function formatTime(milliseconds: number) {
  const seconds = milliseconds / 1000
  if (seconds < 60) return `${seconds.toFixed(seconds < 10 ? 2 : 1)} s`
  if (seconds < 3600)
    return `${Math.floor(seconds / 60)} min ${Math.round(seconds % 60)} s`
  return `${Math.floor(seconds / 3600)} h ${Math.round((seconds % 3600) / 60)} min`
}

export function describeMachine(machine: Machine) {
  const cap = machine.memoryMax ? `, capped at ${machine.memoryMax} per tool` : ""
  return `${machine.cpu} (${machine.cpus} threads), ${machine.memory} RAM${cap}, ${machine.os}`
}

function escapeHtml(text: string) {
  return text
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
}

const STYLE = `
:root {
  color-scheme: light;
  --ground: #f6f5f2; --panel: #fcfcfb; --ink: #0b0b0b; --muted: #52514e; --rule: #e2e1dc;
  --htmlpress: #2a78d6; --rival: #b4b3ab; --bare: #dcdbd6;
  --good: #0ca30c; --serious: #ec835a; --critical: #d03b3b;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    color-scheme: dark;
    --ground: #121211; --panel: #1a1a19; --ink: #f0efec; --muted: #c3c2b7; --rule: #383835;
    --htmlpress: #3987e5; --rival: #6b6a64; --bare: #42413d;
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --ground: #121211; --panel: #1a1a19; --ink: #f0efec; --muted: #c3c2b7; --rule: #383835;
  --htmlpress: #3987e5; --rival: #6b6a64; --bare: #42413d;
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--ground); color: var(--ink);
  font: 16px/1.55 system-ui, -apple-system, "Segoe UI", sans-serif; }
main { max-width: 980px; margin: 0 auto; padding: 40px 16px 64px; }
h1 { font-size: clamp(1.6rem, 4vw, 2.3rem); line-height: 1.15; margin: 4px 0 12px; }
h2 { font-size: 1.25rem; margin: 0 0 4px; }
.eyebrow { margin: 0; color: var(--muted); text-transform: uppercase; letter-spacing: .08em; font-size: .78rem; }
.lede { color: var(--muted); max-width: 70ch; margin: 0; }
section { background: var(--panel); border: 1px solid var(--rule); border-radius: 12px; padding: 20px; margin-top: 20px; }
.caption { color: var(--muted); font-size: .9rem; margin: 0 0 12px; }
.tiles { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(190px, 100%), 1fr)); gap: 12px; }
.tile { border: 1px solid var(--rule); border-radius: 10px; padding: 14px; display: flex; flex-direction: column; gap: 2px; }
.tile .label { color: var(--muted); font-size: .85rem; }
.tile .value { font-size: 1.6rem; font-weight: 650; font-variant-numeric: tabular-nums; }
.tile .note { color: var(--muted); font-size: .85rem; }
.legend { display: flex; gap: 16px; font-size: .85rem; color: var(--muted); margin-bottom: 8px; }
.swatch { display: inline-block; width: 10px; height: 10px; border-radius: 2px; margin-right: 6px; vertical-align: -1px; }
.swatch.htmlpress, .bar.htmlpress { background: var(--htmlpress); }
.swatch.rival, .bar.rival { background: var(--rival); }
.swatch.bare, .bar.bare { background: var(--bare); }
.charts { display: grid; grid-template-columns: repeat(auto-fit, minmax(min(300px, 100%), 1fr)); gap: 24px; margin-bottom: 16px; }
figure { margin: 0; min-width: 0; }
figcaption { font-weight: 600; font-size: .9rem; margin-bottom: 6px; }
.row { display: grid; grid-template-columns: minmax(0, min(11rem, 38%)) minmax(0, 1fr); gap: 8px; align-items: center; padding: 3px 0; position: relative; }
.row:hover { background: color-mix(in srgb, var(--rule) 45%, transparent); border-radius: 4px; }
.row:hover::after { content: attr(data-tip); position: absolute; left: 0; bottom: 100%; z-index: 1;
  background: var(--ink); color: var(--panel); font-size: .8rem; padding: 4px 8px; border-radius: 6px; white-space: nowrap; pointer-events: none; }
.name { font-size: .85rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.track { display: flex; align-items: center; gap: 6px; min-width: 0; }
.bar { height: 14px; border-radius: 0 4px 4px 0; flex: none; max-width: calc(100% - 4.5rem); }
.value, .missing { font-size: .8rem; color: var(--muted); font-variant-numeric: tabular-nums; white-space: nowrap; }
.missing { font-style: italic; }
.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; font-size: .88rem; }
th, td { text-align: left; padding: 7px 8px; border-bottom: 1px solid var(--rule); vertical-align: top; }
th { color: var(--muted); font-weight: 600; }
.num { text-align: right; font-variant-numeric: tabular-nums; white-space: nowrap; }
.chip { display: inline-block; padding: 1px 8px; border-radius: 999px; font-size: .8rem; white-space: nowrap; border: 1px solid; }
.chip.good { color: var(--good); border-color: var(--good); }
.chip.serious { color: var(--serious); border-color: var(--serious); }
.chip.critical { color: var(--critical); border-color: var(--critical); }
code { font-size: .88em; }
ul { padding-left: 1.2rem; margin: 0; }
li + li { margin-top: 6px; }
`
