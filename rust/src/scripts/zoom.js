// Livemark's zoom: any image in a page opens full size, except one a link already wraps

function addZoom() {
  const content = document.querySelector("htmlpress-page > main")
  if (!content) return
  for (const image of content.querySelectorAll("img:not(a img)")) {
    image.classList.add("htmlpress-zoomable")
    image.addEventListener("click", () => openImage(image))
  }
}

function openImage(image) {
  const dialog = document.createElement("dialog")
  dialog.className = "htmlpress-zoom"
  const copy = image.cloneNode()
  copy.removeAttribute("width")
  copy.removeAttribute("height")
  copy.classList.remove("htmlpress-zoomable")
  dialog.append(copy)
  dialog.addEventListener("click", () => dialog.close())
  dialog.addEventListener("close", () => dialog.remove())
  document.body.append(dialog)
  dialog.showModal()
}

addZoom()
