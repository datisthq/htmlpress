// Light-DOM components: each enhances plain HTML that already reads well without them.

class HpTabs extends HTMLElement {
  connectedCallback() {
    if (this.querySelector(":scope > [role=tablist]")) return
    const sections = [...this.querySelectorAll(":scope > section")]
    const list = document.createElement("div")
    list.setAttribute("role", "tablist")
    const buttons = sections.map((section, index) => {
      const button = document.createElement("button")
      button.type = "button"
      button.setAttribute("role", "tab")
      button.textContent = section.getAttribute("title") ?? `Tab ${index + 1}`
      section.setAttribute("role", "tabpanel")
      button.addEventListener("click", () => this.select(index))
      list.append(button)
      return button
    })
    this.prepend(list)
    this.buttons = buttons
    this.sections = sections
    this.select(0)
  }

  select(index) {
    this.buttons.forEach((button, i) => button.setAttribute("aria-selected", i === index))
    this.sections.forEach((section, i) => (section.hidden = i !== index))
  }
}

customElements.define("hp-tabs", HpTabs)
