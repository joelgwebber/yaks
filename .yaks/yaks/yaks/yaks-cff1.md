---
id: yaks-cff1
title: 'Guard: land skill is repo-local, never bundled'
status: shorn
type: chore
priority: 3
created: '2026-10-22T19:40:02Z'
updated: '2026-10-22T19:40:12Z'
labels:
- docs
verify: cargo test --bin yaks skills
---

Delta insists on a skill named land; it is this repo's own landing action. Make it unmistakable it is not bundled: warning in .agents/skills/README.md and land/SKILL.md, a BUNDLED doc comment, and a test (land_is_repo_local_and_never_bundled).

---
▸ 2026-10-22T19:40:11Z [delta:Fix TUI log spam during sync]

moved: hairy -> shaving

---
▸ 2026-10-22T19:40:11Z [delta:Fix TUI log spam during sync]

Added: README warning, land/SKILL.md banner, BUNDLED doc comment, test land_is_repo_local_and_never_bundled. cargo test --bin yaks skills: 38 passed.

---
▸ 2026-10-22T19:40:12Z [delta:Fix TUI log spam during sync]

moved: shaving -> shorn
