# Htmlpress

Htmlpress turns a folder of HTML pages into a site. Each page keeps its own markup, and
htmlpress adds one layout, one theme, a navigation and a live-reloading dev server. It is
made for coding agents: they write plain semantic HTML, and the result looks consistent
without a framework, a build config or Tailwind.

## Installation

```bash
npm install --global gethtmlpress
# or run it without installing
npx gethtmlpress build docs
```

The package carries a prebuilt binary for Linux, macOS and Windows (x64 and arm64)
through a platform-specific optional dependency; no Rust toolchain is needed.

## Usage

```bash
htmlpress build docs        # render docs/ into docs/.htmlpress/build
htmlpress start docs        # serve it on http://127.0.0.1:8080, rebuilding on save
htmlpress build docs --json # the result as JSON, for scripts and agents
htmlpress --llms            # every command's input and output schema, for agents
```

A page is any `.html` file in the folder: a full document or just a fragment. Its `<title>`
(else its first `<h1>`) names it in the navigation, `<meta name="order" content="1">` sorts it,
and `index.html` comes first. Every other file is copied as-is.

```html
<title>Getting started</title>
<meta name="order" content="1" />

<h1>Getting started</h1>
<p>Plain HTML, styled by the built-in theme.</p>
<hp-callout type="warning"><p>Light-DOM components cover what HTML lacks.</p></hp-callout>
```

## Documentation

Read the docs at [htmlpress.dev](https://htmlpress.dev), themselves built with htmlpress.

## License

MIT.
