---
name: htmlpress-pages
description: Use when writing or editing pages in an htmlpress project — a folder of HTML built into a site. Covers the page contract (title, order, head), the theme and htmlpress-* components, the config, and checking the result with build --json and start.
---

# Htmlpress pages

An htmlpress project is a folder of `.html` pages. Htmlpress wraps each in one layout with a
navigation and a theme, so a page carries content only: no `<nav>`, no site header, no styling.

## A page

```html
<title>Getting started</title>
<meta name="description" content="Install and run it in a minute." />
<meta name="order" content="1" />

<h1>Getting started</h1>
<p>Plain semantic HTML.</p>
```

- A full document (`<!doctype html>…<body>`) and a bare fragment are read the same way.
- `<title>` names the page in the navigation and the tab; without it the first `<h1>`, then the
  file name. Give every page one `<title>` and one `<h1>`.
- `<meta name="order" content="N">` (an integer) sorts the navigation, lower first.
  `index.html` always comes first; unordered pages follow by path.
- Other `<head>` tags (`<link>`, `<style>`, `<script>`) are kept and added after the theme.
- `<h2>` and `<h3>` outside components make the page's table of contents; one without an
  `id` gets a slug of its text. Give a heading an `id` only when a link needs it fixed.
- A heading takes `data-toc="false"` (shown, not in the TOC) or `data-toc="only"` (in the TOC,
  not shown); `<htmlpress-toc level="2">` in a page renders its TOC inline, and an image
  with `data-theme="light"` or `"dark"` shows in that theme only.
- The navigation groups pages by top folder (`guides/…`), named by `guides/index.html`, whose title links to it.
- Pages are written to the same path, so link between them with relative hrefs
  (`guides/build.html`). Images and other files are copied as-is.

## Styling: semantic HTML first

The theme styles the elements themselves: headings, `<p>`, lists, `<table>`, `<pre><code>`,
`<blockquote>`, `<details>`/`<summary>`, `<figure>`/`<figcaption>`, `<dl>`, `<kbd>`, `<hr>`.

- No inline `style`, no classes for looks, no utility CSS. The point is that every page looks
  like the same site.
- To change the look site-wide, override a token in a stylesheet: `--htmlpress-accent`, `--htmlpress-font`,
  `--htmlpress-mono`, `--htmlpress-bg`, `--htmlpress-surface`, `--htmlpress-text`, `--htmlpress-muted`, `--htmlpress-border`,
  `--htmlpress-radius`, `--htmlpress-width`.
- LaTeX: `$E = mc^2$` inline and `$$…$$` as a block (pandoc's rule keeps `$5 and $10` text;
  `\$` is a dollar), or `<htmlpress-math display>…</htmlpress-math>`; rendered to MathML at
  build time, and invalid LaTeX fails the build.
- `<htmlpress-include src="partial.html">` pastes in another file by path (relative to
  the page): `.html` body, rendered `.md`, or any other file as a titled code block that
  takes `data-lines` and the other code block attributes. Exclude the partials folder.
- Bare URLs and emails in prose become links and GitHub shortcodes (`:rocket:`) become emoji
  at build time; text in links, code and `data-literal` elements stays as written.
- Escape code inside `<pre><code>`: `&lt;` and `&amp;`. Name its language,
  `<pre><code class="language-rust">`, to highlight it at build time; `data-lines="2,4-5"`
  marks lines, `data-line-numbers` numbers them, `data-add`/`data-remove` show a diff,
  `data-focus` dims the other lines, `data-error`/`data-warning` flag lines and
  `data-words="a,b"` marks words. `<pre data-title="file.yaml">` adds a title bar
  with the language's logo, `<pre data-collapse="10">` shows 10 lines with an Expand bar,
  and an inline `<code class="language-js">` is highlighted too. `codeThemeLight`/`codeThemeDark` in
  the config pick the themes.

## Components

Only these; anything else is plain HTML.

| Element                                          | Use                                                                                 |
| ------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `<htmlpress-callout type="…" label="…">`         | a note; `type` is `info` (default), `success`, `warning`, `danger`                  |
| `<htmlpress-grid cols="2">` + `<htmlpress-card>` | cards in columns (`cols` 2–4, else responsive); `<h3><a>` links a card              |
| `<htmlpress-tabs sync="…">` + `<section title>`  | tabs, one per `<section>`; tabs sharing `sync` switch together                      |
| `<htmlpress-steps>` + `<ol>`                     | numbered steps, one per `<li>`, each led by an `<h3>`                               |
| `<htmlpress-badge type="…">`                     | an inline label; `type` as a callout's, or `neutral`                                |
| `<htmlpress-file-tree>` + nested `<ul>`          | a file tree; an `<li>` holding a `<ul>` is a folder                                 |
| `<htmlpress-table>` + `<table>`                  | a table sorted by a click on a `<thead>` header                                     |
| `<htmlpress-zoom>` + `<img>`                     | an image that opens full screen on click                                            |
| `<htmlpress-icon name="…">`                      | an inline Lucide icon; names at htmlpress.dev/authoring/rich-elements.html#icons    |
| `<htmlpress-button type="…">` + `<a>`            | a call-to-action link; `type="secondary"` for the quieter one                       |
| `<htmlpress-package-tabs>` + `<pre><code>`       | one `npm`/`npx` command, built into npm, pnpm, yarn and bun tabs                    |
| `<htmlpress-terminal title="…">` + `<pre>`       | terminal output: `<kbd>` command, `<samp>` output, `<ins>`/`<del>`/`<mark>` colours |
| `<htmlpress-youtube>` + `<a href>`               | a YouTube video player for the linked video                                         |
| `<htmlpress-soundcloud>` + `<a href>`            | a SoundCloud player for the linked track                                            |
| `<htmlpress-mermaid>` + `<pre>`                  | a Mermaid diagram from the source in `<pre>` (escape `<` and `&`)                   |

