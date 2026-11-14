---
id: yaks-c3d0
title: Display links to source files directly in the TUI
status: shaving
type: feature
priority: 3
created: '2026-11-11T21:09:28Z'
updated: '2026-11-14T22:59:05Z'
depends_on:
- yaks-f505
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

---
▸ 2026-11-14T12:49:40Z [sched-1]
kind: answer
needs: agent
---

1. Perfect.
2. Sounds like a good unification. I dig it.
3. I like it. I hadn't considered the possibility of just dropping a temp copy, but that seems useful. Note that there will also be occasional cases where we can't see the reference, because an agent on another machine decides to drop a local hash. Not an intended state, but one we should be resilient to.
4. :thumbs-up:

---
▸ 2026-11-14T17:51:11Z [linkplan-lead]
kind: pickup
---

picked up

Decision (Joel): all four as proposed: Enter on a `code:` link opens the read-only viewer; `O` is the take-it-out menu everywhere (`e` editor, `o` system open, `p` absolute path, `r` farm or repo path, `g` host link); `p` on a pinned `code:` file is the working-tree path when unchanged since the pin, else a read-only temp copy; no recognised host says so and names `yaks.host`. Added by Joel: resilience when a referenced commit is not visible locally (an agent on another machine dropped or never pushed a hash): that must give a clear message and a fallback, never an error or a hang.

---
▸ 2026-11-14T22:59:04Z [linkplan-lead]
kind: move
from: hairy
to: shaving
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-14T22:59:05Z [linkplan-lead]

Claim: worker `viewer-1` (Sonnet tier; the design is settled by Joel's answers above, so this is implementation, and review is the cost). Owns: the TUI follow dispatch (`src/tui/follow.rs`, `src/tui/source.rs`, `follow_link` in `src/tui/detail_nav.rs`, the three open/follow sites in `src/tui/handlers.rs`: `LinkOpenExternal`, `FileOpenExternal` and the link-follow arm), a new read-only code viewer kind, the `O` menu overlay and its `EXTRAS` help rows, the `LinkOpenExternal` / `FileOpenExternal` rows in `src/tui/keymap.rs`, `Space::content` / `Space::local_path` / `Space::web_url` call sites in `locate.rs` (use them; remove the `#[allow(dead_code)]` that waits on this yak), `docs/tui.md`, tests and snapshots. Out of scope: note parsing and `tui/content.rs` (`notefmt-1`), `sources.rs` and `locate::farm_file` (`srcslug-1`), `check.rs`, the resolver's behaviour, any stored link form, `migrate.rs`. Where a file overlaps (`detail_nav.rs`: `notefmt-1` edits note-anchor lines only; `locate.rs`: `srcslug-1` edits `farm_file` only) keep to your own functions so the three merge cleanly. Judge: the coordinator re-runs the evidence in a real toque session on its own build.
