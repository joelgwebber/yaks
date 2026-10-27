---
id: yaks-a870
title: 'Schema 5 migration: comment header blank line + non-image attachment links'
status: shorn
type: feature
priority: 2
created: '2026-10-17T04:36:55Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-e9de
labels:
- markdown
- cli
needs: agent
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

---
▸ 2026-10-17T04:37:47Z [migrate-1]

Started Sat Oct 17 04:37:47 AM UTC 2026; path: /home/joel/src/yaks/.delta/worktrees/d7cjxmrxs40k/yaks/.yaks/yaks-a870.md; checkout /home/joel/src/yaks/.delta/worktrees/d7cjxmrxs40k/yaks

[migrate-dryrun-report](yaks-a870/migrate-dryrun-report.txt)

---
▸ 2026-10-17T05:01:23Z [migrate-1]

Decision: a schema-4 farm under this binary REFUSES every command except `yaks migrate` (`Farm::open` -> `OpenError::NeedsMigration`, message names `yaks migrate`; `Farm::open_for_migration` is what `migrate` uses). Also refused: `merge` from a schema-4 source, `answer` into a schema-4 shed. Rejected: read-both-refuse-writes (read-only commands would still work, but a farm half old shape, half new is exactly what a schema marker is meant to prevent, and every write path would need the check anyway); read-and-write-either-shape (no point in a schema number then). Readers (`parse_notes`, `content::parse`, `split_note_head`) still accept both shapes, so old commits, `yaks status`/`changes` against HEAD, and sheds on schema 4 still PARSE; they just can't be operated on until migrated. Old binary on a schema-5 farm: refuses as too new (existing SchemaTooNew).

---
▸ 2026-10-17T05:01:23Z [migrate-1]

Decision: fixups are a `FIXUPS: &[Fixup]` list in new `src/migrate.rs` (name, unit, `fn(&Ctx,&str)->(String,usize)` over the body only). `Farm::migrate` = lock + `migrate::run`. A later fixup is one more entry; `Ctx` carries the farm's yak ids so a fixup can be id-aware. Rejected: a per-schema-version ladder (4->5, 5->6): the user said fixups join one migration, and each fixup is idempotent so the whole list runs every time. The schema marker is written LAST, after every file (atomic temp+rename per file), so an interrupted run is just re-run. Frontmatter (incl. `updated:`) is preserved byte for byte: a migration commit is only the shape change, no `updated:` churn.

---
▸ 2026-10-17T05:01:24Z [migrate-1]

Decision: attachment-links fixup only rewrites `![stem](<id>/<file>)` where `<id>` is a yak id of THIS farm, `<file>` has no further `/` or space, and the extension is not in {png,jpg,jpeg,gif,webp,svg} (case-insensitive; no extension = non-image). Skips fenced code blocks. Rejected: rewriting any `![..](x/y.txt)` (would touch foreign/URL links and examples). Single source of the link shape: `store::attachment_link` / `is_image_name`, used by `attach`, `rename_attachment` and the fixup's extension rule.

---
▸ 2026-10-17T05:01:24Z [migrate-1]

Decision: `rename-attachment` rewrites BOTH `![old](path)` and `[old](path)` (new `replace_link`), and the form follows the NEW extension (rename notes.md -> notes.png gains the `!`). Rejected: leaving the form alone on a rename (would leave a `[x](a.png)` that renders as a text link, a `![x](a.txt)` that renders broken).

---
▸ 2026-10-17T05:01:24Z [migrate-1]

Decision: comment-header fixup inserts the blank line only when the header is followed by non-blank text that is not itself the next comment's `---` (an empty comment stays empty). `content::parse` strips ONE leading blank line after the header, so both shapes parse to identical blocks; `assemble` always writes the schema-5 shape (a TUI comment save upgrades that one comment; `parse(old)` -> `assemble` is deliberately not byte-identical to an old body). `store::append_note` writes the new shape; `parse_notes` already trims text so it needed no change.

---
▸ 2026-10-17T05:01:24Z [migrate-1]

Decision: `.yaks` fixtures migrated: tests/fixtures/farm now schema 5 with fix-0003's comment in the new shape, 3 insta snapshots re-accepted (each .snap diff read: only the added blank line after the `▸` header). tests/attribution.rs `has_move` now expects header/blank/text; tests/sheds_layouts.rs `migrate_by_hand` writes schema 5. The schema-3 refusal tests are unchanged (schema 3 is still the old layout, `old_layout` is now `Older(v) if v < 4`).

---
▸ 2026-10-17T05:01:24Z [migrate-1]

Evidence: dry-run/real run over a COPY of this repo's farm (target/farmcopy, not .yaks): attached migrate-dryrun-report (the `.yaks/yaks-a870/` file). Counts: comment-header-blank-line 1283 comments in 396 yaks; attachment-links 77 links in 51 yaks; 396 yak files rewritten. After: `doctor` All clear, second `migrate` = "nothing to migrate: the farm is on schema 5", schema file 5, 0 non-image `![` left, 12 image `![` kept, 77 plain `[stem](id/file)` links. A diff of the copy against the original shows only added blank lines and the 77 dropped `!` (154 diff lines).
New tests (all fail before: no `migrate`, SCHEMA 4): migrate::tests (4: header fixup both shapes + idempotent, empty comments/prose rules untouched, link fixup unbangs non-images only, ignores foreign links + code fences); farm::tests::attach_links_images_with_a_bang_and_other_files_without (attach per extension, rename keeps/gains/loses the bang, rewrites a schema-4 `![..](x.txt)`); farm::tests::migrate_upgrades_a_schema_4_farm_once (open refuses, dry run writes nothing, frontmatter untouched, second run no-op); content::tests::parses_both_comment_shapes_to_the_same_blocks; store note_tests/schema_gate; merge_refuses_a_schema_4_source. Gates: `cargo test --release` and `cargo test -p yaks skills` ok (608 unit + all integration passed).
Docs parity: docs/cli.md (migrate row, attach, rename-attachment, move-entry shape), docs/README.md (schema 5, note shape), bundled skill .agents/skills/yaks/SKILL.md (attach row, move entries, schema-4 note), RELEASING.md (landing order), clap help (Migrate, Attach, RenameAttachment in src/main.rs). README.md describes neither format: no change. Not done / for the coordinator: run the real migration on `.yaks/` after landing (per RELEASING.md), then the TUI detail pane etc. (other workers) should read the blank line fine (parse strips it; detail.rs block_index_per_line only looks at `---` + `▸`).

---
▸ 2026-10-17T05:01:27Z [migrate-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-17T05:01:35Z [migrate-1]

Correction to the evidence note: the new tests were written after the code, so I did NOT observe them failing first; what I observed failing first were the existing tests the change moved (append_note shapes, schema_gate, init schema, attribution has_move, 3 insta snapshots, fixtures). Coordinator: if you want a true red run, stash src/migrate.rs-dependent code and re-run, or review the tests for vacuity.

---
▸ 2026-10-17T05:01:35Z [migrate-1]

moved: shaving -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-a870` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-a870 --drop-request`, which removes this request note, so no new note is left behind.
