---
id: yaks-3f90
title: 'Commits phase 2: commit links in the TUI and a commit view in the pane'
status: shaving
type: feature
priority: 2
created: '2026-11-18T03:03:09Z'
updated: '2026-11-18T15:18:58Z'
parent: yaks-3515
depends_on:
- yaks-274c
tags:
- tui
- git
---

Phase 2 of yaks-3515; needs phase 1 (yaks-274c).

- **Note fields as links:** a note's `commit:` and `source:` values render as short-hash links in its header, with the full hash kept in the link.
- **Commit view:** Enter on a commit link opens a new pane view (beside the text, image and directory views) showing the subject, body, author, date and changed files with `+/-` counts. Enter on a file opens it in the code viewer at that commit. `O` takes the commit out: `g` the host URL, `p` the hash. History works.
- **Squashed worker commits:** a worker hash that is the `source:` of a landed note says "landed as <sha>" and links to it.
- **Missing commits:** a commit not in the local repo says so, and offers the host link and `yaks commits --fetch`.
- Keys go through `BINDINGS`. Update `docs/tui.md` and the `?` help, with a docshot if one fits.

#### 2026-11-18T14:40:32Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T14:41:25Z

*commits-tui*

Start by `commits-tui`: 2026-11-18T14:41:19Z (date -u). Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-3f90.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/h62preyey7a6/yaks (HEAD b0a6d579).

#### 2026-11-18T14:58:13Z

*commits-tui*

Plan (`commits-tui`):

- `Target::Commit { sha }` in `src/tui/detail.rs`, produced by `mdrender::link_target` (replacing the `Artifact` placeholder). A `Target::Url` stays a URL while rendering (no git call per frame); when it is *followed* it goes through `Space::read_url`, so a pasted host commit URL opens the commit view and a pinned blob URL opens the code viewer.
- Note headers: `commit:`/`source:` meta values (when they read as a hash through the `commit:` grammar) become 7-char `Target::Commit` links in the header row; the full sha is kept in the target.
- New `DocView::Commit(commitview::CommitView)` + `Ctx::Commit` (new `src/tui/commitview.rs`): rows built once at open (subject, body, author, date, files), wrapped at render; cursor starts on the first actionable row (landed-as, else first file). Keys reuse existing `Cmd`s through `BINDINGS`; `Enter` on a file = `open_code(path, None, Some(sha))`; `O` = take-out of the commit.
- "landed as": scan the farm's notes once at open for a `landed` note whose `source:` is this hash.
- Missing commit: a view with `missing_commit`'s reason, the host URL row, and the `yaks commits <yak> --fetch` line.
- History: `PaneAt::Doc` with path `commit:<sha>`, `read_doc` re-reads it.

[commit-view-real-farm](yaks-3f90/commit-view-real-farm.txt)

[commit-file-at-commit](yaks-3f90/commit-file-at-commit.txt)

[commit-takeout](yaks-3f90/commit-takeout.txt)

![tui-commit-view](yaks-3f90/tui-commit-view.svg)

![tui-commit-landed](yaks-3f90/tui-commit-landed.svg)

![tui-commit-missing](yaks-3f90/tui-commit-missing.svg)
