---
id: yaks-c019
title: 'Fix /doks/releases/v0.0.41.md''s hero image: drop the @sha pin'
status: shorn
type: chore
priority: 2
created: '2026-11-23T00:16:34Z'
updated: '2026-11-23T00:19:07Z'
parent: yaks-e387
tags:
- release
- images
---

The pin makes the image 404 on GitHub (the @f4211aaa is read as part of the filename) and blocks the TUI's inline draw. `yaks release changelog` already turns a plain farm image into a commit-pinned `raw/<sha>/` URL, so the pin bought nothing. Write it unpinned, keep it under its bullet (the TUI child draws list images), and check CHANGELOG.md is unchanged.

#### 2026-11-23T00:18:39Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-23T00:19:06Z

*herd-files*

Done (herd-files). Dropped the @f4211aaa pin. GitHub's live page now has `<img src="/joelgwebber/yaks/raw/yaks/doks/releases/v0.0.41/archive-browser.png">`, 200 image/png (curl); before it was the `.png@f4211aaa` URL, a 404. `yaks release changelog --release v0.0.41` still writes a commit-pinned `raw/<sha>/` URL (now 5b53f1f9, the edit's farm commit; also 200 image/png). The image stays under its bullet: drawing it in place in the TUI is yaks-bc4a.

#### 2026-11-23T00:19:07Z

*herd-files · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-23T00:19:07Z

*herd-files · kind: change · section: none*

No user-facing change.
