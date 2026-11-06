---
id: yaks-8a92
title: A stale farm checkout says migrate when the remote is already migrated
status: shorn
type: bug
priority: 2
created: '2026-11-06T21:37:13Z'
updated: '2026-11-06T22:23:37Z'
tags:
- cli
- git
---

From yaks-3f35, item 9. A machine holds one farm worktree per host git dir (`store::farm_home`), so a repo can have several on one machine: Joel's own checkout of schism (`schism-38838c3f`) and the Delta-managed repository its Delta threads clone from (`repository-c0e58d55`). The schism import ran in the second; the first, not fetched, was on an older schema and refused with "run `yaks migrate`" (`store::migration_refusal`) although `origin/yaks` was already migrated.

Following that advice would migrate a farm a second time against stale history. The skills tell agents never to migrate on their own, but a human gets the same advice.

## Fix

Before telling anyone to migrate, look at the farm branch's upstream (`origin/yaks`, already fetched or fetched cheaply): when it is on the current schema, say to fetch and fast-forward (`yaks sync`) instead. Consider having `yaks sync` work on a farm that is behind only because it is stale.

## Also worth recording

The skills say the farm is "checked out once per machine", but a Delta-managed repository and the user's checkout are two hosts, so two worktrees whose writes meet only through `yaks sync`. Either say so in the `yaks` skill, or make the two share one worktree (keyed by the origin URL rather than the git dir); yaks-9a13 (`repository-…`) touches the same code.

---
▸ 2026-11-06T22:09:26Z [schism-feedback]

The two-worktrees half is yaks-35b5 (built, not landed). This yak keeps the migrate-message half: suggest fetching when `origin/<branch>` is already on the current schema.

---
▸ 2026-11-06T22:18:25Z [schism-feedback]
kind: move
from: hairy
to: shaving
thread: ksQQNtlWqzuUT5COVXyFQVm8cZLOAEgKKZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-06T22:23:35Z [schism-feedback]

Landed on `main` as 34ded4b5 and installed.

`store::migration_refusal`, for a farm branch worktree: when `<branch>@{upstream}` (as last fetched) has a newer `.yaks/schema`, the refusal says the checkout is behind, not unmigrated, and names `git -C <worktree> pull --ff-only` (ask a human if it won't fast-forward). Otherwise it keeps the `yaks migrate` advice and adds "fetch first". `yaks sync` isn't suggested, because it opens the farm and is refused too.

Evidence: the test `a_stale_farm_whose_upstream_is_migrated_is_told_to_fast_forward` (a pushed schema-7 farm, then a migrated upstream with the worktree reset behind it); full `cargo test` green. The `yaks` skill's "Never migrate" paragraph and `docs/cli.md` `migrate` say what to do with the new message.

---
▸ 2026-11-06T22:23:37Z [schism-feedback]
kind: move
from: shaving
to: shorn
thread: ksQQNtlWqzuUT5COVXyFQVm8cZLOAEgKKZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn
