const ICON =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.34-4.34"/></svg>'
const SHORTCUT = /Mac|iPhone|iPad/.test(navigator.platform) ? "⌘K" : "Ctrl K"
const MAX_PAGES = 10
const MAX_CHUNKS = 3

class HtmlpressSearch extends HTMLElement {
  connectedCallback() {
    if (!this.dataset.index || this.querySelector("button")) return
    const button = document.createElement("button")
    button.type = "button"
    button.setAttribute("aria-label", "Search")
    button.innerHTML = `${ICON}<span>Search</span><kbd>${SHORTCUT}</kbd>`
    button.addEventListener("click", () => this.open())
    document.addEventListener("keydown", event => {
      const shortcut = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k"
      if (shortcut || (event.key === "/" && !isTyping(event.target))) {
        event.preventDefault()
        this.open()
      }
    })
    this.replaceChildren(button)
  }

  open() {
    this.dialog ??= this.createDialog()
    // Fetched on first open only, so a page nobody searches never downloads the index
    this.index ??= fetch(this.dataset.index)
      .then(response => response.json())
      .catch(() => ({ pages: [] }))
    this.dialog.showModal()
    this.input.select()
  }

  createDialog() {
    const dialog = document.createElement("dialog")
    dialog.setAttribute("aria-label", "Search")
    this.input = document.createElement("input")
    this.input.type = "search"
    this.input.placeholder = "Search"
    this.input.setAttribute("aria-label", "Search")
    this.results = document.createElement("div")
    this.results.setAttribute("role", "listbox")
    this.input.addEventListener("input", () => this.search())
    this.input.addEventListener("keydown", event => this.move(event))
    dialog.addEventListener("click", event => event.target === dialog && dialog.close())
    dialog.append(this.input, this.results)
    this.append(dialog)
    return dialog
  }

  async search() {
    const query = this.input.value
    const terms = query.toLowerCase().split(/\s+/).filter(Boolean)
    const index = await this.index
    if (query !== this.input.value) return
    const groups = terms.length ? rankPages(index.pages, terms) : []
    this.results.replaceChildren(...groups.map(group => createGroup(group, terms)))
    if (terms.length && !groups.length) {
      const empty = document.createElement("p")
      empty.textContent = `No results for “${query.trim()}”`
      this.results.append(empty)
    }
    this.select(0)
  }

  move(event) {
    const links = [...this.results.querySelectorAll("a")]
    const current = links.findIndex(link => link.getAttribute("aria-selected") === "true")
    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault()
      const step = event.key === "ArrowDown" ? 1 : -1
      this.select(Math.min(Math.max(current + step, 0), links.length - 1))
    } else if (event.key === "Enter" && links[current]) {
      event.preventDefault()
      links[current].click()
    }
  }

  select(index) {
    const links = [...this.results.querySelectorAll("a")]
    links.forEach((link, i) => link.setAttribute("aria-selected", i === index))
    links[index]?.scrollIntoView({ block: "nearest" })
  }
}

// Every term must appear in the page title, the heading or the text; a title hit
// outweighs a heading hit, which outweighs the text, and a whole word a word's start
function rankPages(pages, terms) {
  const groups = []
  for (const page of pages) {
    const title = page.title.toLowerCase()
    const hits = []
    for (const chunk of page.chunks) {
      const heading = (chunk.heading ?? "").toLowerCase()
      const text = chunk.text.toLowerCase()
      let score = 0
      let local = false
      for (const term of terms) {
        const inChunk = Math.max(scoreField(heading, term) * 5, scoreField(text, term))
        const best = Math.max(scoreField(title, term) * 10, inChunk)
        if (!best) {
          score = 0
          break
        }
        score += best
        local ||= inChunk > 0
      }
      if (score) hits.push({ chunk, score, local })
    }
    if (!hits.length) continue
    const local = hits.filter(hit => hit.local).sort((a, b) => b.score - a.score)
    const shown = local.length ? local.slice(0, MAX_CHUNKS) : hits.slice(0, 1)
    const score = Math.max(...hits.map(hit => hit.score))
    groups.push({ page, hits: shown, score })
  }
  return groups.sort((a, b) => b.score - a.score).slice(0, MAX_PAGES)
}

function scoreField(field, term) {
  const at = field.indexOf(term)
  if (at < 0) return 0
  const starts = at === 0 || /\W/.test(field[at - 1])
  const end = at + term.length
  const whole = starts && (end === field.length || /\W/.test(field[end]))
  return whole ? 3 : starts ? 2 : 1
}

function createGroup(group, terms) {
  const section = document.createElement("section")
  const header = document.createElement("h3")
  const title = document.createElement("span")
  title.append(highlightText(group.page.title, terms))
  header.append(title)
  if (group.page.section) {
    const label = document.createElement("small")
    label.textContent = group.page.section
    header.append(label)
  }
  section.append(header)
  for (const hit of group.hits) {
    const link = document.createElement("a")
    link.href = hit.chunk.id ? `${group.page.url}#${hit.chunk.id}` : group.page.url
    link.setAttribute("role", "option")
    const heading = document.createElement("strong")
    heading.append(highlightText(hit.chunk.heading ?? group.page.title, terms))
    const snippet = document.createElement("span")
    snippet.append(highlightText(cutSnippet(hit.chunk.text, terms), terms))
    link.append(heading, snippet)
    link.addEventListener("click", () => link.closest("dialog")?.close())
    section.append(link)
  }
  return section
}

function cutSnippet(text, terms) {
  const lower = text.toLowerCase()
  const at = Math.min(...terms.map(term => lower.indexOf(term)).filter(at => at >= 0))
  if (!Number.isFinite(at)) return text.slice(0, 120)
  const start = Math.max(0, at - 40)
  const snippet = text.slice(start, at + 100)
  return `${start > 0 ? "…" : ""}${snippet}${at + 100 < text.length ? "…" : ""}`
}

// Built from text nodes, never innerHTML, since the text comes from pages
function highlightText(text, terms) {
  const fragment = document.createDocumentFragment()
  const pattern = new RegExp(`(${terms.map(escapeRegExp).join("|")})`, "gi")
  text.split(pattern).forEach((part, index) => {
    if (index % 2) {
      const mark = document.createElement("mark")
      mark.textContent = part
      fragment.append(mark)
    } else if (part) {
      fragment.append(part)
    }
  })
  return fragment
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")
}

function isTyping(target) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  )
}

customElements.define("htmlpress-search", HtmlpressSearch)
