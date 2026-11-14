---
id: yaks-c3d0
title: Display links to source files directly in the TUI
status: hairy
type: feature
priority: 3
created: '2026-11-11T21:09:28Z'
updated: '2026-11-14T11:21:18Z'
depends_on:
- yaks-f505
tags:
- tui
needs: human
---

Currently selecting a source link immediately opens it in the default editor (or maybe the system open behavior?). This is useful, but it would be nice to at least see it (read-only) in the our existing markdown/text UI. I'm assuming we can mostly reuse what we already have, though it'll need to use whatever syntax highlighting we're embedding in the markdown renderers.

Once in that UI, a *second* affordance can open it with the system editor. Another would be useful for copying the absolute path of the file, so the user can open it however they like. I'm not quite sure how/if to handle that case for source files pinned to specific git-hashes. We'll want to think through that before committing fully to a design.

---
▸ 2026-11-11T21:10:17Z [sched-1]

Come to think of it, the copy absolute path and open in system editor affordances would be useful for yaks, doks, sessions, and sources as well. Hopefully there's a clean way to generalize this so we don't end up with a bunch of weird special cases.

---
▸ 2026-11-11T21:46:11Z [sched-1]

Related to the git-hash linking problem -- we should ensure that we can also link to Github, which should be easier to do for files "at the given hash". If that can be done locally as well, great, but if not at least we can fall back to the GHUI.

---
▸ 2026-11-14T11:19:34Z [linkplan-lead]
kind: ask
needs: human
---

asked: needs human

Design for viewing and taking a link out (plan: [/doks/design/farm-layout-and-links.md](/doks/design/farm-layout-and-links.md#the-seam-one-resolver-one-host-layer); it builds on the resolver yak under yaks-1151). Today Enter on a `code:` link opens `$EDITOR`, or the system opener when none is set; dok, session and source links open in the TUI viewer. `O` does the same as Enter on a pane link and system-opens the file in the viewer.

1. **Enter on a `code:` link opens it in the read-only viewer** (syntax-coloured by extension, landing on `#L` lines), like a dok. Lean: yes.
2. **`O` becomes "take it out", everywhere** (viewer, a pane link, a Doks row, a yak row), opening a small menu: `e` editor at the line, `o` system open, `p` absolute path, `r` farm or repo path, `g` host link (GitHub, GitLab, Gitea). Lean: yes; none of them changes a yak, so no Shift tier is needed. Today `O` on a pane link is a duplicate of Enter, so it loses nothing.
3. **"Copy absolute path" for a pinned `code:path@sha` file**, where no file at that pin exists locally:
   - (a) write the read-only temp copy and copy its path
   - (b) the working-tree path if the file is unchanged since the pin, else (a): the rule opening already uses (`src/tui/source.rs`)
   - (c) copy `path@sha`, or the `git show` command
   - (d) copy the host permalink `blob/<sha>/path#L..`

   Lean: (b). The `g` menu entry already gives the permalink, so `p` stays a local path.
4. **When the farm has no recognised host** (no remote, or a custom domain): `g` says why and points at `git config yaks.host`. Lean: that.
