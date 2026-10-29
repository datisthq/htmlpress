---
name: htmlpress-knowledge
description: Use when writing internal docs for people with htmlpress — a project's knowledge base of research, reports, registers and decisions in `.humans/`, kept beside the code and read as one site.
---

# Htmlpress Knowledge

A knowledge base is what the people on a project read to know why it is the way it is: research,
reports, registers of things counted, and the decisions made. It is not the user docs, which
teach a product, and not agent instructions (`AGENTS.md`, skills), which steer an agent. Htmlpress
builds it the way it builds any site: one HTML page per document, one theme, one navigation.

## When to use this skill

Invoke when the user asks to:

- Write up research, an investigation, a comparison, a market or competitor scan, or a report.
- Keep a register (sources, dealers, vendors) or a decision log.
- Turn notes, a Markdown file or a standalone HTML report into a page people will read.
- Set up a place for a project's internal docs.

How to write a page is in `htmlpress-authoring`; read it first. This skill is what a knowledge
base adds on top.

## Setup

`.humans/` at the repo root is the folder, a dot-directory so code globs skip it. It is its own
htmlpress project:

```yaml
# .humans/htmlpress.yaml
title: Project
description:
  en: Research, reports and decisions for people
```

```json
{ "scripts": { "read": "htmlpress start .humans" } }
```

Ignore `.humans/.htmlpress/build/` in git. Files a page uses (images, CSV) sit beside it in a
folder named after it (`image-quality.html` and `image-quality/`).

## Pages

- **One document, one page**, named for its subject in kebab case (`competitors.html`), never by
  date or author. A family of pages (one per country) gets a folder with an `index.html`.
- **Metas**: `<title>`, `description` (what a reader learns, in one line), `date` (when the
  content was last true) and `order` for the pages a newcomer should read first.
- **Lead with the answer.** The paragraph under the `<h1>` gives the conclusion or the
  recommendation; the evidence follows. Headline numbers go in an `htmlpress-grid` of
  `htmlpress-card`s right after it.
- **Numbers carry their date and source**, near the top: a register goes stale, and the reader
  must see when it was measured and from where. Cite each source with a link.
- **Statuses are badges** with one fixed meaning per site: `success` decided or done, `warning`
  a call still to confirm, `danger` open or blocked. Say so in a legend if a page uses them.
- **Tables hold short cells.** A table never wraps a cell, so a row that needs prose becomes a
  `<dl>`: the item as `<dt>`, its reasons as `<dd>`. Wrap a sortable table in `htmlpress-table`.
- **Diagrams are inline SVG**, drawn for the real structure, in a `<figure>` with a
  `<figcaption>`, styled by a page `<style>` scoped to that figure with `--htmlpress-*` tokens
  only, so it follows light and dark. `htmlpress-mermaid` for a quick flow.

## A decision log

One page, `decisions.html`, with `<h2>` sections Decided, To confirm and Open. Each entry is an
`<h3>` naming the decision, its badge, its date, why, and the alternatives rejected. A decision
changes the log in the same commit as the code that carries it out; an entry moves sections,
it is never deleted.

## Keep it true

A page is a claim about the code or the world on its `date`. When a change makes one false,
fix the page in the same commit, or add a `warning` callout saying what is stale. Delete a page
nobody can keep true rather than leave it wrong.

## Verification

Run `htmlpress build .humans --json` and confirm `pages` counts every page, then
`htmlpress start .humans` and read each changed page at phone width: no sideways scroll, no
double padding, one `<h1>`.
