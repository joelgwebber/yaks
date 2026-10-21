---
id: yaks-1f84
title: 'Editor: auto-continue lists and quotes on Enter (edtui fork)'
status: shorn
type: feature
priority: 3
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T17:09:39Z'
parent: yaks-d2bb
labels:
- markdown
- tui
---

Enter at the end of a `> `, `- `, `* `, `1. ` (renumbered to `2. `) or `- [ ] ` line starts the next line with the same prefix and indent; Enter on an item with an empty body removes the prefix (ends the list/quote). Insert mode only.

Implement in the edtui fork (a line-break hook or a markdown-aware `LineBreak` option on the `yaks-integration` branch), not as a yaks-side key shim: yaks-182f removed those because shims break vim count composition. Then `cargo update -p edtui`.

---
▸ 2026-10-17T05:00:46Z [delta-lead]

asked: needs human

Auto-continue lives in the edtui fork (github.com/joelgwebber/edtui, `yaks-integration`), and Cargo pulls it from GitHub, so landing it means pushing a branch to that fork and then `cargo update -p edtui` here. OK for me (or a worker) to push a feature branch to the fork and merge it into `yaks-integration`? If you'd rather review fork changes yourself first, say so and I'll stop at a pushed feature branch.

---
▸ 2026-10-17T13:22:47Z [Joel Webber]

Yup, works for me. Feel free to commit whatever changes you need to our edtui fork. We'll resolve them later.

---
▸ 2026-10-17T17:02:37Z [delta-lead]

picked up

Joel OK'd fork commits. Worker autocont-1 develops it; the coordinator reviews and merges into yaks-integration.

---
▸ 2026-10-17T17:02:37Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T17:02:37Z [delta-lead]

Claim (yakherd delta-lead, 2026-10-17): owner **autocont-1**.

Two repos:

- **edtui fork.** Clone https://github.com/joelgwebber/edtui into `target/edtui` of your own checkout (`gh repo clone joelgwebber/edtui target/edtui -- -b yaks-integration`). Branch `yaks/md-autocontinue` off `yaks-integration`. Implement an opt-in markdown continuation for Insert-mode Enter, as an option on `EditorState` (e.g. `set_markdown_continuation(bool)`), so plain edtui users are unaffected and vim counts still compose (no yaks-side key shim; see yaks-182f):
  - continue `> `, `- `, `* `, `+ `, `1. ` (renumbered to `2. `; `1)` too), `- [ ] ` / `- [x] ` (as `- [ ] `), keeping leading indent;
  - Enter on an item whose body is empty removes the marker (ends the list or quote) instead of continuing;
  - Enter in the middle of a line splits as today, and the second half gets the prefix.
  - Tests in the fork's own test style. Run the fork's own `cargo test`.
  - Push ONLY that feature branch (`git push origin yaks/md-autocontinue`). Do NOT push or merge `yaks-integration`: the coordinator merges after review.
- **yaks.** Point at your pushed branch *temporarily* to integrate: in Cargo.toml change the edtui `branch` to `yaks/md-autocontinue` and `cargo update -p edtui` (this is the one allowed Cargo edit for this yak). Enable the option for multi-line editors in src/tui/editor.rs (`Editor::new` / `multiline_field` callers), never for single-line fields. Add a yaks-side test driving Enter through the comment editor. Update docs/tui.md (editor section) and help if keys are described there.

Evidence: fork tests; yaks tests; a headless or toque frame of the comment editor after typing `- a<Enter>b<Enter><Enter>` and `> q<Enter>`. Judge: coordinator at landing; the coordinator then merges your branch into `yaks-integration`, switches Cargo.toml back to `branch = "yaks-integration"` and re-runs `cargo update -p edtui`.

---
▸ 2026-10-17T17:03:26Z [autocont-1]

Started autocont-1.

- date -u: Sat Oct 17 05:03:16 PM UTC 2026
- yaks path yaks-1f84: /home/joel/src/yaks/.delta/worktrees/8jz5f03z51k9/yaks/.yaks/yaks-1f84.md (my checkout)

[autocont-list](yaks-1f84/autocont-list.txt)

[autocont-quote](yaks-1f84/autocont-quote.txt)

---
▸ 2026-10-17T17:09:27Z [autocont-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T17:09:39Z [autocont-1]

Decision: continuation lives in `LineBreak` itself, behind `EditorState::set_markdown_continuation(bool)` (default off).

- Rejected: a yaks-side Enter shim (yaks-182f: shims break count composition).
- Rejected: a new action bound to Enter. It would need keymap changes in both vim and emacs maps, and plain `LineBreak` users (Ctrl-J, the `Ctrl-o` chain) would diverge.
- Result: Enter, Ctrl-J, counts and dot-repeat all go through one path, and the undo behaviour is unchanged.

Decision: the empty-item case clears the line and leaves the cursor at column 0 of that same line (the vim and GitHub behaviour). It does not insert a blank line after it.

Decision: a `- [x] ` item continues as `- [ ] `. A `3` count renumbers each extra line (`2.`, `3.`, `4.`).

Decision: Enter with the cursor inside the marker (before its end) is a plain break. At or after the end of the marker, the second half gets the prefix.

Decision: only a single leading-space gap is kept for the checkbox. The text between the bullet and the box is always one space.

Decision: yaks enables the option in `Editor::new` (when not single-line) and in `multiline_field` (the create/edit form's content blocks), so both multi-line surfaces behave the same. This is two one-line changes.

Fork tests: `cargo test` in the fork gave 175 lib tests + 23 doctests ok. It was run from an rsync copy in /tmp because `target/edtui` sits inside the yaks workspace and cargo refuses to build it there. The 10 markdown tests failed first while the option was inert (an `if false &&` toggle), then passed.

yaks tests: `enter_continues_lists_and_quotes_in_the_comment_editor` fails with `- a\nb\n` if the option is off in `Editor::new`, passes with it on. `enter_does_not_continue_in_a_single_line_field` was also added.

Frames attached (headless, my build): `- a<Enter>b<Enter><Enter>` gives `- a / - b / (blank)`, and `> q<Enter>r<Enter><Enter>1. x<Enter>` gives `> q / > r / 1. x / 2. `.

Docs: `docs/tui.md` Edit table row, plus the `?` help overlay entry (`Enter`, `help_content` in `src/tui/render.rs`). No skills describe editor keys. The help bar (`help_hint`) is left alone: Enter is too generic a key to list there.

Fork branch `yaks/md-autocontinue`, head ff5c1a9db3420a8d1d6f84d86a4b87155612d5c3 (one commit on top of yaks-integration 6ba0dbf), pushed. yaks `Cargo.toml` edtui branch points at it, `Cargo.lock` is updated, and it is temporary: the coordinator switches it back after merging into `yaks-integration`.

---
▸ 2026-10-17T17:09:39Z [autocont-1]

moved: shaving -> shorn
