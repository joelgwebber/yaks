---
id: yaks-e387
title: Images and links resolve the same in the TUI and on GitHub
status: shaving
type: task
priority: 2
created: '2026-11-23T00:16:34Z'
updated: '2026-11-23T00:18:38Z'
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
