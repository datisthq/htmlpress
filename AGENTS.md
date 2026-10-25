# htmlpress

Htmlpress turns a folder of HTML pages into a site: one layout, one theme, a navigation and a live-reloading dev server, made for coding agents. It is one Rust binary in `rust/`, shipped three ways: crates.io (`cargo install htmlpress`), npm (`gethtmlpress` in `typescript/`, a shim plus one prebuilt binary per platform) and PyPI (`htmlpress` in `python/`, binary wheels built by maturin). The repo root holds the tooling shared across them and htmlpress' own docs site, written in HTML under `docs/` and built by htmlpress itself.

## Terminology

- **binary** — the `htmlpress` executable `rust/` compiles; the only place behaviour lives
- **wrapper** — a package that only delivers the binary: `typescript/` for npm, `python/` for PyPI. A wrapper never reimplements a command
- **platform package** — `gethtmlpress-<os>-<arch>`, one per entry in `PLATFORMS` (`typescript/settings.ts`), each carrying one binary; the main `gethtmlpress` package pins them all as `optionalDependencies` at its own version
- **project** — a folder htmlpress builds: HTML pages, other files copied as-is, an optional `htmlpress.toml`, and `.htmlpress/` holding the build output and escapes
- **page** — an `.html` file in a project, read by `rust/src/actions/page/read.rs` into title, order, head and content
- **escape** — a project file under `.htmlpress/` at a built-in asset's path (`ASSETS` in `rust/src/settings.rs`), replacing it: `layout.html`, `styles/*.css`, `scripts/*.js`, `components/<tag>.{css,js}`
- **component** — a custom element tag with up to three files, `components/<tag>.{html,css,js}`: its CSS scoped with `@scope (<tag>)`, its JS defining only that element, linked only on pages that use the tag; a project's own tag adds one, and its `.html` replaces the tag at build time
- **docs site** — `docs/`, this repo's own htmlpress project, built by `pnpm build:docs` and deployed to htmlpress.dev

## Rules

- **Never commit unless asked to** — except that a session working directly on `main` commits each task once done. The root's index is shared, so always commit with a pathspec: `git commit -m "…" -- <your paths>` — `git add` new files first, since a pathspec commit silently skips untracked ones.
- **Never add co-authored by Claude Code to commits!**
- **Never mark a commit breaking** — no `!` and no `BREAKING CHANGE:` footer; release-please turns either into a major bump.
- **Never push main or pull requests to origin unless asked to.**
- **Never amend or rewrite existing commits unless asked** (`--amend`, `reset`, `rebase`, force-squash). Fold changes in with a follow-up commit.
- **Never change code unless asked to.** Usually just answer.
- When resolving a TODO, follow its instructions literally. Prefer LSP capabilities where available.
- Start a plan from a summary of the new or changed API (CLI flags, page contract, models) and the hardest points.
- **Every plan ends by asking how to work it** (`AskUserQuestion`, before finalising): **the current branch** or **a `.worktrees/<name>` branch** (**`worktree-workflow`**).
- **Behaviour lives in the binary only.** A wrapper resolves and runs it, nothing more; a feature added to `typescript/` or `python/` would exist for one channel and not the others.
- **The page contract and the CLI are public API.** What a page's `<title>`, `<meta name="order">` and `<head>` mean, the `htmlpress-*` components and `--htmlpress-*` tokens, the layout elements an escape gets filled, the escapable asset paths, `htmlpress.toml` keys and every command's flags and `--json` output: renaming one silently breaks every project and agent relying on it. Treat it like an export change (**`change-review`**).
- **A platform entry is a release contract.** `PLATFORMS` in `typescript/settings.ts` and the build matrix in `.github/workflows/release.yaml` must list the same platforms with the same Rust targets; one missing from the matrix fails the npm pack (**`package-distribution`**).
- Rust: each `actions/<entity>/<verb>.rs` holds one action, named `verb_noun` (`actions/site/build.rs` exports `build_site`); the module tree is declared inline in `rust/src/lib.rs`. TypeScript: the same `actions/<entity>/<verb>.ts` shape, full relative imports with the `.ts` extension, no barrels.
- Don't destructure props in a TypeScript function signature — use `props.x`. Put higher-level items above the ones they compose, public before private.
- **Comments must not outnumber code**, and one is at most ~3 lines. The bar is: **name the specific mistake this prevents** — if you cannot, delete it. A file with none is normal. Rationale and history belong in git; only a non-obvious invariant (an ordering types cannot express, a failure that looks like success, a deliberate deviation from a rule here) earns lines.
- Doc comments only on public API — a brief line on what an exported item does; none on files.
- Update docs when a change requires it, and **after every task check whether a skill needs the lesson it taught** — update it in the same commit, or add one when no skill covers the area. Where a fact belongs: the **`human-documentation`** skill.
- **End every response that finishes a task with a status line: one bold sentence of about ten words, as the final paragraph, nothing after it** (e.g. **Ready to commit; lint, types and units green.**).

