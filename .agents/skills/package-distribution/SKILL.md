---
name: package-distribution
description: Use when touching how htmlpress ships — the npm wrapper and its platform packages, the PyPI maturin wheel, crates.io, the release build matrix, versions, or adding a platform. Covers the invariants between settings, workflow and registries.
---

# Package distribution

One binary, three registries plus GitHub releases, one version. Nothing here may change what the binary does.

## npm: `gethtmlpress`

- `typescript/entrypoints/main.ts` is the `htmlpress` bin: it asks `resolveBinary`
  (`typescript/actions/binary/resolve.ts`) for a path and runs it with inherited stdio,
  passing the exit code and any signal through.
- `resolveBinary` takes `HTMLPRESS_BINARY` first (a cargo build, an unsupported platform),
  else resolves `@htmlpress/<platform>-<arch>/bin/<binary>` from the installed packages.
- `PLATFORMS` in `typescript/settings.ts` is the single list. `scripts/pack.ts` reads it to lay
  out `typescript/compile/`: one platform package per entry (`os`, `cpu`, `bin/`) and the main
  package with `optionalDependencies` pinned to its own version. Those pins are written at pack
  time, never kept in `typescript/package.json`: a pin to an unpublished version would break
  `pnpm install` in this workspace.
- The main package also carries `.skills/` as `skills/`, copied by `scripts/pack.ts`, so
  `npx skills add ./node_modules/gethtmlpress/skills` installs the version in the lockfile.
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
version. A wheel holds the binary under `.data/scripts/` and the skills under
`.data/data/share/htmlpress/skills/`, installed to `<venv>/share/htmlpress/skills`. maturin
reads them from the `htmlpress.data` folder beside `pyproject.toml` (its `data` key), which
`python/scripts/copy-skills.sh` stages and git ignores: the local build and the release
workflow's Stage Skills step both run it, and a wheel built without it ships no skills. CI builds one wheel per target
with `PyO3/maturin-action`; no sdist is published, since the crate sits outside `python/`. The
static musl wheels are retagged `manylinux_2_17` as well (Tag Manylinux), or pip on a glibc
Linux finds no wheel at all. maturin builds those wheels in Docker as root, so the step
chowns `python/build` first; without it `wheel tags --remove` fails with a PermissionError.
Check locally: `pnpm -C python build`, then `uvx --from python/build/<wheel> htmlpress --version`.

## crates.io: `htmlpress`

`cargo publish` from `rust/`. The `include` list in `rust/Cargo.toml` keeps `src/` (with the
embedded assets), `README.md` and `LICENSE.md`; an asset outside `src/` would compile here and
fail for `cargo install`.

## GitHub releases and `install.sh`

The `github` job packs each `binaries` artifact as `htmlpress-<os>-<arch>-<version>.tar.gz`
(`.zip` on Windows) and attaches them to the release-please release, with no `.sha256` files:
GitHub records every asset's sha256 as its `digest` in the releases API.
`<os>` is `linux`, `macos` or `windows`, not the npm `darwin`/`win32`. `rust/docs/install.sh`,
served at `https://htmlpress.dev/install.sh`, builds the same names from the latest release's
tag, verifies the download against that release's `digest` for the asset and installs to
`~/.local/bin`: renaming an asset, or adding a
platform without adding it to the script's `PLATFORMS`, breaks every curl install.
Check locally by running the Pack Binaries script on a `binaries/<platform>/` folder and the
script with a `curl` stub on `PATH` that serves those files.

## The release workflow

`.github/workflows/release.yaml`: release-please → a `binaries` matrix (one job per platform:
`cargo build --release --target`, the wheel, both uploaded as artifacts) → `github`, `npm`, `pypi`
and `crates` jobs. Adding a platform means four edits in one commit: a `PLATFORMS` entry, a matrix
row with the same key and target, the platform table in `rust/docs/introduction.html` and
the `PLATFORMS` list in `rust/docs/install.sh`. The pack
script refuses to run while any platform's binary is missing.

A run that failed after release-please tagged cannot be fixed by a re-run: it reuses the old
workflow file and `release_created` stays false on later pushes. Delete the GitHub release and
tag, relabel the merged Release PR `autorelease: pending`, then push the fix to `main`;
release-please re-creates the release and the fixed workflow publishes it.

## Versions

One release-please component at `.`; `rust/Cargo.toml`, `rust/Cargo.lock` and
`typescript/package.json` are `extra-files`, `version.txt` the simple strategy's own file. Never
bump any of them by hand. The lockfile entry needs `@.name.value` in its jsonpath, since
release-please wraps TOML values; without it `cargo publish` finds a dirty lockfile and refuses.

## First publish

Trusted publishing only covers packages that exist: the first npm publish of each platform
package and the first `cargo publish` need tokens (`NPM_TOKEN`, `CARGO_REGISTRY_TOKEN`); PyPI
accepts a pending trusted publisher for a new project. Publish a `0.0.0` placeholder by hand so
release-please's first version still reaches CI; a token made for that first publish may lack
`publish-update` and cannot publish later versions. Unscoped npm names can trip npm's spam
check, hence the `@htmlpress` scope.
