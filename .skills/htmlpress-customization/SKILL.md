---
name: htmlpress-customization
description: Use when changing an htmlpress site's look or chrome — overriding --htmlpress-* tokens, escaping the layout, styles or a component into .htmlpress/ with `htmlpress escape`, adding layouts and project components.
---

# Htmlpress Customization

Htmlpress ships a layout, a theme and a library of components, all compiled into the binary.
A file under `.htmlpress/` at a built-in's path replaces it; a new name adds one of your own.

## When to use this skill

Invoke when the user asks to:

- Rebrand: colours, fonts, widths, radius.
- Change the header, sidebar, footer or another part of the chrome.
- Restyle a component, or add a block the built-in components do not have.
- Give some pages a different layout, such as a landing page without the sidebar
  (`htmlpress-landing` covers what goes on one).

Do not escape for content: a page's own needs are semantic HTML and components
(`htmlpress-authoring`).

## Tokens first

Most looks are a token. Override it in `.htmlpress/styles/tokens.css`, or in one page's own
`<style>` for that page only:

```css
:root {
  --htmlpress-accent: oklch(0.55 0.24 27);
}
```

`--htmlpress-accent`, `-accent-text`, `-font`, `-mono`, `-bg`, `-surface`, `-text`, `-muted`,
`-prose`, `-prose-strong`, `-prose-marker`, `-border`, `-shadow`, `-radius`, `-width`,
`-nav-width`, `-banner`, `-banner-text`, `-highlight`, `-highlight-text` (search matches on
the page a result opened), and the
callout and badge tones `-info`, `-success`, `-warning`, `-danger`.

## Escaping

```sh
htmlpress escape                         # list every file, marking the escaped ones
htmlpress escape htmlpress-tabs          # a component's .css and .js
htmlpress escape layout.html styles/tokens.css
htmlpress escape htmlpress-tabs -r docs  # in another project folder
```

`escape` copies a default to start from and never overwrites a file you already have. Deleting
an escape restores the default. An escaped copy keeps the code it was copied from, so after an
upgrade it misses the new defaults: escape only what you change.

| Path under `.htmlpress/`                         | Replaces                                                          |
| ------------------------------------------------ | ----------------------------------------------------------------- |
| `layout.html`                                    | the page around the content                                       |
| `layouts/<name>.html`                            | a layout of your own, picked by `<meta name="layout">`            |
| `styles/tokens.css`, `layout.css`, `content.css` | the global theme: tokens, page globals, content elements          |
| `favicon.svg`                                    | the icon linked when the config sets neither `favicon` nor `logo` |
| `components/<name>/<name>.css`, `.js`            | one component, `<htmlpress-<name>>` without its prefix            |

## Layouts

`layout.html` is plain HTML: htmlpress fills its `<head>` and replaces `<htmlpress-content>`,
`<htmlpress-brand>`, `<htmlpress-banner>`, `<htmlpress-nav>`, `<htmlpress-links>`,
`<htmlpress-breadcrumbs>`, `<htmlpress-toc>`, `<htmlpress-pager>`, `<htmlpress-edit>`,
`<htmlpress-toolbar>`, `<htmlpress-footer>`, `<htmlpress-language>` (the language picker) and
`<htmlpress-skip>` at build time, every string in the page's language. The interactive ones,
`<htmlpress-nav-toggle>`, `<htmlpress-search>`, `<htmlpress-theme-toggle>`, `<htmlpress-toc-bar>`
(the phone's contents bar), `<htmlpress-shortcuts>` (the keyboard shortcuts menu, listing every key and disabling the ones no
control on the page announces with `aria-keyshortcuts`) and `<htmlpress-back-to-top>`, run in the browser; the first three get their button at build time,
so the header is whole on its first frame. A head script sets `data-scripted` on `<html>`:
without it those buttons stay hidden, so style anything that needs scripts under
`:root[data-scripted]`. An element with nothing to show is removed. The theme toggle, the
language picker and the sidebar's section picker drop down from `56rem` up and, like the
contents bar, open an `<htmlpress-sheet>` on a phone: one bottom sheet the scripts append to
`<body>`, so escape `components/sheet/sheet.css` to restyle every sheet at once.

Pages crossfade only their content: `<htmlpress-header>` and `<htmlpress-sidebar>` carry the
view transition names `htmlpress-header` and `htmlpress-sidebar`, and a hovered link's page is
prerendered. An escape of either component's `.css` keeps its `view-transition-name`, or it
crossfades with the content.

The frame is elements too, never classes: `<htmlpress-shell>`, `-sidebar`, `-main`, `-header`,
`-page`, `-aside`, `-skip`, each a component whose `.css` can be escaped.

A page with `<meta name="layout" content="landing">` takes `layouts/landing.html` instead. It
starts bare: only the layout elements you put in it are drawn. A missing layout fails the build.

## Project components

A folder under `components/` with a new name adds `<htmlpress-<name>>`, each file named like
the folder:

```html
<!-- .htmlpress/components/credits/credits.html -->
<footer><p>Built by my team · 2026</p></footer>
```

```css
/* .htmlpress/components/credits/credits.css */
@scope (htmlpress-credits) {
  :scope {
    display: block;
  }
  p {
    color: var(--htmlpress-muted);
  }
}
```

- The `.html` fills every such tag in the layouts and pages at build time; the tag stays, so its
  `.css` scopes to it. A custom element is inline until its CSS says otherwise.
- Its `.css` and `.js` load only where the tag appears. A name is lowercase letters, digits and
  hyphens.
- A layout or a component's `.html` may hold any built-in component, but not another project
  component's tag: it is pasted in once, never rewritten again.
- The markup inside a layout element (`nav`, `toc`, `pager`, …) is built from the site; a
  `components/brand/brand.html` replaces it with static HTML, and the built-in `.css` still
  applies to the tag.

## Rules of thumb

- A misspelt built-in name (`calout/`) adds a component rather than replacing one; a file
  directly in `components/` fails the build. Check `htmlpress escape` marks the file escaped.
- To restyle a component from an outer one, beat its `:scope`: write `:scope htmlpress-header`,
  not a bare `htmlpress-header`, which loses on specificity.
- Use tokens inside your CSS (`var(--htmlpress-muted)`) so dark mode keeps working.

## Verification

Run `htmlpress build <root> --json` (exit code 0), then `htmlpress start <root>` and look at
the change in light and dark, wide and phone width. `start` picks up a new or deleted escape
at once.

## References

- Overriding: https://htmlpress.dev/customization/overriding-escaping/
- Layouts: https://htmlpress.dev/customization/custom-routes/
