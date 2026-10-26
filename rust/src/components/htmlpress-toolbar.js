const PROMPT = "Read {url}, I want to ask questions about it."
const CHATS = [
  ["Open in ChatGPT", "https://chatgpt.com/", { hints: "search" }],
  ["Open in Claude", "https://claude.ai/new", {}],
]

class HtmlpressToolbar extends HTMLElement {
  connectedCallback() {
    const link = this.querySelector("a[href]")
    if (!link || this.querySelector("button")) return
    const copy = document.createElement("button")
    copy.type = "button"
    copy.textContent = "Copy Markdown"
    copy.addEventListener("click", async () => {
      try {
        const response = await fetch(link.href)
        await navigator.clipboard.writeText(await response.text())
        copy.textContent = "Copied"
      } catch {
        copy.textContent = "Copy failed"
      }
      setTimeout(() => (copy.textContent = "Copy Markdown"), 2000)
    })
    const chats = CHATS.map(([label, base, extra]) => {
      const query = new URLSearchParams({
        ...extra,
        q: PROMPT.replace("{url}", location.href),
      })
      const chat = document.createElement("a")
      chat.href = `${base}?${query}`
      chat.target = "_blank"
      chat.rel = "noopener noreferrer"
      chat.textContent = label
      return chat
    })
    link.after(copy, ...chats)
  }
}

customElements.define("htmlpress-toolbar", HtmlpressToolbar)
