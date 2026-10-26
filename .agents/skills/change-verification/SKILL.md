---
name: change-verification
description: Use when checking a change works before calling it done — the smallest lint/type/unit slice and visual check for the diff, not the full gate. Covers check costs, the slice ladder, the docs guard and the commit gate.
---

# Verifying a change

**Verify the diff, not the repo.** `pnpm test` is lint + type + unit over everything, including
a full clippy and `cargo test`. A docs fix cannot break the page reader, so running every test
checks nothing. While iterating, run the **slice** that could catch this diff breaking. The full
gate belongs to the lefthook pre-push hook and CI; a commit runs the **commit gate** below.

- Never re-run a check that passed if nothing it covers has changed.
- Never chain two heavy commands in one call. One that fails hides the cost of the other.
- Say in the report which slice ran (e.g. "fmt+clippy, cargo test page::read"). Never write
  "tests pass" for a slice.

## What each check costs

| check                                        | catches                                    |
| -------------------------------------------- | ------------------------------------------ |
| `vp fmt --check <files>` / `vp lint <files>` | format, oxlint on TS, JSON, YAML, Markdown |
| `pnpm -C rust check:lint`                    | rustfmt and clippy over the crate          |
| `cargo test <name>` from `rust/`             | the tests whose path contains the name     |
| `pnpm -C rust test:unit`                     | every Rust unit and `tests/cli.rs`         |
| `pnpm -C typescript test:unit`               | the shim's units                           |
| `vp test .scripts/-test/docs.unit.ts`        | paths, scripts and skill tables in docs    |
| `pnpm build:docs`                            | the real build over `rust/docs/`           |

The first clippy or test run after a dependency change compiles for a minute; later ones take
seconds. The changed files come from `git diff --name-only main...` plus `git status --short`.

## The ladder

Pick the **highest** rung the diff reaches, and run everything that rung lists.

1. **Markdown, copy, a docs page.** `vp fmt --check` on Markdown files. For `AGENTS.md`,
   `README.md`, `CONTRIBUTING.md` or a skill, add the docs guard. For a page under `rust/docs/`, run
   `pnpm build:docs`: a page htmlpress cannot read fails there.
2. **Theme, components, layout** — `rust/src/layout.html`, `404.html` or anything under `rust/src/styles/`, `scripts/` or `components/`. `cargo test page::render`,
   then **look at it**: `pnpm start` (from a worktree, `--port 0`), open the page in Claude in
   Chrome or Playwright, in light and dark and at a phone width. No test renders CSS.
3. **Logic** — an action, model or command. `pnpm -C rust check:lint` and `cargo test` on the
   module. A new or changed test runs on its own first and fails before the fix. When the
   change is visible from a terminal (help text, `--json`, `--llms`), also run
   `cargo run --manifest-path rust/Cargo.toml -- <args>` and read the output. A dev-server
   change gets one live run: start it, edit a page, check one rebuild and one reload.
4. **Build-shaped** — dependencies, `rust/Cargo.toml`, a `vite.config.ts`, a `tsconfig`, the
   wrappers or the release scripts. Run `pnpm test`, then `pnpm build`; for the wrappers, the
   local checks in **`package-distribution`**.

## Guards nothing selects

`.scripts/-test/docs.unit.ts` reads documents from disk and imports nothing. Run it by path
whenever the diff touches `AGENTS.md`, `README.md`, `CONTRIBUTING.md`, a skill, a renamed path or
a renamed script.

## The commit gate

A commit on `main`, and landing a worktree branch (**`worktree-workflow`**), run the **commit
gate** once, at the end: `pnpm check` always, then a unit half **you choose**.

- **`pnpm test:unit`** when the diff is build-shaped, touches a model or `settings.rs`, or you
  are unsure.
- **Cherry-pick** otherwise: the `cargo test` filters the diff reaches, plus every guard above.
- The report names the path taken: "check + test:unit", or "check + cargo test page, docs guard".
