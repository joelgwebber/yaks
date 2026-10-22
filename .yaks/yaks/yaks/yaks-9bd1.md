---
id: yaks-9bd1
title: 'Release 0.0.22: farm branch lifecycle, schema 6, skills 8 to 4'
status: shorn
type: task
priority: 1
created: '2026-10-22T01:16:51Z'
updated: '2026-10-22T01:54:26Z'
labels:
- release
---

---
▸ 2026-10-22T01:16:51Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-22T01:54:25Z [delta:herd-dirs]

Released 0.0.22.
- The first tag (on `9eedc9c`) failed CI: `preflight::tests::a_branch_farm_with_a_merge_conflict_still_fails` ran `git merge` without an identity, and the Linux runner has none. Nothing was published. Fixed in `ff94442`, which was verified under `user.useConfigOnly=true`: the old test failed exactly as on CI, and the full suite passed with the fix.
- Joel approved moving the tag; `v0.0.22` was force-pushed onto `ff94442`.
- Release run 39706273956: test, all 5 builds and publish succeeded. `+ @j15r/yaks@0.0.22` was published with provenance, and `npm view @j15r/yaks version` returns 0.0.22.
- pnpm's minimumReleaseAge (24h) blocks `pnpm add -g` until tomorrow. `~/go/bin/yaks` and `~/.cargo/bin/yaks` are builds of the tagged commit; `yaks --version` reports 0.0.22.

---
▸ 2026-10-22T01:54:26Z [delta:herd-dirs]

moved: shaving -> shorn
