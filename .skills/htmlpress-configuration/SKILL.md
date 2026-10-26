---
name: htmlpress-configuration
description: Use when editing htmlpress.yaml — site metadata, base, include/exclude globs, code themes, sections (blog, changelog, custom), links, languages, versions, patches, head tags, agent skills, content signals and the edit link.
---

# Htmlpress Configuration

`htmlpress.yaml` at the project root is optional; its keys follow Livemark's config. An unknown
key is an error, so a typo fails the build instead of being ignored.

## When to use this skill

Invoke when the user asks to:

- Set the site's title, description, logo, favicon, banner or URL.
- Choose which files become pages, or leave some out.
- Add a blog, a changelog or another section, or links in the header or sidebar.
- Publish the site in several languages, or keep docs for older versions.
- Give a file metas it cannot carry, or publish a Markdown file as a page.
- Add tags to every page's `<head>`, publish skills or set `robots.txt` signals.

## The config shape

```yaml
site: https://example.com
base: /
title: Acme
description: Acme's docs.
logo: /logo.svg
favicon: /favicon.svg
banner: <a href="/blog/">We shipped 2.0</a>
breadcrumbs: false
include: [docs/**/*.html]
exclude: [docs/includes/**]
codeThemeLight: catppuccin-latte
codeThemeDark: catppuccin-mocha
edit: https://github.com/acme/acme/edit/main/
links:
  - { title: GitHub, url: https://github.com/acme/acme, position: header, icon: github }
sections:
  - { title: Blog, prefix: /blog/, type: blog }
patches:
  - { file: README.md, article: { title: Overview, order: 1 } }
skills: [.skills/*/SKILL.md]
contentSignals: { search: true, aiInput: true, aiTrain: false }
head:
  meta: [{ name: theme-color, content: "#0b5" }]
```

- `site` — the canonical origin; with it a build writes `sitemap.xml` and absolute feed URLs.
- `base` — a sub-path such as `/repo/`. Root-relative URLs get it as a prefix, so write them
  without it.
- `title`, `description`, `logo`, `favicon` (URLs) and `banner` (HTML above the header).
- `breadcrumbs` — `false` hides the folder trail above every page; default `true`, and a
  page's `<meta name="breadcrumbs">` overrides it.
- `include` — globs of the `.html` files that become pages, default `**/*.html`. Only files under
  a glob's base, its folders before the first wildcard, are read, and they are served without it:
  `docs/**/*.html` serves `docs/guide.html` at `/guide.html`. Other files there are copied as-is.
- `exclude` — globs left out of the build: partials for includes, drafts.
- `codeThemeLight`, `codeThemeDark` — the code block themes, by Shiki theme name.
- `edit` — a URL prefix for an "Edit on GitHub" link.

## Sections

Each has a `title` and a `prefix`; optional `icon`, `position`, `siteTitle`, `siteLink` and
`siteDescription`. The sidebar and pager then show only the open section's pages.

- `type: article` (the default) — the pages under the prefix.
- `type: blog` — dated posts (`date`, `author`, `keywords` metas) with tag pages and RSS.
- `type: changelog` plus `source` — a page per version of a Markdown changelog, or per
  release of a GitHub repository URL (`GITHUB_TOKEN` when set, cached in `.htmlpress/cache/`); `version: true`
  shows the latest version beside the title.
- `type: custom` plus `url` — a link to anywhere.

## Links

Each has `title`, `url` and `position: header | sidebar`; optional `icon` and `prefix`, which
shows the link only under one folder.

## Languages

`languages` lists each `code` and `label`, optional `locale`, `dir` (`ltr` or `rtl`), `flag` and
`messages` replacing UI strings. The first is the default; the others are served under
`/<code>/`. With `languages`, `title`, `description`, `banner` and link and section titles also
take a map by language code (`title: { en: Docs, es: Documentos }`). Translated pages are in
`htmlpress-authoring`.

## Versions

There is no dedicated option. For a few versions on one branch, put each older version's pages
in a folder with a section of its own (`title: v1`, `prefix: /v1/`, `siteTitle: Acme v1`);
search stays site-wide, so results mix versions. For majors on their own branches, deploy each
branch as its own site (`base`, a `banner` linking to the latest) and link them with a `custom`
section.

## Patches

Each patch is a `file` and an `article` of metas (`title`, `label`, `description`, `icon`,
`order`, `path`, `sidebar`, `breadcrumbs`, `toc`, `image`, `author`, `date`, `tags`, `sourceUrl`,
`sourceAction`), set without touching the file; a patched `.md` file such as `README.md` is
published as a page. A patched path outside the project, through a symlink or into a dot-folder is refused.

## Agents and head

- `skills` — globs of `SKILL.md` files to publish under `.well-known/agent-skills/`.
- `contentSignals` — `search`, `aiInput`, `aiTrain` for `robots.txt`.
- `head` — `meta`, `links`, `scripts`: lists of attribute maps added to every page; a script's
  `children` is its source.

## Rules of thumb

- Change one key at a time and rebuild: the error names the key.
- Keep the config for site-wide facts; a page's own facts are its metas (`htmlpress-authoring`).
- `htmlpress start` rebuilds on a config change; no restart needed.

## Verification

Run `htmlpress build <root> --json`: exit code 0 and the expected `pages` count. An unknown or
misspelt key fails here with its name.

## References

- Configuration: https://htmlpress.dev/customization/configuration-file/
- Sections: https://htmlpress.dev/customization/article-sections/
- Translations: https://htmlpress.dev/customization/translations/
- Versioning: https://htmlpress.dev/customization/versioning/
- Patches: https://htmlpress.dev/customization/article-patches/
