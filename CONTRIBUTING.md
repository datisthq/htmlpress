# Contributing

Htmlpress is an open-source project and we'd love your help — whether that's fixing a bug, polishing the docs, or proposing a new feature. This guide walks through everything you need to get a change from your editor into a release.

## Quick start

You'll need [Node.js](https://nodejs.org/) 24+, [pnpm](https://pnpm.io/) 11+ and [rustup](https://rustup.rs/); for building the Python wheel, [uv](https://docs.astral.sh/uv/). Then:

```bash
git clone https://github.com/datisthq/htmlpress.git
cd htmlpress
pnpm install
pnpm start
```

`pnpm install` also fetches the Rust crates when `cargo` is on your `PATH`. `pnpm start` runs htmlpress from source over the docs site in `docs/`, with live reload. To run every check:

```bash
pnpm test
```

## Repository layout

```
htmlpress/
├── rust/                # the binary and the crates.io package `htmlpress`
│   ├── src/
│   │   ├── actions/     #   config/load, page/read + render, site/scan + build + serve
│   │   ├── commands/    #   the `build` and `start` commands (fireargs)
│   │   ├── models/      #   Config, Page, Site, BuildOptions, Error
│   │   ├── assets/      #   layout, theme, components, reload script — compiled in
│   │   ├── program.rs   #   the CLI
│   │   └── lib.rs       #   the library façade
│   └── tests/           #   CLI integration tests
├── typescript/          # the npm package `gethtmlpress`: a shim that runs the binary
│   ├── actions/         #   resolveBinary — find the platform package's binary
│   ├── entrypoints/     #   main.ts, the `htmlpress` bin
│   ├── scripts/         #   pack and publish the per-platform packages
│   └── settings.ts      #   the platform list
├── python/              # the PyPI package `htmlpress`: maturin binary wheels
├── docs/                # the docs site, HTML built by htmlpress itself
├── skills/              # shipped agent skills for people who use htmlpress
├── .agents/skills/      # repo skills for coding agents (see AGENTS.md)
├── .scripts/            # repo-wide guards and the `pnpm squash` script
└── vite.config.ts       # vite-plus shared config (format, lint, the guards' tests)
```

All behaviour lives in `rust/`. The npm and PyPI packages only deliver the same binary, so a feature never lands in a wrapper.

## Local commands

Run these at the repo root; each chains into the sections with `pnpm -C <section>`.

| Command            | What it does                                                     |
| ------------------ | ---------------------------------------------------------------- |
| `pnpm install`     | Install dependencies                                             |
| `pnpm build`       | Build the release binary, the npm shim and the Python wheel      |
| `pnpm check:types` | Type-check via `tsgo --noEmit` and `cargo check`                 |
| `pnpm check:lint`  | Format check + lint (`vp fmt`, `vp lint`, `cargo fmt`, `clippy`) |
| `pnpm check`       | `pnpm check:lint && pnpm check:types`                            |
| `pnpm format`      | Auto-fix formatting (`vp fmt`, `cargo fmt`)                      |
| `pnpm test:unit`   | Run the repo guards, the shim's tests and `cargo test`           |
| `pnpm test`        | `pnpm check && pnpm test:unit`                                   |
| `pnpm start`       | Serve the docs site from source with live reload                 |
| `pnpm build:docs`  | Production build of the docs site                                |
| `pnpm scan`        | Secret and vulnerability scans (gitleaks, semgrep)               |

A change is ready to push when `pnpm test` is green.

## Code conventions

- **Rust**: edition 2024 on the pinned stable toolchain, clippy-clean with no `unwrap`/`expect` outside tests; unit tests beside the module as `<module>_unit.rs`, run one with `cargo test <name>` from `rust/`.
- **TypeScript**: strict mode is on. Don't use `any`, `as`-casts, or non-null `!` without permission. Full ESM imports with the `.ts` extension; tests beside the module as `<module>.unit.ts`.
- **Comments**: docs on public items only. Skip narrative comments inside function bodies.
- **Docs pages**: semantic HTML and the documented `hp-*` components, no inline styles.
- **Formatting**: `pnpm format`. 2-space indent, no semicolons, double quotes (LF, UTF-8); rustfmt at 90 columns.

## Proposing a change

1. **Open an issue first** for anything bigger than a typo or a small bug. It's easier to align on direction before code.
2. **Branch from `main`** with a short descriptive name (e.g. `fix-fragment-title`, `feat-search`).
3. **Write a focused commit history**. Htmlpress uses [Conventional Commits](https://www.conventionalcommits.org/) — the prefix drives release-please:
   - `fix:` — bug fix → patch release
   - `feat:` — new feature → minor release
   - `chore:`, `docs:`, `refactor:`, `test:`, `perf:` — no release
   - Only changes to what an installer gets use `feat` or `fix`; never mark a commit breaking (`!` or a `BREAKING CHANGE:` footer)
4. **Run `pnpm test`** before pushing.
5. **Open a PR** against `main`. Describe the _why_ (link the issue), include a screenshot or CLI output for any visible change, and call out anything reviewers should pay extra attention to.

## Releases

Releases are driven by [release-please](https://github.com/googleapis/release-please): a `feat` or `fix` landing on `main` opens a Release PR, and merging it cuts the version, tags it, builds the binary for every platform and publishes it to crates.io, npm and PyPI at one shared version. Maintainers don't bump versions manually.

## Where to find help

- **Bugs and feature requests**: [GitHub Issues](https://github.com/datisthq/htmlpress/issues)
- **Discussion / ideas**: [GitHub Discussions](https://github.com/datisthq/htmlpress/discussions)
- **Source**: [github.com/datisthq/htmlpress](https://github.com/datisthq/htmlpress)

Thanks for helping make Htmlpress better.
