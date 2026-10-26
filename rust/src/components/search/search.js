const SEARCH_ICON =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m21 21-4.34-4.34"/><circle cx="11" cy="11" r="8"/></svg>'
const CLOSE_ICON =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>'
const HIGHLIGHT_NAME = "htmlpress-search"
const MAX_RESULTS = 10
const BOOSTS = { title: 3, description: 2, text: 1 }
// Written without spaces between words, so splitting at spaces would index a whole
// sentence as one word that no query ever matches
const UNSPACED_LANGUAGES = new Set(["zh", "ja", "th", "lo", "km", "my"])
const MESSAGES = readMessages()
const FALLBACKS = {
  "search.label": "Search",
  "search.description": "Search documentation",
  "search.placeholder": "Search docs...",
  "search.empty": "No results found.",
  "search.hint": "Type to search across all contents.",
  "search.close": "Close search",
  "search.pages": "Pages",
  "search.button": "Search...",
  "search.navigate": "navigate",
  "search.select": "select",
  "search.dismiss": "close",
}
let lastId = 0

class HtmlpressSearch extends HTMLElement {
  connectedCallback() {
    if (!this.dataset.index || this.wired) return
    this.wired = true
    // The build renders the button, so the header is whole on its first frame
    const button = this.querySelector(":scope > button") ?? createButton()
    button.addEventListener("click", () => this.open())
    document.addEventListener("keydown", event => {
      const modified = event.metaKey || event.ctrlKey || event.altKey
      const shortcut = (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k"
      const key = !modified && /^[s/]$/i.test(event.key) && !isTyping(event.target)
      if (shortcut || key) {
        event.preventDefault()
        this.open()
      }
    })
    if (!button.isConnected) this.replaceChildren(button)
  }

  open() {
    const language = document.documentElement.lang || "en"
    // Fetched on first open only, so a page nobody searches never downloads the index
    this.index ??= fetch(this.dataset.index)
      .then(response => response.json())
      .catch(() => ({ pages: [] }))
      .then(data => createIndex(data.pages ?? [], language))
    if (matchMedia("(max-width: 56rem)").matches) this.openBar()
    else this.openDialog()
  }

  openDialog() {
    if (!this.dialog) {
      this.dialog = document.createElement("dialog")
      this.dialog.setAttribute("aria-label", t("search.label"))
      this.dialog.setAttribute("aria-description", t("search.description"))
      this.palette = createPalette(this, () => this.dialog.close(), true)
      this.dialog.addEventListener("click", event => {
        if (event.target === this.dialog) this.dialog.close()
      })
      const command = document.createElement("div")
      command.className = "command"
      command.append(this.palette.field, this.palette.list, createFooter())
      this.dialog.append(command)
      this.append(this.dialog)
    }
    this.dialog.showModal()
    this.palette.input.select()
  }

  // Below 56rem the field takes over the header, with the results dropping under it
  openBar() {
    if (!this.bar) {
      this.bar = document.createElement("div")
      this.bar.className = "bar"
      this.sheet = createPalette(this, () => this.closeBar(), false)
      const close = document.createElement("button")
      close.type = "button"
      close.className = "close"
      close.setAttribute("aria-label", t("search.close"))
      close.title = t("search.close")
      close.innerHTML = CLOSE_ICON
      close.addEventListener("click", () => this.closeBar())
      const row = document.createElement("div")
      row.className = "row"
      row.append(this.sheet.field, close)
      const panel = document.createElement("div")
      panel.className = "panel"
      panel.append(this.sheet.list)
      this.bar.append(row, panel)
      this.bar.addEventListener("keydown", event => {
        if (event.key === "Escape") this.closeBar()
      })
      // Measured, not a fixed offset: while the banner is in view it pushes the header down
      this.measure = () => {
        panel.style.top = `${this.bar.getBoundingClientRect().bottom}px`
      }
      this.append(this.bar)
    }
    this.bar.hidden = false
    this.measure()
    addEventListener("scroll", this.measure, { passive: true })
    addEventListener("resize", this.measure)
    this.sheet.input.focus()
  }

  closeBar() {
    this.bar.hidden = true
    removeEventListener("scroll", this.measure)
    removeEventListener("resize", this.measure)
  }
}

function createButton() {
  const button = document.createElement("button")
  button.type = "button"
  button.setAttribute("aria-label", t("search.label"))
  button.setAttribute("aria-keyshortcuts", "S")
  button.title = t("search.label")
  button.innerHTML = `${SEARCH_ICON}<span></span><kbd>S</kbd>`
  button.querySelector("span").textContent = t("search.button")
  return button
}

// The input, the listbox and their keyboard handling, shared by the dialog and the bar
function createPalette(host, close, clearable) {
  const listId = `htmlpress-search-${++lastId}`
  const field = document.createElement("div")
  field.className = "field"
  field.innerHTML = SEARCH_ICON
  const input = document.createElement("input")
  input.type = "text"
  input.placeholder = t("search.placeholder")
  input.autocomplete = "off"
  input.spellcheck = false
  input.setAttribute("role", "combobox")
  input.setAttribute("aria-label", t("search.label"))
  input.setAttribute("aria-autocomplete", "list")
  input.setAttribute("aria-expanded", "true")
  input.setAttribute("aria-controls", listId)
  field.append(input)
  const clear = document.createElement("button")
  clear.type = "button"
  clear.className = "clear"
  clear.hidden = true
  clear.setAttribute("aria-label", t("search.close"))
  clear.innerHTML = CLOSE_ICON
  if (clearable) field.append(clear)
  const list = document.createElement("div")
  list.className = "list"
  list.id = listId
  list.setAttribute("role", "listbox")
  list.setAttribute("aria-label", t("search.label"))

  const palette = { field, input, list }
  const options = () => [...list.querySelectorAll("[role=option]")]
  const select = option => {
    for (const item of options()) item.setAttribute("aria-selected", item === option)
    if (option) {
      input.setAttribute("aria-activedescendant", option.id)
      option.scrollIntoView({ block: "nearest" })
    } else {
      input.removeAttribute("aria-activedescendant")
    }
  }
  const run = async () => {
    const query = input.value
    clear.hidden = !query
    const results = query.trim() ? (await host.index).search(query) : []
    if (query !== input.value) return
    renderResults(list, results, query, listId)
    select(options()[0])
  }

  input.addEventListener("input", run)
  clear.addEventListener("click", () => {
    input.value = ""
    run()
    input.focus()
  })
  input.addEventListener("keydown", event => {
    const items = options()
    const current = items.findIndex(item => item.getAttribute("aria-selected") === "true")
    const last = items.length - 1
    const moves = {
      ArrowDown: current >= last ? 0 : current + 1,
      ArrowUp: current <= 0 ? last : current - 1,
      Home: 0,
      End: last,
    }
    if (event.key in moves && items.length) {
      event.preventDefault()
      select(items[moves[event.key]])
    } else if (event.key === "Enter" && items[current]) {
      event.preventDefault()
      items[current].click()
    }
  })
  list.addEventListener("pointermove", event => {
    const option = event.target.closest?.("[role=option]")
    if (option && option.getAttribute("aria-selected") !== "true") select(option)
  })
  list.addEventListener("click", event => {
    if (!event.target.closest?.("[role=option]")) return
    input.value = ""
    run()
    close()
  })
  run()
  return palette
}

function renderResults(list, results, query, listId) {
  if (!results.length) {
    const empty = document.createElement("p")
    empty.className = "empty"
    empty.textContent = query.trim() ? t("search.empty") : t("search.hint")
    list.replaceChildren(empty)
    return
  }
  const groups = new Map()
  for (const result of results) {
    const group = result.group ?? t("search.pages")
    groups.set(group, [...(groups.get(group) ?? []), result])
  }
  const pattern = buildHighlightPattern(query)
  const highlight = new URLSearchParams({ highlight: query.trim() })
  list.replaceChildren(
    ...[...groups].map(([heading, items], groupIndex) => {
      const group = document.createElement("div")
      group.setAttribute("role", "group")
      const label = document.createElement("div")
      label.className = "heading"
      label.id = `${listId}-group-${groupIndex}`
      label.textContent = heading
      group.setAttribute("aria-labelledby", label.id)
      group.append(label)
      for (const result of items) {
        const option = document.createElement("a")
        option.id = `${listId}-${result.url}`
        option.href = `${result.url}?${highlight}`
        option.setAttribute("role", "option")
        option.setAttribute("aria-selected", "false")
        option.tabIndex = -1
        // The icon is htmlpress' own markup, rendered from a built-in icon at build time
        if (result.icon) option.innerHTML = result.icon
        const body = document.createElement("span")
        const title = document.createElement("strong")
        title.textContent = result.title
        body.append(title)
        if (result.snippet) {
          const snippet = document.createElement("span")
          snippet.append(highlightText(result.snippet, pattern))
          body.append(snippet)
        }
        option.append(body)
        group.append(option)
      }
      return group
    }),
  )
}

function createFooter() {
  const footer = document.createElement("div")
  footer.className = "footer"
  const hints = [
    ["↑↓", "search.navigate"],
    ["↵", "search.select"],
    ["esc", "search.dismiss"],
  ]
  for (const [key, message] of hints) {
    const hint = document.createElement("span")
    const kbd = document.createElement("kbd")
    kbd.textContent = key
    hint.append(kbd, t(message))
    footer.append(hint)
  }
  return footer
}

// An inverted index over each page's title, description and text, searched like
// Orama: any query word may match, as a word's prefix or within one typo of a word,
// and pages rank by BM25 with the title and description boosted
function createIndex(pages, language) {
  const tokenize = createTokenizer(language)
  // A typo allowance of one character turns a two-character Chinese word into any
  // other word sharing a character
  const tolerance = UNSPACED_LANGUAGES.has(language.split("-")[0]) ? 0 : 1
  const postings = new Map()
  const lengths = { title: 0, description: 0, text: 0 }
  const documents = pages.map((page, id) => {
    const chunks = (page.chunks ?? []).map(chunk =>
      [chunk.heading, chunk.text].filter(Boolean).join(" "),
    )
    const text = [page.title, ...chunks].join(" ")
    const fields = { title: page.title, description: page.description ?? "", text }
    const sizes = {}
    for (const [field, value] of Object.entries(fields)) {
      const tokens = tokenize(value)
      sizes[field] = tokens.length
      lengths[field] += tokens.length
      const counts = new Map()
      for (const token of tokens) counts.set(token, (counts.get(token) ?? 0) + 1)
      for (const [token, count] of counts) {
        const entries = postings.get(token) ?? []
        entries.push({ id, field, count })
        postings.set(token, entries)
      }
    }
    return { page, text, sizes }
  })
  const vocabulary = [...postings.keys()]

  const search = query => {
    const scores = new Map()
    for (const term of new Set(tokenize(query))) {
      const tokens = vocabulary.filter(
        token => token.startsWith(term) || (tolerance && isOneEditAway(token, term)),
      )
      for (const token of tokens) {
        const entries = postings.get(token) ?? []
        for (const field of Object.keys(BOOSTS)) {
          const hits = entries.filter(entry => entry.field === field)
          const idf = Math.log(
            1 + (documents.length - hits.length + 0.5) / (hits.length + 0.5),
          )
          const average = lengths[field] / documents.length || 1
          for (const { id, count } of hits) {
            const size = documents[id].sizes[field] / average
            const weight = (count * 2.2) / (count + 1.2 * (0.25 + 0.75 * size))
            scores.set(id, (scores.get(id) ?? 0) + idf * weight * BOOSTS[field])
          }
        }
      }
    }
    return [...scores]
      .sort((a, b) => b[1] - a[1])
      .slice(0, MAX_RESULTS)
      .map(([id]) => {
        const { page, text } = documents[id]
        return {
          title: page.title,
          url: page.url,
          icon: page.icon,
          group: page.group,
          snippet: extractSnippet(text, query),
        }
      })
  }
  return { search }
}

function createTokenizer(language) {
  if (
    UNSPACED_LANGUAGES.has(language.split("-")[0]) &&
    typeof Intl.Segmenter === "function"
  ) {
    const segmenter = new Intl.Segmenter(language, { granularity: "word" })
    return text =>
      [...segmenter.segment(text.toLowerCase())]
        .filter(segment => segment.isWordLike)
        .map(segment => segment.segment)
  }
  return text =>
    text
      .toLowerCase()
      .split(/[^\p{L}\p{N}_]+/u)
      .filter(Boolean)
}

function isOneEditAway(a, b) {
  if (Math.abs(a.length - b.length) > 1 || a === b) return a === b
  let i = 0
  while (i < a.length && i < b.length && a[i] === b[i]) i++
  if (a.length === b.length) return a.slice(i + 1) === b.slice(i + 1)
  return a.length > b.length
    ? a.slice(i + 1) === b.slice(i)
    : a.slice(i) === b.slice(i + 1)
}

/** A case-insensitive pattern matching each word of a search query, longest first */
function buildHighlightPattern(query) {
  const words = [...new Set(query.toLowerCase().split(/\s+/))]
    .filter(word => word.length >= 2)
    .sort((a, b) => b.length - a.length)
    .map(word => word.replace(/[.*+?^${}()|[\]\\]/g, "\\$&"))
  if (!words.length) return undefined
  return new RegExp(words.join("|"), "gi")
}

function findTextMatches(text, pattern) {
  return [...text.matchAll(pattern)].map(match => [
    match.index,
    match.index + match[0].length,
  ])
}

function extractSnippet(text, query) {
  const pattern = buildHighlightPattern(query)
  const [match] = pattern ? findTextMatches(text, pattern) : []
  if (!match) return text.slice(0, 120)
  const start = Math.max(0, match[0] - 40)
  const end = Math.min(text.length, match[1] + 80)
  return `${start > 0 ? "…" : ""}${text.slice(start, end)}${end < text.length ? "…" : ""}`
}

// Built from text nodes, never innerHTML, since the text comes from pages
function highlightText(text, pattern) {
  const fragment = document.createDocumentFragment()
  let cursor = 0
  for (const [start, end] of pattern ? findTextMatches(text, pattern) : []) {
    const mark = document.createElement("mark")
    mark.textContent = text.slice(start, end)
    fragment.append(text.slice(cursor, start), mark)
    cursor = end
  }
  fragment.append(text.slice(cursor))
  return fragment
}

// A result opens its page with `?highlight=<query>`: its matches are painted with the
// CSS Custom Highlight API, which leaves the DOM untouched, and the first one shown
function highlightPage() {
  const query = new URLSearchParams(location.search).get("highlight")
  const pattern = query ? buildHighlightPattern(query) : undefined
  const root = document.querySelector("main")
  if (!pattern || !root || typeof CSS === "undefined" || !CSS.highlights) return
  const ranges = []
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, {
    acceptNode: node =>
      node.parentElement?.closest(
        "pre, script, style, svg, textarea, htmlpress-breadcrumbs, htmlpress-pager, htmlpress-footer",
      )
        ? NodeFilter.FILTER_REJECT
        : NodeFilter.FILTER_ACCEPT,
  })
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    for (const [start, end] of findTextMatches(node.textContent ?? "", pattern)) {
      const range = new Range()
      range.setStart(node, start)
      range.setEnd(node, end)
      ranges.push(range)
    }
  }
  CSS.highlights.set(HIGHLIGHT_NAME, new Highlight(...ranges))
  // An explicit anchor in the URL wins over the first match
  if (location.hash) return
  const first = ranges.find(range =>
    range.startContainer.parentElement?.checkVisibility(),
  )
  requestAnimationFrame(() =>
    first?.startContainer.parentElement?.scrollIntoView({ block: "center" }),
  )
}

function isTyping(target) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  )
}

function t(key) {
  return MESSAGES[key] ?? FALLBACKS[key]
}

customElements.define("htmlpress-search", HtmlpressSearch)
highlightPage()

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}
