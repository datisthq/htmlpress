import { EventEmitter } from "node:events"
import { describe, expect, it } from "vite-plus/test"
import { runBinary } from "./run.ts"

const LISTENER = [
  "-e",
  "process.on('SIGTERM', () => process.exit(7)); process.stdout.write('ready'); " +
    "setInterval(() => {}, 1000)",
]

describe("runBinary", () => {
  it("returns the binary's exit status", async () => {
    const result = await runBinary({
      binary: process.execPath,
      args: ["-e", "process.exit(3)"],
      signals: new EventEmitter(),
      stdio: "ignore",
    })
    expect(result).toEqual({ status: 3, signal: null })
  })

  // Windows has no catchable SIGTERM: `kill` there always terminates the child
  it.skipIf(process.platform === "win32")(
    "passes a signal the wrapper gets on to the binary",
    async () => {
      const signals = new EventEmitter()
      const running = runBinary({
        binary: process.execPath,
        args: LISTENER,
        signals,
        stdio: "pipe",
        onSpawn: child =>
          child.stdout?.once("data", () => signals.emit("SIGTERM", "SIGTERM")),
      })
      expect(await running).toEqual({ status: 7, signal: null })
      expect(signals.listenerCount("SIGTERM")).toBe(0)
    },
  )

  it("rejects when the binary cannot run", async () => {
    const running = runBinary({
      binary: "/nonexistent/htmlpress",
      args: [],
      signals: new EventEmitter(),
      stdio: "ignore",
    })
    await expect(running).rejects.toThrow()
  })
})
