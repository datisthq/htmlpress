import { spawn } from "node:child_process"
import { readFileSync, readdirSync } from "node:fs"
import { mkdtemp, readFile, rm } from "node:fs/promises"
import { tmpdir } from "node:os"
import { join } from "node:path"
import type { Command } from "../tools/types.ts"

export type Outcome = "ok" | "timeout" | "oom" | "crash"

export type Run = {
  outcome: Outcome
  wallMs: number
  /** Peak anonymous memory of the tool's process tree, sampled; excludes mapped file pages. */
  peakMemoryMb?: number
  /** Peak resident set from `/usr/bin/time`; counts every mapping of the input file. */
  peakRssMb?: number
  exitCode: number | null
  /** How the run ended, for example `code 1` or `signal 9`. */
  exit: string
  stdout: string
  stderr: string
}

const KEPT_OUTPUT = 1024 * 1024
const SAMPLE_MS = 50

/** Runs a command under `/usr/bin/time`, optionally inside a systemd scope capped at `memoryMax`. */
export async function runCommand(
  command: Command,
  options: { cwd: string; timeoutMs: number; memoryMax?: string },
) {
  const directory = await mkdtemp(join(tmpdir(), "htmlpress-benchmark-"))
  const timePath = join(directory, "time.txt")
  const timed = [
    "/usr/bin/time",
    "-f",
    "%e %M",
    "-o",
    timePath,
    command.file,
    ...command.args,
  ]
  const argv = options.memoryMax
    ? [
        "systemd-run",
        "--user",
        "--scope",
        "--quiet",
        "-p",
        `MemoryMax=${options.memoryMax}`,
        "-p",
        "MemorySwapMax=0",
        "--",
        ...timed,
      ]
    : timed

  const started = performance.now()
  const child = spawn(argv[0] ?? "", argv.slice(1), {
    cwd: options.cwd,
    detached: true,
    env: { ...process.env, NO_COLOR: "1", ...command.env },
  })

  const sampler = startMemorySampler(child.pid ?? 0)
  let stdout = ""
  let stderr = ""
  child.stdout.on("data", chunk => (stdout = keepTail(stdout + chunk)))
  child.stderr.on("data", chunk => (stderr = keepTail(stderr + chunk)))

  let timedOut = false
  let spawnFailed = false
  const timer = setTimeout(() => {
    timedOut = true
    if (child.pid) process.kill(-child.pid, "SIGKILL")
  }, options.timeoutMs)

  // a process that never starts (a missing cwd or binary) emits only `error`, never `close`
  const exitCode = await new Promise<number | null>(resolve => {
    child.on("error", error => {
      spawnFailed = true
      stderr += String(error)
      resolve(null)
    })
    child.on("close", code => resolve(code))
  })
  clearTimeout(timer)
  const wallMs = performance.now() - started
  const peakMemoryMb = sampler.stop()

  // time prefixes "Command exited with non-zero status" or "terminated by signal" to its figures
  const timeLines = (await readFile(timePath, "utf8").catch(() => "")).trim().split("\n")
  const [, maxRssKb] = (timeLines.at(-1) ?? "").split(/\s+/)
  const signal = /terminated by signal (\d+)/.exec(timeLines[0] ?? "")?.[1]
  await rm(directory, { recursive: true, force: true })

  const run: Run = {
    outcome: classifyOutcome({
      timedOut,
      spawnFailed,
      exitCode,
      capped: Boolean(options.memoryMax),
      okExitCodes: [0],
    }),
    wallMs,
    peakMemoryMb,
    peakRssMb: maxRssKb ? Number(maxRssKb) / 1024 : undefined,
    exitCode,
    exit: signal ? `signal ${signal}` : `code ${exitCode}`,
    stdout,
    stderr,
  }
  return run
}

function classifyOutcome(options: {
  timedOut: boolean
  spawnFailed: boolean
  exitCode: number | null
  capped: boolean
  okExitCodes: number[]
}) {
  const { timedOut, spawnFailed, exitCode, capped, okExitCodes } = options
  if (timedOut) return "timeout"
  if (spawnFailed) return "crash"
  // a cgroup OOM kill can take the whole scope, `time` included, so no exit code survives
  const isCappedKill = capped && (exitCode === 143 || exitCode === null)
  if (exitCode === 137 || isCappedKill) return "oom"
  if (exitCode !== null && okExitCodes.includes(exitCode)) return "ok"
  return "crash"
}

/** Sums `RssAnon` over the process tree, since the tool runs under `time` and maybe `systemd-run`. */
function startMemorySampler(rootPid: number) {
  let peakKb = 0
  const sample = () => {
    const totalKb = listProcessTree(rootPid).reduce(
      (sum, pid) => sum + readAnonymousKb(pid),
      0,
    )
    peakKb = Math.max(peakKb, totalKb)
  }
  const timer = setInterval(sample, SAMPLE_MS)
  return {
    stop: () => {
      clearInterval(timer)
      return peakKb ? peakKb / 1024 : undefined
    },
  }
}

function listProcessTree(pid: number): number[] {
  const children: number[] = []
  try {
    for (const task of readdirSync(`/proc/${pid}/task`)) {
      const text = readFileSync(`/proc/${pid}/task/${task}/children`, "utf8").trim()
      if (text) children.push(...text.split(" ").map(Number))
    }
  } catch {
    return []
  }
  return [pid, ...children.flatMap(listProcessTree)]
}

function readAnonymousKb(pid: number) {
  try {
    const status = readFileSync(`/proc/${pid}/status`, "utf8")
    return Number(/^RssAnon:\s+(\d+) kB$/m.exec(status)?.[1] ?? 0)
  } catch {
    return 0
  }
}

function keepTail(text: string) {
  return text.length > KEPT_OUTPUT ? text.slice(-KEPT_OUTPUT) : text
}
