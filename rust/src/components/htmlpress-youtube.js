// Starts as a plain link, so a page without the script still points at the video
class HtmlpressYoutube extends HTMLElement {
  connectedCallback() {
    const link = this.querySelector("a[href]")
    const id = link && readYoutubeId(new URL(link.href))
    if (!id) return
    const frame = document.createElement("iframe")
    frame.src = `https://www.youtube-nocookie.com/embed/${id}`
    frame.title = link.textContent.trim() || link.href
    frame.loading = "lazy"
    frame.allow = "accelerometer; encrypted-media; gyroscope; picture-in-picture"
    frame.allowFullscreen = true
    this.replaceChildren(frame)
  }
}

function readYoutubeId(url) {
  if (url.hostname === "youtu.be") return url.pathname.slice(1)
  if (!url.hostname.endsWith("youtube.com")) return null
  const id = url.searchParams.get("v")
  if (id) return id
  const match = url.pathname.match(/^\/(?:embed|shorts|live)\/([^/]+)/)
  return match ? match[1] : null
}

customElements.define("htmlpress-youtube", HtmlpressYoutube)
