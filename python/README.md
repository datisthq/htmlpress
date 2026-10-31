# Htmlpress

The static site generator for humans and agents. Htmlpress turns a folder of Markdown and
HTML pages into a site. Each page keeps its own markup, and htmlpress adds one layout, one
theme, a navigation and a live-reloading dev server. You or your agent write plain Markdown
or semantic HTML, and the result looks consistent without a framework, a build config or
Tailwind.

## Installation

```bash
pip install htmlpress
# or run it without installing
uvx htmlpress build docs
```

The wheel carries a prebuilt binary for your platform; no Rust toolchain is needed.

## Usage

```bash
htmlpress build docs        # render docs/ into docs/.htmlpress/build
htmlpress start docs        # serve it on http://localhost:8000, rebuilding on save
htmlpress build docs --json # the result as JSON, for scripts and agents
htmlpress --llms            # every command's input and output schema, for agents
```

A page is any `.md` or `.html` file in the folder: Markdown with frontmatter, or HTML from a
fragment to a full document. Its `<title>`
(else its first `<h1>`) names it in the navigation, `<meta name="order" content="1">` sorts it,
and `index.html` comes first. Every other file is copied as-is.

```html
<title>Getting started</title>
<meta name="order" content="1" />

<h1>Getting started</h1>
<p>Plain HTML, styled by the built-in theme.</p>
<htmlpress-callout type="warning"
  ><p>Light-DOM components cover what HTML lacks.</p></htmlpress-callout
>
```

## Documentation

Read the docs at [htmlpress.dev](https://htmlpress.dev), themselves built with htmlpress.

## License

MIT.
