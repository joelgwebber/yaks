---
id: yaks-9f00
title: 'Implement the new note format: reader, writer, converter, anchors'
status: shorn
type: feature
priority: 2
created: '2026-11-14T17:52:43Z'
updated: '2026-11-14T23:35:40Z'
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

---
▸ 2026-11-14T22:37:53Z [notefmt-1]

Start of work (after an offline gap): the checkout is clean, nothing written yet, HEAD contains `local/main` (e4d0ff5a, v0.0.36).

- date: 2026-11-14T22:37Z
- farm file: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-9f00.md`
- checkout: `/Users/joel/src/yaks/.delta/worktrees/bzdf5a446bnv/yaks`

Read so far: `store.rs` note functions, `tui/content.rs`, `mergedriver.rs` note merge, `links.rs` anchors. Plan: reader accepts both shapes; writers take a `new_shape: bool` threaded from the farm schema; `convert_notes` pure.

---
▸ 2026-11-14T23:12:07Z [notefmt-1]
kind: checkpoint
---

Checkpoint, 23:1x. Builds clean (`cargo build`), tests not run yet.

Done in the working tree:

- `store.rs`: `NoteShape` (Old/New), `NEW_NOTES_SCHEMA = 9`, `NoteShape::of_farm(root)` (reads the `schema` marker); writers `append_note_meta_in`, `append_entry_in`, `format_note` (single source of both shapes); readers `scan_notes` (per-body shape: old if any old note, else new), `parse_note_head`, `parse_italic_line`; `convert_notes`; `drop_latest_request` works on both.
- `farm.rs`: every writer passes `self.shape()`; `move_task_with` passes `of_farm(root)`; `rebase_description` uses `store::starts_with_note`.
- `tui/content.rs`: `parse` through `parse_note_head`, `assemble_in(shape, ..)`; `blockedit.rs` passes the body's shape.

Left: anchors in `links.rs` (rebase first, main moved), merge driver `append_entry_in`, sheds/recur/tick/markdown_lint, tests (round trip over 200 real yaks, merge driver both shapes, schema 8 vs 9 farm), docs and skill, GitHub render attachment, verify.

---
▸ 2026-11-14T23:16:23Z [notefmt-1]
kind: checkpoint
---

Checkpoint, ~23:50. Reader, writers, converter, anchors and the merge-driver shape are in and the existing 1164 unit tests pass.

- Anchors: `links::note_anchor(shape, notes, i)` writes the slug (`2026-11-04t221303z`, `-1`) for a new-shape body and the timestamp for an old one; `find_note` and `is_note_anchor` accept both. `peek` of a note prints it in the body's shape.
- Round trip: `store::note_shape_tests::real_yak_bodies_convert_to_the_same_notes` reads yak files from the farm (read only, `YAKS_NOTES_SAMPLE=<dir>` overrides, `YAKS_NOTES_LIMIT` raises the 200): over this farm's first 200 yaks, 185 converted, 766 notes, identical `NoteEntry` lists, idempotent. With `YAKS_NOTES_LIMIT=5000`: 761 yaks, 3307 notes, all identical.
- Left: merge-driver tests in the new shape, `tui/content.rs` round-trip tests, sheds/recur/lint checks, docs, GitHub render, verify.

---
▸ 2026-11-14T23:27:56Z [notefmt-1]

Decision: the writer is gated on the farm's `schema` marker, read where the farm root is known.

- `store::NoteShape::of_farm(root)` reads `<root>/schema` (`>= NEW_NOTES_SCHEMA = 9` is New; a missing or unreadable marker is Old). `Farm::shape()` wraps it; every `Farm` writer and `store::move_task_with` pass it to `append_note_meta_in`. Rejected: a global or env switch (AGENTS.md: no `set_var`, tests would race), and a `Farm` field cached at open (a farm migrated under a running TUI would keep writing the old shape; the marker read is one small file per write).
- A body never mixes shapes: `append_note_meta_in` uses the shape of the notes the body already has, and falls back to the farm's only for a body with no notes. So a half-converted body, or a farm whose marker changed, stays consistent.
- `store::SCHEMA` is still 8. The one-arg `append_note`/`append_note_meta`/`append_entry` stay as the old-shape helpers (the existing tests and fixtures use them).

---
▸ 2026-11-14T23:27:56Z [notefmt-1]

Decision: how a body is read when it holds both shapes, and how the converter refuses.

- `scan_notes` reads a body in one shape: Old if it has any `---` / `▸` note outside a fence, else New. In an old body a `#### <timestamp>` line is text, so a schema-8 yak with such a line in its description (nothing in this farm has one) is never mis-split. Rejected: scanning both and merging (a note's end would depend on which opener came next).
- `convert_notes` returns the body unchanged with 0 when converting would change what it parses to: a `#### <timestamp>` outside a fence anywhere in an old body, or a note whose timestamp is not RFC 3339 (the heading could not carry it). Over this farm's 761 yaks none hits this. `markdown_lint::note_heading_in_description` detects the first case; wiring it into `doctor` is `check.rs`, which I do not own, so it is a follow-up.
- A note with no actor and no metadata whose text opens with an italic line (`*emphasis*`) is written with a `*-*` line so the text is not read as the metadata line; `-` alone as an actor is quoted. Same for an actor that reads as `key:`.
- The anchor follows the body's shape: slug (`2026-11-04t221303z`, `-1`) for a new-shape body, the timestamp for an old one, so a schema-8 yaks keeps printing links an older yaks can follow. `find_note`/`is_note_anchor` accept both everywhere.

