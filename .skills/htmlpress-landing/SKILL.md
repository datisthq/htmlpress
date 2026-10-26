---
name: htmlpress-landing
description: Use when designing or rewriting an htmlpress project's landing page — a layout of its own, hero, sections, screenshots and calls to action as project components, taking htmlpress.dev's own landing as the reference.
---

# Htmlpress Landing

Guidance for designing the page at `/` of an htmlpress project. The mechanics of a layout and a
project component are in `htmlpress-customization`; this skill is about what goes on the page.

## When to use this skill

Invoke when the user asks to:

- Build a landing, home or marketing page for the site, or redesign the existing one.
- Add a hero, feature grid, screenshot carousel or call-to-action section.
- Make the landing look as polished as htmlpress.dev.

## Start from the reference

htmlpress.dev's landing is itself an htmlpress page with its own layout and a set of project
components. Read it before writing anything, see it live, and borrow its structure:

- Live: https://htmlpress.dev/
- Page: https://github.com/datisthq/htmlpress/blob/main/rust/docs/index.html
- Layout and components: https://github.com/datisthq/htmlpress/tree/main/.htmlpress

None of it is in the packages, so fetch it from GitHub. Take the patterns, not the content:
rewrite every heading and line of copy for the project, and use the project's own screenshots
and links. Shipping htmlpress' copy, screenshots or Datist credit on someone else's site is a bug.

## Skeleton

```html
<!-- index.html -->
<title>Home</title>
<meta name="description" content="What the project does, in one line." />
<meta name="layout" content="landing" />
<meta name="sidebar" content="false" />

<htmlpress-landing-hero>…</htmlpress-landing-hero>
<htmlpress-landing-section>…</htmlpress-landing-section>
<htmlpress-landing-cta>…</htmlpress-landing-cta>
```

```html
<!-- .htmlpress/layouts/landing.html -->
<!doctype html>
<html lang="en">
  <head></head>
  <body>
    <htmlpress-skip><a href="#content">Skip to content</a></htmlpress-skip>
    <htmlpress-landing-page>
      <htmlpress-banner></htmlpress-banner>
      <htmlpress-header role="banner">
        <htmlpress-brand></htmlpress-brand>
        <htmlpress-links></htmlpress-links>
        <htmlpress-search></htmlpress-search>
        <htmlpress-theme-toggle></htmlpress-theme-toggle>
      </htmlpress-header>
      <main id="content"><htmlpress-content></htmlpress-content></main>
      <htmlpress-landing-footer></htmlpress-landing-footer>
    </htmlpress-landing-page>
    <htmlpress-back-to-top></htmlpress-back-to-top>
  </body>
</html>
```

The reference runs hero → a differentiator ("Stays out of your way") → a feature grid → an
audience section (agents) → a final call to action → a credit footer. Pick the sections the
project has something real to say in; three strong ones beat seven thin ones.

## Patterns from the reference

- **One component per block** — `landing-hero`, `-section`, `-features`, `-points`, `-cta`,
  `-actions`, `-carousel`, `-window`, `-code`, `-backdrop`, `-footer`, `-page`, each a folder
  under `.htmlpress/components/` with a `.css` scoped to its tag, and `.js` or `.html` only
  where needed. The page holds semantic HTML inside them: `<section>`, `<header>`, `<hgroup>`,
  `<h2>`, `<ul>` of `<li>` cards. No classes.
- **Section shell** — `<htmlpress-landing-section><section><header><h2>` plus a lede, then the
  body; a two-column section pairs a point list with a visual (`landing-points` beside
  `landing-window`).
- **Hero** — a short eyebrow `<p>` in an `<hgroup>` with the `<h1>`, a lede, then
  `landing-actions` with two links: the docs first, the source second. A decorative
  `landing-backdrop` takes `aria-hidden="true"`.
- **Show the product** — real screenshots, not stock art, each shipped light and dark as two
  `<img>` toggled by the theme with an attribute the component's CSS reads; set `width` and
  `height` so nothing shifts. `landing-carousel` turns `<section title>` slides into tabs.
- **Icons** — inline `<svg aria-hidden="true">` with `currentColor` strokes, so they follow the
  theme.
- **Colour** — only `--htmlpress-*` tokens and `color-mix(in oklab, …)` of them, so the page
  rebrands and goes dark with the rest of the site.

## Rules of thumb

- Links are root-relative (`/introduction.html`); `base` is added at build time.
- Keep the landing out of the sidebar (`sidebar` false) but give it a real `<title>` and
  `description`: it is the site's social preview.
- Every component must read well with its JS off: the slides are already in the HTML.
- Check phone width first; a two-column section must fall back to one.

## Verification

Run `htmlpress build <root> --json`, then `htmlpress start <root>` and look at `/` in light and
dark, wide and at phone width. Tab through it: the skip link, the actions and the carousel must
be reachable by keyboard.

## References

- Layouts: https://htmlpress.dev/customization/custom-routes/
- Overriding: https://htmlpress.dev/customization/overriding-escaping/
