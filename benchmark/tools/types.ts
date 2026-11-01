import type { Site } from "../generate/site.ts"

export type Tool = {
  name: string
  title: string
  /** Builds a documentation site: navigation, search and styled pages, not bare templates. */
  isThemed: boolean
  /** What the tool builds differently from htmlpress, shown in the report. */
  caveats: string[]
  getVersion: () => Promise<string>
  /** Writes the tool's project for `site`, once a site, and returns its folder. */
  createProject: (site: Site) => Promise<string>
  /** The command building the project at `directory`, run from it. */
  createCommand: (directory: string) => Command
  /** What a build writes, its output and caches, removed before each build so all are cold. */
  listOutputs: (directory: string) => string[]
  /** The folder the built HTML lands in. */
  readOutput: (directory: string) => string
}

export type Command = {
  file: string
  args: string[]
  env?: Record<string, string>
}
