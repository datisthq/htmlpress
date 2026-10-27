---
name: htmlpress-authoring
description: Use when writing or editing pages in an htmlpress project — the page contract (title, order, head, metas), headings and the TOC, semantic HTML, the htmlpress-* components, code blocks, math, includes and checking the build.
---

# Htmlpress Authoring

An htmlpress project is a folder of `.html` pages. Htmlpress wraps each in one layout with a
navigation and a theme, so a page carries content only: no `<nav>`, no site header, no styling.

## When to use this skill

Invoke when the user asks to:

- Add, edit, move or reorder pages, or group them in the navigation.
- Use a callout, tabs, cards, steps or another component, or a code block, formula or diagram.
- Pull a shared partial or a source file into a page.

The config keys are in `htmlpress-configuration`; a different look in `htmlpress-customization`.

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
  `index.html` always comes first; unordered pages follow by path, then negative orders
  (`-1` pins a page last).
- Other `<head>` tags (`<link>`, `<style>`, `<script>`) are kept and added after the theme; a
  fragment's are the ones before its first content, as a browser reads them.
- The navigation nests pages by folder at any depth (`guides/cli/…` inside `guides/…`), each
  group named by its `index.html`, whose title links to it and whose `order` places the folder.
- Each page is served at its own folder URL: `guides/build.html` at `/guides/build/`,
  `guides/index.html` at `/guides/`. Link to the page file (`guides/build.html`, relative or
  root-relative) or to its URL: a link naming a page's file becomes its URL. Images and other
  files are copied as-is; a relative `href`, `src`, `srcset` or `poster` resolves from the
  source file, even when the page moves.
- A root `404.html` is the not-found page, kept out of the navigation; without one a built-in
  is used.

## Page metas