---
▸ 2026-11-14T23:27:57Z [notefmt-1]

Evidence so far (before `verify`).

- Round trip: `cargo test -p yaks --bins real_yak_bodies_convert_to_the_same_notes -- --nocapture` over the first 200 yak files of `yaks path`: `round trip: 200 yaks, 185 converted, 766 notes`, identical `NoteEntry` lists (meta, actor, text, order), same description, idempotent, and each note's anchor in both forms resolves to the same index in both bodies. With `YAKS_NOTES_LIMIT=5000`: `761 yaks, 702 converted, 3307 notes`. The test only reads the farm files.
- Merge driver: `the_new_note_shape_merges_like_the_old`, `metadata_and_mixed_sides_merge_to_one_shape` beside the unchanged old-shape tests.
- Schema gate: `store::note_shape_tests::a_farm_writes_the_shape_its_schema_says` (marker none/8 write the old shape, 9/10 the new, through `move_task`) and `farm::tests::a_farm_writes_the_note_shape_of_its_schema`. This build refuses a schema-9 farm as newer than `SCHEMA`, so the `Farm::update`/`ask` writers are exercised at schema 8 and the schema-9 half goes through `Farm::shape` and `move_task`; the other `Farm` writers share the same `self.shape()` argument. No schema-9 CLI run is possible until the migration yak bumps `SCHEMA`.
- GitHub render: `converted.md` (yaks-6c65, 6 notes) and `rendered.html` are attached. `gh api markdown -f mode=gfm` returns `<h4 dir="auto">2026-11-14T11:22:51Z</h4>` and `<em>` lines, six `<h4>` and no `id=`: the API returns a fragment, and GitHub adds `user-content-` ids when it renders the page itself (the design dok's own evidence shows the same fragment form). So the `<h4 id="user-content-2026-11-04t221303z">` ids are NOT shown by this check; they remain assumed from GitHub's slug rule until a converted file is viewed on a pushed branch.
- The on-disk shape changes only at schema 9: `store::SCHEMA` is untouched and nothing in this farm changes until the migration yak.

---
▸ 2026-11-14T23:30:20Z [notefmt-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-14T23:30:29Z [notefmt-1]

Summary. Notes can be read and written in the new shape (`#### <ts>` heading, italic `*actor · key: value*` line, no `---`); nothing on disk changes until the migration yak bumps `store::SCHEMA` (still 8).

- `store.rs`: `NoteShape`, `of_farm`, `format_note`, `append_note_meta_in`/`append_entry_in`, `scan_notes`/`parse_note_head`/`parse_italic_line`, `convert_notes`; `parse_notes`, `note_starts`, `description`, `with_description`, `drop_latest_request` read both shapes.
- `farm.rs`/`store::move_task_with` write through `Farm::shape()`; `links.rs` anchors accept timestamp and slug; `mergedriver.rs`, `tui/content.rs` (`assemble_in`), `blockedit.rs`, `sessions.rs` follow the body's shape; `markdown_lint::note_heading_in_description` added (not wired into `doctor`: `check.rs` is not mine).
- Docs: `docs/cli.md`, `docs/README.md`, the `yaks` skill's Metadata paragraph, the tidy-request text.
- Gate: `cargo test -p yaks` all green (1187 unit tests plus integration), `cargo fmt --check` clean; `yaks verify` PASS. The GitHub render shows `<h4>` headings but the markdown API gives no `id=` (see the evidence note).

---
▸ 2026-11-14T23:30:30Z [notefmt-1]
kind: move
from: shaving
to: shorn
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAGKnNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-14T23:31:03Z [linkplan-lead]
kind: landed
commit: 7585597eb78ba8f0b77214e793e968f0beb3da48
source: 2ab64a7ceddf62a6dda9d4a655b7459c35d8afce
branch: main
---

landed: 7585597e yaks-9f00: new note format (#### heading, italic actor/meta line): reader, schema-gated writer, convert_notes, slug anchors

---
▸ 2026-11-14T23:35:40Z [linkplan-lead]

Reviewed and landed (`7585597e`). Gate 1215 passed, `cargo fmt --check` clean. Full round trip over four farms is in the note on yaks-acc0 (762, 113, 145 and 682 yaks, all identical); it found 50 hand-headed yaks in the joel farm that the converter rightly leaves alone, which the worker's test did not allow for (fixed in a follow-up commit).

Not verified by the worker, and not by me: the rendered `<h4 id="user-content-...">` ids. The markdown API returns a fragment without ids, so they are assumed from GitHub's slug rule. They will be seen on a real pushed branch during the migration rehearsal.
