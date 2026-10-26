// The menu is a plain <details>; this only closes it on a click outside or Escape
class HtmlpressLanguage extends HTMLElement {
  connectedCallback() {
    const details = this.querySelector("details")
    if (!details) return
    document.addEventListener("click", event => {
      if (!this.contains(event.target)) details.open = false
    })
    document.addEventListener("keydown", event => {
      if (event.key === "Escape") details.open = false
    })
  }
}

customElements.define("htmlpress-language", HtmlpressLanguage)
