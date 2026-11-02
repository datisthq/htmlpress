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
- `vendor/` holds dependencies htmlpress patches, wired in through `[patch.crates-io]`, as
  textlane's `rust/vendor/` does and copied from it; only the release binaries build with
  them, a crates.io build takes the published crate. `onig-regset` (giallo's Oniguruma
  binding) takes its global lock for the first `onig_new` only: with the lock on every
  compile, the parallel build ran slower than one thread. Drop a copy once upstream ships
  its fix, and keep it out of formatting (`vite.config.ts`'s `rust/vendor/**`)
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
   sorts them by `include` (pages: `.html` and `.md`, both `**/*.html` and `**/*.md` when
   unset) and `exclude` (dropped); other files under a base, `.html` outside `include` too,
   are copied, and so are an `.md` beside its `.html` page (the project's own Markdown
   copy), a changelog section's `source` and the `skills` files; `keep_partials` then turns
   back into files the `.md` pages a page includes (`include/list`, read from each source's
   tree, where Markdown's `::include` is already `<htmlpress-include>`), as before Markdown pages.
   A `.md` page is served at its `.html` path and compiled into hast
   (`page/compile`, no stringify: `create_document` makes its frontmatter, checked against
   `FRONTMATTER_KEYS`, the head's `<title>` and metas, a raw `head` parsed there,
   Livemark's directives, alerts and fences onto htmlpress' elements (`create_rules`, ported from Livemark's `create.ts`), any other `:::name` into `htmlpress-name` of a known component, the config's `plugins.md` after `md.directive` (`plugin/add`; every page pipeline is a textlane `Pipeline`, its textlane plugins by name and htmlpress' own through `use_native`, so a project's entry naming one merges into it as in Livemark; never build with `Processor::with`, which runs a duplicate twice), a fence's meta onto
   `data-*` by the `AnnotateCode` plugin, before `html.raw`, which drops nodes' `data`, and
   `code{:lang}` marked by `MarkInlineCode` after it). Every path is served
   without the longest include base it sits under (Livemark's `resolveArticlePath`, not the
   matched glob's) and slugified (`path/slugify`, each dot part of the name on its own so a
   `.es` suffix survives); a `path` meta or patch sets `Page.literal`, which
   `detect_language` never prefixes, and a suffixed translation without one takes its
   original's (`read_original_path`). Routes skip slugifying: `read_route_path` reads TanStack
   file-route names, their language from the URL prefix. So `Page.path` is the page's path without it
   (`guides/build.html`) and `Page.source` the file's. The nav, breadcrumbs and Markdown copies nest by `Page.path`; a page is
   written to `create_output_path` (`guides/build/index.html`) and served at `create_url`
   (`/guides/build/`), the root `404.html` excepted. Relative URLs become root-relative
   from the source folder (textlane's `html.pathRewrite` in the page pipeline, rooted at the
   page's include base, `list_links` as its `pages`), since every page sits one folder
   deeper, and a link naming a page's file takes its URL. `pathRewrite` matches `pages`
   keys verbatim, never decoding, so `list_links` lists every spelling (percent-encoded, a
   `.md` page's `.html`) per language, and pipelines cache by base and language; `page/rebase` does the same for
   the `image` meta, which no tree holds. Pages sort `index.html`
   first, then by `<meta name="order">` (negatives last), then by path; `nav/build` nests
   them as Livemark's `buildArticleTree` does, under their folder's `index.html` only, for the
   sidebar and the pager: a folder without one makes no entry.
   `scan_site` resolves `<htmlpress-include>` (`page/include`, through `path/contain`) in a
   page's tree before `read_source`, splicing in the partial's nodes, so included content
   reaches the facts, TOC, search and the Markdown copies like the page's own.
2. A page is parsed **once**, into one tree that lives from its parse to its content: no
   stage writes HTML that a later one parses again. `read_site` (behind `scan_site` and,
   rendering too, `build_site`) builds on every core in three steps: workers parse their
   share of the pages and report their includes, then their facts; the main thread alone
   settles what pages need of each other (partials, patches, languages, URLs, generated
   pages, links, each language's view); then each worker finishes and renders its own.
   A tree borrows its arena, which no other thread may touch, so a worker keeps its
   pages' trees from parse to HTML and only `Page` facts and output cross threads; a
   failure carries its page's index, so the first page in order is the one reported. Each
   worker (and the main thread, for generated and patched pages) holds one `Allocator`
   and a `Sources` cache (`models/source.rs`, `page/parse`): every page, partial
   and generated page is parsed into its arena once (`html.parse` through the project's
   `html` plugins, as a document when `is_document`, or compiled), and each use takes a
   `copy_node` of it, so a partial included twice stays as parsed. `read_source` reads the
   facts (title, metas, URL) from the tree, each page held as a `Draft` with its tree, since
   links need every page's URL; then `finish_page` runs `prepare_page` (package tabs,
   icons, fence tabs, `details[summary]`, cards, buttons, media links, themed images and
   mermaid blocks, each into plain markup), then the site's `Pipeline` with `run_sync`
   (`pipeline/create`, one per include base): `html.gemoji`, `html.linkify` and
   `html.math` on text outside links, code and `data-literal`, htmlpress' `WrapMath`
   (every formula into its `<htmlpress-math>`, the LaTeX in `data-latex`), `html.katex`
   (MathML), the config's `plugins.html`, `html.slug` (h2–h6, ids already in the page reserved), `html.pathRewrite` and
   `html.highlight`. From the same tree it reads the headings, writes the missing ids, splits
   the head from the content (`split_page`; a fragment's leading head tags go to the head,
   as a browser parses them), reads the search chunks (`site/index`'s `read_chunks`), and
   runs a `copy_node` of it through `SimplifyPage` and `html.toMarkdown` into
   `Page.markdown`, the agent copy. Then `decorate_page` turns the content into what the
   layout shows: the page's own `<htmlpress-toc>`s,
   callout titles in the page's language, `code/title`, project components filled (their
   tags read first, into `Page.components`), with `base`, `page/prefix`, and last its
   leading top-level `<h1>` moved into `Page.heading`. `render_page` puts that heading (or an
   `<h1>` of the title when the content has none) and the byline where the layout holds
   `<htmlpress-title>`, as Livemark's `Article` draws title and byline; a layout without the
   element, a bare route's included, gets the heading back before the content and no
   byline, so an older escaped `layout.html` renders as before minus the byline. Generated and
   patched pages take the same steps; the public `read_page` has no links, colors or
   decoration. Only then is the tree written, once: textlane writes `&`, `<`, `"` and `'` as
   hex references, which `name_references` names again. Never add a stage that parses
   `page.content` again, or one that reads a page's text instead of its tree.
3. `render_page` parses the layout, a template, into a tree of its own: `layout.html`, or
   `layouts/<name>.html` for a page with `<meta name="layout">` (`scan_site` fails a page
   naming a missing one), as a fragment unless it is a document, so a bare layout gains no
   `<html>` or `<body>`. It fills `<head>`, each project component's tag with its `.html`,
   and swaps each layout element (`htmlpress-nav`, `htmlpress-page-toc`, `htmlpress-pager`,
   …) for markup built from the site, as a raw node written as built: parsing it, as
   `parse_fragment` does, cost a fifth of the docs build, so only a `base`, which prefixes
   its URLs on the tree, parses it. `htmlpress-content` becomes the page's content as a
   raw node, written as the page pipeline left it, and the page's extra head tags go in
   the same way. Markup it fills in is not walked again; an element
   with nothing to show is removed. The CSS it links is for the layout's tags and
   `Page.components`. An escaped layout or component comes out as the same elements,
   respelled, never byte for byte. A route (`Page::is_route`, a file `add_routes` read from
   `.htmlpress/routes/`) without `<meta name="layout">` is drawn bare: its content is its
   layout, the one page content parsed again, so `complete_page` decorates it without the
   base and `render_page` prefixes it whole, or every URL takes the base twice. With `sections`,
   `find_section` (longest prefix) picks the open page's section, and the nav, pager,
   brand and breadcrumbs use only its pages and its `site*` overrides. Code blocks are
   highlighted in the page pipeline (`code/highlight` builds textlane's `html.highlight`
   options: the light theme inline, the dark one as `--shiki-dark` variables `content.css`
   switches to), so `page.content` holds colored code; the search index reads only text,
   and the Markdown copy drops the line numbers. The `data-*` line and word marks map onto its
   `lineAttributes`, `wordAttribute` and `lineNumberAttribute`, and `keepProperties` keeps
   `data-title` and `data-collapse` for
   `code/title` then frames every `<pre data-title>` in a captioned figure with
   its language logo (`src/language_icons.rs`, Livemark's six) and sizes
   `<pre data-collapse>`; inline `<code class="language-…">` is highlighted by the same
   pass (`inline: "class"`). `scan_site` applies `patches` first (`page/patch`; a patched
   `.md` file is compiled by `page/compile` into a new page) and refuses two pages on
   one URL, then adds the pages blog and
   changelog sections build (`section/generate`: a blog's index and tag pages, a page per
   `changelog/split` release) after reading git's dates (`git/read`, one `git log`), so a
   generated page never borrows its source file's date. With `base`, `page/prefix` prefixes
   every root-relative URL attribute, on the page's tree in `decorate_page` and on the
   layout's in `render_page`, which skips the raw content already done, so no URL builder
   needs to know the base; absolute URLs take it through `Config::read_origin`, and the text
   outputs through `Config::read_base`. The built-in `404.html` is read with the default
   language's decoration, so its links take the base too
4. `build_site` refuses an output that holds the project or sits visible inside it, since it
   writes into `<output>.next` and swaps it in whole (`create_build_paths`), since the dev
   server answers while it rebuilds and a half-written output serves pages without their CSS;
   it writes pages, copies files and adds `_htmlpress/` assets,
   `robots.txt`, `sitemap.xml` and, when the project has none, the built-in `404.html`
   (`NOT_FOUND`), which the nav and pager never list. `index_site` writes `_htmlpress/search.json`
   (`Page.chunks`, read in the pipeline: each page's text split at its `h2[id]`/`h3[id]`,
   words separated at element starts, a heading's text its chunk's heading only, and
   `<h1>`, `<pre>`, scripts, styles and SVGs left out), which `htmlpress-search` fetches on
   first open.
   The widget mirrors Livemark's: a result links `?highlight=<query>`, and the opened page
   paints the matches with the CSS Custom Highlight API, so the DOM stays untouched. `describe_site` adds the agent files:
   each page's `Page.markdown` (made in the pipeline: `page/simplify` reduces `htmlpress-*`
   components, bare `<pre>`, titles and line numbers on the tree, since `html.toMarkdown`
   fences only `<pre><code>`, then `html.toMarkdown` with `math` turns KaTeX's MathML back
   into `$…$`; `page/convert` only places it), `llms.txt`, the skills
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
They also answer the exact rules of the output's `_redirects` (`find_redirect`), where the
build sends a missing home to its first page, so `start` never paints the refresh stub
`index.html` that the build leaves for hosts without `_redirects`.
`preview_site` builds without reload and serves through the same request loop, minus the
watcher. `serve_site` builds with `reload: true`, serves the output with tiny_http (a thread per
request) and rebuilds on notify events. Reload is a **long poll**, not SSE: tiny_http buffers
chunked responses, so a stream would sit unsent. `reload.js` asks
`/_htmlpress/reload?since=<version>` and the server answers when the build counter moves,
with the version on the first line and a failed build's error after it. The counter starts at
the server's start time, not 0, or a tab open across a restart already holds the new
server's version and never reloads. `reload.js` retries every second on anything but a 200
with a numeric first line: another server on the port answers at once, so a script trusting
any answer polls it hundreds of times a second. A failed build never stops the server, not
even the first: while one stands, every page request gets the error page, and `reload.js` is
answered from memory. `base` lives in the reload state and is re-read after every build.

The watcher's filter is load-bearing: a build reads every source (inotify `Access` events) and
rewrites the output, so counting either would rebuild forever. `Sources::contains` drops access
events, the output and its `.next`/`.previous` siblings, other dot-folders, the bare
`.htmlpress` folder and anything outside the include bases, `htmlpress.yaml` and
`BuildReport.inputs` (`Site.inputs`: includes, changelog sources, patched files and skills,
which may sit outside every base or in `.skills/`), but keeps files inside `.htmlpress/` such
as an escape. `.htmlpress/cache/`, where a build stores GitHub releases, is dropped too: the
build writes it, so it would rebuild forever. The bases are re-read only on a change event
naming `htmlpress.yaml`: that read makes access events of its own, and parsing the config for
each of a build's thousands of output events held the next edit back by seconds. Only a
source event extends the 100 ms settle, or a `cargo build` beside the docs delays every
rebuild until it ends. The watcher watches the root's real path, since the filter compares
real paths and `start <symlink>` otherwise matched nothing. It never follows symlinks
(`create_watcher`): inotify keeps one watch per folder and notify names its events by the
last path that reached it, so pnpm's `node_modules/.pnpm/node_modules/htmlpress-rust` link
to `rust/` turned every `rust/docs` edit into a `node_modules/` path the filter dropped.

A save that changes only page content rebuilds that page alone (`rebuild_pages`, about
0.15 s against 1 s in debug for the docs): `build_cached` keeps the `Site` and `scan`'s
`Built` (the settled `Finish`, each source's index and `read_source` path), and
`reread_page` reads the page again, runs step 2's per-page settling, and compares
`read_facts` (the page without `head`, `content`, `toc`, `markdown`, `chunks`,
`components`, which only its own render and the indexes read) with the old facts. Equal,
it renders the page in place and `write_indexes` rewrites search, agent files, feeds and
the sitemap from the cached pages; anything else (a fact, a new include, a patched page,
a file that is no page, a failed build) returns None and the full build runs. A path
gone and never read, an editor's temp file, is skipped. A new field on `Page` that another
page reads must stay out of `read_facts`' cleared list, or its edits reach no other page.

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

- A component a script creates at runtime, like `htmlpress-sheet`, is in no page's markup,
  so its files are never linked: list the components whose scripts create it (`SHEET_HOSTS`
  in `page/render`), and append it to `<body>` so the creator's scoped CSS stays off it
- A running `htmlpress start` serves the assets it was compiled with: rebuild and restart it
  after editing one, or copy the file into `<root>/.htmlpress/build/_htmlpress/` to preview
- Fade text with a `color-mix(in srgb, … , <background>)`, never `opacity`: text under
  opacity loses subpixel antialiasing and reads blurred next to Livemark's
- `content.css` styles prose lists and links under `:where(main)`, at a bare type's
  weight, so a component's scoped rule wins; a component inside `main` must still reset
  every `li` margin and padding and give its links a `font-weight`, or it takes prose's
- Mix colors `in oklab`, never `in oklch`: mixing a hue with the neutral `--htmlpress-bg` in oklch
  swings the hue, and every callout tint comes out pink
- A component's per-theme colours can be one `light-dark(light, dark)`: the tokens set
  `color-scheme` for both the media query and the toggle, so it follows either
- The theme toggle's dark set is written twice (media query and `[data-theme="dark"]`);
  change both
- `htmlpress-nav-toggle` sets `data-nav-open` (the phone drawer) or, wider,
  `data-nav-collapsed`, which the head script restores from storage before any CSS
  applies; keep collapse rules inside `(width > 56rem)` or a phone inherits a hidden sidebar
- Mobile drawer rules hang off `:root[data-scripted]:has(htmlpress-nav-toggle)`, never
  `:defined`: the head script sets `data-scripted` before the first frame, while a component
  is defined only after its module runs, which flickers the header on every page. An override
  needs the same `:has()` or it silently loses
- Anything a script reshapes after load shows as a jump on every page, since each page is a
  fresh load the view transition snapshots on its first frame: build the markup in
  `render_page` and let the script adopt it, or add the script to `RENDER_BLOCKING`
- `<htmlpress-header>` and `<htmlpress-sidebar>` carry a `view-transition-name`, and a name
  must be unique per page or the browser skips the transition: never name a component a
  layout may hold twice, such as `htmlpress-brand`
