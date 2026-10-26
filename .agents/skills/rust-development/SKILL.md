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
- `src/icons.rs` — the built-in Lucide icons (`ICONS`, sorted by name for binary search),
  generated once from lucide-react's icon nodes; `icon/render` turns a name into an inline
  `<svg>`, and an unknown name fails the config load or the page
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
   first, then by `<meta name="order">` (negatives last), then by path; `nav/build` nests
   them by folder for the sidebar and the pager.
   `scan_site` resolves `<htmlpress-include>` (`page/include`, through `path/contain`) in a
   page's source before `read_page`, which has no filesystem access, so included content
   reaches the TOC, search and the Markdown copies like the page's own.
2. `read_page` first runs `text/enrich` (`$…$` math, auto links and emoji shortcodes, on whole text
   nodes outside links, code and `data-literal`), then `text/math` (every `<htmlpress-math>` to
   MathML with math-core, the LaTeX kept in `data-latex`), then lol_html over the source three times — facts (title, first `<h1>`, metas,
   whether a `<head>` exists), content (wrappers and layout-owned tags removed) and, only when a
   `<head>` exists, the extra head tags. lol_html builds no tree and adds no implied `<head>` or
   `<body>`: a fragment stays a fragment, which is why the head pass is skipped without one.
   The content pass also expands `htmlpress-package-tabs` into `htmlpress-tabs`: its text
   is collected, removed, and the sections are written from an end-tag handler, since the
   text is only complete there; the asset loader then sees the `htmlpress-tabs` tag
