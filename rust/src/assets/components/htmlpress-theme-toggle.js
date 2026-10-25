const ICON =
  '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/>'

class HtmlpressThemeToggle extends HTMLElement {
  connectedCallback() {
    const button = createButton("Toggle dark mode", ICON)
    button.addEventListener("click", () => {
      const root = document.documentElement
      const dark = matchMedia("(prefers-color-scheme: dark)").matches
      const current = root.dataset.theme ?? (dark ? "dark" : "light")
      root.dataset.theme = current === "dark" ? "light" : "dark"
      writeStorage("htmlpress-theme", root.dataset.theme)
    })
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

customElements.define("htmlpress-theme-toggle", HtmlpressThemeToggle)
