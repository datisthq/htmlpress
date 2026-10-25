// An <abbr title> shows its full form, and a footnote reference its note, on hover and
// focus. Without the script, the title stays a native tooltip and the link still jumps.

const hint = document.createElement("div")
hint.id = "htmlpress-hint"
hint.className = "htmlpress-hint"
hint.setAttribute("role", "tooltip")
hint.hidden = true

function addHints() {
  const content = document.querySelector(".htmlpress-content")
  if (!content) return
  for (const abbr of content.querySelectorAll("abbr[title]")) {
    const title = abbr.title
    // Moved off the element, or the browser's own tooltip shows on top of this one
    abbr.removeAttribute("title")
    abbr.dataset.title = title
    abbr.tabIndex = 0
    addHint(abbr, () => [document.createTextNode(title)])
  }
  for (const link of content.querySelectorAll("sup > a[href^='#']")) {
    const note = document.getElementById(decodeURIComponent(link.hash.slice(1)))
    if (note?.tagName !== "LI") continue
    addHint(link, () => readNote(note, link))
  }
  document.body.append(hint)
}

function addHint(anchor, read) {
  anchor.setAttribute("aria-describedby", hint.id)
  const show = () => {
    hint.replaceChildren(...read())
    hint.hidden = false
    const box = anchor.getBoundingClientRect()
    const width = hint.offsetWidth
    const left = Math.min(
      Math.max(8, box.left + box.width / 2 - width / 2),
      innerWidth - width - 8,
    )
    hint.style.left = `${left + scrollX}px`
    hint.style.top = `${box.bottom + scrollY + 6}px`
  }
  const hide = () => (hint.hidden = true)
  anchor.addEventListener("mouseenter", show)
  anchor.addEventListener("focus", show)
  anchor.addEventListener("mouseleave", hide)
  anchor.addEventListener("blur", hide)
}

// The note without its links back to the reference
function readNote(note, link) {
  const copy = note.cloneNode(true)
  for (const back of copy.querySelectorAll("a[href^='#']")) {
    const target = back.hash.slice(1)
    const returns = target === link.id || target === link.parentElement?.id
    if (returns || back.hasAttribute("data-footnote-backref")) back.remove()
  }
  copy.removeAttribute("id")
  return [...copy.childNodes]
}

addHints()
