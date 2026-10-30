---
name: htmlpress-deployment
description: Use when publishing an htmlpress site — building into .htmlpress/build/, setting site and base, and wiring Cloudflare, GitHub Pages, Vercel, Netlify or any static host, by hand or from CI.
---

# Htmlpress Deployment

`htmlpress build` writes a fully static site; every host that serves a directory of files can
serve it. There is no client-side routing, so no host needs an SPA fallback.

## When to use this skill

Invoke when the user asks to:

- Publish the site, or set up a host or a CI deploy.
- Serve it under a sub-path such as a GitHub Pages project site.
- Fix a deploy that 404s, loses its styles or misses `sitemap.xml`.

## The build

```sh
htmlpress build            # or `npx gethtmlpress build` in CI, without installing it
htmlpress preview          # serves the production build locally, the way a host will
htmlpress clear            # removes .htmlpress/build
```

The output is `.htmlpress/build/`: an `index.html` per page in its own folder
(`guides/build.html` at `guides/build/index.html`), `404.html`,
`robots.txt`, `_headers`, `_redirects` (when a language has no home page, its prefix to
its first page, which Cloudflare and Netlify answer on the server), a Markdown copy of each page and `llms.txt` for agents, the skills
under `.well-known/agent-skills/`, the theme under `_htmlpress/`, and every other project file.
With `site` set it adds `sitemap.xml`. Never edit the output: every build replaces it.

## Config that affects deployment

- `site` — the final origin (`https://docs.example.com`): `sitemap.xml`, canonical URLs and
  absolute social tags need it.
- `base` — only for a sub-path (`/my-docs/`). Write links without it; the build prefixes every
  root-relative URL, and `start` and `preview` serve under it too. A custom domain needs no
  `base`.

## Hosts

| Host               | Setup                                                                                                                                                           |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cloudflare Workers | `wrangler.jsonc`: `assets.directory: ".htmlpress/build"`, `not_found_handling: "404-page"`; `npx wrangler deploy`                                               |
| GitHub Pages       | `site: https://<user>.github.io`, `base: /<repo>/`; `npx gh-pages -d .htmlpress/build --nojekyll --dotfiles`, or the Pages actions uploading `.htmlpress/build` |
| Vercel             | `vercel.json`: `"framework": null`, `"buildCommand": "npx gethtmlpress build"`, `"outputDirectory": ".htmlpress/build"`                                         |
| Netlify            | `netlify.toml`: `[build] command = "npx gethtmlpress build"`, `publish = ".htmlpress/build"`                                                                    |
| nginx              | `try_files $uri $uri/ =404;` and `error_page 404 /404.html;`                                                                                                    |
| Caddy              | `file_server` plus `handle_errors { rewrite * /404.html; file_server }`                                                                                         |
| S3 + CloudFront    | index `index.html`, error `404.html`, a 404 custom error response to `/404.html`                                                                                |

## Rules of thumb

- Point the host's not-found page at `404.html`, so unknown paths keep the site's layout.
- Keep `--nojekyll` and `--dotfiles` on a GitHub Pages branch deploy: Jekyll drops the
  `_htmlpress/` theme, and without dot-files `.well-known/` is lost.
- Files under `_htmlpress/` keep their names between builds: cache them briefly or invalidate
  them on each release.
- A project's dot-files are never copied: keep an `.htaccess` on the server.

## Verification

Run `htmlpress build --json` and `htmlpress preview`, then after the deploy open the home page,
a nested page, an unknown path (the site's 404) and `/robots.txt`. Under a `base`, check that
styles load and links keep the prefix.

## References

- Deployment: https://htmlpress.dev/deployment/
- Static hosting: https://htmlpress.dev/deployment/static-hosting/
