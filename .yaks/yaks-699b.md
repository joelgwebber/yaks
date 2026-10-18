---
id: yaks-699b
title: Inline viewing and editing of txt/md attachments
status: shaving
type: feature
priority: 3
created: '2026-10-04T20:53:40Z'
updated: '2026-10-18T05:54:18Z'
labels:
- tui
- markdown
- attach
---

Now that we're creating attachments pretty often, it's a huge PITA to have them open in the system $EDITOR or file association. There's no reason we can't just display them read-only, and/or making them editable, from directly within the TUI, since we have a half-decent editor implementation in edtui already.

---
▸ 2026-10-18T05:53:21Z [delta-farm]

moved: hairy -> shaving

---
▸ 2026-10-18T05:53:21Z [delta-farm]

Claim (delta-farm, yakherd; trial round 1 of yaks-6a34). Owner: `attach-view-1`.

Goal: open a text attachment (`.txt`, `.md`, `.log`, any file that is valid UTF-8 and not an image) INSIDE the TUI instead of handing it to `xdg-open`/`$EDITOR`. Read-only view first: a full-pane overlay, scrollable, `.md` rendered with the existing markdown renderer (src/tui/mdrender.rs) and everything else verbatim; Esc closes. Then editing if it fits: a key in the viewer opens the text in the existing edtui editor glue (src/tui/editor.rs) and saving writes the file back (under the farm lock: `store::lock` on the farm root, as every mutation must). Images and binary files keep today's external open. If editing does not fit cleanly, land view-only and say so in a note; editing becomes a follow-up yak.

Scope: `App::open_external` / `resolve_attachment` (src/tui/handlers.rs; the path is already resolved across farms), a new overlay (add at the END of the `Overlay` enum in src/tui.rs), its render function in src/tui/render.rs, key handling in handlers.rs. Reuse mdrender/editor; do not fork them.

Out of scope: the CLI, attachment management (rename, delete), image preview.

Evidence: TUI tests that fail before (Enter on a `.md` attachment opens the viewer with rendered content; a `.png` still goes external; if editing ships, an edit+save round-trip changes the file). Headless toque frames of the viewer (md and plain), attached. `?` help + help bar if the key is common, docs/tui.md. Gate: `cargo test -p yaks`. Judge: delta-farm at landing; look-and-feel to Joel with the frames.

---
▸ 2026-10-18T05:54:18Z [attach-view-1]

Started Sun Oct 18 05:54:18 AM UTC 2026. Yak path: /home/joel/src/yaks-farm/.yaks/yaks-699b.md
