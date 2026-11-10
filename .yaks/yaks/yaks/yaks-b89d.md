---
id: yaks-b89d
title: Format src/actor.rs and tests/commits.rs once so cargo fmt stops rewriting them
status: shaving
type: task
priority: 2
created: '2026-11-10T17:53:07Z'
updated: '2026-11-10T17:53:15Z'
tags:
- meta
---

Every session note so far reports `cargo fmt -- <files>` rewriting `src/actor.rs` and `tests/commits.rs`, and every worker reverts them by hand, though `AGENTS.md` warns about it: a lesson already in context that still recurs (yaks-2b40, point 4). The mechanism fix: format the two files once on `main`, and drop the warning from `AGENTS.md` and from briefs.

---
▸ 2026-11-10T17:53:15Z [herd-unlock]
kind: move
from: hairy
to: shaving
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving
