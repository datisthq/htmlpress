---
name: change-review
description: Review a pull request, or the local branch before it lands, for correctness, security and repo conventions — inline PR comments, or a ranked list locally. Invoked by the review workflow or a person; never fires on its own.
disable-model-invocation: true
---

# Review a change

Read the diff, judge it, post findings as inline comments. Be useful, not thorough-looking:
a review that flags twenty things teaches nothing, and the twenty-first — the real one — is lost
in it.

## What to look for

**Correctness**

- Logic that is wrong for an input the change makes reachable: a fragment page, a page without a
  title, a project in a dot-folder path, Windows path separators.
- Error handling that swallows a failure or reports success on a partial build.
- The dev server: a watcher filter that lets the build's own writes or reads trigger a rebuild,
  a request handler that blocks the accept loop, a long poll that never times out.
- Performance where the change makes it structurally worse (a page re-read per nav item, a
  template recompiled per file), not where it is merely not optimal.

**Security — reported only when reachable**

- Path traversal in the dev server (`resolve_file`): a URL that escapes the output folder.
- An output folder that holds the project: `build_site` wipes it first.
- Template injection: page text reaching the layout unescaped outside `content` and `head`,
  which are passed through on purpose.
- Secrets in code, logs or CI; a workflow step with more permissions than it uses.

**This repo's own invariants** — where a generic reviewer is blind:

- behaviour added to a wrapper (`typescript/`, `python/`) instead of the binary;
- a changed page contract, component, token, layout element, config key, flag or `--json`
  field without the matching `rust/docs/` page and the `htmlpress-pages` skill;
- `PLATFORMS` and the release matrix drifting apart (**`package-distribution`**);
- an `unwrap`, `expect` or `unsafe` outside tests, or anything a command prints to stdout under
  `--json` besides the result;
- a commit typed `feat` or `fix` that does not change what an installer gets, or a package change
  typed `chore`; a commit marked breaking;
- `AGENTS.md`, a skill or `CONTRIBUTING.md` that the diff made untrue (**`human-documentation`**).

## How to report

Rate each finding **CRITICAL / HIGH / MEDIUM / LOW**, and say what input or sequence produces
the failure. A finding that cannot name the case that breaks is a hunch: either verify it or
leave it out. Post specific issues as inline comments on the lines they concern; keep the
summary short. If the change is good, say so briefly and stop.

## Reviewing locally

With no pull request, the change is everything this branch adds over `main`:
`git diff main...HEAD` plus the uncommitted and untracked files. Print the findings in the
terminal, worst first, each with its `file:line`. Post nothing, and edit nothing unless the
caller asks for the fixes.
