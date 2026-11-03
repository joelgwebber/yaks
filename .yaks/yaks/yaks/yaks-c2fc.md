---
id: yaks-c2fc
title: 'Upgrade/migrate guidance: upgrade recipe in skills and errors, agents ask before migrate, schema-bump order'
status: shaving
type: task
priority: 3
created: '2026-11-03T14:21:52Z'
updated: '2026-11-03T14:30:06Z'
parent: yaks-ec00
tags:
- skills
- agent
---

---
▸ 2026-11-03T14:21:59Z [delta:skills-upgrade-audit]
kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T14:30:06Z [delta:skills-upgrade-audit]

Landed in 7ed9402.

- `yaks` skill: "Keeping yaks current" (check `yaks --version` vs `npm view @j15r/yaks version`, upgrade with `npm i -g @j15r/yaks@latest` or `npx -y ...@latest`) at session start, and "Never migrate a farm yourself" (always ask a human, per Joel).
- `yaks-working`: one line, upgrade if told, never migrate.
- `yaks-coordinating`: versions on runners, and the schema-bump order (min_version + sync, upgrade every machine, migrate once + commit + sync, restart TUIs).
- `yaks brief` start step says the same. `yaks migrate --help`, docs/cli.md and RELEASING.md carry the order.
- The "newer schema" error names the upgrade command and the binary version.

Evidence: real binary on a scratch farm with `schema: 8`:

```text
error: this farm uses schema v8, newer than this yaks (0.0.30) supports (v7). Upgrade with: npm i -g @j15r/yaks@latest (or run it as npx -y @j15r/yaks@latest <cmd>)
```

Not done: a cold read of the skills by a cheap model.
