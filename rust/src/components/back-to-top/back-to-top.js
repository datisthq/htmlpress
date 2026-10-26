const ICON = '<path d="M12 19V5M5 12l7-7 7 7"/>'

class HtmlpressBackToTop extends HTMLElement {
  connectedCallback() {
    const link = document.createElement("a")
    link.href = "#"
    link.setAttribute("aria-label", readMessages()["nav.backToTop"] ?? "Back to top")
    link.setAttribute("aria-keyshortcuts", "B")
    link.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true">${ICON}</svg>`
    link.addEventListener("click", event => {
      event.preventDefault()
      scrollToTop()
    })
    this.replaceChildren(link)
    const update = () => this.toggleAttribute("data-visible", scrollY > 400)
    addEventListener("scroll", update, { passive: true })
    update()
    document.addEventListener("keydown", event => {
      if (isShortcut(event, "b")) scrollToTop()
    })
  }
}

function scrollToTop() {
  const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches
  scrollTo({ top: 0, behavior: reduced ? "auto" : "smooth" })
}

function isShortcut(event, key) {
  const modified = event.metaKey || event.ctrlKey || event.altKey || event.shiftKey
  return !modified && event.key.toLowerCase() === key && !isTyping(event.target)
}

function isTyping(target) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  )
}

customElements.define("htmlpress-back-to-top", HtmlpressBackToTop)

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}
