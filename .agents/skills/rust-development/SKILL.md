---
name: rust-development
description: Use when working in `rust/` — the htmlpress binary and crate. Covers its layout, the cargo commands, the page pipeline (scan, read, render, build), the dev server and its watcher, the embedded assets and the CLI on fireargs.
---

# Rust development

`rust/` is the whole product: the crates.io package `htmlpress` and the binary every channel
ships. A library (`src/lib.rs`) holds everything; `src/main.rs` only runs
`htmlpress::create_program()`, so the integration tests drive the same CLI in-process.

## Commands

Run from the repo root; each wraps cargo in `rust/package.json`:

```sh
pnpm -C rust setup          # cargo fetch
pnpm -C rust check:lint     # cargo fmt --check + clippy --all-targets -D warnings
pnpm -C rust check:types    # cargo check --all-targets
pnpm -C rust test:unit      # cargo test: units and tests/cli.rs
pnpm -C rust build          # cargo build --release into rust/target/release/
```

A single test: `cargo test name` from `rust/`. `rust/rust-toolchain.toml` pins stable with
clippy and rustfmt, so rustup installs the right toolchain on first use, locally and in CI. A
run from source: `cargo run --manifest-path rust/Cargo.toml -- build docs`.

## Layout

- `src/actions/<entity>/<verb>.rs` exports `verb_noun`; the module tree is declared inline in
  `lib.rs`, so there is no `mod.rs`
- `src/models/` — `Config`, `Page`, `Site`, `BuildOptions` / `BuildReport`, `Error`
- `src/commands/` — one fireargs command per file; `src/program.rs` mounts them
- `src/settings.rs` — every path constant and the four assets, embedded with `include_str!`
- Unit tests sit beside their module as `<module>_unit.rs`, wired in with
  `#[cfg(test)] #[path = "<module>_unit.rs"] mod unit;`; they return `crate::Result<()>` and use
  `?` instead of `unwrap`, which clippy denies

## The pipeline

1. `scan_site` lists the project, skipping dot-entries and `node_modules/`, so `.htmlpress/build`
   is never read back. `.html` files become pages; the rest are copied. Pages sort `index.html`
   first, then by `<meta name="order">`, then by path.
2. `read_page` runs lol_html over the source three times — facts (title, first `<h1>`, metas,
   whether a `<head>` exists), content (wrappers and layout-owned tags removed) and, only when a
   `<head>` exists, the extra head tags. lol_html builds no tree and adds no implied `<head>` or
   `<body>`: a fragment stays a fragment, which is why the head pass is skipped without one
3. `render_page` runs lol_html over the layout, which is plain HTML: it fills `<head>` and
   swaps each layout element (`htmlpress-content`, `htmlpress-nav`, `htmlpress-toc`, `htmlpress-pager`, …) for markup
   built from the site. Inserted markup is never re-parsed, so a page's own `htmlpress-*` tags
   are left to the browser; an element with nothing to show is removed
4. `build_site` refuses an output that holds the project or sits visible inside it, since it
   wipes the output first, then writes pages, copies files and adds `_htmlpress/` assets

## The dev server

`serve_site` builds with `reload: true`, serves the output with tiny_http (a thread per
request) and rebuilds on notify events. Reload is a **long poll**, not SSE: tiny_http buffers
chunked responses, so a stream would sit unsent. `reload.js` asks
`/_htmlpress/reload?since=<version>` and the server answers when the build counter moves.

The watcher's filter is load-bearing: a build reads every source (inotify `Access` events) and
rewrites the output, so counting either would rebuild forever. `is_source_event` drops access
events, the output, other dot-folders and the bare `.htmlpress` folder, but keeps files inside
`.htmlpress/` such as the layout escape.

## The CLI

Commands are fireargs commands: `#[fireargs::input]` / `#[fireargs::output]` structs and an
async handler, so `--json` and `--llms` come free. Anything a command logs goes to **stderr**;
under `--json` stdout carries only the result. Flags, defaults and output fields are public API
(`AGENTS.md`); a change updates `docs/cli.html` and the `htmlpress-pages` skill.

## Assets

`src/assets/` compiles into the binary. `theme.css` styles semantic elements and the `htmlpress-*`
components through `--htmlpress-*` tokens; `components.js` holds light-DOM custom elements that must
leave readable HTML when the script never loads. A new component or token is public API: add
it to `docs/components.html` and the `htmlpress-pages` skill in the same commit, and look at it
in light, dark and phone width (**`change-verification`**).

- A running `htmlpress start` serves the assets it was compiled with: rebuild and restart it
  after editing one, or copy the file into `<root>/.htmlpress/build/_htmlpress/` to preview
- Mix colors `in oklab`, never `in oklch`: mixing a hue with the neutral `--htmlpress-bg` in oklch
  swings the hue, and every callout tint comes out pink
- The theme toggle's dark set is written twice (media query and `[data-theme="dark"]`);
  change both
- Mobile drawer rules hang off `:has(htmlpress-nav-toggle:defined)`, which counts as a type plus a
  pseudo-class: an override needs the same `:has()` or it silently loses
