---
id: yaks-e92e
title: 'sheds landed: an edit the reference made after the fork is not the shed''s unlanded edit (three-way)'
status: shorn
type: bug
priority: 2
created: '2026-10-16T19:53:16Z'
updated: '2026-10-16T20:00:10Z'
parent: yaks-5662
labels:
- cli
- delta
---

`landed_ids` / `outstanding_ids` (src/sheds.rs) compare the shed with the REFERENCE two ways: `yak_changes(reference, theirs)`. For notes and status that is directional (a shed that is merely behind is not outstanding), but `edited` is `store::edited(reference, theirs)`, a plain inequality. So when the reference edits a yak after a shed forked (re-parent, retitle, a field), every shed that touched that yak and is behind on it reads that yak as NOT landed, and all its lines show again (including notes the reference already has).

Seen: review-survey-e0c0-arc (fork e1f11f9) showed survey-209f, b0fd, d13d, eb0c as outstanding. Base and shed both had `parent: survey-e0c0`; main later moved them to survey-3237 (10da8d7). Same for survey-e3f7 in fix-collapse-on-journals (main's 52cf335 dropped the parent).

Fix: three-way. An edit is outstanding only when the SHED made it, i.e. `store::edited(base, theirs)`, and the reference does not already carry it. Simplest sound version: outstanding when `edited(base, theirs) && edited(reference, theirs)`; a field-wise version (which fields the shed changed, and whether the reference holds the shed's values for them) is better when the reference also edited other fields after the shed. Decide and record which. `landed_ids` and `shed_yak_changes` (the `c.landed` flag) both need the base, so `outstanding_ids` grows a base argument; the TUI's `open_asks` reads `outstanding_ids` too, so check that caller.

Evidence: a test that fails first: shed forks, the reference re-parents a yak the shed noted, the reference also has the note. Before: not landed. After: landed. Plus a shed that really edited a field the reference lacks is still outstanding, and one where both edited different fields. Real machine: `yaks sheds` from ~/src/fs/subtext/surveyor no longer lists re-parented yaks for a shed that is only behind.

---
▸ 2026-10-16T19:53:21Z [delta]
moved: hairy -> shaving

---
▸ 2026-10-16T20:00:10Z [delta]
Decision: field-wise three-way, with the reference settling any field it also changed. A field counts as the shed's outstanding edit only when the shed changed it (base -> theirs) AND the reference still holds the base value (`store::edits_landed`). Rejected:
- `edited(base, theirs) && edited(reference, theirs)` (yak-level): still flags a yak when the reference edited a different field after the shed's edit landed.
- "reference holds the shed's value" per field (my first draft): the real review shed failed it. It had dropped survey-209f's `parent: survey-e0c0`; that landed, then main re-parented to survey-3237, so the reference has neither the base nor the shed's value. When both sides changed a field, the reference is where that gets settled, the same way a status the reference moved further along already reads landed.
A shed-created yak (no base) is unchanged: any difference is outstanding.

---
▸ 2026-10-16T20:00:10Z [delta]
Evidence. The new test `sheds::tests::landed_edits_are_three_way` covers: behind only (re-parented in the reference), the shed's own label (outstanding), both edited different fields (outstanding), both edited the same field (landed), and the surveyor review case. Restoring the old `c.edited` condition made it FAIL (`left: {yaks-behind, yaks-both, yaks-mine, yaks-took}`). With the fix it passes. Full `cargo test --release`: 17 suites, 0 failures.

Real data: a scratch `git clone --shared` of ~/src/fs/subtext/surveyor at 8ef0612, with worktrees replaying the two now-gone clones (review: e1f11f9 -> 973d4f6; fix-collapse: 493e7b4 -> a5956a7).
```
            0.0.17                                          this change
review    7 new, 15 moved, 43 new notes, 9 edited (27 landed)  landed (31 yaks)
collapse  1 new notes                                          landed (1 yak)
```
Scratch clone removed afterwards. Docs: the `landed` rule in docs/cli.md `sheds` now states the edit case. README, the skill and `--help` don't spell out the rule (they say "already in the reference"), so they still hold.

---
▸ 2026-10-16T20:00:10Z [delta]
moved: shaving -> shorn
