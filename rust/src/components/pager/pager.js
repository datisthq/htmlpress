// The keys themselves live in scripts/keys.js, which follows the head's links
const KEYS = { prev: "H", next: "L" }

class HtmlpressPager extends HTMLElement {
  connectedCallback() {
    for (const [rel, key] of Object.entries(KEYS)) {
      this.querySelector(`a[rel="${rel}"]`)?.setAttribute("aria-keyshortcuts", key)
    }
  }
}

customElements.define("htmlpress-pager", HtmlpressPager)
