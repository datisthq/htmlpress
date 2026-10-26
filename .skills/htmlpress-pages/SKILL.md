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
- The navigation groups pages by top folder (`guides/…`), named by `guides/index.html`.
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
- Escape code inside `<pre><code>`: `&lt;` and `&amp;`.

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
  `url`, `position: header | sidebar`), `sections` (each `title` and `prefix`, or
  `type: custom` and `url`; optional `position`, `siteTitle`, `siteLink`,
  `siteDescription`: the sidebar and pager then show only the open section's pages), `head` (`meta`, `links`, `scripts`: lists of
  attribute maps; a script's `children` is its source), `contentSignals` (`search`,
  `aiInput`, `aiTrain` for `robots.txt`), `include` (globs of the `.html` files that become pages, default `**/*.html`; only files
  under a glob's base, its folders before the first wildcard, are read, and they are served
  without it, so `docs/**/*.html` serves `docs/guide.html` at `/guide.html`; other files
  there are copied as-is), `exclude` (globs left out of the build), `skills` (globs of
  `SKILL.md` files to publish for agents) and `edit` (URL prefix for an "Edit this page" link). An unknown key is an
  error.
- `<meta name="image" content="/cover.png">` in a page is its social preview image.
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
    `components/<tag>.css` and `.js` one component each, its CSS inside `@scope (<tag>)`.
  - A new tag adds a component: `components/site-footer.html` replaces every
    `<site-footer>` at build time; its `.css`/`.js` load where the tag appears, and since
    the tag is gone, its CSS scopes to a class on the pasted markup. The
    `htmlpress-` prefix is reserved, and a file that replaces nothing fails the build.

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
