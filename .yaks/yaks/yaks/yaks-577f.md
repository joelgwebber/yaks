---
id: yaks-577f
title: 'Skill: research and knowledge in a farm (find, cite, save sources, promote, pages vs yaks vs docs/)'
status: shorn
type: task
priority: 2
created: '2026-11-01T18:53:13Z'
updated: '2026-11-01T22:51:30Z'
parent: yaks-d26a
depends_on:
- yaks-2c42
tags:
- wiki
- skills
verify: cargo test --workspace
---

A bundled skill (or a section of `yaks`) giving research yaks formal guidance: save sources, cite them, promote attachments, when a finding becomes a page, what stays in `docs/` on `main`. Port what applies from Hearth's `hearth-knowledge` skill.

---
▸ 2026-11-01T22:27:16Z [delta:farm-wiki]

kind: move
from: hairy
to: shaving
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-01T22:30:33Z [delta:farm-wiki]

kind: checkpoint
branch: wiki
commit: e2a9aa1
---

The `yaks-research` skill landed on `wiki` (d51e262, then fixes in the commit above). It is a fifth bundled skill rather than a section of `yaks`, so it loads only for research work and `yaks` stays lean (`Decision:`).

It covers:

- where each finding goes: yak notes, page, typed page, source, `docs/` on main, or a new yak;
- finding before writing;
- saving sources before relying on them (yaks fetches nothing);
- writing pages with farm-root links and pins, citing, splitting, open questions and staleness;
- typed and table pages, with a schema example;
- closing the loop.

It is ported from `hearth-knowledge` where that applies. Hearth's MCP tools map onto yaks commands; drafts are hand-edited and committed per change.

**Cold read** by a claude-haiku-5-5 scout ([thread](delta://thread/ksQQeB1CvWFdQrqkwpKQFhpMEJLOAD07P5IBxBCA0aLJyrhJPreUvVwLftHG)): it answered 8 scenarios and found 10 gaps. Fixed:

- `--where tag=` should be `tags=`;
- there was no schema-writing example;
- body editing and images;
- what an unpinned `code:` link resolves against;
- path forms;
- what `P6M`/`verified:` mean;
- when `yaks check` runs and how it differs from doctor;
- judging vendor sources.

Not fixed: range queries ("renews next month") need comparisons in `page query --where`, as Hearth's `page_query` has. That is a follow-up candidate.

`yaks check` is referenced but is being built by check-1 (yaks-c0ac) in this batch; this yak shears once that lands.

---
▸ 2026-11-01T22:51:30Z [delta:farm-wiki]

kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-01T22:51:30Z [delta:farm-wiki]

kind: move
from: shaving
to: shorn
thread: ksQQeB1CvWFdQrqkwpKQFhpMEJLOADtlxpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
