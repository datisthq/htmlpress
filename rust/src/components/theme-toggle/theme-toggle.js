const SUN =
  '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>'
const MOON =
  '<path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/>'

const SYSTEM =
  '<path d="M12 8a2.83 2.83 0 0 0 4 4 4 4 0 1 1-4-4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M6.3 17.7l-1.4 1.4M19.1 4.9l-1.4 1.4"/>'
const CHOICES = ["light", "dark", "system"]

// One button cycles light, dark and the system's choice; "T" flips light and dark
class HtmlpressThemeToggle extends HTMLElement {
  connectedCallback() {
    const messages = readMessages()
    const button = document.createElement("button")
    button.type = "button"
    button.setAttribute("aria-keyshortcuts", "T")
    button.innerHTML =
      createIcon("light", SUN) + createIcon("dark", MOON) + createIcon("system", SYSTEM)
    const show = choice => {
      const label = messages["theme.toggle"] ?? "Toggle dark mode"
      const name = messages[`theme.${choice}`] ?? choice
      button.dataset.choice = choice
      button.setAttribute("aria-label", `${label}: ${name}`)
      button.title = `${label}: ${name}`
    }
    const choose = choice => {
      const root = document.documentElement
      if (choice === "system") delete root.dataset.theme
      else root.dataset.theme = choice
      writeStorage("htmlpress-theme", choice === "system" ? null : choice)
      show(choice)
    }
    show(document.documentElement.dataset.theme ?? "system")
    button.addEventListener("click", () => {
      const index = CHOICES.indexOf(button.dataset.choice ?? "system")
      choose(CHOICES[(index + 1) % CHOICES.length])
    })
    document.addEventListener("keydown", event => {
      if (!isShortcut(event, "t")) return
      const dark = matchMedia("(prefers-color-scheme: dark)").matches
      const current = document.documentElement.dataset.theme ?? (dark ? "dark" : "light")
      choose(current === "dark" ? "light" : "dark")
    })
    this.replaceChildren(button)
  }
}

function createIcon(name, paths) {
  return `<svg viewBox="0 0 24 24" data-icon="${name}" aria-hidden="true">${paths}</svg>`
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

// The page's UI strings; English when a layout leaves them out
function readMessages() {
  try {
    return JSON.parse(document.getElementById("htmlpress-messages")?.textContent ?? "{}")
  } catch {
    return {}
  }
}
