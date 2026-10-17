---
id: yaks-da5e
title: Allow markdown links to repo files / source
status: shorn
type: feature
priority: 2
created: '2026-10-17T14:13:34Z'
updated: '2026-10-17T18:28:57Z'
parent: yaks-faf5
labels:
- markdown
- tui
---

Existing markdown links can only reference files relative to the `.yaks` dir. This is fine for attachments, but it would be really convenient to be able to reference other files in the repo as well.

## Link format
Not entirely sure about link format -- propose something that fits reasonably cleanly into other markdown renderers, so it's likely to do something reasonable. Maybe as dumb as just `[code](../src/whatever.rs)`, which ought to "just work".

## Line ranges
In the case of source code, it would also be helpful to support link number/range references. Again, preferably in a way that won't break other markdown renderers. Eg, is it the norm to use `#fragments` for this, like `[code](../src/whatever.rs#L123:145)`?

## Commit-specific
Ooh, it would be _really_ cool if you could reference repo files by `git` hash as well, so a yak can maintain historical connection to the _actual_ code, before it's changed. Again, open to format ideas; this case matters less for other markdown renderers, though it would be nice if the path still hit the right spot in the filesystem.

---
▸ 2026-10-17T18:14:58Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T18:14:58Z [delta-lead]

Joel agreed the format (2026-10-17, in the Delta thread): plain relative links, GitHub-style `#L` line fragments, and the GitHub blob-URL shape for pinned commits. Showing a resolved git blob can be deferred to a follow-up yak; it may dovetail with yaks-699b (viewing files inside the TUI).

Claim (yakherd delta-lead): owner **srclinks-1**.

- **Format.** `[x](../src/foo.rs)`, `[x](../src/foo.rs#L12)`, `[x](../src/foo.rs#L12-L40)`. Paths are relative to the yak file (`.yaks/`), so they also work on GitHub. Pinned: `[x](../blob/<sha>/src/foo.rs#L12)`, where `../blob/<sha>/` is a GitHub-like prefix that yaks recognises (decide the exact recognised shape and record it as a `Decision:` note; it must stay a harmless relative link elsewhere).
- **Rendering (src/tui/mdrender.rs, `link_target` + `Tag::Link`).** A new `Target::Source { path, lines: Option<(u32, u32)>, rev: Option<String> }` in src/tui/detail.rs for links that resolve to a repo file (outside `.yaks/`, inside the repo); attachments stay `Artifact`. Show the link text; the status-bar destination shows `path:L12-L40` (and `@<sha7>` when pinned). Reuse yaks-3b8d's status-bar line.
- **Following (src/tui/detail_nav.rs, `follow_link` / `O`).** Open the file in `$VISUAL`/`$EDITOR` at the line when set (suspend and restore the TUI around it, like other external opens; check src/tui/runtime.rs for the terminal setup). Otherwise use the OS opener (`xdg-open`/`open`) as attachments do. For a pinned rev: if the file at that rev differs from the working tree (`git diff --quiet <rev> -- <path>`), write `git show <rev>:<path>` to a temp file and open that read-only; otherwise open the working copy. If this turns out hairy, defer the pinned part to a follow-up yak under yaks-da5e and say so.
- **Docs.** docs/tui.md, help_content, the yaks skill's "What the TUI renders" line (add a one-line example of a source link).

Out of scope: an in-TUI file viewer (yaks-699b).

Evidence: failing-first tests for parsing (path, both fragment forms, pinned prefix), Target classification, the status-bar text; a headless frame; a manual run with `EDITOR` set to a script that records its arguments. Judge: coordinator at landing.

---
▸ 2026-10-17T18:20:36Z [srclinks-1]

Start (srclinks-1): Sat Oct 17 06:15:24 PM UTC 2026

- path: /home/joel/src/yaks/.delta/worktrees/23m645m04mk9/yaks/.yaks/yaks-da5e.md; own binary built in this checkout, base a7e5453.
- Plan: `Target::Source { path, lines, rev }` (detail.rs); parse in mdrender.rs (`source_link`, pure, no disk); status-bar arm in render.rs `link_hint`; new src/tui/source.rs (plan + open, pinned rev via `git diff --quiet` / `git show` to a read-only temp file); editor launch = App.pending_editor drained by the event loop in runtime.rs (the loop owns the terminal) with a suspend/resume around it.

[frame-source-links](yaks-da5e/frame-source-links.txt)

---
▸ 2026-10-17T18:28:41Z [srclinks-1]

