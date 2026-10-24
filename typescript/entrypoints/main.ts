#!/usr/bin/env node
import { spawnSync } from "node:child_process"
import { createRequire } from "node:module"
import { resolveBinary } from "../actions/binary/resolve.ts"

const require = createRequire(import.meta.url)

let binary: string
try {
  binary = resolveBinary({
    platform: process.platform,
    arch: process.arch,
    env: process.env,
    resolvePackage: specifier => require.resolve(specifier),
  })
} catch (error) {
  console.error(`error: ${error instanceof Error ? error.message : String(error)}`)
  process.exit(1)
}

const result = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" })
if (result.error) {
  console.error(`error: cannot run ${binary}: ${result.error.message}`)
  process.exit(1)
}
// Re-raise a signal so a parent sees the same death the binary had (Ctrl-C under start)
if (result.signal) process.kill(process.pid, result.signal)
process.exit(result.status ?? 1)
