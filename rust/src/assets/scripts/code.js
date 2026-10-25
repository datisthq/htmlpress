// Every code block gets a copy button and every h2/h3 with an id a link to itself.

function addCopyButtons() {
  for (const pre of document.querySelectorAll(".htmlpress-content pre")) {
    if (pre.parentElement?.classList.contains("htmlpress-code")) continue
    const box = document.createElement("div")
    box.className = "htmlpress-code"
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
  for (const heading of document.querySelectorAll(".htmlpress-content :is(h2, h3)[id]")) {
    if (heading.querySelector(".htmlpress-anchor")) continue
    const anchor = document.createElement("a")
    anchor.className = "htmlpress-anchor"
    anchor.href = `#${heading.id}`
    anchor.setAttribute("aria-label", "Link to this section")
    anchor.textContent = "#"
    heading.append(anchor)
  }
}

addCopyButtons()
addHeadingAnchors()
