const ICON = '<path d="M4 6h16M4 12h16M4 18h16"/>'
const PHONE = matchMedia("(max-width: 56rem)")

// One button: on a phone it opens the sidebar as a drawer, wider it folds the sidebar
// away, and that choice is kept across pages
class HtmlpressNavToggle extends HTMLElement {
  connectedCallback() {
    const label = readMessages()["nav.toggleSidebar"] ?? "Toggle sidebar"
    const button = createButton(label, ICON)
    button.setAttribute("aria-controls", "htmlpress-sidebar")
    button.setAttribute("aria-keyshortcuts", "L")
    const root = document.documentElement
    const update = () => {
      const open = PHONE.matches
        ? root.hasAttribute("data-nav-open")
        : !root.hasAttribute("data-nav-collapsed")
      button.setAttribute("aria-expanded", open)
    }
    const toggle = open => {
      root.toggleAttribute("data-nav-open", open)
      update()
    }
    const flip = () => {
      if (PHONE.matches) return toggle(!root.hasAttribute("data-nav-open"))
      const collapsed = root.toggleAttribute("data-nav-collapsed")
      writeStorage("htmlpress-nav", collapsed ? "collapsed" : "open")
      update()
    }
    button.addEventListener("click", flip)
    document.addEventListener("click", event => {
      if (event.target === document.body) toggle(false)
    })
    document.addEventListener("keydown", event => {
      if (event.key === "Escape") toggle(false)
      if (isShortcut(event, "l")) flip()
    })
    PHONE.addEventListener("change", update)
    update()
    this.replaceChildren(button)
  }
}

function createButton(label, icon) {
  const button = document.createElement("button")
  button.type = "button"
  button.setAttribute("aria-label", label)
  button.innerHTML = createIcon(icon)
  return button
}

function createIcon(paths) {
  return `<svg viewBox="0 0 24 24" aria-hidden="true">${paths}</svg>`
}

// Storage throws in some private modes; a choice that cannot be kept is still applied
function writeStorage(key, value) {
  try {
    localStorage.setItem(key, value)
  } catch {}
}

function isShortcut(event, key) {
  const modified = event.metaKey || event.ctrlKey || event.altKey || event.shiftKey
  return !modified && event.key.toLowerCase() === key && !isTyping(event.target)
}

function isTyping(target) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  )
}

customElements.define("htmlpress-nav-toggle", HtmlpressNavToggle)

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}
