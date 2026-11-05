---
id: yaks-907a
title: ':wq pauses for ~500-1000ms, but ctrl-s finishes quickly'
status: shaving
type: bug
priority: 3
created: '2026-11-03T12:22:51Z'
updated: '2026-11-05T05:33:23Z'
parent: yaks-f677
depends_on:
- yaks-7058
tags:
- tui
- perf
needs: agent
---

Feels like there might be an intentional delay for some reason?

---
▸ 2026-11-03T21:40:35Z [delta:starred-batch]

kind: move
from: hairy
to: shaving
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-03T21:42:04Z [delta:starred-batch]

Investigated (delta:starred-batch), not fixed.

- In code, `:wq`/`:w`/`:x` and Ctrl-S end in the same `commit_edit` ([fn run_command](code:src/tui/handlers.rs#L1682), [fn editor_key](code:src/tui/handlers.rs#L305)); there is no delay or timer on the `:` path.
- Timed headlessly on a clone of this farm (adding a comment): Ctrl-S frame-to-frame 553 ms, `:wq` 587 ms. Both are the per-change farm commit plus reload after the save, i.e. yaks-7058 (49 git calls per write), not anything `:wq`-specific.

So headless shows no difference; whatever Joel sees may be terminal-side.

---
▸ 2026-11-03T21:42:05Z [delta:starred-batch]

kind: ask
needs: human
---

asked: needs human

I can't reproduce a `:wq`-only pause: headlessly both `:wq` and Ctrl-S take ~0.55-0.6 s, all of it the per-change farm commit after the save (yaks-7058). Two questions:

1. In your terminal, does Ctrl-S really feel instant on the *same* kind of edit (a comment or description, not a single-line field)? If both pause, I'd fold this into yaks-7058 (make the commit cheaper, or run it after the frame is drawn).
2. If only `:wq` pauses: which terminal, and is it inside tmux/zellij? An escape-time delay on the `Esc` you press before `:` is my next suspect.

---
▸ 2026-11-03T21:53:17Z [Joel Webber]

kind: answer
needs: agent
---

You're right -- it's actually both; I just didn't notice. Sometimes it seems instantaneous, other times it takes over 500ms (eyeballing it). I just tracked this down to whether it actually makes a change or not (ie, edit/save with no changes is fast), which I suppose is unsurprising.

Are we doing more work on write than just editing the yak file? Might this just be some UI refresh cost that's more expensive than expected?

---
▸ 2026-11-03T22:09:33Z [delta:starred-batch]

kind: pickup
---

picked up

Measuring write cost: commit vs reload.

---
▸ 2026-11-03T22:10:03Z [delta:starred-batch]

Measured (release build, clone of this farm, `yaks update --note` from the CLI, three runs each):

- git farm: 494 / 479 / 501 ms
- the same farm as a plain directory (no git): 47 / 44 / 46 ms
- `yaks list` on the git farm: 35 ms

So the yak write and the TUI's refresh are cheap; ~450 ms of every *changing* write is the per-change farm commit (yaks-a838), whose ~49 git calls are mostly repeated discovery. That is yaks-7058. A no-op save skips the commit, which is why it is instant.

Decision: fold this into yaks-7058 (now a dependency) rather than fix it twice. Two parts there would remove the pause: cache discovery for the process and build the commit from one `status` (yaks-7058's plan), and in the TUI, run the commit after the frame that shows the saved text is drawn, so even a slow git never blocks the redraw.

---
▸ 2026-11-03T22:10:05Z [delta:starred-batch]

kind: move
from: shaving
to: hairy
thread: ksQQGwTub5pWSfWv5Hlp3m1Av5LOAD8vcpIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> hairy

---
▸ 2026-11-04T21:28:12Z [perf-907a]
kind: move
from: hairy
to: shaving
thread: ksQQstI5T-eCRACxAjKJdNo3oJLOAEQG4JIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-04T21:35:34Z [perf-907a]

Where the ~0.5-0.9 s of a changing save goes. Release build, a clone of this farm (880 yaks), driven in a real pty (`yaks tui`, `:wq` then Enter, time to first output), plus a `git` shim logging every spawn with timestamps. No-op saves skip all of it.

| Piece | Cost | Where |
|---|---|---|
| per-change commit | ~350 ms | `autocommit::snapshot` + `finish`, under the farm lock |
| chip refresh in the reload after the save | ~130 ms | `refresh_farm_sync`, 10 git spawns |
| the same reload again, from our own file watcher | ~150 ms | `runtime.rs` `dirty` path |
| reading yak files, doks, drawing | 13-15 / 0 / 0-4 ms | not a factor |

First output after the save: ~700-760 ms with commits, ~300-400 ms with `YAKS_NO_AUTOCOMMIT=1`, ~45 ms (headless) on a plain directory.

Correction to the earlier note here: "the yak write and the TUI's refresh are cheap" was wrong. The reload is about 130-150 ms of git, not cheap, and it runs twice per save. The commit is ~350 ms, not ~450.

Why every git call hurts: one spawn is 11-30 ms on this Mac (a bare `git rev-parse` is ~12 ms; `status` ~30 ms), and a save makes ~57 of them. The commit path alone (`snapshot` + `commit_changes`) runs `rev-parse --show-toplevel`, `config --get yaks.branch`, `symbolic-ref`, `rev-parse --git-common-dir`, `--is-bare-repository`, `ls-files`, `status` (twice), `show HEAD:path`, `hash-object`, `var GIT_COMMITTER_IDENT`, `read-tree`, `update-index`, `write-tree`, `rev-parse ^{tree}`, `commit-tree`, `update-ref`, `reset`.

Filed from this:

- yaks-bcd4: the chip refresh costs ~130 ms of git per reload.
- yaks-1ebf: our own save triggers a second reload via the watcher.
- yaks-7058 (existing): the per-write git calls in the commit itself. Its numbers hold.

Not caused by git config: `core.fsmonitor=true` is set in `~/.gitconfig` and makes `status` ~2x slower than without it here (31 vs 16 ms), but that is the machine, not yaks.

Not found: anything `:wq`-specific, a timer, or an `Esc` delay. `:wq` and Ctrl-S share `commit_edit`.

---
▸ 2026-11-04T21:35:35Z [perf-907a]
kind: ask
needs: human
---

asked: needs human

Three fixes would together take a changing save from ~0.7 s to about the no-git ~0.05-0.1 s. One is a judgment call, so I'm asking before building.

1. Cache git discovery for the process (branch, toplevel, `yaks.branch`, identity) and build the commit and snapshot from one `status` call. This is yaks-7058. No behavior change. Lean: do it.
2. Fold the chip refresh into one `git status --porcelain=v2 --branch -z` (yaks-bcd4) and stop the double reload (yaks-1ebf). No behavior change. Lean: do it.
3. Run the per-change commit after the frame that shows the saved text is drawn (the TUI only). This changes behavior: for a moment the file is written but not yet committed, so quitting right then, a crash, a `yaks sync` from another process, or the watcher seeing a dirty farm could catch it. The CLI path is unaffected.
   - (a) Don't: do 1 and 2 first and re-measure; I expect ~200-250 ms with a cold-cache save, likely fine.
   - (b) Deferred commit, flushed on quit and before any `W`/`U`.
   Lean: (a), and only do (b) if the measurement after 1 and 2 still feels slow.

Question: OK to do 1 and 2 now, with 3(a)? Should yaks-7058 be done first, as it carries most of the weight?

---
▸ 2026-11-04T22:25:22Z [Joel Webber]
kind: answer
needs: agent
---

1, 2: Yes, absolutely. We have a place for caches in ~/.local for exactly this sort of thing.
3. I'm reading this as optimistic concurrency in the TUI. Let's file a yak and defer for re-measurement.

---
▸ 2026-11-04T22:31:41Z [perf-907a]

Decision (Joel, 2026-11-04): do yaks-7058, yaks-bcd4 and yaks-1ebf, with a cache for the git discovery. The deferred commit is filed as its own yak and waits for a re-measure after those three.

---
▸ 2026-11-04T22:31:41Z [perf-907a]
kind: pickup
---

picked up

Decision recorded; checking open questions before starting

---
▸ 2026-11-04T23:24:32Z [perf-907a]

Status after yaks-7058 (dd61f317) and yaks-bcd4 (f62e889c), local `main`, installed with `cargo install --path .` (restart a running `yaks tui`).

Changing save in the TUI, time to first output, clone of this farm, real pty:

- before: 700-760 ms
- after: ~290-400 ms (typically ~330)

What is left of it is the per-change commit itself, ~280 ms: 13 git spawns at 25-35 ms each on this Mac, two of them `status` (before and after the write, to find what the command changed). That is the floor for building the commit with git plumbing; going lower means the commit moves off the critical path (yaks-d262, which you deferred pending this re-measure) or fewer spawns (for example one `status` and `git commit-tree` fed from it).

yaks-1ebf no longer matters (its reload is 44 ms, after the saved frame): recommending it is shorn as not needed.

---
▸ 2026-11-04T23:24:32Z [perf-907a]
kind: ask
needs: human
---

asked: needs human

Re-measured: a changing save is ~330 ms now (was ~730 ms); ~280 ms of that is the per-change commit's 13 git spawns, which a TUI cannot shorten without a different mechanism. Two questions:

1. yaks-1ebf (second reload after our own save): it is now 44 ms and happens after the saved frame is drawn, so it no longer causes the pause. Lean: shear it as not needed rather than build the own-write skip. OK?
2. Is ~330 ms good enough to shear this yak, or do you want the last step? Options: (a) shear yaks-907a now and leave yaks-d262 (commit after the saved frame) open; (b) do yaks-d262 now; (c) try to cut the commit to fewer spawns first (drop the second `status` by diffing the paths the command wrote against the snapshot; lean: only if you still feel it, since it would save ~60 ms). Lean: (a), after you try the installed build and say whether it still feels slow.

---
▸ 2026-11-05T04:36:31Z [sched-1]
kind: answer
needs: agent
---

1. SGTM
2. Let's land what we have, then I'll test manually before doing yaks-d262.

---
▸ 2026-11-05T05:33:23Z [perf-907a]

Decision (Joel, 2026-11-04, in the thread): close yaks-1ebf as not needed; close this yak with the changes on local `main` and keep yaks-d262 open; Joel will try the installed build for a while.

Code: yaks-7058 (`dd61f317`) and yaks-bcd4 (`f62e889c`), on local `main`, not pushed. A changing save in the TUI went from ~700-760 ms to ~330 ms to first output; ~280 ms of that is the per-change commit's 13 git spawns.

Follow-ups: yaks-d262 (commit after the saved frame is drawn; reopen the question if the build still feels slow), yaks-717b (a test that fails on a rerun).
