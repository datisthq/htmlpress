---
name: htmlpress-pages
description: Use when writing or editing pages in an htmlpress project — a folder of HTML built into a site. Covers the page contract (title, order, head), the theme and hp-* components, htmlpress.toml, and checking the result with build --json and start.
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
- Pages are written to the same path, so link between them with relative hrefs
  (`guides/build.html`). Images and other files are copied as-is.

## Styling: semantic HTML first

The theme styles the elements themselves: headings, `<p>`, lists, `<table>`, `<pre><code>`,
`<blockquote>`, `<details>`/`<summary>`, `<figure>`/`<figcaption>`, `<dl>`, `<kbd>`, `<hr>`.

- No inline `style`, no classes for looks, no utility CSS. The point is that every page looks
  like the same site.
- To change the look site-wide, override a token in a stylesheet: `--hp-accent`, `--hp-font`,
  `--hp-mono`, `--hp-bg`, `--hp-surface`, `--hp-text`, `--hp-muted`, `--hp-border`,
  `--hp-radius`, `--hp-width`.
- Escape code inside `<pre><code>`: `&lt;` and `&amp;`.

## Components

Only these; anything else is plain HTML.

| Element                         | Use                                                                |
| ------------------------------- | ------------------------------------------------------------------ |
| `<hp-callout type="…">`         | a note; `type` is `info` (default), `success`, `warning`, `danger` |
| `<hp-grid>` + `<hp-card>`       | cards in responsive columns                                        |
| `<hp-tabs>` + `<section title>` | tabs, one per `<section>`, named by its `title`                    |

Put block content (`<p>`, `<h3>`, `<pre>`) inside a component, not bare text.

## Project files

- `htmlpress.toml` (optional): `title`, `description`, `site`. An unknown key is an error.
- `.htmlpress/build/` is the output: never edit it, every build replaces it.
- `.htmlpress/layout.html` replaces the built-in layout (a minijinja template); only when the
  user asks for a different layout.

## Checking the result

```sh
htmlpress build <root> --json   # {"output": "...", "pages": N, "files": N}; errors on stderr
htmlpress start <root>          # http://127.0.0.1:8080, rebuilds and reloads on save
htmlpress --llms                # every command's input and output schema
```

After editing, run `build --json` and confirm `pages` counts every page and the exit code is 0. For a visual check, run `start` and open the page; look at it narrow as well as wide.

## References

- Docs: https://htmlpress.dev
- Pages: https://htmlpress.dev/pages.html · Components: https://htmlpress.dev/components.html
