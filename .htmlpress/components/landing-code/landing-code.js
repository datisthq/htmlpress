// Colors the code block inside by its `syntax`: html, shell or tree. Text stays text: the
// spans are built from the block's own characters, never parsed as markup.

const SYNTAXES = { html: splitHtml, shell: splitShell, tree: splitTree }

class LandingCode extends HTMLElement {
  connectedCallback() {
    const code = this.querySelector("pre > code")
    const split = SYNTAXES[this.getAttribute("syntax") ?? ""]
    if (!code || !split || code.querySelector("span")) return
    const tokens = split(code.textContent ?? "")
    code.replaceChildren(
      ...tokens.map(([token, text]) => {
        if (!token) return document.createTextNode(text)
        const span = document.createElement("span")
        span.dataset.token = token
        span.textContent = text
        return span
      }),
    )
  }
}

function splitHtml(text) {
  const tokens = []
  let last = 0
  for (const match of text.matchAll(/<!--[\s\S]*?-->|<\/?[a-zA-Z][^>]*>/g)) {
    tokens.push(["", text.slice(last, match.index)])
    if (match[0].startsWith("<!--")) tokens.push(["comment", match[0]])
    else splitTag(match[0], tokens)
    last = match.index + match[0].length
  }
  tokens.push(["", text.slice(last)])
  return tokens
}

function splitTag(tag, tokens) {
  const pattern = /(<\/?)([\w-]+)|([\w-]+)(=)("[^"]*")|([\w-]+)|(\/?>)|(\s+)/g
  for (const match of tag.matchAll(pattern)) {
    if (match[2]) tokens.push(["punct", match[1]], ["tag", match[2]])
    else if (match[3])
      tokens.push(["attr", match[3]], ["punct", "="], ["string", match[5]])
    else if (match[6]) tokens.push(["attr", match[6]])
    else if (match[7]) tokens.push(["punct", match[7]])
    else tokens.push(["", match[0]])
  }
}

function splitShell(text) {
  const tokens = []
  for (const [index, line] of text.split("\n").entries()) {
    if (index > 0) tokens.push(["", "\n"])
    if (!line.startsWith("$ ")) {
      splitWords(line, /("[^"]*")(?=\s*:)|("[^"]*")|(\b\d+\b)/g, tokens, [
        "key",
        "string",
        "number",
      ])
      continue
    }
    tokens.push(["dim", "$ "])
    const [command, ...rest] = line.slice(2).split(" ")
    tokens.push(["command", command])
    splitWords(` ${rest.join(" ")}`, /(\s-[\w-]+)|('[^']*'|"[^"]*")/g, tokens, [
      "flag",
      "string",
    ])
  }
  return tokens
}

function splitTree(text) {
  const tokens = []
  for (const [index, line] of text.split("\n").entries()) {
    if (index > 0) tokens.push(["", "\n"])
    const match = line.match(/^([│├└─\s]*)(\S+)(\s*)(←.*)?$/)
    if (!match) {
      tokens.push(["", line])
      continue
    }
    const name = match[2]
    const token = name.endsWith("/") ? "folder" : name.endsWith(".html") ? "page" : "key"
    tokens.push(
      ["dim", match[1]],
      [token, name],
      ["", match[3]],
      ["note", match[4] ?? ""],
    )
  }
  return tokens
}

function splitWords(text, pattern, tokens, names) {
  let last = 0
  for (const match of text.matchAll(pattern)) {
    tokens.push(["", text.slice(last, match.index)])
    const group = match.slice(1).findIndex(Boolean)
    tokens.push([names[group], match[0]])
    last = match.index + match[0].length
  }
  tokens.push(["", text.slice(last)])
}

customElements.define("htmlpress-landing-code", LandingCode)
