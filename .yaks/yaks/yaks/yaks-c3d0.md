---
id: yaks-c3d0
title: Display links to source files directly in the TUI
status: shaving
type: feature
priority: 3
created: '2026-11-11T21:09:28Z'
updated: '2026-11-15T00:18:30Z'
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

---
▸ 2026-11-14T22:59:43Z [viewer-1]

Start: Sat Nov 14 22:59:38 UTC 2026 (viewer-1).

- farm path: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-c3d0.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/5qj7ed84tg41/yaks

---
▸ 2026-11-14T23:36:07Z [viewer-1]

Checkpoint (viewer-1), `date -u` ~23:40. Honest state: design read, first code in; the viewer and menu are NOT written yet.

Done in the tree (uncommitted):

- `locate.rs`: `Space::link`, `content`, `local_path` are now `pub` with their `#[allow(dead_code)]` removed; new `Space::code_text` (whole file text for the viewer, with the missing-commit message), `Space::rel_path`, and `locate::missing_commit` (`git rev-parse --verify <rev>^{commit}`; "commit abc1234 is not in this checkout; fetch it or open on the host"). `pinned_file` now checks `missing_commit` first, so `p`/`e`/`o` degrade the same way.
- `follow.rs`: `read_farm_doc` and `read_doc` use `locate::farm_file_in(&root, path)` (the bare-id source request from the coordinator), plus test `a_bare_source_id_link_opens_the_slugged_directory`.
- Fast-forwarded my checkout to d3965452 (needed for `farm_file_in`); will rebase on current main before finishing.

Plan (next):

- `DocKind::Code { lang }` viewer: rows coloured with `syntax::Lang` (new `Lang::from_path`), landing on `#L`; `AttachView::code` constructor.
- `Overlay::TakeOut` modal menu (e/o/p/r/g), opened by one new `Cmd::TakeOut` (`O`) in Viewer, Detail pane (yak), List Doc rows and List Yak rows; replaces `LinkOpenExternal`/`FileOpenExternal` on `O` in the text viewer and the pane. Image viewer keeps `O` = open externally (it is the only thing it can do, and it already shares the name `file.open_external`).
- `EXTRAS` rows for the menu keys, docs/tui.md, hints, tests, frames.

---
▸ 2026-11-15T00:05:09Z [viewer-1]

Checkpoint 2 (viewer-1): implementation compiles and `cargo test -p yaks --bin yaks` was green (1192 passed) before the new tests; new `src/tui/takeout.rs` has 9 tests, all passing.

- Enter on `code:` opens `DocKind::Code` in the viewer (line gutter, `syntax::Lang::from_path`, `#L` lands, marked lines in accent gutter). `O` is `Cmd::TakeOut` (`take.out`) on Viewer, Image, Detail/Yak pane, List/Yak row, List/Doc row; menu is `Overlay::TakeOut` (e/o/p/r/g, Esc/q), rows in `EXTRAS`. Missing commit: `locate::missing_commit`, shown as the viewer's one row and as the notification for p/e/o.
- Left: docs/tui.md and README/skills grep, rendered frames (toque + PNG) attached, gate tail, rebase on current main, commit.

![tui-code-viewer](yaks-c3d0/tui-code-viewer.png "(a) Enter on [x](code:src/lib.rs#L4-L9): read-only viewer, line gutter, Rust colours, named lines in the accent gutter")

![tui-code-missing](yaks-c3d0/tui-code-missing.png "(c) pin to a commit not in the checkout: one line, no blank pane")

![tui-take-out](yaks-c3d0/tui-take-out.png "(d) O: the take-out menu over the viewer")

[frames](yaks-c3d0/frames.txt "text frames (a) (b) (c) from the real binary: yaks tui --headless on a scratch checkout with a GitHub origin")

---
▸ 2026-11-15T00:18:13Z [viewer-1]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-15T00:18:30Z [viewer-1]

Decision: `O` takes the one `take.out` command on every surface (viewer, image viewer, yak pane, Doks row, yak row); `LinkOpenExternal` and `FileOpenExternal` are removed, not kept beside it. The image viewer's `O` also opens the menu (its `o` is system open), rather than staying a direct open: one rule, one extra key.

