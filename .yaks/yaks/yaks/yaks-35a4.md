---
id: yaks-35a4
title: create --parent / reparent --parent onto a shorn yak re-opens the shorn-over-unfinished-children state
status: hairy
type: task
priority: 3
created: '2026-11-01T16:31:51Z'
updated: '2026-11-01T16:31:51Z'
parent: yaks-039b
labels:
- cli
---

Found shearing yaks-62f2: `yaks create kid --parent <shorn>` and `yaks reparent <id> --parent <shorn>` succeed silently and leave a shorn yak over a hairy child, which `yaks doctor` now flags (unfinished-children). Decide: refuse (naming `yaks regrow <parent>`), warn, or regrow the parent automatically. Lean: refuse with the regrow hint, same rule as `yaks shorn`.
