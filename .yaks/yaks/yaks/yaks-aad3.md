---
id: yaks-aad3
title: 'yaks sheds: say how old the remote data is (upstream last changed, last fetch)'
status: shorn
type: feature
priority: 2
created: '2026-11-01T16:23:47Z'
updated: '2026-11-01T16:23:48Z'
parent: yaks-039b
tags:
- sheds
- cli
---

Seen 2026-11-01 following the memcraft cloud run: from the laptop, `yaks sheds` showed every runner worker 34–45 minutes quiet. But `writer-1` had finished and been landed on the runner; the coordinator just hadn't synced since. Nothing in the view said how old the remote data was: the header only compared the local farm with `origin/yaks` as of the last fetch.

Done: the header (and the TUI Sheds footer, and `--json`) adds when the upstream last changed and when this farm last fetched:

```text
farm: up to date with origin/yaks · origin/yaks changed 37m ago · fetched just now
```

That tells "nobody wrote" from "nobody synced". Skills say it too (coordinating §11). `FETCH_HEAD` is per worktree, which caught a first version that read the common dir.

---
▸ 2026-11-01T16:23:47Z [yakherd]

kind: move
from: hairy
to: shaving
---

moved: hairy -> shaving

---
▸ 2026-11-01T16:23:47Z [yakherd]

kind: landed
commit: 9077f6c16120aec4b9089bce3f7359b620aec2a0
branch: main
---

Landed `9077f6c`. Evidence: unit test `json_carries_everyone_and_the_farm_freshness` covers the header, JSON and render; the real memcraft farm prints `farm: up to date with origin/yaks · origin/yaks changed 37m ago · fetched just now`, and its JSON carries `upstream_changed: 2026-11-01T15:44:40Z`. `cargo test --workspace` green. The TUI snapshot of the Sheds tab is unchanged (its fixture has no freshness). The same commit fixes the skills' detached-job advice, which failed on the runner: the herd's `setsid nohup … &` as its own command never created its log, and it then polled the missing file for about 10 minutes. The skills now say to confirm the log appears in the same call.

---
▸ 2026-11-01T16:23:48Z [yakherd]

kind: move
from: shaving
to: shorn
---

moved: shaving -> shorn