3. `render_page` runs lol_html over the layout, which is plain HTML: `layout.html`, or
   `layouts/<name>.html` for a page with `<meta name="layout">` (`scan_site` fails a page
   naming a missing one). The page's own project component tags are filled first, since
   its content goes in unparsed, but the tags it links CSS for are read before that; a
   project component keeps its tag, so its CSS scopes to it. It
   fills `<head>` and
   swaps each layout element (`htmlpress-content`, `htmlpress-nav`, `htmlpress-toc`, `htmlpress-pager`, …) for markup
   built from the site. Inserted markup is never re-parsed, so a page's own `htmlpress-*` tags
   are left to the browser; an element with nothing to show is removed. With `sections`,
   `find_section` (longest prefix) picks the open page's section, and the nav, pager,
   brand and breadcrumbs use only its pages and its `site*` overrides. Code blocks are
   highlighted in `render_page` (`code/highlight`, giallo with the dual themes inline
   as `light-dark()`), not in `read_page`, so search, the Markdown copies and
   `page.content` keep plain text; giallo's registry loads once per process. Line marks are attributes added to giallo's `giallo-l` line spans by
   position (the nth span is line n), and word marks only touch text between tags;
   `code/title` then frames every `<pre data-title>` in a captioned figure with
   its language logo (`src/language_icons.rs`, Livemark's six) and sizes
   `<pre data-collapse>`; inline `<code class="language-…">` is highlighted by the same
   pass run on `:not(pre) > code`. `scan_site` applies `patches` first (`page/patch`; a patched
   `.md` file is rendered by `markdown/render` into a new page) and refuses two pages on
   one URL, then adds the pages blog and
   changelog sections build (`section/generate`: a blog's index and tag pages, a page per
   `changelog/split` release) after reading git's dates (`git/read`, one `git log`), so a
   generated page never borrows its source file's date. With `base`, a last
   pass (`page/prefix`) prefixes every root-relative URL attribute of the finished page,
   so no URL builder needs to know the base; absolute URLs take it through
   `Config::read_origin`, and the text outputs through `Config::read_base`
4. `build_site` refuses an output that holds the project or sits visible inside it, since it
   wipes the output first, then writes pages, copies files and adds `_htmlpress/` assets,
   `robots.txt`, `sitemap.xml` and, when the project has none, the built-in `404.html`
   (`NOT_FOUND`), which the nav and pager never list. `index_site` writes `_htmlpress/search.json`
   (each page's text split at its `h2`/`h3`, words separated at element starts since
   lol_html may cut one text node into several chunks), which `htmlpress-search` fetches on
   first open. `describe_site` adds the agent files:
   each page as Markdown (`page/convert` first rewrites `htmlpress-*` components and bare
   `<pre>` into plain HTML, since htmd fences only `<pre><code>`), `llms.txt`, the skills
   `load_skills` matched, and `_headers`; a shipped file of the same name wins, except
   `_headers`, which is extended

## Languages

`scan_site` resolves `languages` first (`language/resolve`): each `SiteLanguage` holds its
prefix, its UI strings (`src/messages/<code>.yaml`, English holding every key, a unit test
checking the others match) and a copy of the config with every `Localized` text, section
prefix and link prefix in that language; `Site.config` is the default language's copy.
`language/detect` moves each page under its language's prefix and sets `Page.key`, the URL
translations share, which the sort, `Site.translations`, the picker and hreflang use.
`build_site` renders each language through `site/localize`: that language's config, its own
pages, and the default pages it lacks, marked `untranslated`, listed at the language's URL but
linked at their key and never built there. Scripts read their strings from the
`#htmlpress-messages` JSON in the head (`SCRIPT_MESSAGES`), each with its English fallback.

## Reading files a page or the config names

Anything read by a path from a page or `htmlpress.yaml` ends up published, so it goes
through `path/contain` (`contain_path`, `read_contained`): canonicalised, under the
project root after symlinks, never in a dot-entry (unless the caller allows it, as
`skills` does for `.skills/`), `node_modules/` or `.htmlpress/`, UTF-8 and at most 4 MB.
Never `root.join(path)` and read it directly.

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
(`AGENTS.md`); a change updates the commands in `rust/docs/introduction.html` and the shipped skill that names it.

## Assets

`src/layout.html`, `src/styles/`, `src/scripts/` and `src/components/` compile into the binary at
the paths a project's `.htmlpress/` escapes them under, and every file there is an escapable asset listed in
`ASSETS` (a unit test fails on one missing). `styles/` joins, in `THEME_STYLES` order, into the
global `theme.css`; `components/<name>/<name>.css` and `.js` are served one file each, and
`render_page` links only those whose tag the layout, the page or a project component's
`.html` holds. A component's CSS sits in `@scope (<tag>)` and its JS is a module that
defines that element alone, with its own copies of any helper: an escaped file must never
depend on another. Components are light DOM and must leave readable HTML when the script
never loads. A new component, token or asset path is public API: add it to
its `rust/docs/authoring/` article and the `htmlpress-authoring` (or, for an escape path,
`htmlpress-customization`) skill in the same commit, and look at it
in light, dark and phone width (**`change-verification`**).

- A component `<htmlpress-<name>>` lives in `components/<name>/<name>.{html,css,js}`, a
  project's own the same as a built-in: build and parse those paths only with
  `create_component_path` / `read_component_path` (`models/asset.rs`). `scan_site` refuses
  a file directly in `components/` and an `htmlpress-*` folder, since either would
  silently stop replacing a built-in
- Inside `@scope`, a bare `ul` ties the global `ul` on specificity and wins on scope
  proximity, but a global `li + li` still beats a scoped `li`: match the global selector's
  weight when a component resets it
- The layout's frame is components too (`htmlpress-shell`, `-header`, `-page`, …), each
  placing what it holds; `styles/layout.css` keeps only globals. A child that sets
  `display` on its own `:scope` beats a container's bare `htmlpress-brand`: write
  `:scope htmlpress-brand`, and an outer scope overriding the container needs one more type

- A running `htmlpress start` serves the assets it was compiled with: rebuild and restart it
  after editing one, or copy the file into `<root>/.htmlpress/build/_htmlpress/` to preview
- Fade text with a `color-mix(in srgb, … , <background>)`, never `opacity`: text under
  opacity loses subpixel antialiasing and reads blurred next to Livemark's
- `content.css` styles prose lists and links under `:where(main)`, at a bare type's
  weight, so a component's scoped rule wins; a component inside `main` must still reset
  every `li` margin and padding and give its links a `font-weight`, or it takes prose's
- Mix colors `in oklab`, never `in oklch`: mixing a hue with the neutral `--htmlpress-bg` in oklch
  swings the hue, and every callout tint comes out pink
- The theme toggle's dark set is written twice (media query and `[data-theme="dark"]`);
  change both
- `htmlpress-nav-toggle` sets `data-nav-open` (the phone drawer) or, wider,
  `data-nav-collapsed`, which the head script restores from storage before any CSS
  applies; keep collapse rules inside `(width > 56rem)` or a phone inherits a hidden sidebar
- Mobile drawer rules hang off `:has(htmlpress-nav-toggle:defined)`, which counts as a type plus a
  pseudo-class: an override needs the same `:has()` or it silently loses
