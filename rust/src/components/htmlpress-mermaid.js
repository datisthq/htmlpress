const MERMAID = "https://cdn.jsdelivr.net/npm/mermaid@11.4.1/dist/mermaid.esm.min.mjs"
let mermaid
let diagrams = 0

class HtmlpressMermaid extends HTMLElement {
  connectedCallback() {
    if (this.source !== undefined) return
    this.source = this.textContent.trim()
    this.render()
    const render = () => this.render()
    new MutationObserver(render).observe(document.documentElement, {
      attributeFilter: ["data-theme"],
    })
    matchMedia("(prefers-color-scheme: dark)").addEventListener("change", render)
  }

  // A diagram that fails to load or parse keeps its source on show
  async render() {
    try {
      mermaid ??= import(MERMAID).then(module => module.default)
      const api = await mermaid
      api.initialize({ startOnLoad: false, theme: readDark() ? "dark" : "default" })
      diagrams += 1
      const { svg } = await api.render(`htmlpress-mermaid-${diagrams}`, this.source)
      this.innerHTML = svg
    } catch (error) {
      console.error("htmlpress-mermaid:", error)
    }
  }
}

function readDark() {
  const theme = document.documentElement.dataset.theme
  if (theme) return theme === "dark"
  return matchMedia("(prefers-color-scheme: dark)").matches
}

customElements.define("htmlpress-mermaid", HtmlpressMermaid)
