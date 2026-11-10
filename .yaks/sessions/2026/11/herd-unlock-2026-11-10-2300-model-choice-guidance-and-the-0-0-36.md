---
type: session-review
title: Model-choice guidance and the 0.0.36 release
source: session-note
actor: herd-unlock
person: Joel Webber
host: joel
harness: delta
thread: ksQQUQUvLIRxSc62aHEDNt0G0pLOAFQ8T5IBxBCA0aLJyrhJPreUvVwLftHG
day: 2026-11-10
written: '2026-11-10T23:00:33Z'
good: 1
---

## Summary

Joel asked for yaks-2788 (farming out to cheaper models in the skills), then to land, push and cut a release. `yaks-coordinating` gained "Choosing a model" (in section 5): tiers not ids, delegable yaks to the mid tier, design and landings kept, review as the cost, models recorded in the batch note; `AGENTS.md` keeps only the standing instruction ([the shear note](yaks-2788)). A Haiku scout cold-read it and found three gaps, fixed before the commit. Then 0.0.36: version bump, full gate with `--locked` green, `main` pushed to `local` and `origin` (`e4d0ff5a`), tag `v0.0.36`, release workflow green, all six npm packages published, the farm synced.

## Turns

- good: "take on 2788, then land everything, push, and cut a new release"

## Saved

- yaks-2788: Skills: farming out yaks to cheaper models (when, which model, what the coordinator keeps) (hairy -> shorn)
  - [Done (herd-unlock-2) in `7bc25139`: `yaks-coordinating` gains "Choosing a mod...](yaks-2788#2026-11-10T22:44:32Z)

## Lessons

- `npx @j15r/yaks@<new>` failed with ETARGET for a few minutes after a successful publish (cached packument); `--prefer-online` or waiting fixed it. Check the registry with `curl` before calling a release broken.
- The farm turned out already pushed: the TUI's interval auto-sync had been pushing all session. The "publishing is the human's call" line in `AGENTS.md` doesn't match how the farm actually behaves (yaks-d426 is the fix).
- A cheap cold-read scout earned its keep again: three real gaps in a 10-line section.
