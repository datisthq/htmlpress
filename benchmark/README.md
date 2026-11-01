# Benchmark

How fast, and in how much memory, htmlpress builds a documentation site: the release binary
side by side with other site generators, all building the same generated Markdown site from
cold:

| tool            | how it runs                                                                         |
| --------------- | ----------------------------------------------------------------------------------- |
| htmlpress       | `htmlpress build`, the release binary of this checkout's `rust/`                    |
| Hugo            | `hugo` with minimal layouts (no theme)                                              |
| Hugo + Hextra   | `hugo` with the Hextra documentation theme: sidebar, table of contents and search   |
| Zola            | `zola build` with minimal templates (no theme), code highlighting on                |
| Zola + book     | `zola build` 0.22.1 with the book theme of Zola's organization and its search index |
| mdBook          | `mdbook build` with its default theme and search                                    |
| VitePress       | `vitepress build` with the default theme, a sidebar of every page and local search  |
| Astro Starlight | `astro build` with Starlight's defaults, its sidebar generated, Pagefind search     |
| Docusaurus      | `docusaurus build` with the classic preset, docs at `/`, no blog                    |
| Eleventy        | `eleventy` with a one-file Nunjucks layout                                          |
| Livemark        | `livemark build`                                                                    |
| MkDocs Material | `mkdocs build` with the Material theme, `pymdownx.highlight` and `superfences`      |

Hugo and Zola also run with a documentation theme: Hextra, and the book theme on Zola
0.22.1, the newest before Tera 2, which neither theme tried (book, AdiDoks) builds on yet. Bare, Hugo, Zola
and Eleventy run minimal templates: they wrap each page and nothing more, so they do
less than a documentation site, and the report marks them apart. Each tool runs as its own
process, timed whole, start-up included, as a build on save or in CI takes it. Linux only:
memory is the peak anonymous memory of the build's processes, sampled from `/proc`, and the
cap comes from `systemd-run`.

The rivals live in `rivals/`, so none reaches the htmlpress packages:

- `rivals/node/` — the npm generators, a pnpm project of its own outside the workspace, with
  the workspace's seven-day quarantine; a `pnpm install` at the root never installs them
- `rivals/python/` — MkDocs Material, a uv project with the same quarantine as an
  `exclude-newer` date
- Hugo, Zola and mdBook are downloaded by `setup:tools` from their GitHub releases, each
  checked against the SHA-256 its project publishes (`RELEASES` in `tools/binaries.ts`), and
  the Hextra and book themes from their source archives, checked against the SHA-256 pinned when it
  was added (`THEME_RELEASES`), since its project publishes none

## Usage

```bash
pnpm -C benchmark setup:tools     # build htmlpress, download the binaries, install the rivals
pnpm -C benchmark measure --size 100
pnpm -C benchmark show:report     # .cache/report.html
```

`measure` runs every size of every case unless narrowed with `--case`, `--size` or `--tool`
(each repeatable). `--runs` (3) and `--warmup` (1) set the cold builds per tool, `--timeout`
its limit in seconds (600), and `--memory-max` caps each build, for example `8G`. Results land
in `.cache/results/`, one file per invocation; `show:report` keeps the latest result per case,
size and tool, and `--out` writes the page elsewhere. `setup:tools` rebuilds htmlpress, so
rerun it after a change to `rust/` before measuring.

`generate:data` writes a site without measuring anything:

```bash
pnpm -C benchmark generate:data --profile docs --size 1000
```

## Data

Every site is generated from a seed, so each tool reads identical Markdown, and cached in
`.cache/data/` until its profile, size or seed changes. A profile in `profiles/` writes one
page's body; `docs` writes a home page and sections of ten pages, each with headings, prose,
lists, the odd table and code blocks in six languages. A case in `cases/` picks a profile and
its page counts.

Each tool's adapter (`tools/`) writes the site into the project the tool expects: its config,
its folders and its frontmatter (TOML for Zola, none for mdBook, which takes its titles from
`SUMMARY.md`), once a site, under `.cache/projects/` or, for the npm rivals, under
`rivals/node/.cache/projects/`, where they resolve their packages.

## Correctness

Every page carries one word no other page holds. A build counts only when its HTML holds the
word of every page: a build that skipped pages is not a fast one.