Decision: the menu is `Overlay::TakeOut`, a bottom panel like the `?` popup, with a yellow status line; its keys are `EXTRAS` rows. Rejected: reusing `Overlay::Pick` (one status-line prompt, no room to say what each key does).

Decision: a code file is a new `DocKind::Code`, rendered by `code_rows` (not the markdown path): a line-number gutter as a no-wrap gutter (so `h`/`l` scroll the text and the numbers stay), colours from `syntax::highlight`, `Lang::from_path` (same table as fence info strings). Plain files get the gutter only.

Decision: an unreadable file or commit still opens the viewer, with one row saying why (so `o` history, `O` and `g` still work); `Err` only when there is no checkout. `locate::missing_commit` is the one check, used by the viewer and by `pinned_file` (p/e/o).

Decision: `e` with no `$VISUAL`/`$EDITOR` says so and points at `o` (it does not fall back to the system opener as the old Enter did).

Decision: `p` in the viewer on a pinned file acts on the file's own pin (unchanged: working path; changed: read-only temp copy), via `Space::local_path`; `g` on a pin links the commit, on an unpinned link the branch (`yaks url`'s rule, `web_url(place, false)`).

Decision: the menu title and the `e` line come from the viewer's cursor (selection range if any) for a code file, from the row's source line for a text/markdown file; a Doks/yak row has no line.

Summary. Enter on `code:` opens the file in the read-only viewer (line gutter, colours by extension, `#L` lines in an accent gutter and landed on, pins read with `git show`); `O` opens a take-out menu (`e` `o` `p` `r` `g`) from the viewer, the pane link, a Doks row, a yak row and the image viewer; a commit not in this checkout is one line in the viewer and a notification for `p`/`e`/`o`, and `g` still gives the URL (noting when `origin` lacks the commit); no host names `git config yaks.host`. Also per the coordinator: `follow.rs` `read_farm_doc`/`read_doc` use `locate::farm_file_in` (test `a_bare_source_id_link_opens_the_slugged_directory`). Docs: `docs/tui.md` (new "Code files and taking things out" section, the old `O` text), `?` overlay (`EXTRAS`, binding lines), hint lines, snapshots, three new docshots (`docshots_code`). Files: src/locate.rs, tui.rs, tui/{detail_nav,docshots,follow,handlers,imageview,keymap,palette,render,source,syntax,takeout,tests}.rs, snapshots, docs.

Evidence: unit tests in `tui/takeout.rs` (9: viewer + colours + landing, plain text, pinned, missing commit, menus on 4 surfaces, p/r/o/e, g with no remote / unknown domain / GitHub / pin / ghost, read-only E, o/i) and `syntax::tests::a_file_is_coloured_by_its_extension`. Real binary (`yaks tui --headless`, scratch checkout with a GitHub origin): (a), (b), (c) frames attached; (d) `r` = `copied src/links.rs`, `p` = `copied /tmp/viewer1-demo/src/links.rs` for the range link and the unchanged pin, `g` = `https://github.com/joelgwebber/yaks/blob/main/src/links.rs#L100-L120`, pin `g` = `.../blob/da1bdca71c070a...47de6b2/src/links.rs#L100`; ghost `p`/`e`/`o` = `commit deadbee is not in this checkout; fetch it or open on the host`, ghost `g` = `.../blob/deadbeefdeadbeef/src/links.rs#L100 (deadbee is not on origin as last fetched: ...)`. PNGs of the viewer, menu and missing-commit frames attached and looked at.

Not done / assumed: the menu's `e` was verified to queue the editor command (and with none set, the message), not by running a real editor; `o` launched the real opener once in the demo. The clipboard is stubbed under `cargo test`; in the demo the real `arboard` was used (`copied ...` shown). `r` on a pane link in a yak (not a file) says "this link names no file" only for non-file places; a Yak place gives the yak's file path. Rebased onto 7585597e with no conflicts; gate re-run after: 1439 passed, 0 failed, `cargo fmt --check` clean.
