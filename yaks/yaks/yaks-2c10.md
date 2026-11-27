---
id: yaks-2c10
title: update --description deletes the yak's notes
status: shorn
type: bug
priority: 1
created: '2026-11-27T02:09:08Z'
updated: '2026-11-27T02:55:52Z'
parent: yaks-7e32
tags:
- cli
---

`yaks update <id> --description <text>` replaces the whole body, notes included. Reproduced: create, `--note n1`, `--description d2` leaves only `d2`; `show --json` goes from one note to none.

The TUI already uses `store::with_description`, which replaces the description and keeps the note blocks byte for byte; the CLI passes the new text straight into `TaskEdit::description` ([main.rs](https://github.com/joelgwebber/yaks/blob/7c1452bd5712911a15ba5e244c9079af914a5c76/src/main.rs#L3137-L3155)). The help text "Replace the body (markdown)" is literally true, and yaks-139b's guidance warns about it, but nobody wants the notes gone.

Fix: `update --description` replaces the description only (through `with_description`), and the help says so; a whole-body replace, if anyone needs one, is an explicit flag.

#### 2026-11-27T02:36:26Z

*herd-unify · kind: move · from: hairy · to: shaving · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAHx7AZIBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-27T02:36:57Z

*herd-unify*

Claim (yakherd `herd-unify`). Owner: worker `desc-1` (Sonnet 5.5), base `4f09d2d2`.

Scope: `yaks update --description` / `--description-file` replace the description only and keep the note blocks byte for byte, via `store::with_description` (what the TUI already does). Code: `Command::Update` in `src/main.rs` (and `update_many` if the replacement must happen per yak, since each yak has its own notes); clap help for those flags; `docs/cli.md`; the `yaks` skill if it describes the flag; check yaks-139b's warning text in skills.

Out of scope: `create` (no notes yet), `bulk`, any new whole-body flag (note it as a follow-up idea instead), `dok set --body`.

Evidence: a test that fails before (create, note, `update --description` drops the note) and passes after; the gate `cargo test -p yaks`; a scratch-farm run pasted in a note. Judge: the yakherd at landing.

#### 2026-11-27T02:37:59Z

*desc-1*

Start: Fri Nov 27 02:37:59 UTC 2026; farm file /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-2c10.md; checkout /Users/joel/src/yaks/.delta/worktrees/xab5c0v8bns4/yaks

#### 2026-11-27T02:49:22Z

*desc-1*

Decision: reuse `store::with_description` inside `update_many` (main.rs), per yak, rather than change `Farm::update` or `TaskEdit` semantics. Rejected: changing `TaskEdit::description` itself (the TUI's block editor and `release.rs` pass whole bodies on purpose), and a new whole-body flag (out of scope; follow-up idea).

Decision: the CLI also sets `base: Some(old body)`, so a note appended between the read and the write is kept (yaks-9f5c rebase) and a concurrent rewrite is refused (exit 1) instead of overwritten. Rejected: no base, which would race a concurrent note.

Decision: `--description ""` clears the description and keeps the notes (`with_description` semantics). Rejected: treating it as "clear the whole body".

Decision: existing tests that asserted the old behaviour were updated (`inline_text_is_unchanged` in tests/body_input.rs, `create_and_update_store_links_a_host_can_follow` in tests/stored_links.rs).

Scratch-farm run (release binary): create with description d1, `--note n1`, `update --description d2`: show prints `d2`, then the n1 note, `notes: 1`. `update --description ""` leaves `notes: 1`.

Test: `update_description_keeps_the_notes` (tests/body_input.rs) fails on the old code (body is just `new`) and passes now.

#### 2026-11-27T02:54:49Z

*desc-1 · kind: verify · result: pass*

verify: `cargo test -p yaks` -> PASS (exit 0)

#### 2026-11-27T02:54:57Z

*desc-1*

Summary: `update --description`/`--description-file` now replace the description only and keep every note byte for byte, per yak (`update_many` in src/main.rs uses `store::with_description` plus `base`). Clap help, `docs/cli.md`, README and the `yaks` skill's command table are updated. `cargo test -p yaks` PASS via `yaks verify`; `cargo fmt --check` clean. New test `update_description_keeps_the_notes` fails on the old code and passes now.

Follow-up idea: an explicit whole-body replace flag, if anyone needs one.

#### 2026-11-27T02:54:58Z

*desc-1 · kind: move · from: shaving · to: shorn · thread: ksQQvf9tyN_TT6-lYi87QcE1GpLOAIK5vJIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-27T02:54:58Z

*desc-1 · kind: change*

yaks update --description now replaces only the description and keeps the yak's notes

#### 2026-11-27T02:55:52Z

*herd-unify · kind: landed · commit: f1dc3b5ca66e5687e1584f944928ddf5dfb19e34 · source: e9c0af49438c2f4077bd85f47b1f554c5f00dfba · branch: main*

landed: [f1dc3b5c](https://github.com/joelgwebber/yaks/commit/f1dc3b5ca66e5687e1584f944928ddf5dfb19e34) yaks-2c10: update --description keeps the notes
