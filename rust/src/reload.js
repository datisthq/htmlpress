// Dev server only: long-poll the build version and reload when it moves.

// Module scripts have no document.currentScript, so find the tag by its attribute
const script = document.querySelector("script[data-endpoint]")
const endpoint = script?.dataset.endpoint ?? "/_htmlpress/reload"
let version

async function waitForBuild() {
  try {
    const query = version === undefined ? "" : `?since=${version}`
    const next = await (await fetch(endpoint + query)).text()
    if (version !== undefined && next !== version) return location.reload()
    version = next
  } catch {
    await new Promise(resolve => setTimeout(resolve, 1000))
  }
  waitForBuild()
}

waitForBuild()
