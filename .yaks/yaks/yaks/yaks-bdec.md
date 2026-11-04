---
id: yaks-bdec
title: 'Release 0.0.31: attachment-relative links, attach --no-link/--caption, inline images in text, svg inline'
status: shaving
type: task
priority: 1
created: '2026-11-04T01:29:03Z'
updated: '2026-11-04T01:49:21Z'
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

---
▸ 2026-11-04T01:49:21Z [delta:Fix relative image links in yak attachme]

Released **0.0.31** (Joel, 2026-11-04: "cut a release once everything's in").

- Bump: `2ef040e` (Cargo.toml, Cargo.lock, npm/yaks/package.json in lockstep), tag `v0.0.31`, pushed to `origin` and `local`.
- Contents since v0.0.30: yaks-be20, yaks-0370, yaks-000b, yaks-7c6e. No schema change (`src/migrate.rs` untouched), so no `min_version` step.
- Gate before tagging: `cargo test --workspace --locked` with an empty HOME and its own TMPDIR passed (1107 tests). One earlier run failed `tests/init_modes.rs` on a leftover `/tmp/share2-web-wt` from a previous run (a fixed temp path, already filed as yaks-50b7), not a code problem.
- Dry run of the `release` workflow on `2ef040e` first: green on all five platforms.
- Tag push: the workflow succeeded, `@j15r/yaks` and the five platform packages are at 0.0.31, `latest` points at it. The registry showed 0.0.30 for about a minute after the run, then caught up.

Installed locally with `cargo install --path .` from the same tree. `yaks tui` sessions started before need a restart.
