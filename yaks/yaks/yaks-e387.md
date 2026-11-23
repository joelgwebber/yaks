---
id: yaks-e387
title: Images and links resolve the same in the TUI and on GitHub
status: shorn
type: task
priority: 2
created: '2026-11-23T00:16:34Z'
updated: '2026-11-23T01:40:28Z'
tags:
- links
- images
- tui
---

Joel, 2026-11-22: "my main concern is making sure we can resolve resources in all supported cases, and getting consistency between the TUI and GHUI."

How it was found: the hero image in [/doks/releases/v0.0.41.md](/doks/releases/v0.0.41.md), `![…](v0.0.41/archive-browser.png@f4211aaa)` indented under a list item. Two different failures:

- **The TUI does not draw it in place.** A pinned farm image gets no inline slot ([mdrender end_image](code:src/tui/mdrender.rs#L707-L717)), and only a top-level paragraph's images are drawn ([image_slot](code:src/tui/detail.rs#L777-L780)), so a list item's image is a link row. Following the link opens the pinned LFS bytes fine (yaks-c955).
- **GitHub does not show it either (it only seemed to).** The page's `<img src>` is `raw/yaks/doks/releases/v0.0.41/archive-browser.png@f4211aaa`, which is a 404 (checked with curl; without the `@f4211aaa` the same URL is 200 image/png). GitHub reads the `@<sha>` as part of the filename. What renders correctly is CHANGELOG.md, where `yaks release changelog` writes `raw/<sha>/…`.

So the rule to settle: **a link a person or agent writes into the farm must resolve in yaks and on the host the same way**, and wherever one side can show a thing (an image drawn in place, a pinned file) the other should too.

Children:
- fix the v0.0.41 dok (drop the needless pin, the changelog pins images itself);
- the pinned-farm-link form (`path@sha`) is yaks-only: GitHub 404s it. Decide its stored form, as decision 4 of [the layout plan](/doks/design/farm-layout-and-links.md) did for pinned `code:` links (host URL, read back by `Space::read_url`);
- the TUI draws an image wherever GitHub does: in list items and quotes, pinned or not, and a host URL to this farm's own file.

#### 2026-11-23T00:18:38Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-23T00:20:26Z

*herd-files*

Batch (herd-files, base 7c1452bd, both claude-sonnet-5-5):

- yaks-c019 (the v0.0.41 dok's pin): done by the coordinator and shorn. GitHub now serves the image (200 image/png).
- yaks-ca59: `hostlinks-1`. Writers canonicalise links on save; pinned farm links are stored as host URLs; `read_url` reads them back. Owns the link layer: `links`, `locate`, `hosting`, `migrate`.
- yaks-bc4a: `inlineimg-1`. The TUI draws images in list items and quotes, and pinned images, through the one resolver. Owns `src/tui/` and `docs/tui.md`.

They meet at `App::resolve` → `Space::read_url`: once both land, a host-URL image is drawn by the TUI. I check that end to end after landing both.

#### 2026-11-23T01:40:27Z

*herd-files*

End to end (herd-files), with my build at `4f09d2d2`. Scratch repo with remote `https://github.com/o/r.git`; the farm commit carries a real 1888×904 PNG; afterwards the working file was replaced with a 40×20 green one.

- `yaks update --note` with `![shot](/doks/shot.png@fe60ac7a)` under a bullet was stored as `![shot](https://github.com/o/r/raw/fe60ac7a…/doks/shot.png)`. A bare `[other](yak-5ac8)` was stored as `/yak/yaks/yak-5ac8.md`.
- `yaks peek <that raw URL>` reads `── doks/shot.png@fe60ac7` back: the URL resolves to the pinned farm file.
- In the TUI, that host-URL image is drawn in place under its bullet, and it is the pinned 1888×904 screenshot, not the green working copy: [frame](yaks-e387/coord-e387-hosturl-image-drawn.png). The two workers' halves meet at `App::resolve` → `read_url` as planned.

All three children are shorn. The 17 bare-id links already in the farm stay as written: rewriting them is Joel's call.

#### 2026-11-23T01:40:28Z

*herd-files · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-23T01:40:28Z

*herd-files · kind: change · section: none*

No user-facing change.
