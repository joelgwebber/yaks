---
id: yaks-3b8d
title: 'Links and attachments: collapse to text, markdown-friendly attachment links'
status: shorn
type: feature
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T05:33:15Z'
parent: yaks-ea13
depends_on:
- yaks-649d
- yaks-a870
labels:
- markdown
- tui
---

On the renderer core.

- `[text](url)` renders as the styled text only; the URL shows in the status bar when the line cursor is on the link (and stays followable with Tab/Enter). Autolinks and bare URLs stay as-is.
- Reference links and footnote-style `[text][ref]` resolve.
- Yak ids and `[[wikilinks]]` keep linking (today's `refs` scan, applied to rendered text spans).
- Attachments: today `yaks attach` writes `![stem](<id>/<file>)` for every file (53 `.txt`, 18 `.md`, 7 `.svg`, 5 `.png` in this farm). Proposal (pending Joel): images keep `![]()`; non-images become plain links `[stem](<id>/<file>)`, which is what GitHub/Delta expect; `rename-attachment` rewrites both forms; a one-time fixup rewrites existing non-image `![]` links. Images render as `🖼 stem` (followable, opens externally), other attachments as `📎 stem`.

---
▸ 2026-10-17T04:26:35Z [delta-lead]

asked: needs human

Attachment links: OK to (a) keep `![stem](<id>/<file>)` for images only, (b) write non-images as plain links `[stem](<id>/<file>)`, (c) have `rename-attachment` handle both forms, and (d) run a one-time fixup that rewrites existing non-image `![]` links (about 80 in this farm, mostly `.txt`/`.md`/`.svg`)? SVG counts as an image here.

---
▸ 2026-10-17T04:35:54Z [Joel Webber]

Yup, completely agreed. This won't be the only fixup we'll need for the next release, so let's leverage the schema migration infrastructure (or extend it as needed) to handle these updates cleanly.

---
▸ 2026-10-17T04:36:55Z [delta-lead]

picked up

Picked up by the yakherd: the attach/link format part moves to the schema-5 migration yaks-a870; this yak keeps the rendering part (collapsed links, 🖼/📎 attachments).

---
▸ 2026-10-17T05:17:20Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T05:17:20Z [delta-lead]

Claim (yakherd delta-lead, 2026-10-17): owner **links-2**, together with yaks-1bbb (both are about link targets in the read pane). The renderer core already collapses `[text](url)` to its text (Tab/Enter follow it), and yaks-a870 rewrote attachment links (images `![]`, everything else `[]`). Remaining scope:

- `src/tui/mdrender.rs` (`Tag::Link` / `Tag::Image` / `link_target` only): a link whose destination is an attachment (`<id>/<file>` relative path, where `<id>` is a known yak id) renders with a `📎 ` prefix (non-image) or keeps `🖼 ` (image); both are `Target::Artifact`, opened externally as today. Note the emoji are 2 cells wide: check alignment and wrapping.
- Show the URL of the link under the line cursor in the status bar (src/tui/render.rs `render_status`, where the notification/hints go), dim, truncated, so a collapsed link's destination is visible without following it.

Evidence: tests on the Target and prefix; headless frame with an attachment, an image and a URL link, plus the status bar line when the cursor is on the link. Judge: coordinator at landing.

---
▸ 2026-10-17T05:21:50Z [links-2]

Start (links-2): Sat Oct 17 05:21:50 AM UTC 2026

- path: /home/joel/src/yaks/.delta/worktrees/dvtctdaewedz/yaks/.yaks/yaks-3b8d.md
- own binary: ./target/release/yaks built in this checkout; base 4c586e9

[frame-links](yaks-3b8d/frame-links.txt)

---
▸ 2026-10-17T05:33:01Z [links-2]

Decision: an attachment link is recognised as `<id>/<file>` where `<id>` is a known yak id in the farm (`is_attachment` in mdrender.rs). A relative link to anything else (`other/f.txt`) stays a plain `Target::Artifact` with no 📎. Rejected: any relative path (would mark `docs/x.md`); checking the file exists (render must not touch disk).

Decision: the status-bar destination is `→ <dest>` in the existing dim help-bar style, replacing the key hints (not a second row), only in the detail pane with no overlay, and only for URL and attachment targets. Yak-id links and footnotes show the normal hints (they are visible in the text). Rejected: also showing task ids; a dedicated status row. `link_hint` in render.rs, truncated with `truncate_disp`.

Decision: when several links share a row the first (leftmost) one's destination is shown, matching what `follow_link`/`O` act on (`find` first jump of the line).

Done:
- mdrender.rs `Tag::Link`: pushes `📎 ` (inside the link, so it is part of the followable run and underlined) for attachments; images already had `🖼 `; both `Target::Artifact`, opened externally as before. `link_target` unchanged.
- render.rs `render_status`/`link_hint`; docs/tui.md (rendered markdown paragraph, `r` rename row now says link 📎 or 🖼), skill line.
- Emoji width: `📎`/`🖼` are 2 cells; the layout already measures with `unicode-width`. `wide_glyph_prefix_wraps_within_the_width` renders at width 8 and asserts every row fits and the wrap point is `📎 abcd` / `efgh`.
- Tests: `attachment_links_get_a_paperclip_and_images_keep_the_picture` (text, targets for 📎 file, 🖼 image, URL, non-attachment relative path) and the wrap test both FAILED with the prefix disabled and pass with it; `status_bar_shows_the_destination_of_the_link_under_the_cursor` in tests.rs.
- Frame attached: frame-links.txt (headless 90x34, scratch farm with a real `yaks attach` of a .txt and a .svg): status bar shows `→ https://example.com/some/long/path`, `→ yak-b573/build-log.txt`, `→ yak-b573/shot.svg`; rows render `📎 build-log` and `🖼 shot`.

---
▸ 2026-10-17T05:33:07Z [links-2]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T05:33:15Z [links-2]

moved: shaving -> shorn
