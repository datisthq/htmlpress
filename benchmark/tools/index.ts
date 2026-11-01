import { hextra, hugo, mdbook, zola, zolaBook } from "./binaries.ts"
import { htmlpress } from "./htmlpress.ts"
import { docusaurus, eleventy, livemark, starlight, vitepress } from "./node.ts"
import { mkdocs } from "./python.ts"
import type { Tool } from "./types.ts"

export const TOOLS: Tool[] = [
  htmlpress,
  hugo,
  hextra,
  zola,
  zolaBook,
  mdbook,
  vitepress,
  starlight,
  docusaurus,
  eleventy,
  livemark,
  mkdocs,
]

export function selectTools(names?: string[]) {
  if (!names?.length) return TOOLS
  const tools = TOOLS.filter(tool => names.includes(tool.name))
  if (tools.length !== names.length) {
    const known = TOOLS.map(tool => tool.name).join(", ")
    throw new Error(`Unknown tool in ${names.join(", ")}; known: ${known}`)
  }
  return tools
}
