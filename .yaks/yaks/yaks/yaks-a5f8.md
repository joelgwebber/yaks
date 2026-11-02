---
id: yaks-a5f8
title: 'Release 0.0.29: the farm as a knowledge store (schema 7)'
status: shaving
type: task
priority: 1
created: '2026-11-02T16:44:16Z'
updated: '2026-11-02T17:10:42Z'
parent: yaks-44b3
tags:
- release
---

Bump to 0.0.29, tag, publish to npm via the release workflow, then publish the migrated farms (Joel, 2026-11-02: land, cut a release, get the migrations done).

---
▸ 2026-11-02T16:44:16Z [delta:farm-wiki]
kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-02T17:10:42Z [delta:farm-wiki]

Released **0.0.30** (Joel, 2026-11-02).

**v0.0.29:** tagged at c4e7257. Its release run failed `cargo test` on the runner: `tests/autocommit.rs`, because a hand-edit commit had no identity fallback. Nothing was published.

**Fix and re-release:**

- Fixed in 9952885: a hand edit is committed as `unattributed` when git has no identity.
- Bumped in 5a6190a.
- Before tagging, the exact commit passed `cargo test --workspace --locked` locally with an empty HOME (1015 passed). That is how I reproduced the CI failure, then checked the fix.
- `main` was pushed to `local` and `origin` (5a6190a), and `v0.0.30` was tagged and pushed.
- [Release run 41486762856](https://github.com/joelgwebber/yaks/actions/runs/41486762856) passed and published all six npm packages with provenance.
- The registry shows `latest` = 0.0.30, and `npx @j15r/yaks@0.0.30 --version` in a clean directory prints `yaks 0.0.30`.

**Farms published after the release:**

- yaks farm: `yaks sync` pushed 14 commits, including the schema-7 migration and LFS. A fresh `--depth 1` clone of `origin/yaks` gets schema 7 and real PNG bytes from LFS.
- `joel` farm: `origin/yaks` = 5e36eaa.
- `joel` main pushed to `origin` (cowpaths/joel) as 3688d71.

Installed 0.0.30 to `~/.cargo/bin` (`cargo install --path .`). Since last night the binary there had gone missing, so `yaks` on PATH had been resolving to the pnpm 0.0.28.
