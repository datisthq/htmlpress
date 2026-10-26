const PHONE = matchMedia("(max-width: 56rem)")

// Livemark's theme menu: light, dark or the system's choice, dropped down from the
// header or, on a phone, in a sheet; "T" flips light and dark
class HtmlpressThemeToggle extends HTMLElement {
  connectedCallback() {
    const details = this.querySelector(":scope > details")
    if (this.wired || !details) return
    this.wired = true
    const summary = details.querySelector("summary")
    const list = details.querySelector("ul")
    const sheet = document.createElement("htmlpress-sheet")
    sheet.setAttribute("label", summary.getAttribute("aria-label") ?? "")
    document.body.append(sheet)
    const mark = () => {
      const current = document.documentElement.dataset.theme ?? "system"
      for (const button of list.querySelectorAll("[data-choice]")) {
        if (button.dataset.choice === current) button.setAttribute("aria-current", "true")
        else button.removeAttribute("aria-current")
      }
    }
    const choose = choice => {
      const root = document.documentElement
      if (choice === "system") delete root.dataset.theme
      else root.dataset.theme = choice
      writeStorage("htmlpress-theme", choice === "system" ? null : choice)
      mark()
    }
    const pick = event => {
      const button = event.target.closest("[data-choice]")
      if (!button) return
      choose(button.dataset.choice)
      details.open = false
      sheet.removeAttribute("open")
    }
    list.addEventListener("click", pick)
    sheet.addEventListener("click", pick)
    summary.addEventListener("click", event => {
      if (!PHONE.matches) return
      event.preventDefault()
      sheet.append(list.cloneNode(true))
      sheet.setAttribute("open", "")
    })
    document.addEventListener("click", event => {
      if (!this.contains(event.target)) details.open = false
    })
    document.addEventListener("keydown", event => {
      if (event.key === "Escape") details.open = false
      if (!isShortcut(event, "t")) return
      const dark = matchMedia("(prefers-color-scheme: dark)").matches
      const current = document.documentElement.dataset.theme ?? (dark ? "dark" : "light")
      choose(current === "dark" ? "light" : "dark")
    })
    mark()
  }
}

// Storage throws in some private modes; a choice that cannot be kept is still applied
function writeStorage(key, value) {
  try {
    if (value === null) localStorage.removeItem(key)
    else localStorage.setItem(key, value)
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

customElements.define("htmlpress-theme-toggle", HtmlpressThemeToggle)
