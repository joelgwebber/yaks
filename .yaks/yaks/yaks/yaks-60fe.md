---
id: yaks-60fe
title: create --set key=value, as update has
status: shaving
type: feature
priority: 2
created: '2026-11-08T00:37:14Z'
updated: '2026-11-08T00:49:18Z'
parent: yaks-44b3
tags:
- cli
---

`yaks create` takes `--due`, `--every`, `--wake` and the other built-in fields, but no `--set key=value` for any other frontmatter field; only `update` has it. A harness that tags new yaks with its own fields has to create, then update: two commits, and a window where the yak exists without the field.

Found on Hearth's first live turn on a farm (2026-11-07, hearth-1ed0 in the Hearth repo's farm): the model followed a skill that said `yaks create "..." --due ... --set person=joel`, got `error: unexpected argument '--set'`, and recovered with `update`. Hearth's reminders and recurring yaks all carry `person:`.

Proposal: `create --set key=value` (repeatable) and `--unset` refused, with exactly `update --set`'s rules (built-in fields refused, lists and maps parse the same, round-trip as written), written in the same commit as the create. `--key` find-or-create leaves an existing yak's fields alone. Update `docs/cli.md`, the `--help` text and the `yaks` skill's command table.

---
▸ 2026-11-08T00:37:21Z [herd-reshape]
kind: move
from: hairy
to: shaving
thread: ksQQBgNNWOHrR9mtaBzuvtn_eJLOADWrTJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-08T00:37:22Z [herd-reshape]

Claim (herd-reshape): worker set-1, from yaks main e8bc2c4f.

---
▸ 2026-11-08T00:42:53Z [set-1]
kind: verify
result: fail
---

verify: `cargo test -p yaks` -> FAIL (exit 101)

---
▸ 2026-11-08T00:49:06Z [set-1]
kind: verify
result: pass
---

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-11-08T00:49:18Z [set-1]

Start 2026-11-08T00:37:42Z. Farm file /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-60fe.md. Checkout /Users/joel/src/yaks/.delta/worktrees/shtfs2rdmezg/yaks.

Implemented: `create --set key=value` (repeatable) in src/main.rs, going through the same `parse_field_args` as `update --set`, into the existing `NewTask.fields`, so the fields are written by the create's one `save` (no farm change needed). Tests in tests/fields.rs: three fail before the change (one-commit write, `--set` beats `--due`, `--key` plus `--set`), pass after; the refusal test passes either way because clap already rejected the unknown flag before.

Decision: no `--unset` on create (nothing to remove on a new yak); an empty value (`--set k=`) sets nothing, as `update` would remove. Rejected: accepting `--unset` and ignoring it.

Decision: `--set` fields go after the sugar flags (`--due/--every/--wake`), so `--set due=...` wins, as on `update`.

Decision: with `--key` on an existing yak, `--set` is validated (a bad key still errors) but not applied. Rejected: updating the existing yak's fields (the spec says leave them alone).

Decision: reworded the shared built-in-field refusal to "(see `--help`)" since it now serves create and update.

Scratch-farm output of ./target/debug/yaks:
$ yaks create "Water plants" --due 2026-12-01 --set person=joel --set 'tags2=[a, b]'
Created t-4e42: Water plants
$ yaks show t-4e42   (tail)
due       2026-12-01
Other fields:
  person: joel
  tags2:
  - a
  - b
$ git -C farm log -1 --stat
yaks: created t-4e42 / Yaks-Actor: set-1 / .yaks/t/yaks/t-4e42.md | 14 ++++ (the one commit)
$ yaks create x --set title=y
Error: --set "title": a built-in field; use its own flag (see `--help`)   (exit 1, farm commit count unchanged)

Docs parity: docs/cli.md (create row and Details), clap help on the flag, .agents/skills/yaks/SKILL.md (create row, and the "Other fields" line). Grepped `--set` across docs/, .agents/skills/, README.md, src/: README has no create flags list; other skills only mention `update --set` or `dok --set`, left alone. The skill and cli.md create rows also now list --due/--every/--wake, which were missing.

Gate: `cargo test -p yaks --no-fail-fast` green; `yaks verify yaks-60fe` PASS. Note: tests/init_modes.rs `farm_works_from_a_git_worktree_and_a_shared_clone...` failed intermittently in some runs ("share2-web-wt already exists" under the shared temp dir), also with my change stashed: not mine.
