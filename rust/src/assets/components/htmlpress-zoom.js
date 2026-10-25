class HtmlpressZoom extends HTMLElement {
  connectedCallback() {
    const image = this.querySelector("img")
    if (!image) return
    image.addEventListener("click", () => {
      const dialog = document.createElement("dialog")
      const copy = image.cloneNode()
      copy.removeAttribute("width")
      copy.removeAttribute("height")
      dialog.append(copy)
      dialog.addEventListener("click", () => dialog.close())
      dialog.addEventListener("close", () => dialog.remove())
      this.append(dialog)
      dialog.showModal()
    })
  }
}

customElements.define("htmlpress-zoom", HtmlpressZoom)
