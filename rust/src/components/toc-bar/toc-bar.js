const MESSAGES = readMessages()
const ICONS = {
  chevron: '<path d="m18 15-6-6-6 6"/>',
}
const RING = 2 * Math.PI * 6

// Livemark's phone contents: a bar under the header naming the heading in view, which
// opens a sheet of the page's contents and, from its toolbar, its actions
class HtmlpressTocBar extends HTMLElement {
  connectedCallback() {
    if (this.querySelector("button")) return
    const toc = document.querySelector("htmlpress-aside htmlpress-toc")
    const links = toc ? [...toc.querySelectorAll("a[href^='#']")] : []
    this.entries = links
      .map(link => ({
        link,
        heading: document.getElementById(decodeURIComponent(link.hash.slice(1))),
      }))
      .filter(entry => entry.heading)
    if (!this.entries.length) return this.remove()
    this.replaceChildren(this.createBar())
    document.body.append(this.createSheet())
    addEventListener("scroll", () => this.update(), { passive: true })
    this.update()
  }

  createBar() {
    const button = document.createElement("button")
    button.type = "button"
    button.setAttribute("aria-haspopup", "dialog")
    button.innerHTML =
      '<svg viewBox="0 0 16 16" data-ring aria-hidden="true">' +
      `<circle cx="8" cy="8" r="6"/><circle cx="8" cy="8" r="6" stroke-dasharray="${RING}"/></svg>` +
      `<span></span>${createIcon("chevron")}`
    button.addEventListener("click", () => this.open())
    this.label = button.querySelector("span")
    this.ring = button.querySelectorAll("circle")[1]
    return button
  }

  // On the body, so this bar's own styles never reach the sheet's rows
  createSheet() {
    const sheet = document.createElement("htmlpress-sheet")
    sheet.setAttribute(
      "label",
      document.querySelector("main h1")?.textContent.trim() ||
        (MESSAGES["toc.title"] ?? "Contents"),
    )
    this.contents = this.createContents()
    this.sheet = sheet
    return sheet
  }

  createContents() {
    const list = document.createElement("ul")
    for (const entry of this.entries) {
      const row = document.createElement("a")
      row.href = entry.link.hash
      row.dataset.level = entry.link.dataset.level ?? "2"
      const label = document.createElement("span")
      label.textContent = entry.link.textContent
      row.append(label)
      row.addEventListener("click", () => this.sheet.removeAttribute("open"))
      entry.row = row
      const item = document.createElement("li")
      item.append(row)
      list.append(item)
    }
    return list
  }

  // The toolbar's script fills it after this one may have run, so its actions are
  // gathered when the sheet opens; a copy button forwards its click to the original
  createActions() {
    const toolbar = document.querySelector("htmlpress-aside htmlpress-toolbar")
    const items = toolbar ? [...toolbar.querySelectorAll(":scope > :is(a, button)")] : []
    const shown = items.filter(item => !item.hidden)
    if (!shown.length) return null
    const list = document.createElement("ul")
    for (const item of shown) {
      const row = item.cloneNode(true)
      if (item.tagName === "BUTTON") {
        row.addEventListener("click", () => {
          item.click()
          this.sheet.removeAttribute("open")
        })
      }
      const entry = document.createElement("li")
      entry.append(row)
      list.append(entry)
    }
    return list
  }

  open() {
    const actions = this.createActions()
    if (actions) {
      const tabs = document.createElement("div")
      tabs.setAttribute("role", "tablist")
      const panels = [this.contents, actions]
      const labels = [
        MESSAGES["toc.title"] ?? "Contents",
        MESSAGES["toc.actions"] ?? "Actions",
      ]
      const buttons = labels.map((label, index) => {
        const tab = document.createElement("button")
        tab.type = "button"
        tab.setAttribute("role", "tab")
        tab.textContent = label
        tab.addEventListener("click", () => select(index))
        return tab
      })
      const select = index => {
        buttons.forEach((tab, i) => tab.setAttribute("aria-selected", i === index))
        panels.forEach((panel, i) => (panel.hidden = i !== index))
      }
      tabs.append(...buttons)
      this.sheet.append(tabs, ...panels)
      select(0)
    } else {
      this.sheet.append(this.contents)
    }
    this.sheet.setAttribute("open", "")
  }

  update() {
    const passed = this.entries.filter(
      entry => entry.heading.getBoundingClientRect().top < 120,
    )
    const current = passed.at(-1) ?? this.entries[0]
    this.label.textContent = current.link.textContent
    for (const entry of this.entries) {
      if (entry === current) entry.row.setAttribute("aria-current", "true")
      else entry.row.removeAttribute("aria-current")
    }
    const room = document.documentElement.scrollHeight - innerHeight
    const progress = room > 0 ? Math.min(Math.max(scrollY / room, 0), 1) : 1
    this.ring.setAttribute("stroke-dashoffset", RING * (1 - progress))
  }
}

function createIcon(name) {
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${ICONS[name]}</svg>`
}

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}

customElements.define("htmlpress-toc-bar", HtmlpressTocBar)
