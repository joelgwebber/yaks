---
id: yaks-56c1
title: Edit text attachments inside the TUI (follow-up to the viewer)
status: hairy
type: feature
priority: 3
created: '2026-10-18T06:15:31Z'
updated: '2026-10-18T06:15:31Z'
parent: yaks-699b
labels:
- tui
- attach
---

yaks-699b shipped a read-only in-TUI viewer for text attachments (`Overlay::Attachment`, `AttachView` in src/tui/handlers.rs); `O` in it still opens the OS opener for editing. This adds editing in place: a key in the viewer opens the text in the edtui editor glue (src/tui/editor.rs, needs a save path for a file rather than a yak field), and saving writes the file back under the farm lock (`store::lock` on the farm root). Markdown re-renders after save. Out of scope: images, binaries, renaming.
