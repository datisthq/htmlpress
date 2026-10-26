const PHONE = matchMedia("(max-width: 56rem)")

// The section picker is a plain <details>; this closes it on a click outside or Escape
// and, on a phone, lists the sections in a sheet instead, as Livemark's sidebar does
class HtmlpressNav extends HTMLElement {
  connectedCallback() {
    const details = this.querySelector("[data-sections] details")
    if (this.wired || !details) return
    this.wired = true
    const sheet = document.createElement("htmlpress-sheet")
    sheet.setAttribute("label", readMessages()["nav.chooseSection"] ?? "Choose section")
    document.body.append(sheet)
    details.querySelector("summary").addEventListener("click", event => {
      if (!PHONE.matches) return
      event.preventDefault()
      sheet.append(details.querySelector("ul").cloneNode(true))
      sheet.setAttribute("open", "")
    })
    document.addEventListener("click", event => {
      if (!details.contains(event.target)) details.open = false
    })
    document.addEventListener("keydown", event => {
      if (event.key === "Escape") details.open = false
    })
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

customElements.define("htmlpress-nav", HtmlpressNav)
