---
id: yaks-1af5
title: 'CI: tests assumed a global git identity; the release dry run failed on a runner without one'
type: bug
priority: 3
created: '2026-10-10T13:59:36Z'
updated: '2026-10-10T13:59:36Z'
labels:
- ci
- tests
---

---
▸ 2026-10-10T13:59:36Z [delta:Delta :Yaks (cont'd)]
moved: hairy -> shaving

---
▸ 2026-10-10T13:59:36Z [delta:Delta :Yaks (cont'd)]
The release dry run (run 38057238039) failed in 'cargo test --workspace --locked': commit::tests (2), tests/status.rs (1) and land.sh --selftest (1) ran git commit with no identity configured; they passed here only because the Mac has a global one. Reproduced with GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null HOME=$(mktemp -d); fix: those repos set user.name/user.email themselves, and land.sh --selftest exports GIT_AUTHOR_*/GIT_COMMITTER_*. After the fix all 16 suites pass with no global identity. Lesson (for AGENTS.md): run the gate with that environment before a release.

---
▸ 2026-10-10T13:59:36Z [delta:Delta :Yaks (cont'd)]
moved: shaving -> shorn
