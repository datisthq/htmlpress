#!/bin/sh

# Installs the htmlpress binary from the GitHub releases
#
#   curl -fsSL https://htmlpress.dev/install.sh | sh
#
# HTMLPRESS_VERSION pins a version (0.2.0) instead of the latest; HTMLPRESS_PLATFORM
# overrides the detected platform (linux-x64, linux-arm64, macos-x64, macos-arm64,
# windows-x64, windows-arm64); HTMLPRESS_INSTALL_DIR overrides where the binary goes.

set -eu

REPO="datisthq/htmlpress"
PLATFORMS="linux-x64 linux-arm64 macos-x64 macos-arm64 windows-x64 windows-arm64"

main() {
  platform=${HTMLPRESS_PLATFORM:-$(detect_platform)}
  case " $PLATFORMS " in
    *" $platform "*) ;;
    *) fail "Unsupported platform \"$platform\": set HTMLPRESS_PLATFORM to one of: $PLATFORMS" ;;
  esac
  os=${platform%-*}

  version=${HTMLPRESS_VERSION:-$(find_version)}
  [ -n "$version" ] || fail "No htmlpress release yet: see https://github.com/$REPO/releases"
  asset=$(name_asset "$os" "$platform" "$version")

  log "Installing: htmlpress $version for $platform"
  tmp=$(mktemp -d)
  trap 'rm -rf "$tmp"' EXIT
  download_asset "v$version" "$asset" "$tmp"
  install_binary "$os" "$tmp/$asset" "$tmp"
}

detect_platform() {
  case "$(uname -m)" in
    arm64* | aarch64*) arch="arm64" ;;
    *64) arch="x64" ;;
    *) arch="$(uname -m)" ;;
  esac
  case "$(uname -s | tr '[:upper:]' '[:lower:]')" in
    linux) echo "linux-$arch" ;;
    darwin) echo "macos-$arch" ;;
    msys* | mingw* | cygwin* | *_nt*) echo "windows-$arch" ;;
    *) echo "unknown-$arch" ;;
  esac
}

find_version() {
  curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" |
    sed -n 's/.*"tag_name": *"v\([^"]*\)".*/\1/p' |
    head -n 1
}

name_asset() {
  case "$1" in
    windows) echo "htmlpress-$2-$3.zip" ;;
    *) echo "htmlpress-$2-$3.tar.gz" ;;
  esac
}

download_asset() {
  url="https://github.com/$REPO/releases/download/$1/$2"
  log "Downloading: $url"
  curl -fL --progress-bar "$url" -o "$3/$2"
  curl -fsSL "$url.sha256" -o "$3/$2.sha256"
  expected=$(cut -d ' ' -f 1 < "$3/$2.sha256")
  actual=$(hash_file "$3/$2")
  [ "$expected" = "$actual" ] || fail "Checksum mismatch for $2"
  log "Verified: sha256 $actual"
}

hash_file() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  else
    shasum -a 256 "$1" | cut -d ' ' -f 1
  fi
}

install_binary() {
  dir=${HTMLPRESS_INSTALL_DIR:-$HOME/.local/bin}
  case "$1" in
    windows)
      binary="htmlpress.exe"
      unzip -oq "$2" -d "$3" 2> /dev/null || tar -xf "$2" -C "$3"
      ;;
    *)
      binary="htmlpress"
      tar -xzf "$2" -C "$3"
      ;;
  esac
  mkdir -p "$dir"
  mv "$3/$binary" "$dir/$binary"
  chmod +x "$dir/$binary"
  log "Installed: $dir/$binary"
  warn_path "$dir"
  log "Done: run \"htmlpress --help\""
}

warn_path() {
  case ":$PATH:" in
    *":$1:"*) ;;
    *) log "Note: $1 is not on your PATH; add it, as in: export PATH=\"$1:\$PATH\"" ;;
  esac
}

log() {
  echo "$@" >&2
}

fail() {
  echo "Error: $*" >&2
  exit 1
}

main "$@"
