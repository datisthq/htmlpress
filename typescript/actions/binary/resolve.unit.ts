import { describe, expect, it } from "vite-plus/test"
import { resolveBinary } from "./resolve.ts"

describe("resolveBinary", () => {
  it("resolves the platform package's binary", () => {
    const binary = resolveBinary({
      platform: "linux",
      arch: "x64",
      env: {},
      resolvePackage: specifier => `/node_modules/${specifier}`,
    })
    expect(binary).toBe("/node_modules/gethtmlpress-linux-x64/bin/htmlpress")
  })

  it("adds .exe on Windows", () => {
    const binary = resolveBinary({
      platform: "win32",
      arch: "arm64",
      env: {},
      resolvePackage: specifier => specifier,
    })
    expect(binary).toBe("gethtmlpress-win32-arm64/bin/htmlpress.exe")
  })

  it("prefers the env override", () => {
    const binary = resolveBinary({
      platform: "plan9",
      arch: "mips",
      env: { HTMLPRESS_BINARY: "/opt/htmlpress" },
      resolvePackage: () => "",
    })
    expect(binary).toBe("/opt/htmlpress")
  })

  it("names the supported platforms for an unknown one", () => {
    const run = () =>
      resolveBinary({
        platform: "plan9",
        arch: "mips",
        env: {},
        resolvePackage: () => "",
      })
    expect(run).toThrow(/no binary for plan9-mips; supported: linux-x64/)
  })

  it("explains a missing optional dependency", () => {
    const run = () =>
      resolveBinary({
        platform: "darwin",
        arch: "arm64",
        env: {},
        resolvePackage: () => {
          throw new Error("Cannot find module")
        },
      })
    expect(run).toThrow(/gethtmlpress-darwin-arm64 is not installed/)
  })
})
