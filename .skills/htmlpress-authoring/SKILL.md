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
- The navigation nests a folder's pages under its `index.html` at any depth (`guides/cli/…`
  inside `guides/…`), whose title names and links the group and whose `order` places it; a
  folder without an `index.html` makes no group, its pages sitting at the top.
- Each page is served at its own folder URL: `guides/build.html` at `/guides/build/`,
  `guides/index.html` at `/guides/`, every name slugified as in Livemark
  (`User Guide/Intro.html` at `/user-guide/intro/`). Link to the page file (`guides/build.html`, relative or
  root-relative) or to its URL: a link naming a page's file becomes its URL, as in Livemark —
  percent-encoded (`User%20Guide/Intro.md`) or not, a `.md` page by its `.html` name too,
  and a page with a translation in the reader's language goes to that one. Images and other
  files are copied as-is; a relative `href`, `src`, `srcset` or `poster` resolves from the
  source file, even when the page moves.
- A root `404.html` is the not-found page, kept out of the navigation; without one a built-in
  is used.

## Markdown pages

A `.md` file is a page too, served where its `.html` twin would be (`guide.md` at `/guide/`),
and goes through the same build: components, math, code blocks, includes, the TOC.

````md
---
title: Getting started
description: Install and run it in a minute.
order: 1
---

# Getting started

:::warning[Back up first]
The build wipes its output folder.
:::

```js title="build.js" lines="2"
await build("docs")
```
````

- The frontmatter keys are the page metas by the same names (`title`, `description`, `order`,
  `layout`, `icon`, `label`, `sidebar`, `date`, `author`, `keywords`, …) plus `head`, raw HTML
  for `<head>`; `author` and `keywords` take a list. Any other key fails the build.
- The syntax is Livemark's: `:::note|tip|info|warning|danger[title]` and `> [!NOTE]` alerts are
  callouts; `:::card{title href icon}`, `:::column` and `:::tab{title sync}` in a row group into
  cards, columns and tabs; `:::details[summary]`, `:::filetree`, `::button[label]{href variant
size}`, `::toc{maxLevel}`, `::video{type="youtube" url}`, `::audio{type="soundcloud" url}`,
  `:badge[label]{variant}`, `:icon{name}`, `:abbr[text]{title}`. Headings take `{#id}`,
  `[step]`, `[!toc]` and `[toc]`; `Term` then `: definition` is a definition list.
- Any other `:::name{attrs}` is the component `<htmlpress-name attrs>`; a block one that names
  no component fails the build, while inline `note:important` stays text. Raw `<htmlpress-*>`
  HTML works too, with Markdown inside after a blank line.
- A fence's meta is the code block attributes without `data-`: `title`, `collapse`, `lines`,
  `line-numbers`, `add`, `remove`, `focus`, `error`, `warning`, `words`; adjacent fences with
  `tab="…"` (and `sync`) are code tabs, ` ```npm ` is package-manager tabs, ` ```mermaid `
  a diagram, and inline `` `code{:ts}` `` is highlighted.
- GitHub Markdown (tables, task lists, footnotes, autolinks), `$…$` math and `:emoji:` work.
  Link to another page by its file, `[Setup](setup.md)`.
