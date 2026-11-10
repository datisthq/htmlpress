import { type ChildProcess, type StdioOptions, spawn } from "node:child_process"
import type { EventEmitter } from "node:events"

/** The signals the wrapper passes on, so a process manager stopping it stops the binary. */
export const FORWARDED_SIGNALS = ["SIGINT", "SIGTERM", "SIGHUP"] as const

/** Run the binary until it exits, passing it every signal `signals` emits meanwhile. */
export function runBinary(props: {
  binary: string
  args: string[]
  signals: EventEmitter
  stdio: StdioOptions
  onSpawn?: (child: ChildProcess) => void
}) {
  return new Promise<{ status: number | null; signal: NodeJS.Signals | null }>(
    (resolve, reject) => {
      const child = spawn(props.binary, props.args, { stdio: props.stdio })
      const forward = (signal: NodeJS.Signals) => child.kill(signal)
      for (const signal of FORWARDED_SIGNALS) props.signals.on(signal, forward)
      const release = () => {
        for (const signal of FORWARDED_SIGNALS) props.signals.off(signal, forward)
      }
      child.once("error", error => {
        release()
        reject(error)
      })
      child.once("exit", (status, signal) => {
        release()
        resolve({ status, signal })
      })
      props.onSpawn?.(child)
    },
  )
}
