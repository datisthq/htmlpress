// J and K scroll the page down and up by most of a screen and D to its end, as in
// Livemark; H and L follow the head's prev and next links, which cross sections

document.addEventListener("keydown", event => {
  const modified = event.metaKey || event.ctrlKey || event.altKey || event.shiftKey
  const key = event.key.toLowerCase()
  const step = { j: 0.8, k: -0.8 }[key]
  const rel = { h: "prev", l: "next" }[key]
  if (modified || !(step || rel || key === "d") || isTyping(event.target)) return
  const link = rel && document.querySelector(`link[rel="${rel}"]`)
  if (link) location.assign(link.href)
  else if (key === "d")
    scrollTo({ top: document.documentElement.scrollHeight, behavior: "smooth" })
  else if (step) scrollBy({ top: innerHeight * step, behavior: "smooth" })
})

function isTyping(target) {
  return (
    target instanceof HTMLElement &&
    (target.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName))
  )
}