- Never pages: an `.md` beside its `.html` page (that page's own Markdown copy for agents),
  an `.md` a page includes (a partial, with directives), a changelog section's `source`, and
  the `skills` files.

## Page metas

| Meta                                         | Effect                                                       |
| -------------------------------------------- | ------------------------------------------------------------ |
| `description`                                | the page's summary for search, previews and `llms.txt`       |
| `image`                                      | its social preview image, relative to the file; needs `site` |
| `date` (`YYYY-MM-DD`), `author` (repeatable) | dates and signs it (a byline under the `<h1>`)               |
| `keywords` (`a, b`)                          | tags it (a blog post's tag pages)                            |
| `label`                                      | a short name in the navigation                               |
| `icon`                                       | a Lucide icon; top-level entries and folder index pages      |
| `path` (`/start/`)                           | serves the page at that exact URL, never prefixed            |
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
language prefix: a `page.<code>.html` without a `path` takes its original's, under the prefix.
A `path` is literal, as Livemark's: slashes are added (`start` → `/start/`), the language prefix
never is, so a translation's starts with it (`/es/start/`) or the build fails. Untranslated pages stay
listed in every language with a badge and link to the original. Without a translated home,
`/<code>/` redirects to the language's first page.
The same holds for the default language without an `index.html`.

## Headings

- `<h2>` to `<h4>` outside components make the page's table of contents; any `<h2>` to `<h6>`
  without an `id` gets a slug of its text, as GitHub makes them (`intro`, then `intro-1`), never
  an `id` the page already uses, and an anchor link. Give a heading an `id` only when a link
  needs it fixed.
- A heading takes `data-toc="false"` (shown, not in the TOC) or `data-toc="only"` (in the TOC,
  not shown); `<htmlpress-toc max-level="2">` (2–4, default 4) in a page renders its TOC inline.

## The layout owns the frame

The layout already gives every page its `<main>`, width, gutters, header, footer and spacing.
A page that brings its own frame nests one inside the other and gets double padding.

- **No wrapper.** Content starts at the top level: no `<main>`, `<article>`, `<div class="container">`
  or `<section>` around the whole page, and no `<header>` or `<footer>` of the page's own.
- **No frame CSS.** Never style `html`, `body`, `main` or a page root: no `max-width`, `margin`,
  `padding`, `font-family`, `background` or `color` scheme. These come from tokens.
- **One `<h1>`, first.** The `<h1>` is the first content element, says what `<title>` says, and is
  the only one; no eyebrow, logo or date above it (`date` and `author` metas make the byline).
- **Headings descend in steps**: `<h2>` for sections, `<h3>` inside them, never skipping a
  level to get a smaller size. A component's heading is `<h3>` (cards, steps).
- **No spacing hacks**: no `<br>` runs, empty `<p>`, `&nbsp;` indents or `<hr>` between sections;
  headings already space the page.
- **Converting a standalone HTML page** (a report, an export, an artifact): keep its body content,
  drop its `<style>` frame, wrapper, header and footer, map its stat tiles, badges and callouts to
  the components below, and keep a page `<style>` only for a figure the theme cannot draw, scoped
  to that figure and using `--htmlpress-*` tokens only.

## Semantic HTML first

The theme styles the elements themselves: headings, `<p>`, lists, `<table>`, `<pre><code>`,
`<blockquote>`, `<details>`/`<summary>`, `<figure>`/`<figcaption>`, `<dl>`, `<kbd>`, `<hr>`.

- No inline `style`, no classes for looks, no utility CSS: every page should look like the same
  site. A site-wide change is a token (`htmlpress-customization`).
- An image with `data-theme="light"` or `"dark"` shows in that theme only.
- Every image opens full screen on click, unless a link wraps it; nothing to write.
- `<abbr title>` and footnote references (`<sup><a href="#n1">` to an `<li id="n1">`) show a
  hover hint; headings get a link on their own.
- Bare URLs and emails in prose become links and GitHub shortcodes (`:rocket:`) become emoji at
  build time; text in links, code and `data-literal` elements stays as written.
- LaTeX: `$E = mc^2$` inline and `$$…$$` as a block (pandoc's rule keeps `$5 and $10` text; `\$`
  is a dollar), or `<htmlpress-math display>…</htmlpress-math>`; rendered to MathML at build
  time, and invalid LaTeX fails the build.
- `<htmlpress-include src="partial.html">` pastes in another file by path, relative to the page:
  an `.html` body, compiled `.md` (directives and fence meta work), or any other file as a titled code block that takes
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

| Element                                                  | Use                                                                                                                        |
| -------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `<htmlpress-callout type="…" title="…">`                 | a note; `type` is `info` (default), `note`, `tip`, `warning`, `danger`                                                     |
| `<htmlpress-cards>` + `<htmlpress-card title href icon>` | cards in two columns; `title` is the heading, `href` links the whole card, `icon` leads it                                 |
| `<htmlpress-tabs sync="…">` + `<htmlpress-tab title>`    | tabs, one per `<htmlpress-tab>`; tabs sharing `sync` switch together                                                       |
| `<h3 data-step>`                                         | a numbered step, counted through the page; kept out of the TOC                                                             |
| `<htmlpress-badge variant="…">`                          | an inline label; `variant` `secondary`/`outline`/`positive`/`warning`/`destructive`, accent without it                     |
| `<htmlpress-filetree>` + nested `<ul>`                   | a file tree; an `<li>` holding a `<ul>` is a folder                                                                        |
| `<htmlpress-icon name="…">`                              | an inline Lucide icon; names at htmlpress.dev/authoring/rich-elements/#icons                                               |
| `<htmlpress-button href label variant="…">`              | a call-to-action link; `variant` `secondary`/`outline`/`ghost`/`link`/`positive`/`warning`/`destructive`, `size` `sm`/`lg` |
| `<htmlpress-package-tabs>` + `<pre><code>`               | one `npm`/`npx` command, built into npm, pnpm, yarn and bun tabs                                                           |
| `<htmlpress-terminal title="…">` + `<pre>`               | terminal output: `<kbd>` command, `<samp>` output, `<ins>`/`<del>`/`<mark>` colours                                        |
| `<htmlpress-video type="youtube" url>`                   | a YouTube video player for the video at `url`                                                                              |
| `<htmlpress-audio type="soundcloud" url>`                | a SoundCloud player for the track at `url`                                                                                 |
| `<pre><code class="language-mermaid">`                   | a Mermaid diagram from the source in the block (escape `<` and `&`)                                                        |

## Verification

```sh
htmlpress build <root> --json   # {"output": "...", "pages": N, "files": N}; errors on stderr
htmlpress start <root>          # http://localhost:8000 (or the next free port to 8009), --host for the network, --open [path] in the browser
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