Decision: pinned prefix = `../blob/<sha>/<path>` with `<sha>` 7-40 hex digits (GitHub's blob-URL shape, relative). Anything else under `../blob/` (short or non-hex) stays an ordinary repo path `blob/...`. Rejected: `@sha` suffix or `?rev=` query (no GitHub equivalent, breaks as a filesystem path); any `../blob/<x>/` regardless of hex (would hijack a real `blob/` directory).

Decision: a source link is `../<path>` that stays inside the repo: `..` after the first is refused (`../../etc/x` is not a source link, stays an Artifact), `.`/empty segments are dropped, a trailing `/` (directory) is refused, `%XX` is decoded. Classification is pure string work in `source::parse_source` (no disk), after the URL check and BEFORE the attachment fallback; attachments (`<id>/<file>`) never start with `../`, so yaks-31e2 resolution is untouched (the attachment/paperclip test still passes).

Decision: a bad fragment (`#L`, `#L0`, `#L1-2`, `#L1:2`, `#intro`) still yields a Source link, just with `lines: None`. A reversed range is reordered. Rejected: treating the link as non-source (it still points at the right file). The issue's `#L123:145` guess is not accepted; GitHub's `#L123-L145` is.

Decision: editor launch is queued in `App.pending_editor` and run by the event loop (`runtime::run_suspended`: restore the terminal, wait, re-enter, `term.clear()`), because the loop owns the terminal; handlers stay pure. `setup`'s mode-entering code is split into `enter_modes` to be reused on resume. Editor line syntax by program name: `code/codium/cursor -g file:N`; `subl/zed/hx file:N`; everything else `+N file`; `$VISUAL` then `$EDITOR`, may carry args (`code -w`). Rejected: spawning the editor from the key handler (the TUI would draw under it). No editor set -> `xdg-open`/`open` on the file, no line.

Decision: pinned open = `git cat-file -e <rev>:<path>` (error if the commit lacks the file; needed because `git diff --quiet <rev> -- <untracked file>` exits 0), then `git diff --quiet <rev> -- <path>`: 0 and file present -> open the working copy; 1 -> `git show` into `<tmp>/yaks-src/<rev as written>/<path>`, made read-only. Done synchronously on Enter (one or two quick git calls). Deferred: nothing; no follow-up yak. Base dir for resolution is the parent of the farm root (`.yaks/`), as the links are relative to the yak file.

Evidence (failing first):
- With `link_target`'s call to `parse_source` disabled, `links_to_repo_files_are_source_targets` and `status_bar_shows_path_lines_and_pin_of_a_source_link` FAILED; re-enabled, pass.
- `a_pinned_link_opens_a_read_only_copy_only_when_the_file_changed` first FAILED (`unwrap_err()` on Ok for a file the commit lacked) before the `cat-file` check; passes now.
- Real pty run of `yaks tui` with `EDITOR=rec.sh` on a scratch git repo (target/srcscratch, src/foo.rs edited after the pin): Enter on the pinned link `[pinned](../blob/da07025/src/foo.rs#L2-L4)` made the real event loop run the script, which logged `+2 /tmp/.../yaks-src/da07025/src/foo.rs`. The headless driver does not reach the event loop (it only sets the notification: `edited src/foo.rs:L2-L4@da07025 (read-only copy)`, `edited src/foo.rs` for the unpinned, `src/nope.rs: no such file`).
- Status bar frames in the attached frame-source-links.txt (`→ src/foo.rs:L2-L4@da07025`); the same run showed `→ src/foo.rs`, `→ src/foo.rs:L2-L4`, `→ src/nope.rs:L1`.
- Docs parity: docs/tui.md (rendered-markdown paragraph), help_content Enter line (render.rs), skill "What the TUI renders" line.

---
▸ 2026-10-17T18:28:46Z [srclinks-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T18:28:57Z [srclinks-1]

Summary (srclinks-1): source links done, nothing deferred (pinned part included, so no follow-up yak). New `Target::Source`, `src/tui/source.rs` (parse, label, editor args, pinned-copy resolution, `App::open_source`), classification in `mdrender::link_target`, status-bar arm in `link_hint`, Enter/`O` in `follow_link`/`open_current_external`, editor suspend/resume in runtime.rs; `os_open` is now shared with `open_external`. Docs: docs/tui.md, help_content, yaks skill line. Gates: `cargo test --workspace` and `cargo test --release` both green (701 unit tests).

---
▸ 2026-10-17T18:28:57Z [srclinks-1]

moved: shaving -> shorn
