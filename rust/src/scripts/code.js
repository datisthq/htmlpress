// Every code block gets wrap and copy buttons, and every h2 to h6 with an id a link to itself.

const MESSAGES = readMessages()
const COPY = MESSAGES["code.copy"] ?? "Copy"
const ICONS = {
  copy: '<rect width="8" height="4" x="8" y="2" rx="1" ry="1"/><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2"/>',
  check: '<path d="M20 6 9 17l-5-5"/>',
  wrap: '<path d="m16 16-3 3 3 3"/><path d="M3 12h14.5a1 1 0 0 1 0 7H13"/><path d="M3 19h6"/><path d="M3 5h18"/>',
}

// Livemark's two always-visible actions: wrap the lines, copy the code
function addCopyButtons() {
  for (const pre of document.querySelectorAll("htmlpress-page > main pre")) {
    if (pre.parentElement?.classList.contains("htmlpress-code")) continue
    if (pre.closest("htmlpress-mermaid")) continue
    const box = document.createElement("div")
    box.className = "htmlpress-code"
    pre.replaceWith(box)
    const actions = document.createElement("div")
    actions.className = "htmlpress-code-actions"
    const wrap = createButton("wrap", MESSAGES["code.wrapOn"] ?? "Enable wrap")
    wrap.addEventListener("click", () => {
      const on = pre.toggleAttribute("data-wrap")
      wrap.setAttribute("aria-pressed", on)
      wrap.setAttribute(
        "aria-label",
        on
          ? (MESSAGES["code.wrapOff"] ?? "Disable wrap")
          : (MESSAGES["code.wrapOn"] ?? "Enable wrap"),
      )
    })
    const copy = createButton("copy", COPY)
    copy.addEventListener("click", async () => {
      // Line numbers are part of the text but not of the code
      const code = pre.cloneNode(true)
      code.querySelectorAll(".giallo-ln").forEach(number => number.remove())
      await navigator.clipboard.writeText(code.textContent ?? "")
      copy.innerHTML = createIcon("check")
      copy.setAttribute("aria-label", MESSAGES["code.copied"] ?? "Copied")
      setTimeout(() => {
        copy.innerHTML = createIcon("copy")
        copy.setAttribute("aria-label", COPY)
      }, 2000)
    })
    actions.append(wrap, copy)
    box.append(pre, actions)
  }
}

function createButton(icon, label) {
  const button = document.createElement("button")
  button.type = "button"
  button.setAttribute("aria-label", label)
  button.innerHTML = createIcon(icon)
  return button
}

function createIcon(name) {
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${ICONS[name]}</svg>`
}

function addHeadingAnchors() {
  for (const heading of document.querySelectorAll(
    "htmlpress-page > main :is(h2, h3, h4, h5, h6)[id]",
  )) {
    if (heading.querySelector(".htmlpress-anchor")) continue
    const anchor = document.createElement("a")
    anchor.className = "htmlpress-anchor"
    anchor.href = `#${heading.id}`
    anchor.setAttribute("aria-label", MESSAGES["code.link"] ?? "Link to this section")
    anchor.textContent = "#"
    heading.append(anchor)
  }
}

// Only a block that really is taller than its lines gets the bar. That is known only
// once layout settles (stylesheets, web fonts), so each block is watched until it is
const expandable = new ResizeObserver(entries => {
  for (const { target: pre } of entries) {
    if (pre.scrollHeight <= pre.clientHeight + 1) continue
    expandable.unobserve(pre)
    pre.dataset.collapsible = ""
    const button = document.createElement("button")
    button.type = "button"
    button.className = "htmlpress-expand"
    button.textContent = MESSAGES["code.expand"] ?? "Expand"
    button.addEventListener("click", () => {
      pre.dataset.expanded = ""
      button.remove()
    })
    pre.after(button)
  }
})

function addExpandButtons() {
  for (const pre of document.querySelectorAll(
    "htmlpress-page > main pre[data-collapse]",
  )) {
    expandable.observe(pre)
  }
}

addCopyButtons()
addExpandButtons()
addHeadingAnchors()

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}
