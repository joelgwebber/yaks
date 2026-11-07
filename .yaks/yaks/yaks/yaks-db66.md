---
id: yaks-db66
title: 'Images and files in doks and session notes: a sibling-directory convention'
status: shaving
type: feature
priority: 2
created: '2026-11-06T21:37:11Z'
updated: '2026-11-07T17:22:20Z'
parent: yaks-3f35
tags:
- doks
- design
---

From yaks-3f35, item 4. Yaks have attachments (`yaks attach`, stored in `<herd>/yaks/<id>/`); doks and session notes have nothing. The schism import put each image in a directory named after its document, beside it:

- `doks/screenshots/2026-03-08-composite-objects.md` embeds `2026-03-08-composite-objects/composite_objects.png`
- `sessions/2025/03/<note>.md` has `<note>/screenshot.png`

That works (a relative link going down resolves), but no skill describes it and no command knows it. The `yaks-research` skill only says to embed an image saved as a source (`/sources/<id>/shot.png`), which is wrong for a screenshot the farm itself made.

## To decide

Bless the sibling directory as the attachment directory of any farm document, mirroring yaks (`<id>.md` + `<id>/`), or send all images through sources. Lean: bless it.

If blessed:

- `yaks attach <dok-or-session-path> <file>...` stores into it and writes the link, with the same `--caption` / `--no-link`.
- `yaks mv` moves the directory with its dok, and `yaks doctor` checks links into it (check what `mv` does today).
- `yaks lfs` / `yaks weight` count it.
- The `yaks` skill ("Doks", "Links") and `yaks-research` say when to attach to a dok versus save a source: the farm's own artifact is an attachment; outside material is a source.

## Evidence

A test attaching to a dok and moving it; the skill and `docs/` text.

---
▸ 2026-11-06T21:37:49Z [schism-feedback]
kind: ask
needs: human
---

asked: needs human

Images and other files beside a dok or session note: bless the sibling directory (`doks/x.md` + `doks/x/`, as yaks have `<id>.md` + `<id>/`), with `yaks attach <dok-path>` writing into it and `yaks mv` moving it? Or route every image through `yaks source add`?

Lean: bless the sibling directory for the farm's own artifacts (screenshots it made, diagrams), and keep sources for outside material.

---
▸ 2026-11-06T21:48:01Z [sched-1]
kind: answer
needs: agent
---

Agreed, I think that's a very easy pattern to follow, and also easy to browse on the filesystem.

---
▸ 2026-11-07T17:04:28Z [herd-stars]
kind: pickup
---

picked up

herd-stars: blessed sibling directory; a worker builds attach/mv/doctor/weight for it, I write the skill text.

