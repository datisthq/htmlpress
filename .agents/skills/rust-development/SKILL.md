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
- `src/models/` — `Asset` / `AssetStatus`, `Config`, `Page`, `Site`, `BuildOptions` /
  `BuildReport`, `Error`
- `Config` mirrors Livemark's config keys and is read from `htmlpress.yaml` by
  `serde-saphyr` (only its `deserialize` feature); `deny_unknown_fields` everywhere, and an
  empty file means the defaults, since an empty YAML document is not a mapping
- `src/commands/` — one fireargs command per file; `src/program.rs` mounts them
- `src/settings.rs` — every path constant, and `ASSETS`: each built-in asset embedded with
  `include_str!` through the `asset!` macro, keyed by its path under `.htmlpress/`
- Unit tests sit beside their module as `<module>_unit.rs`, wired in with
  `#[cfg(test)] #[path = "<module>_unit.rs"] mod unit;`; they return `crate::Result<()>` and use
  `?` instead of `unwrap`, which clippy denies

## The pipeline

1. `scan_site` lists only the include bases (`read_bases`: each glob's folders before its
   first wildcard), skipping dot-entries and `node_modules/`, so `.htmlpress/build` is
   never read back and a repo-root project never walks `rust/target`. `select_paths`
   sorts them by `include` (pages, `**/*.html` when unset) and `exclude` (dropped); other
   files under a base, `.html` outside `include` too, are copied. Every path is served
   without its base, so `Page.path` is the output path and `Page.source` the file's. Pages sort `index.html`
   first, then by `<meta name="order">`, then by path.
2. `read_page` runs lol_html over the source three times — facts (title, first `<h1>`, metas,
   whether a `<head>` exists), content (wrappers and layout-owned tags removed) and, only when a
   `<head>` exists, the extra head tags. lol_html builds no tree and adds no implied `<head>` or
   `<body>`: a fragment stays a fragment, which is why the head pass is skipped without one.
   The content pass also expands `htmlpress-package-tabs` into `htmlpress-tabs`: its text
   is collected, removed, and the sections are written from an end-tag handler, since the
   text is only complete there; the asset loader then sees the `htmlpress-tabs` tag
3. `render_page` runs lol_html over the layout, which is plain HTML: `layout.html`, or
   `layouts/<name>.html` for a page with `<meta name="layout">` (`scan_site` fails a page
   naming a missing one). The page's own project component tags are swapped first, since
   its content goes in unparsed, but the tags it links CSS for are read before the swap. It
   fills `<head>` and
   swaps each layout element (`htmlpress-content`, `htmlpress-nav`, `htmlpress-toc`, `htmlpress-pager`, …) for markup
   built from the site. Inserted markup is never re-parsed, so a page's own `htmlpress-*` tags
   are left to the browser; an element with nothing to show is removed. With `sections`,
   `find_section` (longest prefix) picks the open page's section, and the nav, pager,
   brand and breadcrumbs use only its pages and its `site*` overrides. With `base`, a last
   pass (`page/prefix`) prefixes every root-relative URL attribute of the finished page,
   so no URL builder needs to know the base; absolute URLs take it through
   `Config::read_origin`, and the text outputs through `Config::read_base`
4. `build_site` refuses an output that holds the project or sits visible inside it, since it
   wipes the output first, then writes pages, copies files and adds `_htmlpress/` assets,
   `robots.txt`, `sitemap.xml` and, when the project has none, the built-in `404.html`
   (`NOT_FOUND`), which the nav and pager never list. `describe_site` adds the agent files:
   each page as Markdown (`page/convert` first rewrites `htmlpress-*` components and bare
   `<pre>` into plain HTML, since htmd fences only `<pre><code>`), `llms.txt`, the skills
   `load_skills` matched, and `_headers`; a shipped file of the same name wins, except
   `_headers`, which is extended

## The dev server

Both servers strip the base from each request (`strip_base`) and redirect `/` to it.
`preview_site` builds without reload and serves through the same request loop, minus the
watcher. `serve_site` builds with `reload: true`, serves the output with tiny_http (a thread per
request) and rebuilds on notify events. Reload is a **long poll**, not SSE: tiny_http buffers
chunked responses, so a stream would sit unsent. `reload.js` asks
`/_htmlpress/reload?since=<version>` and the server answers when the build counter moves.

The watcher's filter is load-bearing: a build reads every source (inotify `Access` events) and
rewrites the output, so counting either would rebuild forever. `is_source_event` drops access
events, the output, other dot-folders, the bare `.htmlpress` folder and anything outside
the include bases and `htmlpress.yaml`, but keeps files inside `.htmlpress/` such as an
escape.

## The CLI

Commands are fireargs commands: `#[fireargs::input]` / `#[fireargs::output]` structs and an
async handler, so `--json` and `--llms` come free. Anything a command logs goes to **stderr**;
under `--json` stdout carries only the result. Flags, defaults and output fields are public API
(`AGENTS.md`); a change updates the commands in `rust/docs/introduction.html` and the `htmlpress-pages` skill.

## Assets

`src/layout.html`, `src/styles/`, `src/scripts/` and `src/components/` compile into the binary at
the paths a project's `.htmlpress/` escapes them under, and every file there is an escapable asset listed in
`ASSETS` (a unit test fails on one missing). `styles/` joins, in `THEME_STYLES` order, into the
global `theme.css`; `components/<tag>.css` and `.js` are served one file each, and
`render_page` links only those whose tag the layout, the page or a project component's
`.html` holds. A component's CSS sits in `@scope (<tag>)` and its JS is a module that
defines that element alone, with its own copies of any helper: an escaped file must never
depend on another. Components are light DOM and must leave readable HTML when the script
never loads. A new component, token or asset path is public API: add it to
its `rust/docs/authoring/` article and the `htmlpress-pages` skill in the same commit, and look at it
in light, dark and phone width (**`change-verification`**).

- Inside `@scope`, a bare `ul` ties the global `ul` on specificity and wins on scope
  proximity, but a global `li + li` still beats a scoped `li`: match the global selector's
  weight when a component resets it
- `styles/layout.css` only places components (`.htmlpress-header htmlpress-brand`); one
  that sets `display` on its own `:scope` beats a layout rule hiding it unless the layout
  rule carries a class

- A running `htmlpress start` serves the assets it was compiled with: rebuild and restart it
  after editing one, or copy the file into `<root>/.htmlpress/build/_htmlpress/` to preview
- Mix colors `in oklab`, never `in oklch`: mixing a hue with the neutral `--htmlpress-bg` in oklch
  swings the hue, and every callout tint comes out pink
- The theme toggle's dark set is written twice (media query and `[data-theme="dark"]`);
  change both
- Mobile drawer rules hang off `:has(htmlpress-nav-toggle:defined)`, which counts as a type plus a
  pseudo-class: an override needs the same `:has()` or it silently loses