| Meta                                         | Effect                                                       |
| -------------------------------------------- | ------------------------------------------------------------ |
| `description`                                | the page's summary for search, previews and `llms.txt`       |
| `image`                                      | its social preview image, relative to the file; needs `site` |
| `date` (`YYYY-MM-DD`), `author` (repeatable) | dates and signs it (a byline; none with `layout`)            |
| `keywords` (`a, b`)                          | tags it (a blog post's tag pages)                            |
| `label`                                      | a short name in the navigation                               |
| `icon`                                       | a Lucide icon; top-level entries and folder index pages      |
| `path` (`/start/`)                           | serves the page at another URL                               |
| `sidebar` (`false`)                          | built, not listed                                            |
| `toc` (`false`)                              | no table of contents                                         |
| `breadcrumbs` (`false` / `true`)             | hides or shows the folder trail, over the config's           |
| `layout` (`<name>`)                          | another layout, from `htmlpress-customization`               |
| `language` (`<code>`)                        | marks the page a translation                                 |
| `source-url`, `source-action` (`view`)       | the page's source link                                       |

A file that cannot carry metas, such as a `README.md`, gets them from `patches` in the config.

## Translations

With `languages` in the config, a translation is `page.<code>.html` beside `page.html`, or a
file with `<meta name="language" content="<code>">`. It pairs with the original by URL below the
language prefix, so a renamed one needs the same `<meta name="path">`. Untranslated pages stay
listed in every language with a badge and link to the original. Without a translated home,
`/<code>/` redirects to the language's first page.

## Headings

- `<h2>` to `<h4>` outside components make the page's table of contents; any `<h2>` to `<h6>`
  without an `id` gets a slug of its text and an anchor link. Give a heading an `id` only when a link needs it fixed.
- A heading takes `data-toc="false"` (shown, not in the TOC) or `data-toc="only"` (in the TOC,
  not shown); `<htmlpress-toc level="2">` (2–4, default 3) in a page renders its TOC inline.

## Semantic HTML first

The theme styles the elements themselves: headings, `<p>`, lists, `<table>`, `<pre><code>`,
`<blockquote>`, `<details>`/`<summary>`, `<figure>`/`<figcaption>`, `<dl>`, `<kbd>`, `<hr>`.

- No inline `style`, no classes for looks, no utility CSS: every page should look like the same
  site. A site-wide change is a token (`htmlpress-customization`).
- An image with `data-theme="light"` or `"dark"` shows in that theme only.
- `<abbr title>` and footnote references (`<sup><a href="#n1">` to an `<li id="n1">`) show a
  hover hint; headings get a link on their own.
- Bare URLs and emails in prose become links and GitHub shortcodes (`:rocket:`) become emoji at
  build time; text in links, code and `data-literal` elements stays as written.
- LaTeX: `$E = mc^2$` inline and `$$…$$` as a block (pandoc's rule keeps `$5 and $10` text; `\$`
  is a dollar), or `<htmlpress-math display>…</htmlpress-math>`; rendered to MathML at build
  time, and invalid LaTeX fails the build.
- `<htmlpress-include src="partial.html">` pastes in another file by path, relative to the page:
  an `.html` body, rendered `.md`, or any other file as a titled code block that takes
  `data-lines` and the other code block attributes. Exclude the partials folder in the config.

## Code blocks

- Escape code inside `<pre><code>`: `&lt;` and `&amp;`. Name its language,
  `<pre><code class="language-rust">`, to highlight it at build time; an inline
  `<code class="language-js">` is highlighted too.
- `data-lines="2,4-5"` marks lines, `data-line-numbers` numbers them, `data-add`/`data-remove`
  show a diff, `data-focus` dims the other lines, `data-error`/`data-warning` flag lines and
  `data-words="a,b"` marks words.
- `<pre data-title="file.yaml">` adds a title bar with the language's logo;
  `<pre data-collapse="10">` shows 10 lines with an Expand bar.
- `<pre><code class="language-ansi">` renders terminal color escapes (raw or `&#27;[31m`).
- Every block gets copy and wrap buttons. Tabs holding only code blocks join into one framed block.

## Components

Only these; anything else is plain HTML. Put block content (`<p>`, `<h3>`, `<pre>`) inside a
component, not bare text.

| Element                                          | Use                                                                                                                     |
| ------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------- |
| `<htmlpress-callout type="…" label="…">`         | a note; `type` is `info` (default), `note`, `tip`, `success`, `warning`, `danger`                                       |
| `<htmlpress-grid cols="2">` + `<htmlpress-card>` | cards in columns (`cols` 2–4, else responsive); `<h3><a>` links a card                                                  |
| `<htmlpress-tabs sync="…">` + `<section title>`  | tabs, one per `<section>`; tabs sharing `sync` switch together                                                          |
| `<htmlpress-steps>` + `<ol>`                     | numbered steps, one per `<li>`, each led by an `<h3>`                                                                   |
| `<htmlpress-badge type="…">`                     | an inline label; `type` as a callout's, `neutral`, `outline`, `ghost` or `link`                                         |
| `<htmlpress-file-tree>` + nested `<ul>`          | a file tree; an `<li>` holding a `<ul>` is a folder                                                                     |
| `<htmlpress-table>` + `<table>`                  | a table sorted by a click on a `<thead>` header                                                                         |
| `<htmlpress-zoom>` + `<img>`                     | an image that opens full screen on click                                                                                |
| `<htmlpress-icon name="…">`                      | an inline Lucide icon; names at htmlpress.dev/authoring/rich-elements/#icons                                            |
| `<htmlpress-button type="…">` + `<a>`            | a call-to-action link; `type` `secondary`/`outline`/`ghost`/`link`/`positive`/`warning`/`destructive`, `size` `sm`/`lg` |
| `<htmlpress-package-tabs>` + `<pre><code>`       | one `npm`/`npx` command, built into npm, pnpm, yarn and bun tabs                                                        |
| `<htmlpress-terminal title="…">` + `<pre>`       | terminal output: `<kbd>` command, `<samp>` output, `<ins>`/`<del>`/`<mark>` colours                                     |
| `<htmlpress-youtube>` + `<a href>`               | a YouTube video player for the linked video                                                                             |
| `<htmlpress-soundcloud>` + `<a href>`            | a SoundCloud player for the linked track                                                                                |
| `<htmlpress-mermaid>` + `<pre>`                  | a Mermaid diagram from the source in `<pre>` (escape `<` and `&`)                                                       |

## Verification

```sh
htmlpress build <root> --json   # {"output": "...", "pages": N, "files": N}; errors on stderr
htmlpress start <root>          # http://localhost:8000 (or the next free port to 8009); --host for the network
htmlpress start <root> --json   # {"url": "...", "network": []} on one line once serving
htmlpress --llms                # every command's input and output schema
```

After editing, run `build --json` and confirm `pages` counts every page and the exit code is 0.
For a visual check, run `start` and open the page; look at it narrow as well as wide. Never edit
`.htmlpress/build/`: every build replaces it.

## References

- Pages: https://htmlpress.dev/authoring/
- Components: https://htmlpress.dev/authoring/rich-elements/
- Code blocks: https://htmlpress.dev/authoring/code-blocks/
