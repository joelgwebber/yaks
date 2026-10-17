---
id: yaks-a870
title: 'Schema 5 migration: comment header blank line + non-image attachment links'
status: shaving
type: feature
priority: 2
created: '2026-10-17T04:36:55Z'
updated: '2026-10-17T04:37:18Z'
parent: yaks-e9de
labels:
- markdown
- cli
---

Joel (2026-10-17, on yaks-2362 and yaks-3b8d): do these format changes as a real schema migration in the next release, and expect more fixups to join it.

## Shape

- Bump `store::SCHEMA` 4 → 5. Today `schema_status` treats Older as "refuse" (the 3→4 path points at `npx yaks@0.0.16 migrate`). For 4 → 5 the migration lives in this binary: a `yaks migrate` (or reuse/extend the existing migrate path; check git history for 0.0.16's `Farm::migrate`) that rewrites files in place under the farm lock, idempotent, `--dry-run` prints what changes; every other command on a schema-4 farm refuses and names `yaks migrate`.
- Built as a list of named, per-file fixups so later ones slot in. The first two:
  1. **Comment header blank line** (yaks-2362): `---` / `▸ <ts> [actor]` / text → `---` / `▸ <ts> [actor]` / blank / text. `store::append_note` and `content::assemble` write the new shape; `content::parse` / `split_note_head` and every other `▸` reader (`grep -rn 25b8 src`: sheds, status, main, store) accept both, so un-migrated sheds and old commits still parse.
  2. **Non-image attachment links** (yaks-3b8d): `![stem](<id>/<file>)` where the file is not an image (png, jpg, jpeg, gif, webp, svg) → `[stem](<id>/<file>)`. `Farm::attach` writes the right form; `rename-attachment` rewrites both.
- Team-farm care: a migration commit touches nearly every yak file; sheds (other checkouts) on schema 4 must keep reading. Document the landing order in RELEASING.md / docs.

## Evidence

Round-trip tests for both fixups and both shapes; a dry-run over a copy of this repo's farm attached (counts per fixup); `yaks doctor` clean after; docs (`docs/cli.md`), `--help`, skills (Task format, attach) updated in the same change.

---
▸ 2026-10-17T04:37:18Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-17T04:37:18Z [delta-lead]
Claim (yakherd delta-lead, 2026-10-17): owner **migrate-1**.

Scope: src/store.rs (`SCHEMA`, `schema_status` handling, `append_note`, `split_note_head`/note parsing), src/farm.rs (a `Farm::migrate` under the lock; `attach` / `rename_attachment` link forms), src/main.rs (a `Migrate` command variant + dispatch only), src/tui/content.rs (`parse` accepts both shapes, `assemble` writes the new one), any other `▸` reader (`grep -rn 25b8 src`: sheds.rs, status.rs) made to accept both shapes, docs/cli.md, RELEASING.md, README/skills text describing the comment or attach format, `--help`. Earlier art: `git show adfd0c1` (0.0.16's `yaks migrate`, removed in 9b9f02b): reuse its shape (dry-run, idempotent, lock, report) where it fits.

Out of scope: src/tui/detail.rs, render.rs, editor.rs, markdown.rs, docshots.rs (other workers own them right now). Do NOT migrate this repo's own `.yaks/` in your commit: the yakherd runs the migration on the farm after landing, as its own commit. Attach a `--dry-run` report over a COPY of this repo's farm instead.

Design decisions already made (Joel, on yaks-2362 / yaks-3b8d): new comment shape `---` / `▸ <ts> [actor]` / blank line / text; non-image attachments (anything but png, jpg, jpeg, gif, webp, svg) linked as `[stem](<id>/<file>)`, images keep `![stem](...)`; done as schema 4 → 5 with a migration in this binary, structured as a list of named fixups so more can join. Open choice for you: how a schema-4 farm behaves under a schema-5 binary (refuse everything but `migrate`/`doctor`/read-only? or read both and only refuse writes?). Prefer the least disruptive rule that keeps files authoritative; record it as a `Decision:` note. If it turns out to be a real user-visible fork, `yaks ask`.

Evidence: failing-first round-trip tests (parse/assemble of both shapes; append_note new shape; attach writes the right form per extension; rename-attachment rewrites both forms; migrate idempotent: a second run changes nothing); the dry-run report attached; gate `cargo test --release` + `cargo test --workspace`. Judge: coordinator at landing.