Code blocks get a copy button and headings a link on their own; a plain `<details>` is
already styled. Tabs holding only code blocks join into one framed block. `<abbr title>`
and footnote references (`<sup><a href="#n1">` to an `<li id="n1">`) show a hover hint.
A root `404.html` is the not-found page, kept out of the navigation; without one a
built-in is used.

Put block content (`<p>`, `<h3>`, `<pre>`) inside a component, not bare text.

## Project files

- `htmlpress.yaml` (optional, keys as in Livemark's config): `site`, `base` (a sub-path
  such as `/repo/`; root-relative URLs get it as a prefix, so write them without it), `title`,
  `description`, `logo` and `favicon` (URLs), `banner` (HTML), `links` (each `title`,
  `url`, `position: header | sidebar`, optional `icon` and `prefix` to show it only
  under one folder), `sections` (each `title` and `prefix`, with
  `type: blog` for dated posts with tag pages and RSS, `type: changelog` plus `source` for
  a page per version of a Markdown changelog, or `type: custom` and `url`; optional `icon`, `position`, `siteTitle`, `siteLink`,
  `siteDescription`: the sidebar and pager then show only the open section's pages), `head` (`meta`, `links`, `scripts`: lists of
  attribute maps; a script's `children` is its source), `contentSignals` (`search`,
  `aiInput`, `aiTrain` for `robots.txt`), `include` (globs of the `.html` files that become pages, default `**/*.html`; only files
  under a glob's base, its folders before the first wildcard, are read, and they are served
  without it, so `docs/**/*.html` serves `docs/guide.html` at `/guide.html`; other files
  there are copied as-is), `exclude` (globs left out of the build), `skills` (globs of
  `SKILL.md` files to publish for agents) and `edit` (URL prefix for an "Edit this page" link). An unknown key is an
  error.
- `<meta name="image" content="/cover.png">` in a page is its social preview image;
  `<meta name="date" content="YYYY-MM-DD">`, `<meta name="author">` (repeatable) and
  `<meta name="keywords" content="a, b">` date, sign and tag it (a blog post's metadata).
- `<meta name="label">` (short nav name), `<meta name="icon">` (top-level entries and
  folder index pages only), `<meta name="path"
content="/start/">` (another URL), `<meta name="sidebar" content="false">` (built, not
  listed), `<meta name="toc" content="false">`, `<meta name="source-url">` and
  `<meta name="source-action" content="view">` (the source link). `patches` in the config
  sets any of these for a file, and publishes a `.md` file such as `README.md` as a page.
- A build writes `robots.txt`, with `site` a `sitemap.xml`, and for agents a Markdown copy
  of each page (`a.html` → `a.md`), `llms.txt`, the skills under `.well-known/agent-skills/`
  and `_headers`; a project file of the same name wins (`_headers` is extended instead).
- `.htmlpress/build/` is the output: never edit it, every build replaces it.
- Escapes, only when the user asks for a different look or a new block: a file under
  `.htmlpress/` at a built-in's path replaces it. `htmlpress escape <name>` copies the
  default to start from (a path, or a component's tag); `htmlpress escape` lists them all.
  - `layout.html` is plain HTML: htmlpress fills its `<head>` and replaces
    `<htmlpress-content>`, `<htmlpress-brand>`, `<htmlpress-banner>`, `<htmlpress-nav>`,
    `<htmlpress-links>`, `<htmlpress-breadcrumbs>`, `<htmlpress-toc>`, `<htmlpress-pager>`,
    `<htmlpress-edit>`, `<htmlpress-toolbar>` and `<htmlpress-footer>` at build time.
  - `layouts/<name>.html` is another layout, used by a page with
    `<meta name="layout" content="<name>">` (a landing page without the sidebar); a
    missing one fails the build.
  - `styles/tokens.css`, `layout.css`, `content.css` are the global theme;
    `components/<name>/<name>.css` and `.js` one component each, the tag
    `<htmlpress-<name>>` without its prefix, its CSS inside `@scope (<tag>)`.
    The layout's frame is elements too, never classes: `<htmlpress-shell>`,
    `-sidebar`, `-main`, `-header`, `-page`, `-aside`, `-skip`.
  - A new name adds a component: `components/credits/credits.html` fills every
    `<htmlpress-credits>` at build time; the tag stays, so its `.css` scopes to the tag
    (`:scope { display: block }` when it should be a box) and its `.css`/`.js` load where
    the tag appears. A misspelt built-in name (`calout/`) adds a component rather than
    replacing one; a file directly in `components/` fails the build.
  - To restyle a component from an outer one, beat its `:scope`: write
    `:scope htmlpress-header`, not a bare `htmlpress-header`, which loses on specificity.

## Checking the result

```sh
htmlpress build <root> --json   # {"output": "...", "pages": N, "files": N}; errors on stderr
htmlpress escape -r <root>      # every escapable file, marking the project's own
htmlpress start <root>          # http://localhost:8000 (or the next free port to 8009); --host for the network
htmlpress preview <root>        # the production build, served without reload
htmlpress clear <root>          # remove .htmlpress/build
htmlpress --llms                # every command's input and output schema
```

After editing, run `build --json` and confirm `pages` counts every page and the exit code is 0. For a visual check, run `start` and open the page; look at it narrow as well as wide.

## References

- Docs: https://htmlpress.dev
- Pages: https://htmlpress.dev/authoring/ · Components: https://htmlpress.dev/authoring/rich-elements.html
