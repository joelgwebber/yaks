---
id: yaks-9f00
title: 'Implement the new note format: reader, writer, converter, anchors'
status: shaving
type: feature
priority: 2
created: '2026-11-14T17:52:43Z'
updated: '2026-11-14T17:53:16Z'
parent: yaks-1151
tags:
- links
- markdown
---

Implement the note format decided in [yaks-6c65](yaks-6c65) ([/doks/design/note-format.md](/doks/design/note-format.md), Joel's decision recorded there). Does not migrate any farm; [yaks-acc0](yaks-acc0) uses the converter this yak ships.

## The new shape

```markdown
#### 2026-11-04T22:13:03Z

*linkplan-lead · kind: move · from: hairy · to: shaving*

moved: hairy -> shaving
```

- A note starts at a line `#### <RFC 3339 timestamp>` outside a code fence and runs to the next one or the end. No `---` separator.
- The next non-blank line, if it starts and ends with `*`, is `actor` and metadata, split on ` · ` and then on the first `: `. A value that contains ` · ` or starts with a quote is quoted like a frontmatter value (reuse `write::scalar`). A note with neither actor nor metadata has no such line.
- The anchor for a note is the GitHub slug of its heading: `2026-11-04t221303z`, with `-1`, `-2` for notes of the same second (the rule `links::note_anchor` uses now).

## Scope

- **Reader:** `store::parse_notes` and its helpers accept BOTH shapes, per file, so a schema-8 farm and an old commit still parse. Never mix shapes inside one body when writing.
- **Writer:** `append_note_meta`, `append_entry`, `note_head`, `meta_block` write the new shape only for a farm at schema 9 or later, else the old one (a farm that has not migrated must still be readable by schema-8 yaks). Find the single place that knows the farm schema; if the writers do not have it, thread it through by the existing pattern rather than a global. Record a `Decision:` note on how you gate it.
- **Converter:** a pure function `store::convert_notes(body) -> (String, usize)` from old shape to new, idempotent, for the migration (same style as the fixups in `migrate.rs`; do NOT register it in `FIXUPS`, the migration yak does that).
- **Anchors:** `links::note_anchor`, `find_note`, `is_note_anchor` and `latest_note_link` produce and accept both the old timestamp form and the new slug form.
- **Other readers and writers:** `mergedriver.rs` (its note-aware merge splits bodies by note; it must merge new-shape bodies and not mangle old ones), `tui/content.rs` (comment assembly and in-place editing round-trip), `tui/detail_nav.rs`, `sheds.rs`, `recur.rs`, `markdown_lint.rs`. The TUI must look the same; only the file changes.
- **Known ambiguity:** a description that holds a `#### <timestamp>` heading cannot be told from a note; `markdown_lint` warns on one in a description.

## Out of scope

Migrating any farm, `store::SCHEMA` bump, link rewriting, the resolver, docs beyond `docs/` lines that describe the note format, the `yaks` skill's "Metadata" paragraph (update both only where they show the old shape).

## Evidence

Round-trip tests (old to parse to new to parse to the same `NoteEntry`s, over the real notes in this farm: write a test that runs the converter over a sample of 200 real yak bodies copied into `tests/fixtures` or generated, then parses both and compares); merge-driver tests for both shapes; a test that a schema-8 farm still writes the old shape and a schema-9 scratch farm the new one; a `gh api -X POST markdown -f mode=gfm -f text=@file` render of one converted real yak, attached, showing headings with the expected ids.

---
▸ 2026-11-14T17:53:12Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T17:53:16Z [linkplan-lead]

Claim: worker `notefmt-1` (Sonnet tier, per AGENTS.md; the format is fully specified in the yak and the design dok, so this is implementation, with my review as the cost). Owns: `store.rs` note functions (`parse_notes`, `note_starts`, `append_note_meta`, `append_entry`, `note_head`, `meta_block`, `parse_meta`), a new `convert_notes`, `links.rs` note-anchor functions only, `mergedriver.rs` note merge, `tui/content.rs`, `sheds.rs`, `recur.rs`, `markdown_lint.rs`. Not `migrate.rs`: yaks-acc0 registers the converter. Other workers in flight: `keys-1` (`keymap.rs`, `docs/tui.md`), `check-1` (`check.rs`); `yaks-c3d0` follows and works in `handlers.rs`, `follow.rs`, `source.rs`, `detail_nav.rs`'s `follow_link`: if you must touch `detail_nav.rs`, keep to note-anchor lines. Judge: the coordinator re-runs the evidence and the round trip over this farm's real yaks.
