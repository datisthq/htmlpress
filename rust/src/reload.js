// Dev server only: long-poll the build version and reload when it moves; a failed build is
// shown over the page instead, and cleared by the reload the next good build brings.

// Module scripts have no document.currentScript, so find the tag by its attribute
const script = document.querySelector("script[data-endpoint]")
const endpoint = script?.dataset.endpoint ?? "/_htmlpress/reload"
// The server's own error page already shows the error, so it only reloads
const errorPage = script?.hasAttribute("data-error-page") ?? false
let version

async function waitForBuild() {
  try {
    const query = version === undefined ? "" : `?since=${version}`
    const response = await fetch(endpoint + query)
    const [next, ...error] = (await response.text()).split("\n")
    // Another server on the port, or none: answering at once, it would be asked again
    // at once, hundreds of times a second, and its 404 page shown as a build error
    if (!response.ok || !/^\d+$/.test(next)) throw new Error("not the dev server")
    const message = error.join("\n")
    const changed = version !== undefined && next !== version
    if (changed && (!message || errorPage)) return location.reload()
    if (message && !errorPage) showError(message)
    version = next
  } catch {
    await new Promise(resolve => setTimeout(resolve, 1000))
  }
  waitForBuild()
}

function showError(message) {
  let overlay = document.getElementById("htmlpress-error")
  if (!overlay) {
    overlay = document.createElement("div")
    overlay.id = "htmlpress-error"
    overlay.setAttribute("role", "alert")
    overlay.style.cssText =
      "position:fixed;inset:auto 1rem 1rem;z-index:2147483647;max-height:50vh;" +
      "overflow:auto;padding:1rem 1.25rem;border-radius:0.5rem;font:14px/1.5 ui-monospace," +
      "monospace;white-space:pre-wrap;background:#2b0b0e;color:#ffd7d9;" +
      "box-shadow:0 8px 32px #0006;border:1px solid #ff6b78"
    document.body.append(overlay)
  }
  overlay.textContent = `Build failed\n\n${message}`
}

waitForBuild()
