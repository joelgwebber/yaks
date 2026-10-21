---
id: yaks-699b
title: Inline viewing and editing of txt/md attachments
status: shorn
type: feature
priority: 3
created: '2026-10-04T20:53:40Z'
updated: '2026-10-18T06:15:31Z'
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

[viewer-markdown-frame](yaks-699b/viewer-markdown-frame.txt)

[viewer-plain-frame](yaks-699b/viewer-plain-frame.txt)

---
▸ 2026-10-18T06:13:46Z [attach-view-1]

Decision: shipped view-only; editing is a follow-up. Saving through the editor glue needs a new `EditAction` variant (or a save path) in `src/tui/editor.rs`, which is outside this brief's file scope. In the viewer, `O` hands the file to the OS opener so it can be edited there. I rejected two ways of fitting editing in: reusing `BlockEdit` (it is tied to a yak's content blocks) and a second edtui wiring inside `handlers.rs` (that would fork `editor.rs`). Because nothing writes to an attachment, no farm lock is needed yet.

---
▸ 2026-10-18T06:13:46Z [attach-view-1]

Decision: what counts as a text attachment. It is shown in the viewer when it is not an image (`store::is_image_name`), is at most 1 MiB (`VIEW_MAX_BYTES`), is valid UTF-8 and has no NUL byte. Anything else goes to the OS opener as before. I rejected an extension allow-list (`.txt`/`.md`/`.log`) because the claim says "any file that is valid UTF-8 and not an image". I rejected having no size cap because the whole file is read on the keypress.

---
▸ 2026-10-18T06:13:46Z [attach-view-1]

Decision: markdown means a `.md` or `.markdown` extension (case-insensitive), rendered by `mdrender::render` exactly as a yak body is. Other text is shown verbatim (tabs become 4 spaces) and soft-wrapped by `detail::wrap`. I rejected sniffing the content for markdown, which is unpredictable.

---
▸ 2026-10-18T06:13:46Z [attach-view-1]

Decision: the viewer is a right-pane overlay like `?` help (the list stays visible when the terminal is wide enough), not a full-screen takeover. That is consistent with every other right-pane overlay. Keys: `j`/`k`, `d`/`u`, PgUp/PgDn, `g`/`G` and Home/End scroll; `O` opens the file externally; `Esc`, `q`, `h` and `←` close. Links inside the viewer are not followable (there is no cursor), which is a possible follow-up.

---
▸ 2026-10-18T06:13:46Z [attach-view-1]

Decision: under `cfg(test)`, `App::os_open` does not launch `xdg-open`; it only sets the notification. Without that, the "png still opens externally" test would pop up a real viewer on the machine running it. I rejected injecting an opener function, which is heavier for one test seam.

---
▸ 2026-10-18T06:13:54Z [attach-view-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-18T06:14:02Z [attach-view-1]

Shorn summary (attach-view-1). Pressing `Enter` or `O` on a text attachment link now opens it in an in-TUI viewer (`Overlay::Attachment(AttachView)`). A `.md` file is rendered by mdrender; any other text is shown verbatim and wrapped. Images and binary files still open with the OS opener. The viewer is **view-only**: editing is a follow-up (see the Decision note above).

- Code: `AttachView`, `App::open_external`, `App::os_open` and `App::handle_attachment_key` in `src/tui/handlers.rs`; `render_attachment`, the status-bar hint and the `help_content` line in `src/tui/render.rs`; the enum variant in `src/tui.rs`; the docs in `docs/tui.md`.
- Tests that fail before this change: `enter_on_a_markdown_attachment_opens_the_viewer_rendered`, `a_plain_text_attachment_is_shown_verbatim_and_scrolls` and `an_image_or_binary_attachment_still_opens_externally`, in `src/tui/tests.rs`.
- Evidence: headless frames from my own release build, attached (`viewer-markdown-frame.txt`, `viewer-plain-frame.txt`). The `?` help line was checked headlessly: the description column is aligned. No insta snapshot changed.
- Gate: `yaks verify` PASS (`cargo test --workspace`).

---
▸ 2026-10-18T06:14:02Z [attach-view-1]

moved: shaving -> shorn

---
▸ 2026-10-18T06:15:31Z [delta-farm]

Landed (delta-farm): worker commit e08957a merged by SHA as 8102833 (land.sh: base an ancestor, result identical to the computed merge, doctor clear). With all three round-1 workers in: full `cargo test --release` 854 passed. Real run on the live farm with my build: Enter on this yak's own attachments opens the in-TUI viewer (the .txt frames verbatim, the markdown inside them rendered as a body is), Esc closes. Shipped view-only by the claim's own fallback; editing filed as a follow-up.
