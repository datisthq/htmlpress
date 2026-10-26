// Dev server only: long-poll the build version and reload when it moves; a failed build is
// shown over the page instead, and cleared by the reload the next good build brings.

// Module scripts have no document.currentScript, so find the tag by its attribute
const script = document.querySelector("script[data-endpoint]")
const endpoint = script?.dataset.endpoint ?? "/_htmlpress/reload"
let version

async function waitForBuild() {
  try {
    const query = version === undefined ? "" : `?since=${version}`
    const [next, ...error] = (await (await fetch(endpoint + query)).text()).split("\n")
    const message = error.join("\n")
    if (message) showError(message)
    else if (version !== undefined && next !== version) return location.reload()
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
