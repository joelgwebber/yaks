---
id: yaks-640a
title: Explicit label definitions in config.yaml
status: shaving
type: feature
priority: 3
created: '2026-08-23T02:49:59Z'
updated: '2026-11-14T18:04:14Z'
tags:
- config
---

While the yaks CLI/TUI doesn't need explicit label definitions for anything, it could still be useful for auto-complete in the UI.

More importantly, if we can find a good way to get context into the calling agent (cf yak-011e), we can use label definitions to help nudge it in the right direction.

---
▸ 2026-09-21T18:37:07Z [agent]

Multi-herd angle: with the farm config herds: map + one-level global->herd cascade (yaks-98d5), label definitions should be per-herd-overridable (project A's labels differ from B's in a consolidated farm). Resolve label defs through that cascade rather than a single global list.

---
▸ 2026-11-14T18:03:46Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T18:04:14Z [linkplan-lead]

Shearing as superseded. This asked for predefined tag definitions with descriptions so the UI can complete them and an agent can be steered. The design settled differently from `config.yaml`, and the work is done:

- **Where the definitions live:** the `tags:` map of `schemas/yak.md` ([yaks-f04d](yaks-f04d), shorn 2026-11-10), not `config.yaml`. This farm has 22, with retired ones named. `yaks dok types` shows them and the `yaks` skill's Tags section tells agents to pick from them.
- **UI use:** Tab completes known tags first ([yaks-8d6a](yaks-8d6a), then [yaks-bf1b](yaks-bf1b) and [yaks-1a16](yaks-1a16), landed `dc8aaa92`, for the `L` editor and the filter drawer).
- **Agent use:** `unknown_tag_warnings` in `src/schemas.rs` ([yaks-bcd7](yaks-bcd7)) warns on `create`, `update` and `bulk` when a tag is not known and names the five nearest, which is the nudge this yak wanted.

**One piece is not covered:** the 2026-09-21 note asked that definitions be overridable per herd, through the cascade from [yaks-98d5](yaks-98d5). `schemas::load` reads one farm-wide `schemas/yak.md`, so a consolidated farm with several herds shares one tag list. Filed as its own idea rather than left buried here.
