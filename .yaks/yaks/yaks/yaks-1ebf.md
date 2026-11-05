---
id: yaks-1ebf
title: TUI reloads twice per own save (apply_edit, then the watcher)
status: shaving
type: bug
priority: 3
created: '2026-11-04T21:35:11Z'
updated: '2026-11-05T05:33:23Z'
parent: yaks-907a
tags:
- tui
- perf
---

Found while pinning down yaks-907a. A save writes the yak file under `.yaks/`, which the TUI's own filesystem watcher (`setup_watcher` in `src/tui/runtime.rs`) reports as an external change, so the loop reloads again once the editor is closed, right after `apply_edit` already did.

Evidence, from a `git` call log of one comment save: two full runs of `ls-files`, `status`, then the `symbolic-ref` ... `rev-list` chip chain, back to back (~+0.97 s and ~+1.3 s after the save key), and a traced `watcher reload: 146-152 ms` after the save. Each reload is ~130-150 ms (see the chip yak, child of yaks-907a), so the pause is doubled on the screen only when the second one lands before the next key.

Fix to consider: ignore watcher events caused by our own write (compare the farm's change generation or the file's mtime/content against what `apply_edit` just loaded), or drop the reload in `apply_edit` and let the watcher do it. Not a design call yet: check first which of the two is the cheaper rule.

---
▸ 2026-11-04T21:35:34Z [perf-907a]

Correction to the description: in my traced run the save went through `needs_edit` (the `M` I scripted turned out to be a request note), not `apply_edit`, so no `apply_edit` trace printed; that path reloads on its own too. The two back-to-back reload chains in the git log and the one traced `watcher reload` still stand. Check which save paths reload themselves before choosing the fix.

---
▸ 2026-11-04T23:14:56Z [perf-907a]
kind: move
from: hairy
to: shaving
thread: ksQQstI5T-eCRACxAjKJdNo3oJLOAEQG4JIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-04T23:24:31Z [perf-907a]

Re-measured after yaks-7058 and yaks-bcd4 (local `main`, f62e889c; temporary trace in the watcher branch of `runtime.rs`, reverted). Real pty, `:wq` on a comment in a clone of this farm (880 yaks):

- The watcher's reload after our own save takes 42-44 ms (it was ~150 ms: it ran the whole chip chain).
- It runs after the frame that shows the saved text is drawn: first output after the save at ~290-400 ms, the second repaint ~250 ms later. So it no longer delays the save the user waits for; it is a ~44 ms stall on the next key at worst.

So this is no longer a cause of the pause. Skipping our own writes costs a rule that has to tell our write from an external one landing at the same moment, to save ~44 ms off the critical path. Recommend not doing it: shear as not needed unless the stall shows up on a bigger farm (the reload also re-reads every yak file, 13-15 ms here at 880 yaks; yaks-e61e is the farm-size scaling).

---
▸ 2026-11-05T05:33:23Z [perf-907a]

Decision (Joel, 2026-11-04): not needed. The reload after our own save is 44 ms and runs after the saved frame; see the measurement above.
