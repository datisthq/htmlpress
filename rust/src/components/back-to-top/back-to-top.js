const ICON = '<path d="M12 19V5M5 12l7-7 7 7"/>'

class HtmlpressBackToTop extends HTMLElement {
  connectedCallback() {
    const link = document.createElement("a")
    link.href = "#"
    link.setAttribute("aria-label", "Back to top")
    link.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true">${ICON}</svg>`
    this.replaceChildren(link)
    const update = () => this.toggleAttribute("data-visible", scrollY > 400)
    addEventListener("scroll", update, { passive: true })
    update()
  }
}

customElements.define("htmlpress-back-to-top", HtmlpressBackToTop)
