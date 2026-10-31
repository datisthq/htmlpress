---
name: agent-documentation
description: Use when a task ends — to write the lesson it taught (a correction, a convention, a trap, a decision) into the skill or AGENTS.md line the next session reads — and when adding, splitting or trimming a skill.
---

# Agent documentation

`AGENTS.md` and `.agents/skills/` are this repo's memory. A **lesson** lives on only once it is
written into the line the next session reads. Docs for people belong to **`human-documentation`**.

## After every task

1. **Find the lesson**: what would the next session get wrong without being told? It comes from a
   correction by the user, a command that failed before it worked, a trap hit, or a decision about
   how an area should look or behave. Most tasks teach none; then "no lesson" is the whole answer.
2. **Place it** at the one spot a reader of that area hits first:
   - the area has a skill → edit that skill
   - every session needs it before picking an area → one line in `AGENTS.md`
   - the area recurs and has no skill → a new `<subject>-<facet>` skill, plus its entry in the
     `AGENTS.md` skills list
   - it will not recur → nowhere
3. **Write it as a rule** with its reason, in one or two lines, in the present tense. Rewrite the
   line it supersedes in place, so the skill reads as current guidance with no history in it.
4. **Ship it in the task's commit.**

## `AGENTS.md` versus a skill

`AGENTS.md` loads into every session, so it carries only what must hold before anyone knows which
area they are in: hard rules, invariants that fail silently, terminology, one line per command
and per source folder, and the skills tables. Procedure, rationale and an area's traps go in that
area's skill. Keep `AGENTS.md` near 10 KB.

## Keeping skills sharp

- The `description` is the trigger: when a skill's scope moves, its description moves with it.
- Each fact has one home; elsewhere, a pointer names the skill.
- A line proven wrong is deleted; so is a line that restates a config file or `--help`.
- A skill past ~150 lines splits by area.
