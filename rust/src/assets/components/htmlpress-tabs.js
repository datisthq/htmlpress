class HtmlpressTabs extends HTMLElement {
  connectedCallback() {
    if (this.querySelector(":scope > [role=tablist]")) return
    const sections = [...this.querySelectorAll(":scope > section")]
    const list = document.createElement("div")
    list.setAttribute("role", "tablist")
    this.titles = sections.map((section, index) => {
      return section.getAttribute("title") ?? `Tab ${index + 1}`
    })
    const buttons = sections.map((section, index) => {
      const button = document.createElement("button")
      button.type = "button"
      button.setAttribute("role", "tab")
      button.textContent = this.titles[index]
      section.setAttribute("role", "tabpanel")
      button.addEventListener("click", () => this.choose(index))
      list.append(button)
      return button
    })
    this.prepend(list)
    this.buttons = buttons
    this.sections = sections
    const sync = this.getAttribute("sync")
    const saved = sync ? readStorage(`htmlpress-tabs:${sync}`) : null
    this.select(Math.max(0, this.titles.indexOf(saved)))
    if (sync) {
      window.addEventListener("htmlpress-tabs-sync", event => {
        const index = this.titles.indexOf(event.detail.title)
        if (event.detail.sync === sync && index >= 0) this.select(index)
      })
    }
  }

  choose(index) {
    this.select(index)
    const sync = this.getAttribute("sync")
    if (!sync) return
    const title = this.titles[index]
    writeStorage(`htmlpress-tabs:${sync}`, title)
    window.dispatchEvent(
      new CustomEvent("htmlpress-tabs-sync", { detail: { sync, title } }),
    )
  }

  select(index) {
    this.buttons.forEach((button, i) => button.setAttribute("aria-selected", i === index))
    this.sections.forEach((section, i) => (section.hidden = i !== index))
  }
}

// Storage throws in some private modes; a choice that cannot be kept is still applied
function readStorage(key) {
  try {
    return localStorage.getItem(key)
  } catch {
    return null
  }
}

function writeStorage(key, value) {
  try {
    localStorage.setItem(key, value)
  } catch {}
}

customElements.define("htmlpress-tabs", HtmlpressTabs)
