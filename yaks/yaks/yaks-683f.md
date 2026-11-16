---
id: yaks-683f
title: Save incomplete edits of yaks and descriptions/comments
status: shorn
type: feature
priority: 3
created: '2026-08-23T02:49:48Z'
updated: '2026-11-10T19:07:03Z'
tags:
- tui
---

Preserve a draft when an edit form or comment is dismissed or interrupted, so in-progress text is not lost.

---
▸ 2026-08-26T13:02:05Z

Related: yaks-275f (confirm-on-cancel for dirty yaks). Both address losing work when an edit is dismissed; coordinate the designs.

[683f-design](yaks-683f/683f-design.md)

---
▸ 2026-09-23T21:39:11Z [design-scout]

Design proposal attached

---
▸ 2026-09-23T21:39:11Z [design-scout]

Design attached (683f-design.md). Decisions:
1. Dirty cancel: replace 275f's y/N with 'k=keep draft (default) / d=discard / Esc=back', or drop the prompt and always keep silently? Lean: 3-way prompt, keep default.
2. Restore: auto-restore with a 'draft' badge + Ctrl-R to revert, vs a 'restore draft?' prompt? Lean: auto-restore.
3. Phase 1 saves on dismiss/quit only (autosave via the 250ms loop in P2), or autosave from day one? Lean: dismiss/quit first.
4. Mid-edit external changes clobbered on commit: in scope or separate yak? Lean: separate; 683f only warns on restore when base 'updated' differs. Drafts stored at `~/.config/yaks/<slug>/drafts/<key>.json`, keys `edit:<id>/comment:<id>/create:<parent|root>`.

---
▸ 2026-11-06T04:58:33Z [sched-1]
kind: answer
needs: agent
---

1. I like the three-way prompt idea.
2. I'm good to start by trying auto-restore.
3. Yeah, we can start there and see how it goes. I originally wanted this becauuse of crash bugs, but we haven't had that problem in a while.
4. SGTM.

---
▸ 2026-11-10T17:49:46Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-10T17:50:28Z [herd-unlock]
kind: pickup
---

picked up

herd-unlock: delegating phase 1 of the attached design to worker drafts-1 (Sonnet), with the four answers as decided.

---
▸ 2026-11-10T17:50:30Z [herd-unlock]

Claim (yakherd herd-unlock): worker `drafts-1`, Sonnet 5.5. Phase 1 of [the design](yaks-683f/683f-design.md), with the answers of 2026-11-06.

## Scope

