---
id: yaks-9f91
title: 'Flat farm, phase 2: git reporting (yaks status, yaks commit, changes.rs) for in-place status changes'
type: task
priority: 1
created: '2026-10-12T19:53:57Z'
updated: '2026-10-12T19:53:57Z'
parent: yaks-8b7a
depends_on:
- yaks-95be
labels:
- cli
- git
---

Phase 2 of yaks-8b7a (plan note: D7; runs after phase 1, yaks-95be). Sonnet worker.
Scope: src/changes.rs (`classify`), src/status.rs (`describe_yak`, the `yaks status` lines and message), src/commit.rs (`yaks commit`'s generated message), their tests, and the docs rows for `status`/`commit` in docs/cli.md and README.md.
Rule: in a flat farm a status change is a modified `<id>.md` whose `status:` changed between HEAD and the working tree: report it as `moved <from> -> <to>` exactly as today, and `yaks commit` says `shorn <id>` etc. as today. Keep understanding a deleted plus an added path for the same id (legacy history, and a v3 farm read by this binary). The migration itself (every yak deleted from a status dir and added at the root with the same status) reports as ONE line, `farm layout migrated: N yaks`, not N moves, and `yaks commit` uses that as its message.
Out of scope: src/store.rs, migrate, doctor, sheds, skills.
Evidence (fail first): a flat-farm status change reports `moved`; a combined status change plus note reports both; a staged migration reports the one line; the existing v3-shaped tests still pass.
