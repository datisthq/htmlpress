// Livemark's shortcuts menu: lists the keys this page answers to, read as it opens, and a
// row presses its key
class HtmlpressShortcuts extends HTMLElement {
  connectedCallback() {
    const details = this.querySelector(":scope > details")
    if (this.wired || !details) return
    this.wired = true
    details.addEventListener("toggle", () => {
      if (!details.open) return
      for (const button of details.querySelectorAll("[data-key]")) {
        button.parentElement.hidden =
          button.hasAttribute("data-targeted") && !isAnnounced(button.dataset.key)
      }
    })
    details.addEventListener("click", event => {
      const button = event.target.closest("[data-key]")
      if (!button) return
      details.open = false
      const init = { key: button.dataset.key.toLowerCase(), bubbles: true }
      document.body.dispatchEvent(new KeyboardEvent("keydown", init))
    })
    document.addEventListener("click", event => {
      if (!this.contains(event.target)) details.open = false
    })
    document.addEventListener("keydown", event => {
      if (event.key === "Escape") details.open = false
    })
  }
}

function isAnnounced(key) {
  const controls = document.querySelectorAll(`[aria-keyshortcuts="${key}"]`)
  return [...controls].some(control => control.checkVisibility())
}

customElements.define("htmlpress-shortcuts", HtmlpressShortcuts)
