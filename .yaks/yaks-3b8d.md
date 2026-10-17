---
id: yaks-3b8d
title: 'Links and attachments: collapse to text, markdown-friendly attachment links'
status: shaving
type: feature
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T05:17:20Z'
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
