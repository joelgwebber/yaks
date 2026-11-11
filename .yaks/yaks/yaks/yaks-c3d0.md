---
id: yaks-c3d0
title: Display links to source files directly in the TUI
status: hairy
type: feature
priority: 3
created: '2026-11-11T21:09:28Z'
updated: '2026-11-11T21:46:11Z'
tags:
- tui
---

Currently selecting a source link immediately opens it in the default editor (or maybe the system open behavior?). This is useful, but it would be nice to at least see it (read-only) in the our existing markdown/text UI. I'm assuming we can mostly reuse what we already have, though it'll need to use whatever syntax highlighting we're embedding in the markdown renderers.

Once in that UI, a *second* affordance can open it with the system editor. Another would be useful for copying the absolute path of the file, so the user can open it however they like. I'm not quite sure how/if to handle that case for source files pinned to specific git-hashes. We'll want to think through that before committing fully to a design.

---
▸ 2026-11-11T21:10:17Z [sched-1]

Come to think of it, the copy absolute path and open in system editor affordances would be useful for yaks, doks, sessions, and sources as well. Hopefully there's a clean way to generalize this so we don't end up with a bunch of weird special cases.

---
▸ 2026-11-11T21:46:11Z [sched-1]

Related to the git-hash linking problem -- we should ensure that we can also link to Github, which should be easier to do for files "at the given hash". If that can be done locally as well, great, but if not at least we can fall back to the GHUI.
