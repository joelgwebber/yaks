---
id: yaks-1f84
title: 'Editor: auto-continue lists and quotes on Enter (edtui fork)'
status: shaving
type: feature
priority: 3
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T17:02:37Z'
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
