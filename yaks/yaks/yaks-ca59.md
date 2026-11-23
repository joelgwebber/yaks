---
id: yaks-ca59
title: 'Writers store links the host can follow: canonicalise on save, pinned farm links as host URLs'
status: shaving
type: feature
priority: 2
created: '2026-11-23T00:18:30Z'
updated: '2026-11-23T00:19:08Z'
parent: yaks-e387
tags:
- links
- cli
---

Under yaks-e387 (TUI and GitHub resolve links the same way).

Two gaps between what yaks reads and what GitHub can follow, both in text written today:

1. **Bare yak ids are not canonicalised on save.** Decision 2 of [yaks-1151](/yaks/yaks/yaks-1151.md) (Joel, 2026-11-14): "Stored yak links become `/<herd>/yaks/<id>.md`; bare ids stay accepted and are canonicalised on save." [yaks-acc0](/yaks/yaks/yaks-acc0.md) did the one-time migration (`migrate::canonical_dest`) but no writer calls it: 17 `](yaks-1a2b)` / `](yaks-1a2b#note)` links have been written since, and every one 404s on GitHub.
2. **A pinned farm link (`path@<sha>`) is a yaks-only form.** GitHub reads `@f4211aaa` as part of the filename: the v0.0.41 release dok's hero image `<img src>` is `raw/yaks/doks/releases/v0.0.41/archive-browser.png@f4211aaa`, a 404 (curl; the unpinned URL is 200 image/png). Decision 4 of the same plan already settled this for pinned `code:` links: stored as the host URL, read back by `locate::Space::read_url`.

Want:

- One canonicaliser, applied by every farm writer as it saves. Move the yak and attachment rules of `canonical_dest` out of `migrate.rs` into the link layer, so the migration and the writers share it. Writers: `create`/`update` description and notes, `dok new`/`dok set`, the TUI editors (through the same `Farm` methods), `session note`, `attach --caption`. Idempotent; code spans and fences untouched (`links::rewrite`).
- A pinned farm link is stored as the farm remote's host URL at that commit: `raw/<sha>/<path>` for an image (what an `<img>` loads; `hosting::Repo::raw_url`), `blob/<sha>/<path>[#anchor]` otherwise. Only when the farm branch's remote is a recognised host and the farm is not private; else left as written. A pinned `code:` link likewise (decision 4; check whether its write side shipped).
- `Space::read_url` reads this farm's own host URLs back (blob and raw, at a sha) as pinned farm links, as it does for code. The farm and the code repo can share one GitHub repo (this one does): tell them apart by which repository has the commit, or by the farm remote when `yaks.farm` points elsewhere.
- `@sha` stays readable forever (old text).
- Docs: the yaks skill's link section and "Notes and descriptions", `docs/cli.md`, the notes line in `src/brief.rs`.

#### 2026-11-23T00:19:08Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving
