// J and K scroll the page down and up by most of a screen, as in Livemark

document.addEventListener("keydown", event => {
  const modified = event.metaKey || event.ctrlKey || event.altKey || event.shiftKey
  const step = { j: 0.8, k: -0.8 }[event.key.toLowerCase()]
  if (modified || !step || isTyping(event.target)) return
  scrollBy({ top: innerHeight * step, behavior: "smooth" })
})

function isTyping(target) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  )
}
