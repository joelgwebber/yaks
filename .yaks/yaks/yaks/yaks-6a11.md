---
id: yaks-6a11
title: 'Design: structured comment metadata (per-note frontmatter-like fields)'
status: hairy
type: idea
priority: 2
created: '2026-10-31T15:18:16Z'
updated: '2026-10-31T15:18:16Z'
parent: yaks-039b
labels:
- schema
- design
---

Today a note is `---`, then `▸ <ts> [actor]`, then a body. Everything else is inferred from body conventions: `moved: a -> b`, `verify: … -> PASS`, `Decision:`, claim notes, `Tidy request:`, and which note is the ask and which the answer. Joel (2026-10-31): consider designing explicitly for note metadata, like the yak's own frontmatter.

## Candidate fields

- `kind`: note, move, claim, ask, answer, request, pickup, verify, decision, checkpoint, landed.
- `for` / `owner`: who a claim is for (a coordinator claims on a worker's behalf). This would settle sheds ownership without heuristics.
- `commit`, `branch`, `remote`: checkpoint and landing records (sibling designs).
- `reply_to`: an answer names the ask it answers. This needs a note id; today notes have none (timestamp plus actor is not unique).
- `thread`: a stable harness id (the parked actor-id idea).

## Ask, answer and needs

Move the ask/answer/request semantics into the notes themselves, so neither the human nor the agent has to infer which comment is the question. Keep the top-level `needs:` for findability (Joel's suggestion), written by the same command under the farm lock. Decide which one is authoritative, and have `yaks doctor` check that they agree.

## Syntax candidates

Each must stay readable on GitHub and in the TUI, greppable and diffable.

1. Extend the marker line: `▸ <ts> [actor] ask · commit=abc123`. Least disruptive; gets crowded.
2. A YAML block right after the marker, closed by a blank line. Renders as a run-on paragraph on GitHub.
3. An HTML comment (`<!-- yaks: kind=ask -->`). Invisible on GitHub; the TUI would have to strip it (it shows inline HTML literally today).
4. A fenced `yaks` code block. Visible but honest everywhere.

## Constraints

- Optional and additive, like the actor stamp was: a note without metadata stays valid, so no migration is forced.
- The merge driver (`yaks-4a53`), `store::parse_notes` and `split_note_head`, the TUI comment parser and the schema-5 rendering rules all read this.
- AGENTS.md: "don't change the on-disk format lightly". This yak should end in a recorded decision, backed by a spike on a copy of a real farm, before any code.
