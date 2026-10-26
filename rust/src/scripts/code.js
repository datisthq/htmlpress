// Every code block gets wrap and copy buttons, and every h2 to h6 with an id a link to itself.

const MESSAGES = readMessages()
const COPY = MESSAGES["code.copy"] ?? "Copy"
const WRAP_ON = MESSAGES["code.wrapOn"] ?? "Enable wrap"
const WRAP_OFF = MESSAGES["code.wrapOff"] ?? "Disable wrap"
const WRAP =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M3 6h18M3 12h15a3 3 0 1 1 0 6h-4"/><path d="m16 16-2 2 2 2M3 18h7"/></svg>'

function addCopyButtons() {
  for (const pre of document.querySelectorAll("htmlpress-page > main pre")) {
    if (pre.parentElement?.classList.contains("htmlpress-code")) continue
    if (pre.closest("htmlpress-mermaid")) continue
    const box = document.createElement("div")
    box.className = "htmlpress-code"
    pre.replaceWith(box)
    const button = document.createElement("button")
    button.type = "button"
    button.textContent = COPY
    button.addEventListener("click", async () => {
      // Line numbers are part of the text but not of the code
      const code = pre.cloneNode(true)
      code.querySelectorAll(".giallo-ln").forEach(number => number.remove())
      await navigator.clipboard.writeText(code.textContent ?? "")
      button.textContent = MESSAGES["code.copied"] ?? "Copied"
      setTimeout(() => (button.textContent = COPY), 2000)
    })
    const actions = document.createElement("span")
    actions.append(createWrapButton(pre), button)
    box.append(pre, actions)
  }
}

function createWrapButton(pre) {
  const button = document.createElement("button")
  button.type = "button"
  button.innerHTML = WRAP
  const show = () => {
    const wrapped = pre.dataset.wrap !== undefined
    button.setAttribute("aria-pressed", String(wrapped))
    button.setAttribute("aria-label", wrapped ? WRAP_OFF : WRAP_ON)
    button.title = wrapped ? WRAP_OFF : WRAP_ON
  }
  button.addEventListener("click", () => {
    pre.toggleAttribute("data-wrap")
    show()
  })
  show()
  return button
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
