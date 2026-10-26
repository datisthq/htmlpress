// When the header's links outgrow it, search first shrinks to its icon, then the last
// links fold into a "More" menu. Without the script the links scroll, as before.

const CHEVRON =
  '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 9 6 6 6-6"/></svg>'

class HtmlpressLinks extends HTMLElement {
  connectedCallback() {
    const nav = this.querySelector(":scope > nav")
    if (!nav || this.querySelector("details")) return
    const more = document.createElement("details")
    more.innerHTML = `<summary>${readMessages()["nav.more"] ?? "More"}${CHEVRON}</summary><div></div>`
    more.hidden = true
    nav.append(more)
    document.addEventListener("click", event => {
      if (!more.contains(event.target)) more.open = false
    })
    more.addEventListener("keydown", event => {
      if (event.key !== "Escape") return
      more.open = false
      more.querySelector("summary")?.focus()
    })
    new ResizeObserver(() => fit(this, nav, more)).observe(this)
  }
}

function fit(element, nav, more) {
  const header = element.closest("htmlpress-header")
  const links = [...nav.querySelectorAll(":scope > a")]
  const list = more.querySelector("div")
  header?.removeAttribute("data-compact")
  for (const link of links) link.hidden = false
  more.hidden = true
  list.replaceChildren()
  if (!overflows(nav)) return
  header?.setAttribute("data-compact", "")
  more.hidden = false
  const shown = links.filter(link => link.checkVisibility())
  const folded = []
  while (overflows(nav) && shown.length > 0) {
    const link = shown.pop()
    link.hidden = true
    folded.unshift(link)
  }
  if (folded.length === 0) more.hidden = true
  for (const link of folded) list.append(link.cloneNode(true))
  for (const copy of list.children) copy.hidden = false
}

function overflows(nav) {
  return nav.scrollWidth > nav.clientWidth + 1
}

customElements.define("htmlpress-links", HtmlpressLinks)

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}
