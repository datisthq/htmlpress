// Starts as the link the build writes, so a page without the script still points at the track
class HtmlpressAudio extends HTMLElement {
  connectedCallback() {
    const url = this.getAttribute("url")
    if (this.getAttribute("type") !== "soundcloud" || !url) return
    const frame = document.createElement("iframe")
    frame.src = `https://w.soundcloud.com/player/?url=${encodeURIComponent(url)}`
    frame.title = this.getAttribute("title") || url
    frame.loading = "lazy"
    this.replaceChildren(frame)
  }
}

customElements.define("htmlpress-audio", HtmlpressAudio)
