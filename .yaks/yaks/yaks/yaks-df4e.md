---
id: yaks-df4e
title: Docs, bundled skills and clap help for the new layout and links
status: shaving
type: task
priority: 3
created: '2026-11-14T11:21:02Z'
updated: '2026-11-15T08:45:32Z'
parent: yaks-1151
depends_on:
- yaks-acc0
tags:
- docs
- skills
---

Last of yaks-1151: update every surface that mentions `.yaks/`, a link form or the note format, once the migration is in (AGENTS.md "Keeping docs current"): `docs/cli.md`, `docs/tui.md`, `README.md`, `AGENTS.md`, the bundled skills (`yaks`, `yaks-research`, `yaks-upkeep`, `yaks-coordinating`, `yaks-working`), the clap help in `src/main.rs`, and the farm doks `knowledge-store.md` and `migrate-a-farm.md`. Includes the yaks-5f70 session-citation example if its ask has been answered.

Evidence: a grep for `.yaks/` and the old link forms across `docs/`, `.agents/skills/`, `README.md` and `src/`, showing only the intended hits.

---
▸ 2026-11-15T08:45:30Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-15T08:45:32Z [linkplan-lead]

Done in a7a34957. Swept `.yaks/` from README.md, docs/ (cli, tui, README, skills), AGENTS.md, the `yaks`, `yaks-coordinating`, `yaks-scheduling` and `land` skills, `src/brief.rs` and the clap help; the remaining `.yaks/` hits are the intended ones (the stub, the pre-schema-9 notes, the stray-layout finding, the migrate docs). New text: the schema-9 section of `migrate` (cli.md and clap help), `stray-layout` under `check`, the path forms in the link table, and the farm dok `howto/migrate-a-farm.md`. `update --note` and `session touched` now print the host-followable note link on a new-shape body, with tests. Not done: `doks/design/knowledge-store.md` mentions no layout, so left alone; no cold-read scout of the skills prose.
