---
id: yaks-4a53
title: 'Frontmatter-aware merge driver for farm files: concurrent edits always merge into a VALID yak'
status: hairy
type: feature
priority: 2
created: '2026-10-18T19:35:20Z'
updated: '2026-10-18T19:35:20Z'
parent: yaks-73f4
labels:
- git
- cli
---

Needed once teammates push one farm branch (yaks-73f4). Observed: `merge=union` merges two concurrent notes on one yak without a conflict but leaves two `updated:` lines in the frontmatter (an invalid file). Revive yaks-74f5's design as `yaks merge-driver %O %A %B %P` registered via `.gitattributes` on the farm branch + `git config merge.yaks.driver` (yaks init sets it): union the note entries (dedupe by header, order by timestamp), max(`updated`), union labels/depends_on; a scalar both sides changed differently (status, priority, title, parent) never fails the merge: keep one value and record the other in a `conflict:` frontmatter field / an attributed note that `yaks doctor` flags and the TUI shows, so the farm is always valid and the human resolves at leisure (Joel's idea, cf. yaks-9f5c). A `yaks sync` (pull --rebase with the driver, push, retry) wraps it.
