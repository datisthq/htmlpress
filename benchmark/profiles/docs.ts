import type { Random } from "../generate/random.ts"
import type { Profile } from "./types.ts"

// Plain CommonMark and GFM tables every tool renders the same: no links between pages, which
// each tool resolves its own way and some fail a build over, and no syntax of one tool
export const docs: Profile = {
  name: "docs",
  description: "documentation pages: sections of prose, lists, tables and code blocks",
  sectionSize: 10,
  writeBody: options => {
    const random = options.random
    const blocks = [writeParagraph(random, options.token)]
    const sections = random.integer(4, 8)
    for (let index = 0; index < sections; index++) {
      blocks.push(`## ${writeTitle(random)}`, writeParagraph(random))
      if (random.chance(0.5)) blocks.push(`### ${writeTitle(random)}`)
      if (random.chance(0.6)) blocks.push(writeParagraph(random))
      if (random.chance(0.4)) blocks.push(writeList(random))
      if (random.chance(0.5)) blocks.push(writeCode(random))
      if (random.chance(0.15)) blocks.push(writeTable(random))
    }
    return `${blocks.join("\n\n")}\n`
  },
}

const WORDS = [
  "build",
  "page",
  "layout",
  "theme",
  "section",
  "folder",
  "config",
  "plugin",
  "render",
  "content",
  "server",
  "client",
  "request",
  "cache",
  "output",
  "source",
  "value",
  "option",
  "field",
  "table",
  "index",
  "search",
  "link",
  "image",
  "style",
  "script",
  "markdown",
  "component",
  "release",
  "version",
  "deploy",
  "preview",
  "local",
  "remote",
  "project",
  "default",
  "custom",
  "simple",
  "fast",
  "small",
  "large",
  "every",
  "other",
  "first",
  "next",
  "the",
  "a",
  "of",
  "to",
  "and",
  "in",
  "with",
  "for",
  "from",
  "on",
  "is",
  "it",
  "as",
  "by",
]

const LANGUAGES = ["rust", "typescript", "bash", "json", "yaml", "python"] as const

const CODE: Record<(typeof LANGUAGES)[number], string[]> = {
  rust: [
    'let value = options.read("{word}")?;',
    "fn {word}_{word}(input: &str) -> Result<String, Error> {",
    "    Ok(input.trim().to_owned())",
    "}",
    "for item in items.iter().filter(|item| item.is_{word}()) {",
  ],
  typescript: [
    "const {word} = await load{Word}(options.path)",
    "export function create{Word}(props: { name: string }) {",
    "  return { ...props, {word}: true }",
    "}",
    "if ({word}.length > 0) console.log({word})",
  ],
  bash: [
    "htmlpress build docs --json",
    "npm install --save-dev {word}",
    "export {WORD}_PATH=./{word}",
    "cat {word}.json | jq '.{word}'",
    'for file in *.md; do echo "$file"; done',
  ],
  json: ['{ "{word}": "{word}", "count": 42 }', '{ "{word}": [1, 2, 3] }'],
  yaml: ["{word}: {word}", "- name: {word}", "  enabled: true", "{word}:\n  - {word}"],
  python: [
    "def {word}_{word}(path: str) -> str:",
    "    return path.strip()",
    "for {word} in {word}s:",
    "    print({word})",
    "import {word}",
  ],
}

function writeParagraph(random: Random, token?: string) {
  const sentences = random.integer(2, 5)
  const parts = token ? [`This page is ${token}.`] : []
  for (let index = 0; index < sentences; index++) parts.push(writeSentence(random))
  return parts.join(" ")
}

function writeSentence(random: Random) {
  const words = Array.from(
    { length: random.integer(8, 18) },
    () => random.pick(WORDS) ?? "",
  )
  const index = random.integer(0, words.length - 1)
  if (random.chance(0.3)) words[index] = `\`${words[index]}\``
  else if (random.chance(0.2)) words[index] = `**${words[index]}**`
  else if (random.chance(0.1))
    words[index] = `[${words[index]}](https://example.com/${words[index]})`
  const sentence = words.join(" ")
  return `${sentence.charAt(0).toUpperCase()}${sentence.slice(1)}.`
}

function writeTitle(random: Random) {
  const title = Array.from({ length: random.integer(2, 4) }, () =>
    random.pick(WORDS),
  ).join(" ")
  return `${title.charAt(0).toUpperCase()}${title.slice(1)}`
}

function writeList(random: Random) {
  const ordered = random.chance(0.3)
  return Array.from({ length: random.integer(3, 6) }, (_, index) => {
    const marker = ordered ? `${index + 1}.` : "-"
    return `${marker} ${writeSentence(random)}`
  }).join("\n")
}

function writeCode(random: Random) {
  const language = random.pick(LANGUAGES) ?? "bash"
  const lines = Array.from({ length: random.integer(3, 16) }, () => {
    const line = random.pick(CODE[language]) ?? ""
    return line.replaceAll(/\{(word|Word|WORD)\}/g, (_, form: string) => {
      const word = random.pick(WORDS.slice(0, 45)) ?? "value"
      if (form === "Word") return `${word.charAt(0).toUpperCase()}${word.slice(1)}`
      return form === "WORD" ? word.toUpperCase() : word
    })
  })
  return `\`\`\`${language}\n${lines.join("\n")}\n\`\`\``
}

function writeTable(random: Random) {
  const columns = random.integer(3, 4)
  const header = Array.from({ length: columns }, () => writeTitle(random))
  const rows = Array.from({ length: random.integer(3, 6) }, () =>
    Array.from({ length: columns }, () => random.pick(WORDS) ?? ""),
  )
  return [header, header.map(() => "---"), ...rows]
    .map(row => `| ${row.join(" | ")} |`)
    .join("\n")
}
