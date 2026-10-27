// J and K scroll the page down and up by most of a screen and D to its end, as in Livemark

document.addEventListener("keydown", event => {
  const modified = event.metaKey || event.ctrlKey || event.altKey || event.shiftKey
  const step = { j: 0.8, k: -0.8 }[event.key.toLowerCase()]
  const end = event.key.toLowerCase() === "d"
  if (modified || !(step || end) || isTyping(event.target)) return
  if (end) scrollTo({ top: document.documentElement.scrollHeight, behavior: "smooth" })
  else scrollBy({ top: innerHeight * step, behavior: "smooth" })
})

function isTyping(target) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  )
}
