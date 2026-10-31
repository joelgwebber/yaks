---
id: yaks-6a11
title: 'Design: structured comment metadata (per-note frontmatter-like fields)'
status: shaving
type: idea
priority: 2
created: '2026-10-31T15:18:16Z'
updated: '2026-10-31T16:16:35Z'
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

---
▸ 2026-10-31T15:56:39Z [delta:Cloud Runners]

Decision (Joel, 2026-10-31): do this design before `yaks-bdf9` and `yaks-a3fc`, since it decides where their fields live.

- **Shape:** simple, open-ended key/value metadata per note, with a handful of reserved keys (`kind`, `for`, `commit`, `branch`, `reply_to`, `thread`, …). Unknown keys round-trip untouched.
- **Syntax instinct:** consistent with the yak's own YAML frontmatter, even if it renders oddly on GitHub; a fenced code block is also acceptable. Open as an ask on this yak.
- **`needs:` stays** at the top of the yak, written with the note by the same command; `yaks doctor` catches drift between them.
- **Doctor noise:** explicit `kind`s (a shear summary, an answer, a verify result) should let doctor check for the right note instead of heuristics, cutting false alarms like "shorn but has no recorded note" (five today under `doctor --strict`).
- **Design Q&A in yaks:** the skills should push design-phase questions and answers into yak asks/answers rather than chat, for provenance. Tracked as its own skills yak, which depends on this one.

---
▸ 2026-10-31T15:57:19Z [delta:Cloud Runners]

asked: needs human

Which syntax for note metadata? Lean: (A), bare YAML lines directly under the marker. (The examples omit the `---` above each marker: inside a fence it would still be parsed as a real note boundary, see the note below.)

## (A) YAML lines under the marker, up to the first blank line

```markdown
▸ 2026-10-31T15:20:00Z [tex-1]
kind: checkpoint
branch: yaks/tex-1/textures
commit: 47076b0

Atlas and UVs done; next, per-face texture choice.
```

- Same syntax as the yak's frontmatter, as you suggested.
- Unambiguous with every existing note: since schema 5 the writer always leaves a blank line after the marker, so a legacy note has no lines there. The parser accepts only `key: value` YAML there; `yaks doctor` flags anything else.
- GitHub renders the marker and the fields as one compact line ("▸ 2026-… [tex-1] kind: checkpoint branch: … commit: 47076b0"), which reads like a byline. Not pretty, not broken. The TUI renders them as chips.
- It must not be closed with `---`: on GitHub that turns the last field into a heading.

## (B) A fenced YAML block right after the marker

````markdown
▸ 2026-10-31T15:20:00Z [tex-1]

```yaml yaks
kind: checkpoint
commit: 47076b0
```

Atlas and UVs done.
````

- Renders cleanly everywhere, with YAML highlighting on GitHub (the first word of the info string is the language).
- Heavier to write and read raw. The `yaks` tag is needed to tell it apart from a note that starts with an ordinary YAML snippet.

## Other formats I checked

None of them is a portable standard for metadata on a section:

- **Pandoc** allows YAML metadata blocks anywhere, but they need `---` above and `---` or `...` below. That collides with our `---` separator and with GitHub's setext headings.
- **Obsidian/Dataview** has inline `key:: value` fields, which nothing else renders.

So the choice is (A), consistent and terse, or (B), pretty on GitHub. Reserved keys and note ids (`reply_to` needs one) are separate follow-ups for this design.

---
▸ 2026-10-31T15:58:01Z [delta:Cloud Runners]

Related bug found while writing the ask above: the note parser isn't fence-aware, so example notes inside code blocks split into fake notes. Filed as `yaks-4cd7`; fix it before this design ships examples in the docs.

---
▸ 2026-10-31T16:04:10Z [Joel Webber]

What about (A), but with a `---` HR underneath, like in frontmatter? At least then it would render separated by the line.

---
▸ 2026-10-31T16:08:34Z [yakherd]

picked up

Picking up Joel's answer: (A) with a frontmatter-style `---` under the metadata. Decision recorded below; implementing now as the yakherd's prep commit (core parse/write), then the command and TUI surface via a worker.

---
▸ 2026-10-31T16:08:35Z [yakherd]

Decision (Joel, 2026-10-31, answering the syntax ask): metadata is YAML `key: value` lines directly under the `▸` marker, closed by a `---` line, like the yak's own frontmatter.

```text
---
▸ 2026-10-31T15:20:00Z [tex-1]
kind: checkpoint
commit: 47076b0
---

Atlas and UVs done.
```

## Rules

- **Only when present.** A note with no metadata keeps today's shape (marker, blank line, text). No migration, no schema bump.
- **Parse.** After the marker, a run of non-blank `key: value` lines ended by a `---` line, with no blank line in between, is metadata. Anything else (a blank line first, a line that isn't `key: value`, no closing `---`) leaves the note as plain text, exactly as today. Keys match `[A-Za-z_][A-Za-z0-9_-]*`; values are single-line scalars, unquoted like frontmatter values (`'…'` / `"…"`). Order is kept and unknown keys round-trip untouched.
- **Reserved keys (v1):** `kind`, `for`, `commit`, `branch`, `remote`, `thread`. `reply_to` waits for note ids.
- **Kinds (v1):** `move`, `ask`, `answer`, `request`, `pickup`, `verify`, `checkpoint`, `landed`. The commands that write these keep writing their existing body text too (`moved: a -> b`, `asked: …`), so older binaries and every text-based reader keep working.

## Rendering trade-off, accepted

On GitHub the metadata lines followed by `---` become a setext H2 heading, so a note with metadata shows its byline as a heading with GitHub's rule under it. It is visually separated, as Joel wanted, but larger than the byline of a note without metadata. The TUI parses notes itself and is unaffected. Putting a blank line before the `---` would make it a plain rule instead, but it would no longer look like frontmatter; revisit only if the heading reads badly in practice.

---
▸ 2026-10-31T16:08:35Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:16:35Z [yakherd]

Core landed in 7f78a28: `NoteEntry.meta` / `kind()` / `get()`, `append_note_meta`, `append_entry`, `meta_block`, `parse_meta`, the TUI comment block's `meta`, merge-driver preservation. The command and TUI surface follow from a worker.
