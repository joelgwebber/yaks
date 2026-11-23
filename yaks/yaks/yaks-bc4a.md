---
id: yaks-bc4a
title: 'TUI draws an image wherever GitHub does: list items, quotes, pinned and host-URL images'
status: shaving
type: feature
priority: 2
created: '2026-11-23T00:18:31Z'
updated: '2026-11-23T01:17:16Z'
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

![rel41](yaks-bc4a/rel41.png "The real /doks/releases/v0.0.41.md: its screenshot drawn under the 'Archives open in place' bullet, in the bullet's gutter, with its caption")

[rel41](yaks-bc4a/rel41.txt "Text frames of the same headless run (v0.0.41 dok)")

![list-quote-pinned](yaks-bc4a/list-quote-pinned.png "Scratch farm: image in a list item (unpinned, green on disk), a pinned @sha image in a list (red/blue as committed, working copy is green), a nested-quote pinned image at =30% with the quote bars beside it; the table-cell image stays a link row (scrolled just off)")

[frame-yak](yaks-bc4a/frame-yak.txt "Text frame (100x50) of the scratch yak")

![pinned-dok](yaks-bc4a/pinned-dok.png "Pinned document doks/pics.md@89b311d: its own image drawn at that commit (red/blue) although the working copy is green")

![scrolled-quote-table](yaks-bc4a/scrolled-quote-table.png "Same scratch yak scrolled: the nested-quote pinned image with its caption and both quote bars running beside it; the table-cell image stays a link row")

#### 2026-11-23T01:12:22Z

*inlineimg-1*

Decision: the gutter and indent rule. Each `MdImage` carries the prefix stack of its containers as they read on a *continuation* row (`MdGutter`: text + styled spans, from the same `Container::cont` the blank rows use). The picture, its reserved rows and its caption all start at `gutter.cols()`, the column the container's text hangs at; the reserved and caption rows begin with the gutter text, so a quote's bar runs down beside the picture. The pane width the picture is sized against is `width - gutter`. Rejected:
- first-row prefix (`• `): puts the picture under the bullet glyph;
- a draw offset only, rows left blank: a quote's bar would break at every picture.

Decision: where the image's rows go. An image rides on the last row of the inline text it is in (`flush`), not the first row of its marker and not "the blank row after the paragraph" as before. A paragraph's blank row is no longer the trigger, because a tight list item has no blank row after it. Rejected: keeping the pending list drained at the next blank row: it puts an image at the end of a whole tight list.

Decision: table cells never draw. A table is emitted as aligned grid rows sharing one no-wrap block that scrolls sideways; a picture needs N reserved rows that the grid cannot hold, and its caption would break the column widths. The renderer drops the cell's image (`TagEnd::Table` clears them), so it stays the `🖼 alt` link row and has no caption; docs/tui.md says so. Rejected: drawing it under the whole table (it would detach from its cell).

Decision: the cache key is `(path, rev, mtime, cols, rows)`. A pinned image has mtime 0 (a commit never changes), an unpinned one the file's mtime as before; the rev is in the key because a pinned and an unpinned image of one path are different pictures (test: two entries). Rejected: keying on path+mtime only (a pinned image would show today's file once cached), or hashing content (needs the bytes before the lookup).

Decision: sizing a pinned image reads its bytes at layout time. A header read needs the bytes, so `pinned_dims` calls `links::read_bytes_at` and memoises the (farm, path, rev) -> pixel size in a thread-local, so a resize or a keypress that rebuilds the layout does not ask git again. Rejected: sizing from the working copy (wrong aspect when the pin differs).

Decision: resolution goes through `App::resolve`. mdrender keeps the link's own `Target` (a URL stays `Target::Url` so `Space::read_url` can read a farm host URL back once yaks-ca59 lands); `ImageEnv.resolve` is a closure over a `LazySpace` (follow.rs) that makes the `Space` on the first image of a layout, since building one asks git where the code checkout is. Only `Place::Attachment` and `Place::FarmFile` draw; anything else (URL, code file, commit, directory, missing) stays the link row.

Decision: a pinned document draws its images at its rev unless an image pins another (`ImageEnv.rev`). A code file with a rev is excluded: its rev is a commit of the code repo, not of the farm its links read.

#### 2026-11-23T01:16:53Z

*inlineimg-1 · kind: verify · result: pass*

verify: `cargo test --workspace` -> PASS (exit 0)

#### 2026-11-23T01:17:16Z

*inlineimg-1*

Summary. An image in a list item or a quote (nested too) now draws in place, in the container's gutter, with its caption. Targets resolve through `App::resolve`, so `@sha` images read their bytes at the commit, and a pinned document draws its own images at its commit.

Shape of the fix:
- `mdrender`: `MdImage` carries its `Target` and `MdGutter`. Images ride on the last row of their text (`flush`). A table drops them.
- `detail::md_rows` / `image_slot` / `image_block`: resolve through `ImageEnv.resolve`, size against `width - gutter`, and indent the rows. `ImageSlot` gained `rev` and `x`.
- `imageview`: the cache key includes the rev; a pinned image is read with `read_bytes_at` and drawn at `x`.
- `follow`: `LazySpace` shares one `Space` per layout. `handlers`: the "pinned document draws no images" rule is lifted.
- `detail_nav` / `memo`: `image_env` takes the resolver, and a cheap `image_key` is the cache key.
- Docs: `docs/tui.md` inline-image and caption sections, the README config comment, the `?` help line.

Evidence:
- `yaks verify yaks-bc4a`: PASS, whole workspace.
- `cargo fmt --check`: clean.
- `cargo test -p yaks docshots -- --ignored`: 15 passed; no asset changed.
- Attached frames: the real v0.0.41 dok with its image under the bullet; a scratch farm with a list image, a pinned image (the pin is red/blue and the working copy green, and the pinned one is drawn), a nested-quote pinned image, a table-cell image that stays a link row, and a pinned dok's own image drawn at its commit.
- New tests: `an_image_carries_its_containers_gutter`, `a_list_items_image_is_on_the_last_row_of_its_text`, `an_image_in_a_list_item_or_a_quote_is_drawn_in_the_containers_gutter`, `a_pinned_image_is_drawn_as_it_was_at_its_commit` and `a_pinned_document_draws_its_images_at_its_commit`.

Not tested: the host-URL form (yaks-ca59 is unlanded). `Target::Url` is preserved for images, so it should work once `read_url` handles it.
