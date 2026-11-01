// Its buttons become tabs, each showing the element its `aria-controls` names and hiding
// the others' — mark all but the first `hidden` in the HTML, so the page reads without JS.

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
      const panel = document.getElementById(tab.getAttribute("aria-controls") ?? "")
      if (!panel) continue
      panel.setAttribute("role", "tabpanel")
      panel.setAttribute("aria-labelledby", tab.id)
      panel.hidden = !selected
    }
  }
}

customElements.define("htmlpress-landing-switch", LandingSwitch)
