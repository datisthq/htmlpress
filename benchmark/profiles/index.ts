import { docs } from "./docs.ts"
import type { Profile } from "./types.ts"

export const PROFILES: Profile[] = [docs]

export function getProfile(name: string) {
  const profile = PROFILES.find(profile => profile.name === name)
  if (!profile) {
    const known = PROFILES.map(profile => profile.name).join(", ")
    throw new Error(`Unknown profile ${name}; known: ${known}`)
  }
  return profile
}
