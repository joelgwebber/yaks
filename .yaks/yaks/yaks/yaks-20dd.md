---
id: yaks-20dd
title: 'Triage: the starred tag-completion and TUI yaks against the link work'
status: shorn
type: task
priority: 3
created: '2026-11-11T21:53:57Z'
updated: '2026-11-11T22:39:29Z'
parent: yaks-1151
tags:
- tui
---

Part of yaks-1151. The starred yaks yaks-bf1b, yaks-1a16, yaks-b2ed, yaks-df01, yaks-8874, yaks-640a: which are independent and ready for a worker, which overlap the link work (yak-id completion inserts links), which are already done or superseded.

---
▸ 2026-11-11T22:08:04Z [triage-1]

Triage (triage-1), against main e4d0ff5a.

| yak | vs code | size | worker-ready? | blocker / question |
|---|---|---|---|---|
| b2ed | done: yak/dok completion (yaks-5656), tag completion (yaks-8d6a) | – | shear | (a) was approved; label completion already shipped |
| 640a | superseded: `schemas/yak.md` `tags:` (yaks-f04d, shorn) | – | shear | per-herd tags is the only leftover (schemas::load is farm-global, src/schemas.rs:168); file it only if wanted |
| 0e15 | shorn, fixed (links.rs:140) | – | – | – |
| 1a16 | accurate | S | yes (bundle with bf1b) | – |
| bf1b | accurate: only the form's tags field completes (handlers.rs:452, docs/tui.md:796) | S–M | yes | – |
| df01 | accurate (actions.rs:642) | S | yes | – |
| e853 | accurate: link jump only scrolls vertically (linenav.rs:143-170) | S–M | yes | – |
| 8874 | accurate (keymap.rs:671,737) | S | no | which key? |
| 5f70 | accurate (yaks-upkeep SKILL.md:53,67) | S | after a decision | citation form |
| 6cf2 | accurate (no code: check in src/check.rs) | M | after the 3 design points | see sketch |
| bede | wrong premise, plus a real bug | S | partly | layout belongs to yaks-1151 |
| c3d0 | accurate | M–L | no | design (below) |
| 9173 | other repo's farm | S | no | the human's call |

**1a16.** `finish_insert` (handlers.rs:2358) puts the tag at the cursor and adds nothing after it, so `tag_completion` (editor.rs:334) still sees a partial tag and Tab opens the picker again. Fix: when `fp.tags.is_some()`, add `", "` unless the next char is already a separator. Evidence: extend `tab_after_a_partial_tag_opens_the_picker_and_cancel_keeps_the_text` (tests.rs:8019) with pick, then Tab, to show the cursor moves to the next row.

**bf1b.** (1) `L` editor: in `editor_key` (handlers.rs:331), when `ed.action` is `EditAction::Labels`, Tab uses `tag_completion`. The `Overlay::Edit` branch of `finish_insert` already inserts generically. (2) Hint: render.rs:1580 skips single-line editors; add the Labels case. (3) Drawer: today Tab means next row (handlers.rs:1806). On row 3 (`d.tags`, drawer.rs:95), Tab on a partial tag opens the picker; `finish_insert` needs an `Overlay::Drawer` arm and then has to rerun `drawer_live_preview` (handlers.rs:1752). (4) Update docs/tui.md:619 and :796. Evidence: unit tests plus a toque frame of the picker over the drawer.

**df01.** `jump_block` sets the line and calls `scroll_line_into_view`. Change: block end = next start - 1. If the block fits in `rows_shown`, scroll so start through end is visible; otherwise scroll = start. Prev: if the cursor is past the current block's start, go to that start, else to the previous block's. Extend `ctrl_n_p_navigate_content_blocks_in_detail` (tests.rs:693) and add a scroll assertion. Only actions.rs and tests.

**e853.** In `LineNav::jump_link`, after `reveal`, if `lines[to.0].nowrap_key()` is set and the link column falls outside `[off, off+width)`, set `hscroll` (see `block_hscroll`/`scroll_block`, linenav.rs:313-333). This needs the width passed in from detail_nav.rs:174 and from the viewer caller. Unsure: whether `Jump.col` is in unscrolled columns. Check that first. Evidence: a snapshot of a wide table with Tab onto an off-screen link.

