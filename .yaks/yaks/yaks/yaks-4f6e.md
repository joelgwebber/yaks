---
id: yaks-4f6e
title: Release v0.0.17
status: shorn
type: task
priority: 2
created: '2026-10-12T23:02:13Z'
updated: '2026-10-12T23:13:52Z'
labels:
- release
---

---
▸ 2026-10-12T23:02:14Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-12T23:03:22Z [delta-lead]

Version 0.0.17 (Cargo.toml, Cargo.lock, npm/yaks/package.json): phase 4 of the flat farm (yaks-bbda: no more yaks migrate or old-layout reading; a pre-0.0.16 farm refuses to open and names 0.0.16's migrate). Gate: see the commit note.

---
▸ 2026-10-12T23:13:00Z [delta-lead]

Released 0.0.17. Dry run 38320104901 and tag run 38320360092 green; real publish of all six packages, zero npm errors. npm pack check: all six fetchable by 23:12:46Z (propagation ~2 minutes); npx -y @j15r/yaks@0.0.17 --version prints 'yaks 0.0.17'.

---
▸ 2026-10-12T23:13:51Z [delta-lead]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-12T23:13:52Z [delta-lead]

moved: shaving -> shorn
