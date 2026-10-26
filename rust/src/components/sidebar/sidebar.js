const KEY = "htmlpress-sidebar-scroll"

// Each page is a fresh load, so the sidebar would jump back to the top on every link;
// keep its scroll across pages as Livemark's single-page sidebar does
class HtmlpressSidebar extends HTMLElement {
  connectedCallback() {
    this.restore()
    const save = () => writeStorage(KEY, String(this.scrollTop))
    // A prerendered next page is revealed before this one's pagehide, so a click saves too
    this.addEventListener("click", event => event.target.closest("a") && save())
    addEventListener("pagehide", save)
    document.addEventListener("prerenderingchange", () => this.restore())
  }

  // A first visit starts at the top, as Livemark's does; only a walk between pages
  // keeps the scroll and brings the open page into view
  restore() {
    const saved = readStorage(KEY)
    if (saved === null) return
    this.scrollTop = Number(saved)
    this.revealCurrent()
  }

  // scrollIntoView would scroll the page as well; only the sidebar may move
  revealCurrent() {
    const link = this.querySelector("a[aria-current='page']")
    if (!link) return
    const box = this.getBoundingClientRect()
    const { top, bottom } = link.getBoundingClientRect()
    if (top >= box.top && bottom <= box.bottom) return
    this.scrollTop += top - box.top - (box.height - (bottom - top)) / 2
  }
}

// Storage throws in some private modes; the sidebar then just starts at the top
function readStorage(key) {
  try {
    return sessionStorage.getItem(key)
  } catch {
    return null
  }
}

function writeStorage(key, value) {
  try {
    sessionStorage.setItem(key, value)
  } catch {}
}

customElements.define("htmlpress-sidebar", HtmlpressSidebar)
