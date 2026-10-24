import { BINARY_ENV, PACKAGE_NAME, PLATFORMS } from "../../settings.ts"

/** Find the htmlpress binary: the env override first, else the matching platform package. */
export function resolveBinary(props: {
  platform: string
  arch: string
  env: Record<string, string | undefined>
  resolvePackage: (specifier: string) => string
}) {
  const override = props.env[BINARY_ENV]
  if (override) return override

  const key = `${props.platform}-${props.arch}`
  const platform = PLATFORMS[key]
  if (!platform) {
    const supported = Object.keys(PLATFORMS).join(", ")
    throw new Error(
      `${PACKAGE_NAME} has no binary for ${key}; supported: ${supported}. ` +
        `Set ${BINARY_ENV} to a binary built with cargo to run it anyway.`,
    )
  }
  try {
    return props.resolvePackage(`${platform.packageName}/bin/${platform.binary}`)
  } catch {
    throw new Error(
      `${platform.packageName} is not installed. Reinstall ${PACKAGE_NAME} without ` +
        `--no-optional / --omit=optional so the binary for ${key} comes along.`,
    )
  }
}
