---
id: yaks-ed5e
title: 'yaks sheds: show a shed as landed when its farm changes are all already in this checkout''s farm'
type: feature
priority: 3
created: '2026-10-10T13:40:07Z'
updated: '2026-10-10T15:44:45Z'
parent: yaks-ee0a
labels:
- cli
- delta
---

Raised by Joel's check of `title-1` (2026-10-10). `yaks sheds` showed that shed as `+1 -5` with "1 moved, 5 new notes" although everything it did is in main: its commit 0c98505 landed as a cherry-pick (49c2d5d) because its docs/cli.md row conflicted with another worker's, so git does not count 0c98505 as merged, and `sheds` measures by commits (known limit, yaks-df61 O55). Verified by hand: its yak file is byte-identical to main's, its code differs from main only by the other worker's later change, its clone is clean.
Goal: a shed whose farm changes are ALL already in this checkout's farm reads as `landed`, not as outstanding work. Per yak the shed changed (reuse `yak_changes`): landed when this checkout has the yak, in the same status, with every note entry the shed has (timestamp, actor, text), and the same `needs:`. A shed is `landed` when it has at least one change and every changed yak is landed; a shed with no changes keeps "no changes of its own"; a shed with some landed and some outstanding yaks lists the outstanding ones and says how many are landed. Show it on the `farm:` line (e.g. `farm: landed (1 yak)`) and in JSON (`farm.landed`: bool or count; record the shape as a Decision:). Keep ahead/behind exactly as they are (they are about commits).
Also: `yaks changes <shed>` marks each yak `landed` when it is. `inbox --sheds` already ignores asks this checkout has.
Scope: src/sheds.rs (a function beside compare_farms/yak_changes, the render of the farm line and `changes`, JSON), docs/cli.md, README.md, the yaks skill sheds/changes rows. Out of scope: any write, parent/child display (Joel deferred it until Delta adds CLI support).
Evidence (tests FAIL first): a worktree shed whose yak change was cherry-picked/squashed into main with a different commit shows `landed`; one with an extra note main lacks does not; mixed case. Real-machine check by the coordinator: title-1's shed (606y05kk15w7) reads landed. Judge: coordinator.

---
▸ 2026-10-10T15:44:45Z [delta-lead]
Second real-machine case (2026-10-10, ~/src/j15r clones 47g751xth9ea and nh3dwrb2as8f, viewed from clone bcasg4bjrcxp): both are at 07fa8a4, +0 -7, and each holds seven uncommitted yak files that are Delta-applied copies of work the primary has since committed. `yaks sheds` says `farm: 5 new, 2 moved` for each; but j15r-7384, 9e6d, 9ea3, a24d, ab39 are `shorn` here with notes (the shed's copies are `shaving`/`hairy` with none), and j15r-7de1, ad91 are identical here. Nothing in those sheds is outstanding.
The `landed` rule above (same status, every note the shed has) does not catch it, because OUR copy is further along, not equal. Same family, other direction: a shed copy that is an earlier state of ours (same id, our status later in hairy -> shaving -> shorn, our notes a superset of its) should also read as already in this checkout. Decide whether `landed` covers "superseded by ours" or it is a separate word (see the shed state vocabulary yak: `stale`). Add this case to the tests: a shed whose yak is `shaving` with no notes while ours is `shorn` with notes.
