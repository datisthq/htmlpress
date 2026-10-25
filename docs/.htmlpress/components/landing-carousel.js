// Every slide is in the HTML, stacked in one grid cell; only the active one is shown and
// reachable. A slide's source chip moves out of the window, so the window's crop misses it.

class LandingCarousel extends HTMLElement {
  connectedCallback() {
    if (this.querySelector(":scope > figure")) return
    const slides = [...this.querySelectorAll(":scope > section")]
    const frame = document.createElement("figure")
    const header = document.createElement("header")
    const dots = document.createElement("span")
    dots.setAttribute("aria-hidden", "true")
    const list = document.createElement("div")
    list.setAttribute("role", "tablist")
    const panels = document.createElement("div")
    const chips = document.createElement("aside")
    chips.setAttribute("aria-label", "Page source")
    this.items = slides.map((slide, index) => {
      const id = `landing-slide-${index + 1}`
      const tab = document.createElement("button")
      tab.type = "button"
      tab.id = `${id}-tab`
      tab.setAttribute("role", "tab")
      tab.setAttribute("aria-controls", id)
      const number = document.createElement("span")
      number.textContent = String(index + 1).padStart(2, "0")
      const title = document.createElement("span")
      title.textContent = slide.title
      tab.append(number, title)
      tab.addEventListener("click", () => this.stop(index))
      list.append(tab)
      slide.id = id
      slide.setAttribute("role", "tabpanel")
      slide.setAttribute("aria-labelledby", tab.id)
      slide.removeAttribute("title")
      panels.append(slide)
      const chip = slide.querySelector(":scope > figure")
      if (chip) chips.append(chip)
      return { tab, slide, chip }
    })
    list.addEventListener("keydown", event => this.moveFocus(event))
    header.append(dots, list)
    frame.append(header, panels)
    this.replaceChildren(frame, chips)
    this.addEventListener("pointerenter", () => this.toggleAttribute("paused", true))
    this.addEventListener("pointerleave", () => this.toggleAttribute("paused", false))
    this.addEventListener("focusin", () => this.toggleAttribute("paused", true))
    this.addEventListener("focusout", () => this.toggleAttribute("paused", false))
    this.select(0)
  }

  select(active) {
    this.active = active
    for (const [index, item] of this.items.entries()) {
      const selected = index === active
      item.tab.setAttribute("aria-selected", String(selected))
      item.tab.tabIndex = selected ? 0 : -1
      item.slide.inert = !selected
      if (item.chip) item.chip.inert = !selected
    }
    this.progress?.remove()
    // A fresh bar restarts the animation; its end, unless stopped, moves to the next slide
    this.progress = document.createElement("span")
    this.progress.setAttribute("aria-hidden", "true")
    this.progress.addEventListener("animationend", () => {
      if (!this.hasAttribute("stopped")) this.select((active + 1) % this.items.length)
    })
    this.items[active].tab.append(this.progress)
  }

  stop(index) {
    this.toggleAttribute("stopped", true)
    this.select(index)
  }

  moveFocus(event) {
    const last = this.items.length - 1
    const target = {
      ArrowLeft: this.active === 0 ? last : this.active - 1,
      ArrowRight: this.active === last ? 0 : this.active + 1,
      Home: 0,
      End: last,
    }[event.key]
    if (target === undefined) return
    event.preventDefault()
    this.stop(target)
    this.items[target].tab.focus()
  }
}

customElements.define("landing-carousel", LandingCarousel)
