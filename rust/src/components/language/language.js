const PHONE = matchMedia("(max-width: 56rem)")

// The menu is a plain <details>; this closes it on a click outside or Escape and, on a
// phone, lists the languages in a sheet instead
class HtmlpressLanguage extends HTMLElement {
  connectedCallback() {
    const details = this.querySelector("details")
    if (this.wired || !details) return
    this.wired = true
    const summary = details.querySelector("summary")
    const sheet = document.createElement("htmlpress-sheet")
    sheet.setAttribute("label", summary.getAttribute("aria-label") ?? "")
    document.body.append(sheet)
    summary.addEventListener("click", event => {
      if (!PHONE.matches) return
      event.preventDefault()
      sheet.append(details.querySelector("ul").cloneNode(true))
      sheet.setAttribute("open", "")
    })
    document.addEventListener("click", event => {
      if (!this.contains(event.target)) details.open = false
    })
    document.addEventListener("keydown", event => {
      if (event.key === "Escape") details.open = false
    })
  }
}

customElements.define("htmlpress-language", HtmlpressLanguage)
