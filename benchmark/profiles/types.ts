import type { Random } from "../generate/random.ts"

/** The shape of a generated site's pages. */
export type Profile = {
  name: string
  description: string
  /** How many pages a section holds, its index among them. */
  sectionSize: number
  /** One page's Markdown body, without a title: every tool adds its own header. */
  writeBody: (options: { random: Random; token: string }) => string
}
