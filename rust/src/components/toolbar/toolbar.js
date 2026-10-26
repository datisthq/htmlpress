const MESSAGES = readMessages()
const PROMPT =
  MESSAGES["page.llmPrompt"] ?? "Read {url}, I want to ask questions about it."
const COPY = MESSAGES["page.copyMarkdown"] ?? "Copy Markdown"
const CHATS = [
  [
    MESSAGES["page.openChatGpt"] ?? "Open in ChatGPT",
    "https://chatgpt.com/",
    { hints: "search" },
  ],
  [MESSAGES["page.openClaude"] ?? "Open in Claude", "https://claude.ai/new", {}],
]

const ICONS = {
  copy: '<rect width="14" height="14" x="8" y="8" rx="2" ry="2"/><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  external:
    '<path d="M15 3h6v6"/><path d="M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
}

class HtmlpressToolbar extends HTMLElement {
  connectedCallback() {
    const link = this.querySelector('a[type="text/markdown"]')
    if (!link || this.querySelector("button")) return
    const copy = document.createElement("button")
    copy.type = "button"
    const show = (icon, label) => (copy.innerHTML = createIcon(icon) + escapeHtml(label))
    show("copy", COPY)
    copy.addEventListener("click", async () => {
      try {
        const response = await fetch(link.href)
        await navigator.clipboard.writeText(await response.text())
        show("check", MESSAGES["page.copied"] ?? "Copied")
      } catch {
        show("copy", MESSAGES["page.copyFailed"] ?? "Copy failed")
      }
      setTimeout(() => show("copy", COPY), 2000)
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
      chat.innerHTML = createIcon("external") + escapeHtml(label)
      return chat
    })
    // Copying covers what the link offers without the script
    link.hidden = true
    link.after(copy, ...chats)
  }
}

customElements.define("htmlpress-toolbar", HtmlpressToolbar)

function createIcon(name) {
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${ICONS[name]}</svg>`
}

function escapeHtml(text) {
  return text.replace(/[&<>"]/g, char => `&#${char.charCodeAt(0)};`)
}

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}