- Surfaces: create form, edit form (`E`), comment (`M`): exactly the ones 275f guards.
- Storage: `$XDG_CONFIG_HOME/yaks/<slug>/drafts/<key>.json`, via `views_store::config_dir`; atomic writes; best effort.
- Save on dismiss (the dirty-cancel prompt's keep) and on quit with a dirty overlay open. No periodic autosave (P2).
- The 275f `Discard changes? (y/N)` becomes `k`=keep draft (default, also Enter) / `d`=discard / `Esc`=back to editing.
- Auto-restore on open, a `draft` badge, `Ctrl-R` reverts to the yak's current content. Warn (badge colour) when the yak's `updated` moved since the draft.
- Clear on commit, on discard; GC at startup (30 days, or yak gone/dead).
- Keys and help through `src/tui/keymap.rs` (modal keys: `EXTRAS`), `docs/tui.md`.

## Out of scope

Autosave, ask/answer drafts, a drafts picker, the `✎` tree marker, clobber-on-commit (design Q4: separate).

## Evidence

Tests: dismiss-keep then reopen restores; `d` discards; commit clears; stale warning. A toque text frame of a restored form with its badge and of the three-way prompt, attached. Gate `cargo test --workspace`. Judge: the yakherd at landing.

---
▸ 2026-11-10T17:52:18Z [drafts-1]

Start: Tue Nov 10 17:52:18 UTC 2026. Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-683f.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/v49c5a8gdr4r/yaks.

---
▸ 2026-11-10T18:27:52Z [drafts-1]

Plan (before coding), read of the code:

- New module `src/tui/drafts.rs`: the store (JSON file per draft, atomic write, GC) and the `impl App` pieces (keep/discard prompt key handler, restore on open, `Ctrl-R` revert, flush on exit).
- Drafts exist for `Overlay::Create` (create and edit forms) and `BlockEdit` with `EditAction::Comment`. The other 275f-guarded editors (block edit, attachment edit, ask/answer/request) keep the old `y/N` confirm: there is no draft for them, so `k=keep` would be a lie.
- `views_store::slug` becomes `pub(super)`; the drafts dir is `<config_home>/yaks/<slug>/drafts/`. Under `cfg(test)` the base is a pid-scoped dir in the OS temp dir, so no test touches the real `~/.config/yaks`.

[prompt](yaks-683f/prompt.txt "toque-style text frame (TestBackend, 100x16): the three-way prompt after dismissing a dirty edit form")

[restored](yaks-683f/restored.txt "text frame: edit form reopened, restored from the draft, with its 'draft (saved just now) · Ctrl-R original' badge")

---
▸ 2026-11-10T19:06:03Z [drafts-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-10T19:06:37Z [drafts-1]

Decision: the three-way prompt is a new `ConfirmAction::KeepDraft`, used only when the stashed overlay is a form or a new comment (`keeps_drafts`). Rejected: extending `PickAction` (a Pick's Esc means cancel, but here Esc must go back to editing) and replacing the `y/N` everywhere (block edits, attachment edits and ask/answer/request have no draft, so `k=keep` would be false). They keep `Discard changes? (y/N)`.

Decision: draft store lives in `src/tui/drafts.rs` (JSON file per key, tmp+rename); `views_store::config_dir` made `pub(super)` to share the per-farm config dir. Under `cfg(test)` the dir is `<tmp>/yaks-test-config-<pid>/<slug>/drafts`, so no test touches `~/.config/yaks` (checked: none created).

Decision: a save that is refused (conflict, name taken, parent finished) keeps what was typed as a draft, since the form closes on a failed save. `commit_create`/`commit_edit_form`/`commit_comment` now return whether the write happened; `commit_form` deletes the draft on success, else saves one. Not in the claim's list, but without it a closed-on-failure form loses text, which is the yak's whole point. Rejected: reopening the form after a failed save (changes 9f5c's clipboard behaviour).

Decision: the stale warning compares the yak's `updated` at the draft's start with its `updated` now (a field in the draft, not the design's body hash). A restored draft keeps its original `base_updated`, so it stays stale until reverted or saved.

Decision: `Ctrl-R` in the form/comment editor reverts only while a draft is restored (`is_draft_revert`); otherwise it is vim's redo as before. Rejected: always binding it (would remove redo in the description).

Decision: quit saves via `flush_open_draft` after the event loop ends. `q`/Ctrl-C are swallowed by an open overlay (they cancel the editor, which asks), so the only way to exit with an editor open is the loop failing; I did not add a panic hook (phase 2).

Decision: draft badge is magenta (restored) or yellow (yak changed since), in the form header; the comment's badge is text in its header label.

Decision: GC is `sweep_drafts` at `with_farm`: older than 30 days, or its yak is gone or dead. Create drafts have no yak and only age out.

Evidence: 15 new tests in `tui::tests::live` (prompt text, keep+restore, Enter keeps, parent-keyed create drafts, edit fields, `d` deletes, Esc writes nothing, commit clears, create clears, Ctrl-R, stale warning, comment keep/restore/Ctrl-R/send clears, flush on exit, clean form, `:q!`) + 4 unit tests in `drafts.rs` (round trip, key files, bad/foreign version, sweep). Three existing tests changed from `y` to `d` at the prompt (create_cancelled_with_esc, double_esc_cancels_the_comment_editor, edit_form_cancel_leaves_task_unchanged): the frames they assert changed on purpose. No insta snapshot changed (none showed the prompt or the form header).

Frames attached: prompt.txt and restored.txt.

Clippy: `cargo clippy --all-targets` reports 104 warnings before and after my change, none in drafts.rs.

Docs parity: `docs/tui.md` "Drafts" section; `?` overlay rows in `EXTRAS` (k/Enter, d, Ctrl-R). Grep `Discard changes` over docs/ .agents/ README.md src/: remaining hits are docs/tui.md (says the other editors keep it), `handlers.rs` (the y/N prompt for non-draft editors), `drafts.rs` doc comment. skills/yaks do not mention cancel.

Summary: dirty cancel of the create form, edit form and a new comment now offers keep/discard/back (keep default); keep saves a JSON draft under the farm's config dir; reopening restores it with a badge and Ctrl-R reverts; commit, discard and refused-save paths update the draft; startup GC. Gate: `yaks verify` PASS.

---
▸ 2026-11-10T19:06:37Z [drafts-1]
kind: move
from: shaving
to: shorn
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFR7IZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-10T19:07:03Z [herd-unlock]
kind: landed
commit: 686d328a0b281e8f6570ed738e7ea083e9a1037a
source: 61db35cd36c66c65155390004b4f3e04acbff7e4
branch: main
---

landed: 686d328a yaks-683f: keep unfinished form and comment edits as drafts (keep/discard/back prompt, auto-restore, Ctrl-R revert)
