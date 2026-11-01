import { execFile } from "node:child_process"
import { join } from "node:path"
import { promisify } from "node:util"
import { PYTHON_RIVALS, writeProject, writeYaml } from "./project.ts"
import type { Tool } from "./types.ts"

const MKDOCS = join(PYTHON_RIVALS, ".venv", "bin", "mkdocs")

export const mkdocs: Tool = {
  name: "mkdocs",
  title: "MkDocs Material",
  isThemed: true,
  caveats: [],
  getVersion: async () => {
    const python = join(PYTHON_RIVALS, ".venv", "bin", "python")
    const script = "import importlib.metadata as m; print(m.version('mkdocs-material'))"
    const { stdout } = await promisify(execFile)(python, ["-c", script])
    return stdout.trim()
  },
  createProject: site =>
    writeProject({
      tool: "mkdocs",
      site,
      files: {
        "mkdocs.yml": `site_name: Benchmark
theme:
  name: material
markdown_extensions:
  - tables
  - pymdownx.highlight
  - pymdownx.superfences
`,
      },
      writePage: (page, body) => ({
        path: `docs/${page.path}.md`,
        content: `${writeYaml({ title: page.title })}# ${page.title}\n\n${body}`,
      }),
    }),
  createCommand: directory => ({
    file: MKDOCS,
    args: [
      "build",
      "--quiet",
      "-f",
      join(directory, "mkdocs.yml"),
      "-d",
      join(directory, "site"),
    ],
  }),
  listOutputs: directory => [join(directory, "site")],
  readOutput: directory => join(directory, "site"),
}
