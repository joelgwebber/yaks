---
id: yaks-a57c
title: 'yaks attach <dir>: attach a directory with its layout'
status: shorn
type: feature
priority: 2
created: '2026-11-17T21:41:18Z'
updated: '2026-11-17T22:12:32Z'
parent: yaks-0229
tags:
- cli
- doks
---

Part of yaks-0229 (scope agreed by Joel). `yaks attach <id> <dir>` fails with `error reading d: Is a directory`, so an agent that wants to attach a small code directory attaches a `.tgz` instead.

## Want

- `yaks attach <id|dok|session> <dir>` stores the directory with its layout under the attachment directory (`<herd>/yaks/<id>/<dir-name>/...`), in one write and one commit.
- Use the same limits and skip rules as a directory source (`sources::MAX_DIR_FILES`, `MAX_DIR_BYTES`, its skip list: `.git`, `node_modules`, `target`, ...). Reuse that walk; don't write a second one.
- The link appended to the body names the directory (`[<dir-name>/](<id>/<dir-name>/)`), unless `--no-link`.
- Refuse `--name` with a directory only if it can't be made to mean "store under this directory name".
- The `yaks` skill says: attach a directory, never an archive. Update `--help`, `docs/cli.md`, and `.agents/skills/yaks` in the same change.

## Evidence

Tests: a nested directory attached to a yak and to a dok keeps its layout. Over-limit refuses and writes nothing. Skip names are skipped. A real run of the built binary, shown in the note.

#### 2026-11-17T21:42:27Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:43:00Z

*sources-triage*

Claim (yakherd `sources-triage`). Owner: worker `attach-2`. Scope: `yaks attach` of a directory, for yaks and for doks and session notes (`Farm::attach_many`, `attach_doc_many`, the `Attach` arm in `src/main.rs`), reusing the directory walk and limits from `src/sources.rs`. Expose that walk as a function; don't copy it.

- Out of scope: the TUI (yaks-3329), the source layout, and the skip rules themselves, which `srccli-1` is changing in yaks-3da7. Call the walk; don't edit its skip list.
- Evidence: the tests in the description, the gate, and a real run.

#### 2026-11-17T21:44:32Z

*attach-2*

Start: 2026-11-17T21:44:28Z (`date -u`). Farm file: `/Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-a57c.md`. Checkout: `/Users/joel/src/yaks/.delta/worktrees/h4mbnq0px7df/yaks`.

#### 2026-11-17T22:08:04Z

*attach-2 · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)

#### 2026-11-17T22:08:25Z

*attach-2*

Decision: reuse the walk by making `sources::dir_files` `pub(crate)` (was private `fn dir_files`). One-word visibility change; body, skip list, dot-file rule and limits untouched, so `srccli-1`'s yaks-3da7 edits to that function merge cleanly (keep their body, keep the `pub(crate)`). Rejected: copying the walk (a second skip list), or splitting it out of `add_on` (it already is a function).

Decision: a directory is passed to `Farm::attach_many`/`attach_doc_many` as files named `<dir>/<rel>` (no signature change); `store_attachments` makes the parent directories and writes one link per top-level directory (`store::attachment_dir_link`: `[proj/](<id>/proj/)`). Rejected: a new `attach_dir` method on Farm (duplicates lock, link and doc paths) and linking every file.

Decision: `--name` with a directory means "store under this directory name" (sanitized by `farm::attachment_name`, no extension borrowed), so it works for one directory, as for one file. A second path that would land under the same top-level name is refused.

Decision: an empty walk (everything skipped) is refused, and `--caption` with a directory is refused, rather than writing nothing silently or captioning nothing.

Decision: on a dok or session note a `.md` file inside the directory is refused for the whole attach (existing rule, unchanged: it would read as another document). Rejected: silently dropping `.md` files.

Evidence: new `tests/attach_dir.rs` (5 tests): nested layout on a yak (one commit, one link, skip names skipped), on a dok (and the `.md` refusal writes nothing), `--no-link`/`--name`, over-limit (201 files, over 5 MiB) refuses and writes nothing and no commit, empty/caption refused. `cargo fmt --check` clean; `cargo test -p yaks` all green; `yaks verify yaks-a57c` PASS.

Real run (release binary, scratch farm): `attached proj/ (3 files) to yak-8164 (.yaks/yak/yaks/yak-8164)`; stored `yak-8164/proj/README.md`, `proj/src/main.rs`, `proj/src/util/m.rs` (node_modules and .git skipped); body ends `[proj/](yak-8164/proj/)`; one commit `yaks: updated yak-8164; artifacts for yak-8164`. Dok: `attached proj/ (2 files) to doks/x.md`, link `[proj/](x/proj/)`. 201 files: `error: .../big holds more than 200 files; save a smaller directory`, exit 1.

Docs: `Attach` clap help, `docs/cli.md` (table row + "A directory" paragraph), `.agents/skills/yaks/SKILL.md` ("Attach a directory, never an archive").

#### 2026-11-17T22:08:26Z

*attach-2 · kind: move · from: shaving · to: shorn · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGzKtpIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-17T22:12:32Z

*sources-triage · kind: landed · commit: 230f026eb568292ee9bae623baa16c4de599c20f · source: c1010b1c47fd06c98306590e9533ca3fa5882467 · branch: main*

landed: 230f026e yaks-a57c: yaks attach <dir> stores a directory with its layout
