// Light-DOM components: each enhances plain HTML that already reads well without them.

const ICONS = {
  menu: '<path d="M4 6h16M4 12h16M4 18h16"/>',
  theme:
    '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>',
  up: '<path d="M12 19V5M5 12l7-7 7 7"/>',
}

class HpTabs extends HTMLElement {
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
    const saved = sync ? readStorage(`hp-tabs:${sync}`) : null
    this.select(Math.max(0, this.titles.indexOf(saved)))
    if (sync) {
      window.addEventListener("hp-tabs-sync", event => {
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
    writeStorage(`hp-tabs:${sync}`, title)
    window.dispatchEvent(new CustomEvent("hp-tabs-sync", { detail: { sync, title } }))
  }

  select(index) {
    this.buttons.forEach((button, i) => button.setAttribute("aria-selected", i === index))
    this.sections.forEach((section, i) => (section.hidden = i !== index))
  }
}

class HpTable extends HTMLElement {
  connectedCallback() {
    const headers = [...this.querySelectorAll("thead th")]
    headers.forEach((header, column) => {
      header.tabIndex = 0
      const sort = () => this.sort(headers, header, column)
      header.addEventListener("click", sort)
      header.addEventListener("keydown", event => event.key === "Enter" && sort())
    })
  }

  sort(headers, header, column) {
    const ascending = header.getAttribute("aria-sort") !== "ascending"
    headers.forEach(other => other.removeAttribute("aria-sort"))
    header.setAttribute("aria-sort", ascending ? "ascending" : "descending")
    const body = this.querySelector("tbody")
    if (!body) return
    const read = row => row.cells[column]?.textContent.trim() ?? ""
    const rows = [...body.rows].sort((a, b) => {
      const order = read(a).localeCompare(read(b), undefined, { numeric: true })
      return ascending ? order : -order
    })
    body.append(...rows)
  }
}

class HpZoom extends HTMLElement {
  connectedCallback() {
    const image = this.querySelector("img")
    if (!image) return
    image.addEventListener("click", () => {
      const dialog = document.createElement("dialog")
      dialog.className = "hp-zoom-dialog"
      const copy = image.cloneNode()
      copy.removeAttribute("width")
      copy.removeAttribute("height")
      dialog.append(copy)
      dialog.addEventListener("click", () => dialog.close())
      dialog.addEventListener("close", () => dialog.remove())
      document.body.append(dialog)
      dialog.showModal()
    })
  }
}

class HpThemeToggle extends HTMLElement {
  connectedCallback() {
    const button = createButton("Toggle dark mode", ICONS.theme)
    button.addEventListener("click", () => {
      const root = document.documentElement
      const dark = matchMedia("(prefers-color-scheme: dark)").matches
      const current = root.dataset.theme ?? (dark ? "dark" : "light")
      root.dataset.theme = current === "dark" ? "light" : "dark"
      writeStorage("hp-theme", root.dataset.theme)
    })
    this.replaceChildren(button)
  }
}

class HpNavToggle extends HTMLElement {
  connectedCallback() {
    const button = createButton("Menu", ICONS.menu)
    button.setAttribute("aria-controls", "hp-sidebar")
    button.setAttribute("aria-expanded", "false")
    const root = document.documentElement
    const toggle = open => {
      root.toggleAttribute("data-nav-open", open)
      button.setAttribute("aria-expanded", open)
    }
    button.addEventListener("click", () => toggle(!root.hasAttribute("data-nav-open")))
    document.addEventListener("click", event => {
      if (event.target === document.body) toggle(false)
    })
    document.addEventListener("keydown", event => event.key === "Escape" && toggle(false))
    this.replaceChildren(button)
  }
}

class HpToc extends HTMLElement {
  connectedCallback() {
    const links = [...this.querySelectorAll("a[href^='#']")]
    const headings = links
      .map(link => document.getElementById(decodeURIComponent(link.hash.slice(1))))
      .filter(Boolean)
    const update = () => {
      // The current heading is the last one scrolled past the header
      const passed = headings.filter(heading => heading.getBoundingClientRect().top < 120)
      const current = passed.at(-1) ?? headings[0]
      for (const link of links) {
        if (link.hash === `#${current?.id}`) link.setAttribute("aria-current", "true")
        else link.removeAttribute("aria-current")
      }
    }
    addEventListener("scroll", update, { passive: true })
    update()
  }
}

class HpBackToTop extends HTMLElement {
  connectedCallback() {
    const link = document.createElement("a")
    link.href = "#"
    link.setAttribute("aria-label", "Back to top")
    link.innerHTML = createIcon(ICONS.up)
    this.replaceChildren(link)
    const update = () => this.toggleAttribute("data-visible", scrollY > 600)
    addEventListener("scroll", update, { passive: true })
    update()
  }
}

function addCopyButtons() {
  for (const pre of document.querySelectorAll(".hp-content pre")) {
    if (pre.parentElement?.classList.contains("hp-code")) continue
    const box = document.createElement("div")
    box.className = "hp-code"
    pre.replaceWith(box)
    const button = document.createElement("button")
    button.type = "button"
    button.textContent = "Copy"
    button.addEventListener("click", async () => {
      await navigator.clipboard.writeText(pre.textContent ?? "")
      button.textContent = "Copied"
      setTimeout(() => (button.textContent = "Copy"), 2000)
    })
    box.append(pre, button)
  }
}

function addHeadingAnchors() {
  for (const heading of document.querySelectorAll(".hp-content :is(h2, h3)[id]")) {
    if (heading.querySelector(".hp-anchor")) continue
    const anchor = document.createElement("a")
    anchor.className = "hp-anchor"
    anchor.href = `#${heading.id}`
    anchor.setAttribute("aria-label", "Link to this section")
    anchor.textContent = "#"
    heading.append(anchor)
  }
}

function createButton(label, icon) {
  const button = document.createElement("button")
  button.type = "button"
  button.setAttribute("aria-label", label)
  button.innerHTML = createIcon(icon)
  return button
}

function createIcon(paths) {
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${paths}</svg>`
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

customElements.define("hp-tabs", HpTabs)
customElements.define("hp-table", HpTable)
customElements.define("hp-zoom", HpZoom)
customElements.define("hp-theme-toggle", HpThemeToggle)
customElements.define("hp-nav-toggle", HpNavToggle)
customElements.define("hp-toc", HpToc)
customElements.define("hp-back-to-top", HpBackToTop)
addCopyButtons()
addHeadingAnchors()