## Skills

Depth lives in skills: this file carries invariants, the skills carry procedure. Reach for one before working in its area. **A repo skill beats a plugin skill on the same subject** — `change-verification` over verification-before-completion, `change-review` over a generic code review. Each is named `<subject>-<facet>`, subject first.

Repo skills live in `.agents/skills/` (reached by Claude Code through the `.claude/skills` symlink `pnpm install` creates) and are never published:

| skill                  | when                                                                       |
| ---------------------- | -------------------------------------------------------------------------- |
| `change-review`        | reviewing a pull request or the working diff — also what CI review runs    |
| `change-verification`  | checking a change works — the smallest lint/type/unit slice and a look     |
| `human-documentation`  | the docs-truth pass before a commit, and where a fact belongs              |
| `issue-resolution`     | taking one issue or bug report to one tested commit in a worktree          |
| `package-distribution` | the npm and PyPI wrappers, platform packages and the release build matrix  |
| `rust-development`     | working in `rust/` — layout, cargo commands, the page pipeline, dev server |
| `worktree-workflow`    | starting, running and landing a `.worktrees/<name>` branch                 |

Shipped skills live in `.skills/` and teach coding agents to **use** htmlpress; they are linked from the docs site. Read the matching one before changing the behaviour it describes, and update it in the same commit:

| skill             | when                                                         |
| ----------------- | ------------------------------------------------------------ |
| `htmlpress-pages` | writing or editing pages in an htmlpress project, and checks |

**There is exactly one `AGENTS.md`, at the root.** `.scripts/-test/docs.unit.ts` enforces that, along with every path and `pnpm` command these documents name and the two tables above.

## Commands

- `pnpm install` — dependencies, lefthook, the `.claude/skills` symlink and `cargo fetch`
- `pnpm start` — `htmlpress start docs` from source, the docs site with live reload · `pnpm build:docs` — its production build into `docs/.htmlpress/build`
- `pnpm build` — the release binary (`cargo build --release`), the npm shim into `typescript/build/` with `tsgo`, then the wheel into `python/build/` with maturin (needs `uv`)
- `pnpm test` — full gate: lint + type + unit · `pnpm check` — lint + type
- `pnpm check:lint` — `vp fmt --check && vp lint`, then `cargo fmt --check` and clippy · `pnpm format` — `vp fmt`, `cargo fmt` · `pnpm check:types` — `tsgo --noEmit`, then `cargo check`
- `pnpm test:unit` — the repo guards, the shim's units, then `cargo test`. Single test: `cargo test name` from `rust/`, or `vp test path/to/file.unit.ts` from `typescript/`. While iterating, verify the slice the diff reaches, not the full gate (**`change-verification`**)
- `pnpm measure:coverage` — the unit tests with coverage
- `pnpm scan:leaks` / `pnpm scan:vulns` / `pnpm scan` — gitleaks / semgrep / both
- `pnpm update:deps` — bump npm dependencies past a 7-day cooldown
- `pnpm -C typescript pack:platforms --binaries <dir>` / `pnpm -C typescript release` / `pnpm -C python release` / `pnpm -C rust release` — lay out and publish npm / PyPI / crates.io; run by the release workflow only
- `pnpm squash <branch> -m "<subject>"` — land a worktree branch on local `main` as one commit (**`worktree-workflow`**)

## Code structure

The root is a private pnpm workspace (`package.json`, `pnpm-workspace.yaml`): format, lint, the repo guards and the scripts that chain each section with `pnpm -C <section>` — never `pnpm -r`.

