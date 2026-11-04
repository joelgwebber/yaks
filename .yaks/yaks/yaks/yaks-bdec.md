---
id: yaks-bdec
title: 'Release 0.0.31: attachment-relative links, attach --no-link/--caption, inline images in text, svg inline'
status: shaving
type: task
priority: 1
created: '2026-11-04T01:29:03Z'
updated: '2026-11-04T01:29:09Z'
tags:
- release
---

Bump to 0.0.31, tag `v0.0.31`, publish to npm via the release workflow (Joel, 2026-11-04: "cut a release once everything's in").

Since v0.0.30: yaks-be20 (a bare name in an attachment is a sibling attachment; `attach` takes several files and `--no-link`; doctor, rename-attachment and promote follow links inside attachments) and yaks-0370 (images in a paragraph draw under it, svg drawn in place, image title as caption, `attach --caption`), plus yaks-000b and yaks-7c6e. No schema change (`src/migrate.rs` untouched), so no `min_version` step.

Gate before tagging, as in yaks-a5f8: the exact commit passes `cargo test --workspace --locked` with an empty HOME (reproduces a runner with no git identity).

---
▸ 2026-11-04T01:29:09Z [delta:Fix relative image links in yak attachme]
kind: move
from: hairy
to: shaving
thread: ksQQNSYysfbwRGKL-KGL8e-aQpLOAEKHPJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
