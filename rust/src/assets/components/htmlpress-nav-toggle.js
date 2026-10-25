const ICON = '<path d="M4 6h16M4 12h16M4 18h16"/>'

class HtmlpressNavToggle extends HTMLElement {
  connectedCallback() {
    const button = createButton("Menu", ICON)
    button.setAttribute("aria-controls", "htmlpress-sidebar")
    button.setAttribute("aria-expanded", "false")
    const root = document.documentElement
    const toggle = open => {
      root.toggleAttribute("data-nav-open", open)
      button.setAttribute("aria-expanded", open)
    }
    button.addEventListener("click", () => toggle(!root.hasAttribute("data-nav-open")))
    document.addEventListener("click", event => {
      if (event.target === document.body) toggle(false)
    })
    document.addEventListener("keydown", event => event.key === "Escape" && toggle(false))
    this.replaceChildren(button)
  }
}

function createButton(label, icon) {
  const button = document.createElement("button")
  button.type = "button"
  button.setAttribute("aria-label", label)
  button.innerHTML = createIcon(icon)
  return button
}

function createIcon(paths) {
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${paths}</svg>`
}

customElements.define("htmlpress-nav-toggle", HtmlpressNavToggle)
