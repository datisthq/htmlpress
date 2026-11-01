import { execFile } from "node:child_process"
import { join } from "node:path"
import { promisify } from "node:util"
import { BENCHMARK, writeProject, writeYaml } from "./project.ts"
import type { Tool } from "./types.ts"

/** The release binary `setup:tools` builds from this checkout's `rust/`. */
export const HTMLPRESS = join(BENCHMARK, "..", "rust", "target", "release", "htmlpress")

export const htmlpress: Tool = {
  name: "htmlpress",
  title: "htmlpress",
  isThemed: true,
  caveats: [],
  getVersion: async () => {
    const { stdout } = await promisify(execFile)(HTMLPRESS, ["--version"])
    return stdout.trim().split(/\s+/).at(-1) ?? "unknown"
  },
  createProject: site =>
    writeProject({
      tool: "htmlpress",
      site,
      files: { "htmlpress.yaml": "title: Benchmark\n" },
      writePage: (page, body) => ({
        path: `${page.path}.md`,
        content: `${writeYaml({ title: page.title, order: page.order })}${body}`,
      }),
    }),
  createCommand: directory => ({ file: HTMLPRESS, args: ["build", directory] }),
  listOutputs: directory => [join(directory, ".htmlpress")],
  readOutput: directory => join(directory, ".htmlpress", "build"),
}
