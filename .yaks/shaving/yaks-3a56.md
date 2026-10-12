---
id: yaks-3a56
title: 'Flat farm, phase 3: brief + skills/docs sweep, migrate this farm, real-machine checks, release 0.0.16'
type: task
priority: 1
created: '2026-10-12T19:54:05Z'
updated: '2026-10-12T21:02:26Z'
parent: yaks-8b7a
depends_on:
- yaks-86f4
- yaks-9f91
- yaks-ca28
labels:
- cli
- skills
- release
---

Phase 3 of yaks-8b7a (coordinator; after all three phase-2 workers land): src/brief.rs (the finish step is `git add $(yaks path <id>)` alone), the skills sweep (.agents/skills: staging guidance, the glob pathspec, "a move relocates the file", status dirs in examples), docs/README.md and any remaining prose; then, at a quiet point with every `yaks tui` closed, `yaks migrate` this repo's own farm in one commit; real-machine checks: docshots unchanged, `yaks sheds` against a clone forked before the migration, `yaks migrate --dry-run` on ~/src/j15r; release 0.0.16.

---
▸ 2026-10-12T20:00:39Z [delta-lead]
Amended (see yaks-8b7a): brief.rs's finish step stages `.yaks/<id>.md` and `.yaks/<id>/`; skills drop every `.yaks/artifacts/<id>` staging line. Also migrate ~/src/j15r (Joel: we are the only users, migrate our farms quickly, then phase 4 removes the migration).

---
▸ 2026-10-12T21:02:26Z [delta-lead]
moved: hairy -> shaving
