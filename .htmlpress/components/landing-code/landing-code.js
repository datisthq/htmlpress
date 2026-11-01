// Colors a file tree, `syntax="tree"`, which Shiki has no grammar for. Text stays text:
// the spans are built from the block's own characters, never parsed as markup.

const SYNTAXES = { tree: splitTree }

class LandingCode extends HTMLElement {
  connectedCallback() {
    const code = this.querySelector("pre > code")
    const split = SYNTAXES[this.getAttribute("syntax") ?? ""]
    // Shiki colors a block naming a language; one it left plain has only bare spans
    if (!code || !split || code.querySelector("[data-token], span[style]")) return
    // Else Shiki's dark-theme `.shiki span` rule paints every token its plain color
    code.parentElement?.classList.remove("shiki")
    code.parentElement?.removeAttribute("style")
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

customElements.define("htmlpress-landing-code", LandingCode)
