---
id: yaks-bc4a
title: 'TUI draws an image wherever GitHub does: list items, quotes, pinned and host-URL images'
status: shaving
type: feature
priority: 2
created: '2026-11-23T00:18:31Z'
updated: '2026-11-23T00:20:27Z'
parent: yaks-e387
tags:
- tui
- images
- markdown
---

Under yaks-e387 (TUI and GitHub resolve links the same way).

GitHub draws every markdown image: in a paragraph, a list item, a quote, a table. The TUI draws one in place only when it is a local attachment or an unpinned farm file in a top-level paragraph ([image_slot](code:src/tui/detail.rs#L777-L780), [end_image](code:src/tui/mdrender.rs#L707-L717)); everything else is a `🖼 alt` link row. Found on [/doks/releases/v0.0.41.md](/doks/releases/v0.0.41.md): a pinned image under a bullet. yaks-0bf0 and yaks-0370 deferred "images inside list items and quotes (need the container gutter as a draw offset)".

Want:

- An image in a list item or a quote (nested too) is drawn in place, indented to its container's gutter, with its caption, as a top-level one is now. A table cell stays the link row (a grid row cannot hold a picture): say so in docs/tui.md.
- An image's target resolves through the one resolver (link -> `Place`, applying `Space::read_url`) instead of the string forms in mdrender, so whatever a link can name, an image can show: an attachment, a farm file, a pinned farm file (`@sha`, and the host-URL form a sibling yak adds to `read_url`). A pinned image reads its bytes at the commit (`links::read_bytes_at`, LFS smudged, as `farm_image_at` does), so the inline cache key must include the rev, not only path and mtime.
- A pinned document's own images (the viewer of `x.md@sha`) are drawn at that commit, which lifts the "pinned document draws no images" rule in `handlers.rs`.
- docs/tui.md's inline-image section; a docshot or an attached frame of a list image and a pinned image drawn.

#### 2026-11-23T00:19:09Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-23T00:19:36Z

*herd-files*

Claim (herd-files): worker `inlineimg-1` (claude-sonnet-5-5), base `7c1452bd`, in parallel with `hostlinks-1` on yaks-ca59.

**Scope:** the description's four points: images in list items and quotes drawn in place; image targets resolved through the one resolver; pinned images drawn at their commit, with the cache keyed on the rev; a pinned document's own images drawn.

**Files you own:** `src/tui/mdrender.rs`, `src/tui/detail.rs`, `src/tui/imageview.rs`, `src/tui/handlers.rs`, `src/tui/follow.rs` (you may call `App::resolve`; don't change its contract), `src/tui/render.rs`, `src/tui/tests.rs` + snapshots, `src/tui/docshots.rs`, `docs/tui.md`, `docs/assets/`.

**Not yours:** `src/locate.rs`, `src/links.rs`, `src/hosting.rs`, `src/migrate.rs`. yaks-ca59 is adding a farm host URL form to `Space::read_url` there. Resolve images through `App::resolve` (which applies `read_url`) and you pick it up for free; don't parse URLs yourself.

**Judge:** the real dok `/doks/releases/v0.0.41.md` shows its screenshot drawn under the "Archives open in place" bullet. In a scratch farm, the following each draw in place:

- an image in a list item;
- an image in a nested quote;
- a pinned `@sha` image whose working file differs from the pin (the drawn picture is the pinned one);
- the images of a pinned dok.

A table-cell image stays a link row.

**Evidence:**

- `yaks tui --headless` frames written with `svg <path>` / `png <path>` and attached. Look at them (`yaks render x.svg -o x.png`, or the png). The headless half-block renderer does draw pictures; see the `doc=image` frames;
- whole-frame tests in `src/tui/tests.rs`;
- if the docs gain a scene, a docshot (`cargo test -p yaks docshots -- --ignored`).

**Decisions to record, not ask:** the indent and gutter rule; whether table cells could ever draw; the cache key.

#### 2026-11-23T00:20:27Z

*inlineimg-1*

Start 2026-11-23T00:20:26Z. Checkout: /Users/joel/src/yaks/.delta/worktrees/13mq924tg7fh/yaks. Farm: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-bc4a.md. Base 7c1452bd.
