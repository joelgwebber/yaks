---
id: yaks-56c1
title: Edit text attachments inside the TUI (follow-up to the viewer)
status: shorn
type: feature
priority: 3
created: '2026-10-18T06:15:31Z'
updated: '2026-10-18T20:13:09Z'
parent: yaks-699b
labels:
- tui
- attach
---

yaks-699b shipped a read-only in-TUI viewer for text attachments (`Overlay::Attachment`, `AttachView` in src/tui/handlers.rs); `O` in it still opens the OS opener for editing. This adds editing in place: a key in the viewer opens the text in the edtui editor glue (src/tui/editor.rs, needs a save path for a file rather than a yak field), and saving writes the file back under the farm lock (`store::lock` on the farm root). Markdown re-renders after save. Out of scope: images, binaries, renaming.

---
▸ 2026-10-18T20:00:00Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T20:00:00Z [delta-farm]

Claim (delta-farm, yakherd; trial round 2 of yaks-6a34). Owner: `attach-edit-1`.

Goal: edit a text attachment inside the TUI from the viewer yaks-699b shipped (`Overlay::Attachment`, `AttachView`, `handle_attachment_key` in src/tui/handlers.rs). A key in the viewer (pick a free one; `E` matches the detail pane's edit key) opens the text in the existing editor glue (src/tui/editor.rs): add ONE `EditAction` variant for "save this attachment file" (beside `Attach`/`RenameAttachment`, not at the end). Saving writes the file back under the farm lock (`store::lock(farm root)`), then the viewer re-opens on the new content (markdown re-rendered). The file is resolved from the live farm (the viewer already holds the path). Only text the viewer accepts is editable; images/binaries unchanged.

Scope: `handle_attachment_key`, `AttachView` and the save path in src/tui/handlers.rs; the one `EditAction` variant + its commit in src/tui/editor.rs; the viewer's status hint in src/tui/render.rs; `help_content`; docs/tui.md. Do NOT touch `Farm::update` / `TaskEdit` (clobber-1 changes them) or src/tui/blockedit.rs.

Evidence: tests that fail first: open viewer, edit key, change text, save: file on disk changed, viewer shows the new content; Esc/cancel leaves the file unchanged. A headless frame of the editor over an attachment, attached. Gate `cargo test --workspace`. Judge: delta-farm; look-and-feel to Joel with the frame.

---
▸ 2026-10-18T20:00:58Z [attach-edit-1]

Start (attach-edit-1): `date -u` = Sun Oct 18 08:00:55 PM UTC 2026. `yaks path yaks-56c1` = /home/joel/src/yaks-farm/.yaks/yaks-56c1.md

[attachment-edit-frames](yaks-56c1/attachment-edit-frames.txt)

---
▸ 2026-10-18T20:11:30Z [attach-edit-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-18T20:11:53Z [attach-edit-1]

Decision: the variant is `EditAction::EditAttachment(AttachView)`, which carries the whole view, not just a path. Saving needs the path, the target and the shed; the dirty check and the on-disk guard need the text the edit started from; and cancel has to return to the viewer. Carrying the view gives all of that without re-reading the file. Rejected: `EditAttachment { path, target }`, which would have to re-read the file to rebuild the viewer and could not tell whether it changed meanwhile.

---
▸ 2026-10-18T20:11:53Z [attach-edit-1]

Decision: the save happens under `store::lock(farm root)`. It re-reads the file first, and if the file changed since `E` it writes nothing, keeps the editor open with the typed text and says "changed on disk: not saved (Ctrl-S again overwrites)". A second Ctrl-S then overwrites on purpose. Rejected:
- a blind overwrite, which silently clobbers an agent's `yaks attach` or an external editor;
- a three-way merge, which is out of proportion for an attachment.

The write is `store::write::atomic_write` (temp + rename), the same as yak files.

---
▸ 2026-10-18T20:11:53Z [attach-edit-1]

Decision: `E` refuses (with a notification) when the viewer found the file in another shed (`AttachView.shed` is Some). That copy belongs to another checkout's farm, and locking or writing a different farm from this TUI is a cross-shed write the claim did not decide. `O` still opens it externally. Rejected: editing it under that shed's lock.

---
▸ 2026-10-18T20:11:53Z [attach-edit-1]

Decision: cancel and dirty rules for the attachment editor.
- Dirty means the text differs from the file as opened, not "non-empty" (a file always has text).
- Ctrl-C, a double Esc and `:q` on unchanged text go straight back to the viewer.
- On changed text they ask "Discard changes?"; `y` returns to the viewer with the file untouched, `n` returns to the editor.
- Saving unchanged text says "unchanged" and writes nothing.

This needed two small helpers in handlers.rs: `edit_is_dirty` (replacing the inline check at its two call sites) and `viewer_behind` (used by `request_cancel`, `cmd_quit(force)` and the `DiscardEdit` confirm). Rejected: closing all the way out, which loses the viewer's context.

---
▸ 2026-10-18T20:11:53Z [attach-edit-1]

Done (attach-edit-1). `E` in the attachment viewer opens the text in the multi-line editor (vim opens in Normal, as for any existing text). Ctrl-S writes it back under the farm lock and returns to the viewer re-rendered; cancel returns unchanged. Images and binaries never reach the viewer, so they have no `E`.

Files:
- src/tui/editor.rs: one variant, `EditAction::EditAttachment`, after `RenameAttachment`.
- src/tui/handlers.rs: `AttachView` doc, `handle_attachment_key` gets `E`, new `open_attachment_edit`, `commit_attachment_edit` and `reopen_attachment_edit`, the `commit_edit` arm, the `edit_is_dirty` and `viewer_behind` helpers.
- src/tui/render.rs: viewer hint and `help_content` line.
- docs/tui.md: the viewer row.
- src/tui/tests.rs: 4 tests beside the viewer tests.

Evidence:
- Tests that fail first: `e_in_the_viewer_edits_the_attachment_and_saves_it_back`, `cancelling_an_attachment_edit_leaves_the_file_unchanged`, `an_attachment_changed_on_disk_meanwhile_is_not_overwritten`, `an_image_attachment_offers_no_edit`. With the `E` arm removed the first three FAIL ("14 passed; 3 failed"); restored, all 17 attachment tests pass. The image test passes either way: it guards the existing rule that images skip the viewer.
- Headless session with target/release/yaks (`tui --headless --size 100x24`) on a temp farm: the editor frame over `notes.md`, the edited frame, then the viewer re-rendered with "saved …/notes.md". The file on disk was then "Edited **in the TUI**.\n\n# Findings…". Attached as attachment-edit-frames.txt.
- No insta snapshot changed.
- Gate `yaks verify yaks-56c1`: PASS (731 unit tests + the integration suites).

---
▸ 2026-10-18T20:11:56Z [attach-edit-1]

moved: shaving -> shorn

---
▸ 2026-10-18T20:13:09Z [delta-farm]

Landed (delta-farm): worker commit 8fd8f15 squashed onto main as ca15de8 by land.sh (clean against clobber-1's changes in the same files, tree identical to the computed merge, doctor clear). Full `cargo test --release` 869 passed. Read the save path: under `store::lock`, re-reads the file and refuses (editor reopened with the text) if it changed since E, atomic write. Real run on a scratch farm with my build: attach notes.md, open it in the viewer, E, type, Ctrl-S: 'saved', the viewer re-renders, the file on disk changed. Accepted: E refuses a copy found in another shed (moot under the live farm).
