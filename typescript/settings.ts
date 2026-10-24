export interface Platform {
  /** The npm package that carries this platform's binary. */
  packageName: string
  /** The Rust target the binary is compiled for. */
  target: string
  /** The binary's file name inside the package's `bin/`. */
  binary: string
}

/** The npm package users install; it pulls in the one platform package that matches. */
export const PACKAGE_NAME = "gethtmlpress"

/** An environment variable that points at a binary to run instead of the installed one. */
export const BINARY_ENV = "HTMLPRESS_BINARY"

/** Every published platform, keyed by `${process.platform}-${process.arch}`. */
export const PLATFORMS: Record<string, Platform> = {
  "linux-x64": createPlatform("linux-x64", "x86_64-unknown-linux-musl"),
  "linux-arm64": createPlatform("linux-arm64", "aarch64-unknown-linux-musl"),
  "darwin-x64": createPlatform("darwin-x64", "x86_64-apple-darwin"),
  "darwin-arm64": createPlatform("darwin-arm64", "aarch64-apple-darwin"),
  "win32-x64": createPlatform("win32-x64", "x86_64-pc-windows-msvc"),
  "win32-arm64": createPlatform("win32-arm64", "aarch64-pc-windows-msvc"),
}

function createPlatform(key: string, target: string): Platform {
  const binary = key.startsWith("win32") ? "htmlpress.exe" : "htmlpress"
  return { packageName: `${PACKAGE_NAME}-${key}`, target, binary }
}
