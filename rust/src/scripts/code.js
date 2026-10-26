// Every code block gets a copy button and every h2/h3 with an id a link to itself.

function addCopyButtons() {
  for (const pre of document.querySelectorAll("htmlpress-page > main pre")) {
    if (pre.parentElement?.classList.contains("htmlpress-code")) continue
    if (pre.closest("htmlpress-mermaid")) continue
    const box = document.createElement("div")
    box.className = "htmlpress-code"
    pre.replaceWith(box)
    const button = document.createElement("button")
    button.type = "button"
    button.textContent = "Copy"
    button.addEventListener("click", async () => {
      // Line numbers are part of the text but not of the code
      const code = pre.cloneNode(true)
      code.querySelectorAll(".giallo-ln").forEach(number => number.remove())
      await navigator.clipboard.writeText(code.textContent ?? "")
      button.textContent = "Copied"
      setTimeout(() => (button.textContent = "Copy"), 2000)
    })
    box.append(pre, button)
  }
}

function addHeadingAnchors() {
  for (const heading of document.querySelectorAll(
    "htmlpress-page > main :is(h2, h3)[id]",
  )) {
    if (heading.querySelector(".htmlpress-anchor")) continue
    const anchor = document.createElement("a")
    anchor.className = "htmlpress-anchor"
    anchor.href = `#${heading.id}`
    anchor.setAttribute("aria-label", "Link to this section")
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
    button.textContent = "Expand"
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