**8874.** Keys free on Shift in both the yak list and the yak pane: only `I` and `Y`. Lean: tags to `T` (the common edit), type to `Y` (Shift tier; the clash with `y` copy is mnemonic only). Alternative: tags on `I`. `keys:` overrides key by `yak.tags`, so they survive. Docs: tui.md:209/619/796.

**5f70.** Lean: link the date or the phrase the session settled. No `sessions:` frontmatter for now. yaks-85a6 (sessions open in the TUI) has shipped.

**6cf2.** Lean: unpinned links count from `verified:`, else from the dok's last farm commit. No threshold. With no checkout, skip and print one info line (as `peek` does). It reuses `links::code_base`/`git`.

**bede.** Sessions and sources live at the farm root (`sessions::DIR` src/sessions.rs:28, `SOURCES_DIR` src/sources.rs:26, reserved in store.rs:588), not under `doks/`. The only `/doks/sessions` link in the farm is in this yak. `/sessions/2026/11/…md` works (`yaks peek` reads it). Real bug: `/sources/<id>` names a directory, and both `peek` and `read_farm_doc` (follow.rs:97) say "no such file". Fix: resolve a `sources/<id>` directory to its `source.md`, in links.rs (read side) and follow.rs. GitHub resolution waits on yaks-7d13/9714.

**c3d0 audit (what happens today):**
- Pane Enter (`follow_link`, detail_nav.rs:387):
  - `code:` runs `open_source` (source.rs:162): `$VISUAL`/`$EDITOR` at the line, else `open`/`xdg-open`. A pinned link to a file that has changed since opens a read-only temp copy `$TMP/yaks-src/<sha>/<path>` (source.rs:131).
  - Attachment: the viewer if text or image, else OS open (handlers.rs:1193).
  - Dok, session or source: the viewer (follow.rs:225; pinned via `git show`, links.rs:673).
  - Yak or note: goes to it. URL: OS open.
- Pane `O` (handlers.rs:1615): same as Enter. A dok link goes to the viewer, not out.
- Viewer:
  - Enter follows the same way (handlers.rs:1444).
  - `O` OS-opens `v.path` (handlers.rs:1398). For a pinned dok that is today's file: `read_farm_doc` passes `root.join(path)` (follow.rs:95,112), a likely bug.
  - `y` copies the farm-relative path (handlers.rs:137). `E` edits in place and refuses pinned files.
- Proposal:
  - Enter on `code:` opens the viewer, read through `links::read_at(code_base, path, rev)` the way peek does. It uses a new `DocKind::Code` with syntax colour from the file extension (needs `Lang::from_path`; plain text has no colour today, handlers.rs:236). It lands on `#L` via `anchor_row` (follow.rs:50).
  - `O` becomes "take it out" everywhere: viewer, pane link, Doc row, and yak row (`O` is free in List/Doc and List/Yak). It opens a small modal menu: `e` editor at line, `o` system open, `p` copy absolute path, `r` copy farm/repo path, `g` host permalink. Being modal, it goes in EXTRAS.
  - Lowercase "looks" keys are unchanged; nothing mutates, so no Shift key is needed.
- Pinned "copy path" options:
  - (a) materialize the temp copy and copy that path;
  - (b) the worktree path when the file is unchanged since the pin, else (a). This is what open already does (source.rs:118). My lean.
  - (c) copy `path@sha` / `git show sha:./path`;
  - (d) the host permalink `blob/<sha>/path#L…`, the natural one for a pin. Needs a remote → web URL from yaks-7d13.

**Overlaps and ordering.**
- 1a16+bf1b share `finish_insert`/`open_ref_picker`/`tag_completion` and docs/tui.md: one worker.
- df01 (actions.rs) and e853 (linenav.rs, detail_nav.rs) are disjoint from each other and from that pair.
- 8874 and c3d0 both edit keymap.rs and docs/tui.md: land 8874 first.
- c3d0 and bede share follow.rs/links.rs with the 1151 layout work. Yak/dok link completion (`dok_link_tail` editor.rs:285, `/doks/` only) has to follow any change to link forms. Tag completion doesn't touch links.
- Parallel today: (1a16+bf1b), df01, e853, plus 6cf2/5f70 once answered.

---
▸ 2026-11-11T22:39:29Z [triage-1]
kind: move
from: hairy
to: shorn
thread: ksQQw3f_HRXkT-eF6oV8kx3Nn5LOAFi59ZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shorn
