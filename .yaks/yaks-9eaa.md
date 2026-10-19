---
id: yaks-9eaa
title: Stamp the shed on every write; build the sheds view from stamps (group-by), delete the farm-diff half of sheds.rs
status: hairy
type: feature
priority: 2
created: '2026-10-18T05:15:36Z'
updated: '2026-10-19T04:29:29Z'
parent: yaks-6a34
labels:
- sheds
- cli
- tui
---

With one live farm (yaks-6a34) "what is shed X doing" cannot come from diffing farm copies. Every write (note, move, ask, answer, attach) records the shed it ran in beside the actor, named by the rule `yaks sheds` already uses (`main`, the Delta thread-title slug, ...), e.g. a `shed:` on the entry header or an `actor@shed` form; decide the on-disk shape carefully (schema bump) and keep it readable.

Then the Sheds tab and `yaks sheds` compute per shed, from the farm alone: yaks it claimed (`shaving` with its stamp), yaks it wrote to, newest stamp (liveness), open asks it raised; plus the git facts discovery already gives (branch, ahead/behind, dirty, merged). Ordering per yaks-0309/yaks-9a06 falls out as a plain sort.

Delete once the trial holds: fork points / base farms / `compare_farms` / `yak_changes` / `landed` / `needs_superseded` / `outstanding_ids` in sheds.rs, `inbox --sheds`, `show --sheds`, `answer <id>@<shed>`, `changes <shed>`, the TUI ghost rows, badges and `copies:` chip. Keep: discovery (host, pins, worktrees), names, git state. Blocked on the trial's verdict.

---
▸ 2026-10-19T04:29:29Z [delta:herd-pass]

Herd pass 2026-10-19: unblocked, the trial's verdict is in (yaks-6a34). The per-shed farm-diff commands it deletes (`changes <shed>`, `inbox --sheds`, `show --sheds`, `answer <id>@<shed>`, ghost rows) are also what the skills rewrite (yaks-b048) stops teaching, so land the two together or skills first. yaks-0309 and yaks-9a06 (ordering) now sit under this yak.
