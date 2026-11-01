import { join } from "node:path"
import { parseArgs } from "node:util"
import { describeSite, generateSite } from "./generate/site.ts"
import { getProfile } from "./profiles/index.ts"
import { CACHE } from "./tools/project.ts"

const { values } = parseArgs({
  options: {
    profile: { type: "string", default: "docs" },
    size: { type: "string", default: "100" },
    seed: { type: "string", default: "1" },
  },
})

const site = await generateSite({
  profile: getProfile(values.profile),
  size: Number(values.size),
  directory: join(CACHE, "data"),
  seed: Number(values.seed),
})
console.log(`${describeSite(site)}\n${site.directory}`)
