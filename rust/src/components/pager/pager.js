const KEYS = { prev: "p", next: "n" }

class HtmlpressPager extends HTMLElement {
  connectedCallback() {
    for (const [rel, key] of Object.entries(KEYS)) {
      this.querySelector(`a[rel="${rel}"]`)?.setAttribute(
        "aria-keyshortcuts",
        key.toUpperCase(),
      )
    }
    document.addEventListener("keydown", event => {
      const rel = Object.keys(KEYS).find(rel => isShortcut(event, KEYS[rel]))
      if (rel) this.querySelector(`a[rel="${rel}"]`)?.click()
    })
  }
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

customElements.define("htmlpress-pager", HtmlpressPager)
