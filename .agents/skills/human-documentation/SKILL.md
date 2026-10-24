---
name: human-documentation
description: Use before committing a change — to find what AGENTS.md, CONTRIBUTING.md, the READMEs, the skills and the docs site now falsely claim after the diff — and when deciding where a new fact or rule belongs.
---

# Documentation

Keep every document true when the code changes, and put each fact where the next reader will
find it.

## Keeping the docs true

Run this **before committing**, once the work is done and the diff is final:

```sh
git diff main...HEAD --stat
git diff main...HEAD
git status --short
```

The guard in `.scripts/-test/docs.unit.ts` already fails on a path or `pnpm` command that no
longer exists. This is the other half: a paragraph that still parses, still points at real
files, and is no longer true.

- **A rename.** Grep the docs for the _old_ name — flag, component, token, config key, script,
  folder. The single most common miss.
- **A new or deleted command, script or source folder.** The command list and code structure in
  `AGENTS.md` and the tables in `CONTRIBUTING.md` enumerate them.
- **A changed default or contract.** A flag's default, how a page's title or order is read, a
  `--json` field, a layout element.
- **A removed mechanism.** Search the docs for the thing you deleted.

## Where to look

- `AGENTS.md` — the only one, at the repo root.
- `.agents/skills/*/SKILL.md` and the shipped `skills/*/SKILL.md`, including each
  `description`, which decides whether a skill gets picked up at all.
- `README.md`, `rust/README.md`, `typescript/README.md`, `python/README.md` — the three package
  READMEs share their usage section; change all three together.
- `docs/` — the user manual, in HTML. Code in `<pre>` must be escaped (`&lt;`), and a page
  should build cleanly with `pnpm build:docs`.

## Where a fact belongs

`AGENTS.md` loads into every session, so it carries only what must hold before anyone knows which
area they are in: hard rules, invariants that fail silently, terminology, one line per command
and per source folder, and the skills tables. Procedure, rationale and an area's traps go in that
area's skill. Keep `AGENTS.md` near 10 KB.

- How to **use** htmlpress (pages, components, CLI) → the matching `docs/` page and the shipped
  `skills/htmlpress-pages/SKILL.md`, kept in step.
- How to **work on** htmlpress → a repo skill in `.agents/skills/`.
- How a person contributes → `CONTRIBUTING.md`.

Do not restate the change in the docs; the story is what `git log` is for. If the diff adds no
invariant and removes none, the right outcome is **no documentation change**.
