// Starts as a plain link, so a page without the script still points at the track
class HtmlpressSoundcloud extends HTMLElement {
  connectedCallback() {
    const link = this.querySelector("a[href]")
    if (!link) return
    const frame = document.createElement("iframe")
    frame.src = `https://w.soundcloud.com/player/?url=${encodeURIComponent(link.href)}`
    frame.title = link.textContent.trim() || link.href
    frame.loading = "lazy"
    this.replaceChildren(frame)
  }
}

customElements.define("htmlpress-soundcloud", HtmlpressSoundcloud)
