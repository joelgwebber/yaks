---
id: yaks-56c1
title: Edit text attachments inside the TUI (follow-up to the viewer)
status: shaving
type: feature
priority: 3
created: '2026-10-18T06:15:31Z'
updated: '2026-10-18T20:00:58Z'
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
