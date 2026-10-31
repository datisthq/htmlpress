# Htmlpress

The static site generator for humans and agents. Turn a folder of Markdown and HTML pages
into a site: one layout, one theme, a navigation and a live-reloading dev server — no
framework, no templating language, so you or your agent just write the pages.

| Channel   | Package                                                           | Install                                             |
| --------- | ----------------------------------------------------------------- | --------------------------------------------------- |
| Script    | [GitHub releases](https://github.com/datisthq/htmlpress/releases) | `curl -fsSL https://htmlpress.dev/install.sh \| sh` |
| npm       | [`gethtmlpress`](https://www.npmjs.com/package/gethtmlpress)      | `npm install --global gethtmlpress`                 |
| PyPI      | [`htmlpress`](https://pypi.org/project/htmlpress/)                | `pip install htmlpress`                             |
| crates.io | [`htmlpress`](https://crates.io/crates/htmlpress)                 | `cargo install htmlpress`                           |

Every channel ships the same Rust binary at the same version. The docs live at
[htmlpress.dev](https://htmlpress.dev), built with htmlpress from `docs/`. This repository holds
the binary in `rust/` and the npm and PyPI wrappers in `typescript/` and `python/`; see
[Contributing](CONTRIBUTING.md) for the layout and the commands.
