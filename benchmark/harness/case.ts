import { readFile, readdir } from "node:fs/promises"
import { basename, join } from "node:path"

export type Case = {
  name: string
  description: string
  profile: string
  /** Page counts the case builds. */
  sizes: number[]
  seed?: number
  /** Only these tools run at the case's sizes past the first, when one would take minutes. */
  tools?: string[]
}

export const CASES = join(import.meta.dirname, "..", "cases")

export async function loadCases(names?: string[]) {
  const files = (await readdir(CASES)).filter(file => file.endsWith(".json")).sort()
  const cases: Case[] = []
  for (const file of files) {
    const name = basename(file, ".json")
    if (names?.length && !names.includes(name)) continue
    cases.push({ name, ...JSON.parse(await readFile(join(CASES, file), "utf8")) })
  }
  if (names?.length && cases.length !== names.length) {
    throw new Error(`Unknown case in ${names.join(", ")}; known: ${files.join(", ")}`)
  }
  return cases
}