- `rust/` — the crates.io package and the binary, a cargo project on the toolchain `rust/rust-toolchain.toml` pins (**`rust-development`**)
  - `src/actions/<entity>/<verb>.rs` — `asset/escape` and `asset/list`, `config/load`, `page/read` and `page/render`, `site/scan`, `site/build` and `site/serve`
  - `src/commands/` — the fireargs commands `build`, `start` and `escape`; `src/program.rs` the CLI; `src/main.rs` runs it
  - `src/models/`, `src/settings.rs` (paths and the embedded assets), `src/lib.rs` the façade; `src/assets/` the layout, `styles/`, `scripts/`, `components/` and the reload script compiled into the binary
  - Unit tests beside their module as `<module>_unit.rs`; `tests/` the CLI integration tests
- `typescript/` — the npm package `gethtmlpress`: `entrypoints/main.ts` runs the binary `actions/binary/resolve.ts` finds; `settings.ts` lists the platforms; `scripts/` packs and publishes the platform packages into `typescript/compile/` (**`package-distribution`**)
- `python/` — the PyPI package `htmlpress`: only `pyproject.toml`, which points maturin at `rust/Cargo.toml` with `bindings = "bin"`
- `docs/` — the docs site: HTML pages and `docs/htmlpress.toml`
- `.skills/` — the shipped agent skills
- `README.md`, `CONTRIBUTING.md` — the GitHub overview and the contributor guide; `rust/README.md`, `typescript/README.md` and `python/README.md` are each registry's page

Dot-directories hold non-package code: `.agents/` repo skills, `.skills/` shipped skills, `.release/` release-please config, `.scripts/` repo-wide guards (`.scripts/-test/`) and the `pnpm squash` script, whose tests the root `vite.config.ts` runs.

## Conventions

- pnpm `^11`, Node `^24`, ES Modules; vite-plus format and lint (2-space, LF, UTF-8, no semicolons, 90 columns)
- Strict TypeScript with null checks; no explicit return types. Never use `any`, an `as` cast or `!` without permission
- PascalCase types; camelCase values; TypeScript functions are **verbNoun** (`resolveBinary`)
- `package.json` script names are `verb[:object]` (`check:lint`, `build:docs`), never noun-first
- Specs: Vitest via `vite-plus/test`, `<module>.unit.ts`, no "Arrange / Act / Assert" comments
- Rust: edition 2024, rustfmt (90 columns), clippy with `-D warnings`; snake_case **verb_noun** functions; no `unwrap` / `expect` / `unsafe` outside tests (**`rust-development`**)
- HTML in `docs/` and `.skills/`: semantic elements and the documented `htmlpress-*` components only, no inline styles — the docs are the first project htmlpress must serve well
- Commits: Conventional Commits, typed by what an installer gets (below)

## Releases

Conventional Commits drive [release-please](https://github.com/googleapis/release-please): a `feat` or `fix` landing on `main` opens a Release PR, and merging that PR cuts the version, tags it and publishes. Merging is the gate — nothing ships without it.

- **One version for everything.** The repo is one release-please component at `.` (`release-type: simple`, `version.txt`), tagged `vX.Y.Z` with one root `CHANGELOG.md`; `rust/Cargo.toml` and `typescript/package.json` are its `extra-files`, and the wheel takes the crate's version through maturin. The manifest holds the current version and release-please maintains it — never hand-edit it
- **IMPORTANT: only changes to what an installer gets use `feat` or `fix`** — `rust/src/`, the `[package]` and `[dependencies]` tables of `rust/Cargo.toml`, `typescript/actions/`, `typescript/entrypoints/`, `typescript/settings.ts`, the `bin` and `files` of `typescript/package.json`, `python/pyproject.toml`. Everything else MUST be `chore` or `docs`: CI and workflows, release config, dependencies, tooling, `AGENTS.md`, `.agents/skills/`, the READMEs, the docs site, the shipped `.skills/` (linked from the site, not in any package)
- This is not a style preference. release-please reads the commit type: a `fix:` on a workflow file cuts a release and burns a version number on a change no consumer can observe. Version numbers can never be reused
- When in doubt, ask whether the change alters what someone installing the package gets. If it does not, it is `chore`
