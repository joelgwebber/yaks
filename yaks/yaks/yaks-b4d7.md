---
id: yaks-b4d7
title: Direct links to doks, with no anchor, often jump to the bottom
status: shorn
type: bug
priority: 3
created: '2026-11-16T21:20:31Z'
updated: '2026-11-21T22:31:16Z'
tags:
- tui
---

I haven't quite pinned this down, but it looks like it happens most of the time when linking directly to a dok, with no anchor in the link. The last line is focused and pinned to the top of the viewport, so you have to scroll up to see much of anything.

#### 2026-11-19T23:29:33Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-19T23:40:13Z

*herd-files*

Root cause found and fixed (herd-files).

A dok opened by following a link has never been drawn, so its viewer's `width` is 0. An anchor (`#heading`, `#L12`) was placed by `scroll_to_anchor` on rows laid out at that width: one column per row, thousands of rows. The heading's row index there is far past the end of the real (drawn) rows, so on the first frame the cursor was clamped to the last row and the scroll pinned it at the top: "the last line is focused and pinned to the top of the viewport".

Repro in a scratch farm (headless, before the fix): a yak links `[mid](/doks/anch.md#middle)`; Enter opens the dok on blank rows with nothing on screen. Links with no anchor were fine in my runs; the "no anchor" in the report may be the status bar hiding the fragment of a collapsed link, or a link that had one.

Fix: `App::show_doc` gives any viewer not drawn yet the pane's width as last drawn (`App::doc_width`: the right pane's inner width, recorded in `render`; else the detail pane's; else 80). The member viewer's own width hack (yaks-93ee) goes, since it is now the general rule.

Test: `a_link_from_a_yak_lands_on_its_heading_in_the_first_frame` (fails without the fix: the viewer's width is 0). Headless after: `#middle` lands with "Middle" as the top row.

#### 2026-11-19T23:52:19Z

*herd-files · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-20T00:07:10Z

*herd-files*

Evidence frame: [the `#middle` link lands on its heading](yaks-b4d7/coord-b4d7-anchor-lands.txt) (release build, headless). Committed as `b9f31552`.

#### 2026-11-21T22:31:16Z

*herd-files · kind: change · section: fixed*

Following a link to a dok's heading lands on that heading the first time the viewer opens, not at the bottom.
