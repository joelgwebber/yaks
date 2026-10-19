---
id: yaks-dc63
title: Release 0.0.21
status: shorn
type: task
priority: 2
created: '2026-10-19T03:20:23Z'
updated: '2026-10-19T03:20:50Z'
labels:
- release
---

---
▸ 2026-10-19T03:20:31Z [delta:farm-bootstrap]

moved: hairy -> shaving

---
▸ 2026-10-19T03:20:50Z [delta:farm-bootstrap]

Version 0.0.21 (Cargo.toml, Cargo.lock, npm/yaks/package.json). Headline since v0.0.20:

- **The farm is a branch** (yaks-73f4): orphan `yaks` branch holding only `.yaks/`, in its own worktree; discovery resolves it from any checkout (yaks-4809, yaks-87a3); `yaks migrate --to-branch` moves a committed farm there (yaks-bd7e); default worktree home `$XDG_DATA_HOME/yaks/farms/<repo>-<hash>`.
- **Bootstrap on first use** (yaks-a260): a fresh clone needs no setup; the first yaks command creates the farm worktree tracking `origin/yaks`. `YAKS_NO_BOOTSTRAP=1` disables.
- Live-farm trial fixes (yaks-5924, yaks-6c4f, yaks-9f5c), shed lands as squash.

Gate: `cargo test --release` green except tests/attachment_sheds.rs (fails identically on the pre-change baseline on macOS; not caused by this release), `node npm/yaks/test-mapping.mjs` ok.

---
▸ 2026-10-19T03:20:50Z [delta:farm-bootstrap]

moved: shaving -> shorn
