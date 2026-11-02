// Its buttons become tabs, each showing the elements its `aria-controls` names (the first
// is the tabpanel) and hiding the others' — mark all but the first tab's `hidden` in the
// HTML, so the page reads without JS.

class LandingSwitch extends HTMLElement {
  connectedCallback() {
    if (this.tabs) return
    this.tabs = [...this.querySelectorAll(":scope > button[aria-controls]")]
    this.setAttribute("role", "tablist")
    this.setAttribute("aria-label", this.getAttribute("label") ?? "Variant")
    for (const [index, tab] of this.tabs.entries()) {
      tab.setAttribute("role", "tab")
      tab.addEventListener("click", () => this.select(index))
    }
    this.addEventListener("keydown", event => {
      const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key]
      if (!step) return
      event.preventDefault()
      const count = this.tabs.length
      const active = this.tabs.findIndex(tab => tab.ariaSelected === "true")
      this.select((active + step + count) % count)
      this.tabs[(active + step + count) % count].focus()
    })
    this.select(0)
  }

  select(index) {
    for (const [position, tab] of this.tabs.entries()) {
      const selected = position === index
      tab.setAttribute("aria-selected", String(selected))
      tab.tabIndex = selected ? 0 : -1
      const ids = (tab.getAttribute("aria-controls") ?? "").split(/\s+/).filter(Boolean)
      for (const [order, id] of ids.entries()) {
        const element = document.getElementById(id)
        if (!element) continue
        if (order === 0) {
          element.setAttribute("role", "tabpanel")
          element.setAttribute("aria-labelledby", tab.id)
        }
        element.hidden = !selected
      }
    }
  }
}

customElements.define("htmlpress-landing-switch", LandingSwitch)
