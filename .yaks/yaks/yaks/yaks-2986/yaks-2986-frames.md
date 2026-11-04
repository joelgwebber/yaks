## Edit form on a recurring yak (E, then Tab x5)
=== frame 6 · 78x16 · focus=list · view=🕒 Recent · cursor=0 · rows=50 · sel=yaks-0d7c · blocked=[yaks-79a6,yaks-8d6a,yaks-8fd1,yaks-d262] · collapsed=62 · filter=(all) · overlay=create ===
 🕒 Recent (50)   ⭐ Starred (5)   📥 Inbox (5)   🦬 Hairy (58)   🪒 Shaving (

◂ Edit yaks-0d7c
│   title       Write the release notes
│   type         task bug feature idea
│   priority     p1 p2 p3 p4 p5
│   tags
│   source
│ ▸ due         2026-11-02
│   every       P1W, P1D, P1M: makes it recur
│   wakes        none agent human
│
│
│
│
Tab/↑↓ rows · ←→ chips · Ctrl-S save · EscEsc/Ctrl-C cancel
=== end ===

## Z: the snooze prompt
=== frame 3 · 78x16 · focus=list · view=🕒 Recent · cursor=1 · rows=50 · sel=yaks-3979 · blocked=[yaks-79a6,yaks-8d6a,yaks-8fd1,yaks-d262] · collapsed=62 · filter=(all) · overlay=edit ===
 🕒 Recent (50)   ⭐ Starred (5)   📥 Inbox (5)   🦬 Hairy (58)   🪒 Shaving (

 yaks-0d7c p3 task     🦬 Write the release notes                   2026-11-02
 yaks-3979 p3 task     🦬 Sweep the review queue                    2026-11-01
 yaks-907a p3 bug      🪒 :wq pauses for ~500-1000ms, but ctrl- [tui, perf] ⏳
 yaks-1ebf p3 bug      🪒 TUI reloads twice per own save (apply_ed [tui, perf]
 yaks-bcd4 p3 bug      🐑 TUI reload spends ~130 ms in git to refr [tui, perf]
 yaks-717b p4 bug      🦬 tests/init_modes: farm_works_from_a_git_work [tests]
 yaks-5e8d p2 feature  🐑 yaks tick: regrow what is due, then report wha [cli]
 yaks-2986 p2 feature  🪒 TUI: set due and every, and a snooze key       [tui]
 yaks-7058 p3 task     🐑 Per-change commits: cut the git ca [git, perf, doks]
 yaks-a041 p2 feature  🐑 Regrow routes a recurring yak to the inbox (ne [cli]
 yaks-bdae p2 feature  🐑 create --key <source-url> and list --source: f [cli]
 yaks-0079 p2 feature  🐑 Relative --due (+2d, +1w) and yaks snooze      [cli]
 yaks-5550 p2 task     🐑 Migrate the joel research repo onto a farm (y [doks]
Snooze yaks-3979 until (Enter = +1d · +1w · 2026-11-15 · Ctrl-C cancel) INSERT
=== end ===

## after Enter
=== frame 4 · 78x16 · focus=list · view=🕒 Recent · cursor=0 · rows=50 · sel=yaks-3979 · blocked=[yaks-79a6,yaks-8d6a,yaks-8fd1,yaks-d262] · collapsed=62 · filter=(all) · overlay=none ===
 🕒 Recent (50)   ⭐ Starred (5)   📥 Inbox (5)   🦬 Hairy (58)   🪒 Shaving (
yaks-3979 snoozed until 2026-11-07
 yaks-3979 p3 task     🦬 Sweep the review queue                    2026-11-07
 yaks-0d7c p3 task     🦬 Write the release notes                   2026-11-02
 yaks-907a p3 bug      🪒 :wq pauses for ~500-1000ms, but ctrl- [tui, perf] ⏳
 yaks-1ebf p3 bug      🪒 TUI reloads twice per own save (apply_ed [tui, perf]
 yaks-bcd4 p3 bug      🐑 TUI reload spends ~130 ms in git to refr [tui, perf]
 yaks-717b p4 bug      🦬 tests/init_modes: farm_works_from_a_git_work [tests]
 yaks-5e8d p2 feature  🐑 yaks tick: regrow what is due, then report wha [cli]
 yaks-2986 p2 feature  🪒 TUI: set due and every, and a snooze key       [tui]
 yaks-7058 p3 task     🐑 Per-change commits: cut the git ca [git, perf, doks]
 yaks-a041 p2 feature  🐑 Regrow routes a recurring yak to the inbox (ne [cli]
 yaks-bdae p2 feature  🐑 create --key <source-url> and list --source: f [cli]
 yaks-0079 p2 feature  🐑 Relative --due (+2d, +1w) and yaks snooze      [cli]
 yaks-5550 p2 task     🐑 Migrate the joel research repo onto a farm (y [doks]
Tab:view  j/k:move  l:detail  F:family  `:views  c/C:new  E:edit  X:del  S:sta
=== end ===

## a due that does not read keeps the form open
=== frame 9 · 78x16 · focus=list · view=🕒 Recent · cursor=0 · rows=50 · sel=yaks-3979 · blocked=[yaks-79a6,yaks-8d6a,yaks-8fd1,yaks-d262] · collapsed=62 · filter=(all) · overlay=create ===
 🕒 Recent (50)   ⭐ Starred (5)   📥 Inbox (5)   🦬 Hairy (58)   🪒 Shaving (
due: "tomorrow" is not a date, a datetime or +2d / +1w
◂ New yak
│   title       Standup
│   type         task bug feature idea
│   priority     p1 p2 p3 p4 p5
│   tags
│   source
│ ▸ due         tomorrow
│   every       P1W, P1D, P1M: makes it recur
│   wakes        none agent human
│   description ─────────────────────────────────────────────────────────────
│ (no description)
│
│
Tab/↑↓ rows · ←→ chips · Ctrl-S create · EscEsc/Ctrl-C cancel
=== end ===

