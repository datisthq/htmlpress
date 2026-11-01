---
name: benchmark-development
description: Use when working in `benchmark/` — htmlpress' build time and memory against Hugo and Zola (bare and themed), mdBook, VitePress, Starlight, Docusaurus, Eleventy, Livemark and MkDocs. Covers commands, the adapter contract and each rival's traps.
---

# Benchmark

`benchmark/` times htmlpress building a generated documentation site against nine other site
generators. It ships nothing and is not a release component: its commits are
`chore(benchmark)`. `benchmark/README.md` is the user reference; this skill is how to change it
without making the numbers lie.

## Commands

- `pnpm -C benchmark test:unit` — the units in `benchmark/-test/`; the root `test:unit` chains
  it and the root `tsgo --noEmit` types it. Nothing there runs a real generator, and the
  section has no CI job: its numbers need a fixed, idle machine
- `pnpm -C benchmark setup:tools` — `cargo build --release` of this checkout, the three
  binaries checked against their pinned SHA-256, `pnpm install --frozen-lockfile` in
  `rivals/node/` and `uv sync --frozen` in `rivals/python/`. Rerun it after a change to
  `rust/`: `measure` runs whatever binary `rust/target/release/` holds
- `pnpm -C benchmark measure` — narrowed with `--case`, `--size`, `--tool`; `--runs 1 --warmup 0`
  for a smoke run. Never measure while something else builds on the machine: a cargo build in
  another worktree doubles the slow rivals' times
- `pnpm -C benchmark show:report` — merges `.cache/results/`, latest per case, size and tool

## The adapter contract

A `Tool` in `tools/` writes its project for a site once (`writeProject`, stamped by a hash of
every byte it writes, so a changed adapter writes its projects again), names the outputs and
caches removed before each build, so every build is cold, and the folder its HTML lands in.
A build counts only when that HTML holds every page's token. Write each page into the layout
and frontmatter the tool documents, and keep the tool's own defaults otherwise; anything it
does unlike htmlpress (no build-time highlighting, no search, bare templates) goes in its
`caveats` and `isThemed`, which the report prints.

## Traps the rivals set

- The npm rivals' projects sit under `rivals/node/.cache/projects/` so Node resolves their
  packages; `rivals/node/` is a workspace of its own (its own `pnpm-workspace.yaml`), with the
  workspace's quarantine and no exemption: pin each package to a release past it
- Docusaurus 3.10 fails with `require.resolveWeak is not a function` when the nearest
  `package.json` says `"type": "module"`: `rivals/node/package.json` must not
- Livemark strips each include pattern's base, so `section-*/*.md` loses its folders and pages
  collide; the release pinned routes by file name alone, so each page sets its `path`
- Hugo takes a folder's `index.md` for a leaf bundle that swallows the folder's pages: a
  section's index is `_index.md` there, as in Zola
- Hugo and Zola render nothing without templates: their adapters write minimal ones, which
  makes them bare, not documentation sites; `hugo-hextra` is Hugo as a documentation site,
  its home page cascading `type: docs` to every page. Hextra renders the whole sidebar into
  every page, so its time grows with the square of the pages: measure it at 1000, not 100.
  `zola-book` runs Zola 0.22.1: 0.23 moved to Tera 2, which neither the book theme nor
  AdiDoks speaks (`filter`, `macros::name`); its project links `themes/` to the downloaded
  ones, as Zola reads no other folder, and overrides the home template, which drops its text
