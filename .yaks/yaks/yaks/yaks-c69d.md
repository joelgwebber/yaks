---
id: yaks-c69d
title: 'Farm min_version in config.yaml: an older yaks warns to upgrade'
status: shorn
type: feature
priority: 3
created: '2026-11-03T14:21:53Z'
updated: '2026-11-03T14:30:08Z'
parent: yaks-ec00
tags:
- cli
---

---
▸ 2026-11-03T14:22:00Z [delta:skills-upgrade-audit]

kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T14:30:06Z [delta:skills-upgrade-audit]

Landed in 7ed9402: `min_version:` in `config.yaml` (`store::Config::min_version`, `min_version_notice`). An older binary prints `warning: this farm asks for yaks X or newer ...` on every command (not `--json`) and as a TUI notification at startup; it still runs. Docs: README, docs/cli.md, docs/tui.md, RELEASING.md.

Evidence: unit test `min_version_warns_only_an_older_yaks`, and the real binary on a scratch farm:

```text
warning: this farm asks for yaks 9.9.9 or newer (this is 0.0.30): upgrade with npm i -g @j15r/yaks@latest (or run it as npx -y @j15r/yaks@latest <cmd>)
```

---
▸ 2026-11-03T14:30:08Z [delta:skills-upgrade-audit]

kind: move
from: shaving
to: shorn
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
