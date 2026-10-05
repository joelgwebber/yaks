---
id: yaks-c29a
title: 'yaks lanes: label each lane with who is working in it and on which yaks'
type: feature
priority: 2
created: '2026-10-05T19:48:22Z'
updated: '2026-10-05T19:53:50Z'
parent: yaks-38dc
labels:
- cli
- delta
---

Joel's question (2026-10-04): the lanes output shows only Delta's opaque per-mount directory, so it cannot be tied to a thread. Option 1 (agreed): derive a label from the lane's OWN farm changes (already computed against the merge-base): the actors on its new notes and moved: entries (a YAKS_ACTOR name, else delta:<thread title> from DELTA_THREAD_TITLE, see src/actor.rs) and the yaks it has in progress, e.g. 'lanes-2 · yaks-d738 shaving'. Human-readable table and --json (a 'who' list and an 'in_progress' list). A lane with no yak activity stays unlabeled (later option: an identity stamp written under the checkout's git dir by yaks commands run under Delta; not part of this yak). Works for plain git worktrees too (actor is whatever wrote the entries). No Delta database, no new tracked format. Evidence: tests with temp repos (worker-named lane, delta:<title> lane, git-user fallback, idle lane unlabeled, two actors in one lane) and a real run on a Delta thread; docs parity.

---
▸ 2026-10-05T19:49:09Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-05T19:49:09Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker lbl-1. Scope: src/lanes.rs, the lanes rows in docs/cli.md, README.md, .agents/skills/yaks/SKILL.md, the Lanes help in src/main.rs. Do NOT touch yaks-1b39 (ahead/behind '?'), a separate yak I do next in the same file. Actor extraction: parse the lane's own new entries with store::parse_notes; the 'moved: hairy -> shaving' entries give in-progress yaks; reuse store::is_transition_text. Judge: coordinator re-runs the gate and 'yaks lanes' on this thread.

---
▸ 2026-10-05T19:53:04Z [lbl-1]
Start 2026-10-05T19:49:42Z (date -u: Mon Oct 5 19:49:42 UTC 2026). ls .yaks/*/yaks-c29a.md -> .yaks/shaving/yaks-c29a.md. Built release binary at HEAD 5dd2f4d.

In-progress rule chosen (stated here; deviates from both of the brief's two alternatives on purpose): a yak is in progress in a lane iff it is in shaving IN THE LANE and the lane has OWN entries on it (notes beyond the merge-base's count, or every note of a yak the base lacks). Why not 'latest own transition is -> shaving' or 'shaving in lane, not at base': a worker's checkout forks AFTER the coordinator's claim, so the claim is in the base and the worker's own entries are plain notes; both alternatives would leave a real worker lane (e.g. pre-1's, with yaks-d1bd) unlabeled. A yak shorn in the lane, or untouched by it, is not in progress. who = distinct actors on own entries, first-seen by timestamp; unattributed entries name no one. Lane with no merge-base (vs checkout) or no own entries: unlabeled. Caveat: a coordinator's lane that wrote notes/claims on workers' yaks lists those yaks too.

---
▸ 2026-10-05T19:53:50Z [lbl-1]
Done. src/lanes.rs: label() (pure; own entries = notes beyond the merge-base's per-yak count via store::parse_notes; who ordered by ts, first-seen; in_progress = shaving in lane with own entries, rule in the note above), Lane.who / Lane.in_progress (set only on the merge-base path, so vs-checkout lanes stay unlabeled), label_line() in render (one line under the row), JSON who/in_progress. is_transition_text not needed: a moved: entry is just an own entry. Tests (lanes::tests): a_worker_named_lane_is_labeled_with_the_worker_and_its_yak, a_delta_thread_lane_is_labeled_with_its_delta_title, a_git_user_fallback_actor_labels_the_lane_like_any_other, an_idle_lane_is_unlabeled, entries_without_an_actor_are_not_a_who, two_actors_in_one_lane_are_listed_once_each_in_first_seen_order, a_yak_shorn_within_the_lane_is_not_in_progress, a_lane_compared_vs_checkout_is_unlabeled. With the label assignment disabled 6 of 8 fail (the idle and vs-checkout ones are negative guards and pass trivially). cargo test -p yaks green (390+8+28). Real run in own Delta checkout: before no label lines; after 'who: pre-1 · shaving: yaks-d1bd' (pre-1 lane) and 'who: delta-lead · shaving: yaks-b5a0' (coordinator lane); stale lanes unlabeled. Docs parity: docs/cli.md, README.md, SKILL.md, Lanes --help. Finished 2026-10-05T19:53:50Z.

---
▸ 2026-10-05T19:53:50Z [lbl-1]
moved: shaving -> shorn
