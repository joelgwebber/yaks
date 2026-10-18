---
id: yaks-27c6
title: Shear-with-comment in TUI
status: shorn
type: feature
priority: 3
created: '2026-09-23T04:27:01Z'
updated: '2026-10-18T06:04:43Z'
labels:
- tui
---

If `yaks doctor` is going to complain about yaks shorn-without-comment, then the TUI should at least give you a chance to add a brief comment when shearing.

---
▸ 2026-10-18T05:53:21Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T05:53:21Z [delta-farm]

Claim (delta-farm, yakherd; trial round 1 of yaks-6a34). Owner: `shear-1`.

Goal: when a yak is moved to shorn in the TUI, offer an optional comment in the same action, so a TUI shear can satisfy `yaks doctor --strict` ("shorn yak with no recorded note"). Empty input = shear with no note (never block the shear). The note is written BEFORE the move (one `Farm::update` with the note, then `Farm::transition`), attributed like other TUI notes (`crate::actor::resolve(None)`).

Scope: the single-yak state pick (`resolve_pick`, `PickAction::State` with `'n'`, src/tui/handlers.rs) and any other single-yak path to Shorn you find (record which). Bulk shear (`PickAction::BulkState`): one prompt whose note goes on every moved yak, or no prompt; your call, recorded as a `Decision:` note. Prefer reusing an existing input overlay (the editor/`BlockEdit`, or the `a` ask/answer input) over a new `Overlay` variant; if you must add one, add it at the END of the enum.

Out of scope: CLI changes (`yaks shorn` takes no note by design), other transitions, doctor.

Evidence: a TUI test in src/tui/tests.rs that fails before (shear via the picker with a comment -> the yak is shorn AND carries the note; empty comment -> shorn, no extra note). A headless toque frame of the prompt, attached (`yaks attach`). `?` help (`help_content`, src/tui/render.rs) and docs/tui.md updated. Gate: `cargo test -p yaks` (the `tui` label default). Judge: delta-farm at landing; look-and-feel goes to Joel with the frame.

---
▸ 2026-10-18T05:54:03Z [shear-1]

Started Sun Oct 18 05:54:03 AM UTC 2026. yaks path yaks-27c6: /home/joel/src/yaks-farm/.yaks/yaks-27c6.md

[shear-comment-prompt](yaks-27c6/shear-comment-prompt.txt)

---
▸ 2026-10-18T06:02:31Z [shear-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-18T06:02:55Z [shear-1]

Decision: reuse the single-line `Overlay::Edit` (an `Editor` like the `a`/`A` ask/answer prompts) with a new `EditAction::Shear(ShearTarget)` variant (`ShearTarget::{One, Bulk}` in src/tui/editor.rs). No new `Overlay` variant. Rejected: a new overlay (conflicts with the parallel worker; this one gets Enter/cancel/vim handling for free); the multiline comment editor (too heavy for a "brief comment", and Enter has to shear).

---
▸ 2026-10-18T06:02:55Z [shear-1]

Decision: bulk shear (`S` with marks, then `n`) prompts once, and the one comment goes on every yak that actually moves. Yaks already shorn in the set get no note and no move. Rejected: no prompt for bulk (then a bulk TUI shear could never satisfy `doctor --strict`, which is the point of this yak); one prompt per yak (tedious for a set).

---
▸ 2026-10-18T06:02:55Z [shear-1]

Decision: cancel (Ctrl-C, Esc in emacs, double-Esc in vim; the editor's usual cancel) aborts the shear entirely and the yak stays where it was. Only Enter shears; an empty or whitespace-only line shears with no note. Rejected: cancel = shear without a note (a cancel that still mutates would surprise people).

---
▸ 2026-10-18T06:02:55Z [shear-1]

Decision: the transition code was factored out of `resolve_pick` into `App::transition_one` and `App::bulk_transition` (both take an optional note, which is written first via `Farm::update` with `actor::resolve(None)`, then `Farm::transition`), so the picker and the comment-commit share one path. Behaviour for h/s/x is unchanged. Prompt label: `Shear <id> — optional comment (Enter shears · Ctrl-C cancel): `, shortened from a first draft that pushed typed text off a 100-col line.

---
▸ 2026-10-18T06:02:55Z [shear-1]

Summary (shear-1):

- Single-yak paths to Shorn in the TUI: `S` then `n`, and `E` on the detail `Status:` row (it opens the same picker). Both now open a one-line comment prompt. Bulk `S` with marks then `n` prompts once (see Decision). The CLI is unchanged.
- Files: src/tui/handlers.rs (`resolve_pick`, new `open_shear_comment`, `commit_shear`, `note_before_move`, `transition_one`, `bulk_transition`, and the `EditAction::Shear` arm in `commit_edit`), src/tui/editor.rs (`EditAction::Shear`, `ShearTarget`), src/tui/render.rs (`help_content`: a `S → n` line beside the state entry), docs/tui.md (Edit table plus the bulk row), src/tui/tests.rs.
- Tests (in `mod live`, beside the state-pick tests): `state_pick_shear_records_the_optional_comment`, `state_pick_shear_with_empty_comment_adds_no_note`, `state_pick_shear_cancel_leaves_the_yak`, `bulk_shear_comment_goes_on_every_moved_yak`. The existing `state_pick_transitions_and_reloads` and `bulk_state_transition_shorns_the_marked_set` now press Enter at the prompt. Against HEAD's handlers.rs/editor.rs the new tests fail to compile (`no variant ... named Shear found for enum EditAction`).
- No insta snapshots changed. The `?` overlay snapshot, if any, did not move. `cargo test --release --bin yaks`: 716 passed.
- Real run (my release build, throwaway farm in target/shear-demo): headless `key S`, `key n`, `type release notes drafted`, `key Enter` gives the frame `yak-2283 → shorn`. `yaks show` lists the note `release notes drafted` and then `moved: hairy -> shorn` (note first), and `yaks doctor --strict` reports "All clear". The prompt frame is attached as shear-comment-prompt.txt.
- Gate: `yaks verify yaks-27c6` PASS.

---
▸ 2026-10-18T06:02:55Z [shear-1]

moved: shaving -> shorn

---
▸ 2026-10-18T06:04:43Z [delta-farm]

Landed (delta-farm): worker commit d870bcb merged by SHA as 4a6a3e4 (land.sh: base an ancestor, result identical to the computed merge, doctor clear). My checks: full `cargo test --release` 844 passed; real run on a scratch farm with my build, driving `S`, `n`, a comment, Enter: the comment note then `moved: hairy -> shorn`, `doctor --strict` All clear; `S`, `n`, Ctrl-C leaves the yak hairy. Accepted the out-of-brief edit to src/tui/editor.rs (the `EditAction` enum lives there). Cosmetic, not acted on: the long prompt label scrolls typed text off a ~110-column line.
