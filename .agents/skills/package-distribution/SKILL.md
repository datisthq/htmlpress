---
name: package-distribution
description: Use when touching how htmlpress ships — the npm wrapper and its platform packages, the PyPI maturin wheel, crates.io, the release build matrix, versions, or adding a platform. Covers the invariants between settings, workflow and registries.
---

# Package distribution

One binary, three registries, one version. Nothing here may change what the binary does.

## npm: `gethtmlpress`

- `typescript/entrypoints/main.ts` is the `htmlpress` bin: it asks `resolveBinary`
  (`typescript/actions/binary/resolve.ts`) for a path and runs it with inherited stdio,
  passing the exit code and any signal through.
- `resolveBinary` takes `HTMLPRESS_BINARY` first (a cargo build, an unsupported platform),
  else resolves `gethtmlpress-<platform>-<arch>/bin/<binary>` from the installed packages.
- `PLATFORMS` in `typescript/settings.ts` is the single list. `scripts/pack.ts` reads it to lay
  out `typescript/compile/`: one platform package per entry (`os`, `cpu`, `bin/`) and the main
  package with `optionalDependencies` pinned to its own version. Those pins are written at pack
  time, never kept in `typescript/package.json`: a pin to an unpublished version would break
  `pnpm install` in this workspace.
- `scripts/release.ts` publishes platform packages before the main one and skips any version
  already on the registry, so rerunning a half-failed release finishes it.
- Never a `postinstall` download: pnpm 10+ blocks lifecycle scripts by default, and it breaks
  offline installs.
- Local check without publishing: build the binary, fill a folder with
  `<platform>/<binary>` for every entry, `pnpm -C typescript build`,
  `pnpm -C typescript pack:platforms --binaries <dir>`, then `npm pack` the folders and install
  the tarballs in a scratch project. Installing the folders directly symlinks them, and the
  shim then resolves from the symlink's real path and finds nothing.

## PyPI: `htmlpress`

`python/pyproject.toml` is the whole package: maturin with `bindings = "bin"` and
`manifest-path = "../rust/Cargo.toml"`, `dynamic = ["version"]` so the wheel takes the crate's
version. A wheel holds only the binary under `.data/scripts/`. CI builds one wheel per target
with `PyO3/maturin-action`; no sdist is published, since the crate sits outside `python/`.
Check locally: `pnpm -C python build`, then `uvx --from python/build/<wheel> htmlpress --version`.

## crates.io: `htmlpress`

`cargo publish` from `rust/`. The `include` list in `rust/Cargo.toml` keeps `src/` (with the
embedded assets), `README.md` and `LICENSE.md`; an asset outside `src/` would compile here and
fail for `cargo install`.

## The release workflow

`.github/workflows/release.yaml`: release-please → a `binaries` matrix (one job per platform:
`cargo build --release --target`, the wheel, both uploaded as artifacts) → `npm`, `pypi` and
`crates` jobs. Adding a platform means three edits in one commit: a `PLATFORMS` entry, a matrix
row with the same key and target, and the platform table in `rust/docs/introduction.html`. The pack
script refuses to run while any platform's binary is missing.

## Versions

One release-please component at `.`; `rust/Cargo.toml` and `typescript/package.json` are
`extra-files`, `version.txt` the simple strategy's own file. `rust/Cargo.lock` follows on the
next cargo build. Never bump any of them by hand.

## First publish

Trusted publishing only covers packages that exist: the first npm publish of each platform
package and the first `cargo publish` need tokens (`NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`); PyPI
accepts a pending trusted publisher for a new project.