---
▸ 2026-11-07T17:04:44Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-07T17:05:35Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-dokattach` builds the commands; herd-stars writes the skill text after landing.

**Scope.** The sibling directory is a farm document's attachment directory (`doks/x.md` + `doks/x/`, `sessions/.../n.md` + `sessions/.../n/`), as Joel answered:

- `yaks attach <dok-or-session-path> <file>...` stores into it and appends the link to the document's body (a relative link going down, `x/file.png`), with `--caption` and `--no-link` as for yaks.
- `yaks mv` moves the directory with its dok (check what it does today), and links into it follow.
- `yaks doctor` reports a dead link into it; `yaks weight` and `yaks lfs` count it (check each; fix only what does not already hold).
- `docs/cli.md`, `src/main.rs` help for `attach` and `mv`.

**Out of scope.** The TUI; `rename-attachment` for doks (a follow-up if it is not trivial); skill files.

**Evidence.** Tests: attach to a dok writes the file and link; `mv` of that dok moves the directory and the link still resolves (`doctor` clean); same for a session note. Gate `cargo test -p yaks`.

**Judge.** herd-stars at landing.

---
▸ 2026-11-07T17:06:47Z [stars-dokattach]

Start: 2026-11-07 17:06:44 UTC (`date -u`).

- `yaks path yaks-db66`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-db66.md
- checkout: /Users/joel/src/yaks/.delta/worktrees/vqz90cnfqhr3/yaks

---
▸ 2026-11-07T17:12:15Z [stars-dokattach]

What already held (checked, not changed):

- `mv` today: moved only the `.md` file and rewrote links to it; the sibling directory was left behind, and a relative `x/shot.png` link in the moved dok was rewritten to the farm-root form `/doks/<old>/x/shot.png`, so it went dead. Fixed (below).
- `doctor` already reports a dead link into a dok's or session's sibling directory: `links::edges` reads dok and session bodies, `parse(.., From::Dok)` resolves `x/shot.png` to `doks/.../x/shot.png`, and `exists` stats it. Test added (session test deletes the file and expects the broken-link line).
- `weight`: `kind_of` counts anything under a path with a `doks` segment as `doks`, so a dok's directory already counts there; a session note's directory (like the notes themselves) falls under `other`. Counted, so left; asserted in the `kinds_follow_the_farm_layout` unit test. A `sessions` bucket would change the report's shape: follow-up if wanted.
- `lfs`: rules are `.yaks/**/*.<ext>` globs, which already cover any depth, so the sibling directories are routed. No change.

Not done: `rename-attachment` for doks (follow-up; it needs the same doc-path dispatch plus rewriting links in the doc and every doc that links it), and `mv` rewriting links held in session notes (`dok_move` scans yaks and doks only, as before).

---
▸ 2026-11-07T17:12:16Z [stars-dokattach]

Decision: a document path is told from a yak id by `id.contains('/') || id.ends_with(".md")` in `yaks attach` (yak ids are flat). Rejected: trying the yak id first and falling back (an unknown id would silently be searched as a path), or a new `--dok` flag (the task asks for the first argument to be the path).

Decision: the doc attach shares `store_attachments` (write files, build body) with the yak path in `src/farm.rs`; the dok/session variant `Farm::attach_doc_many` edits the file text directly (trim, append link, newline) instead of parse+render, so a session note or dok keeps its frontmatter byte for byte, and there is no `updated:` to bump.

Decision: `attach` of a `.md` file to a doc is refused. `dok_paths` and `sessions::paths` read every `.md` under a doks root or `sessions/` as a document, so an attached markdown file would turn into a dok. Rejected: allowing it (it would become a phantom dok with no frontmatter, flagged by `doctor`).

Decision: `--note` with a doc path is refused (a dok has no notes log). Rejected: silently ignoring it.

Decision: `mv` refuses when the dok's sibling directory also holds doks (`doks/a.md` + `doks/a/child.md`): moving the directory would drag them without fixing their links, and leaving it would split attachments from their dok. Rejected: moving only non-`.md` files (partial, surprising), moving everything.

Decision: link rewrite in `mv`: a link into the sibling directory stays relative-down in the moved dok (`<new-stem>/file`), and becomes `/<new-dir>/file` from anywhere else (the farm-root form `mv` already uses). Rejected: rewriting the moved dok's own sibling links to the absolute form (that is what `mv` did for other relative links, but it makes a dok with attachments not viewable standalone).

---
▸ 2026-11-07T17:22:08Z [stars-dokattach]
kind: verify
result: pass
---

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-11-07T17:22:20Z [stars-dokattach]

Scratch-farm run with `./target/debug/yaks` (YAKS_DIR = a temp farm), output as observed:

```
$ yaks attach doks/topics/a.md shot.png --caption "The shot"
attached shot.png to doks/topics/a.md (.yaks/doks/topics/a)
$ cat doks/topics/a.md        -> ... Intro.\n\n![shot](a/shot.png "The shot")
$ yaks doctor                 -> All clear: no farm-integrity issues found. (exit 0)
$ yaks mv doks/topics/a.md doks/design/b.md
Moved doks/topics/a.md -> doks/design/b.md
  doks/topics/a.md: 1 link
$ find . -name '*.png'        -> .../.yaks/doks/design/b/shot.png
$ cat doks/design/b.md        -> ![shot](b/shot.png "The shot")
$ yaks doctor                 -> All clear (exit 0)
$ rm doks/design/b/shot.png; yaks doctor
Broken link (a farm path or attachment that does not exist, or a ../ link):
  doks/design/b.md:L3: (b/shot.png) no such file in the farm   (exit 1)
$ yaks attach sessions/2026/11/scratch-2026-11-07-1212-shots.md shot.png
attached shot.png to sessions/2026/11/scratch-2026-11-07-1212-shots.md (.yaks/sessions/2026/11/scratch-2026-11-07-1212-shots)
  link appended: ![shot](scratch-2026-11-07-1212-shots/shot.png); doctor: All clear
$ yaks attach doks/none.md shot.png   -> no such dok or session note: doks/none.md (exit 1)
$ yaks attach <session> n.md          -> Error: n.md: a .md file beside a dok or session note would read as another document; ... (exit 1)
```

Tests (failed before the change: "no such task: doks/topics/a.md" and `mv` not refusing): `tests/links.rs::attach_to_a_dok_and_mv_it`, `a_dok_directory_holding_doks_is_not_an_attachment_directory`, `tests/sessions.rs::attach_to_a_session_note`, `src/weight.rs` `kinds_follow_the_farm_layout`.

Summary: `yaks attach` takes a dok or session path (`Farm::attach_doc_many`, sharing `store_attachments` with the yak path); `mv` moves the directory and rewrites links into it; docs/cli.md and clap help for `attach` and `mv` updated. Gate `yaks verify yaks-db66`: PASS.
