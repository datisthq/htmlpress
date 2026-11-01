// Starts as the link the build writes, so a page without the script still points at the video
class HtmlpressVideo extends HTMLElement {
  connectedCallback() {
    if (this.getAttribute("type") !== "youtube") return
    const url = readUrl(this.getAttribute("url"))
    const id = url && readYoutubeId(url)
    if (!id) return
    const frame = document.createElement("iframe")
    frame.src = `https://www.youtube-nocookie.com/embed/${id}`
    frame.title = this.getAttribute("title") || url.href
    frame.loading = "lazy"
    frame.allow = "accelerometer; encrypted-media; gyroscope; picture-in-picture"
    frame.allowFullscreen = true
    this.replaceChildren(frame)
  }
}

function readUrl(value) {
  try {
    return new URL(value ?? "", document.baseURI)
  } catch {
    return null
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

customElements.define("htmlpress-video", HtmlpressVideo)
