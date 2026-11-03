---
id: yaks-efc0
title: Indent shed contents a bit
status: shaving
type: task
priority: 3
created: '2026-11-03T14:25:00Z'
updated: '2026-11-03T21:40:27Z'
parent: yaks-f677
tags:
- sheds
- tui
---

You can see here that it's difficult to tell the sheds apart from their contents:
```
⏳ design-scout              idle   40d ago      last: yaks-1bbd Carry search context through the UI globally   ▶ 4
   yakherd                   active 1m ago       shaving: yaks-039b                                            ▶ 34
   delta:skills-upgrade-aud  active 2m ago       shaving: yaks-9786, yaks-c69d, yaks-c2fc, yaks-ec00
   yaks-9786 p2 bug      🪒 TUI keeps writing after a sync pulls a farm on a newer schema         shaving [tui, git]
   yaks-c69d p3 feature  🪒 Farm min_version in config.yaml: an older yaks warns to upgrade            shaving [cli]
   yaks-c2fc p3 task     🪒 Upgrade/migrate guidance: upgrade recipe in skills and errors, agents as [skills, agent]
   yaks-ec00 p2 task     🪒 Research and plan: cloud runners always run a current yaks binary and [skills, agent] ⭐
   Joel Webber               active 1h ago       last: yaks-6919 TUI phase 7: the key pass (collisions, which- ▶ 77
   herd-tui                  active 1h ago       last: yaks-947a Implement the unified TUI model (/wiki/design ▶ 19
   delta:farm-wiki           stale  21h ago      shaving: yaks-44b3                                            ▶ 32
   keys-2                    idle   8h ago       last: yaks-6919 TUI phase 7: the key pass (collisions, which-k ▶ 1
   hist-1                    idle   8h ago       last: yaks-a2c5 TUI phase 5: one history of locations          ▶ 1
   find-1                    idle   8h ago       last: yaks-c215 TUI phase 6b: one search register for every text su
```

---
▸ 2026-11-03T21:24:34Z [delta:starred-batch]
kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

![tui-sheds-docshot](yaks-efc0/tui-sheds-docshot.png)

---
▸ 2026-11-03T21:40:27Z [delta:starred-batch]

Done in d3aa34a: a shed's yak rows are depth 2 (`Row::shed_yak`), so their ids start two columns right of the shed names instead of level with them. Docshot `tui-sheds.svg` regenerated and rasterized (attached).
