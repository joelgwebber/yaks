---
id: yaks-62f2
title: 'Invariant: a shorn yak has no unfinished children (doctor check, shorn refuses, skills rule)'
status: hairy
type: task
priority: 2
created: '2026-10-31T19:28:04Z'
updated: '2026-10-31T19:28:04Z'
labels:
- cli
- skills
---

Joel (2026-10-31): a yak is shorn only when all its children are shorn (or dead). If the parent really is done enough, graduate the unfinished children first: reparent them to another family, or unparent them.

Found when `yaks-da23` was shorn while it still held `yaks-f1e6` and `yaks-19c3` (hairy); fixed by moving them to `yaks-039b`.

## Scope

- **`yaks doctor`:** a check, "shorn parent with unfinished children", listing each parent and the children that are not shorn or dead.
- **`yaks shorn <id>`:** refuse (or at least warn) when the yak has children that are not shorn or dead, naming them and the fix (`yaks reparent <child> --parent <other>` / `--unparent`). Decide refuse vs warn; the lean is refuse, with `--force` only if a real need appears.
- **Skills:** the `yaks` skill's Workflow "Done" step already says to shear the parent when its last child is shorn. Add the converse as a hard rule: never shear a yak over unfinished children; graduate them first. The same goes for `yaks-working`'s finish step and the coordinating skill's reconcile step.
- **Docs and help:** `docs/cli.md` (`shorn`, `doctor`) and `--help`.
