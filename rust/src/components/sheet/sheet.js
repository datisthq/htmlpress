const CLOSE = '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>'
const MOTION = matchMedia("(prefers-reduced-motion: no-preference)")

// Livemark's phone sheet: a host appends the content and sets `open`; a swipe down, the
// backdrop, Escape or the close button closes it, removing `open` again
class HtmlpressSheet extends HTMLElement {
  static observedAttributes = ["open", "label"]

  connectedCallback() {
    if (this.dialog) return
    const dialog = document.createElement("dialog")
    const header = document.createElement("header")
    this.heading = document.createElement("h2")
    const close = document.createElement("button")
    close.type = "button"
    close.setAttribute("aria-label", readMessages()["nav.close"] ?? "Close")
    close.innerHTML = `<svg viewBox="0 0 24 24" aria-hidden="true">${CLOSE}</svg>`
    close.addEventListener("click", () => this.removeAttribute("open"))
    header.append(this.heading, close)
    // Focused on open instead of the first row, which would scroll the list to it
    this.body = document.createElement("div")
    this.body.tabIndex = -1
    this.body.autofocus = true
    const handle = document.createElement("hr")
    dialog.append(handle, header, this.body)
    this.dialog = dialog
    this.wire(handle, header)
    this.append(dialog)
    this.sync()
  }

  attributeChangedCallback() {
    this.sync()
  }

  wire(...grips) {
    const dialog = this.dialog
    // Escape must not also reach the document, where it closes the phone sidebar
    dialog.addEventListener("keydown", event => {
      if (event.key === "Escape") event.stopPropagation()
    })
    dialog.addEventListener("cancel", event => {
      event.preventDefault()
      this.removeAttribute("open")
    })
    // Chrome closes a dialog outright on a repeated Escape, skipping `cancel`
    dialog.addEventListener("close", () => {
      document.documentElement.style.overflow = this.overflow ?? ""
      this.removeAttribute("open")
    })
    // The handle's margin is the dialog's own box too, so only a click outside it counts
    dialog.addEventListener("click", event => {
      if (event.target !== dialog) return
      const box = dialog.getBoundingClientRect()
      if (
        event.clientY < box.top ||
        event.clientX < box.left ||
        event.clientX > box.right
      )
        this.removeAttribute("open")
    })
    for (const grip of grips) {
      grip.addEventListener("pointerdown", event => this.drag(event))
    }
  }

  // Follows the finger down and closes past a quarter of the sheet or on a flick
  drag(start) {
    if (start.button !== 0 || start.target.closest("button")) return
    const dialog = this.dialog
    const grip = start.currentTarget
    grip.setPointerCapture(start.pointerId)
    let last = { y: start.clientY, time: start.timeStamp }
    let speed = 0
    let offset = 0
    dialog.dataset.dragging = ""
    const move = event => {
      offset = Math.max(0, event.clientY - start.clientY)
      speed = (event.clientY - last.y) / Math.max(1, event.timeStamp - last.time)
      last = { y: event.clientY, time: event.timeStamp }
      dialog.style.translate = `0 ${offset}px`
    }
    const end = event => {
      grip.removeEventListener("pointermove", move)
      grip.removeEventListener("pointerup", end)
      grip.removeEventListener("pointercancel", end)
      delete dialog.dataset.dragging
      dialog.style.translate = ""
      const far = offset > dialog.offsetHeight / 4 || speed > 0.5
      if (event.type === "pointerup" && far) this.removeAttribute("open")
    }
    grip.addEventListener("pointermove", move)
    grip.addEventListener("pointerup", end)
    grip.addEventListener("pointercancel", end)
  }

  sync() {
    const dialog = this.dialog
    if (!dialog) return
    const label = this.getAttribute("label") ?? ""
    dialog.setAttribute("aria-label", label)
    this.heading.textContent = label
    const open = this.hasAttribute("open")
    if (open && (!dialog.open || "closing" in dialog.dataset)) this.show()
    if (!open && dialog.open && !("closing" in dialog.dataset)) this.hide()
  }

  show() {
    const dialog = this.dialog
    const content = [...this.childNodes].filter(node => node !== dialog)
    if (content.length) this.body.replaceChildren(...content)
    delete dialog.dataset.closing
    if (dialog.open) return
    const root = document.documentElement
    this.overflow = root.style.overflow
    root.style.overflow = "hidden"
    dialog.showModal()
  }

  // Slides out before closing: a closed dialog leaves the top layer at once
  hide() {
    const dialog = this.dialog
    dialog.dataset.closing = ""
    const done = () => {
      if (!("closing" in dialog.dataset)) return
      delete dialog.dataset.closing
      dialog.close()
    }
    if (!MOTION.matches) return done()
    // transitionend bubbles: a row's hover fade must not cut the slide short
    const slid = event => {
      if (event.target !== dialog || event.propertyName !== "translate") return
      dialog.removeEventListener("transitionend", slid)
      done()
    }
    dialog.addEventListener("transitionend", slid)
    setTimeout(done, 400)
  }
}

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}

customElements.define("htmlpress-sheet", HtmlpressSheet)
