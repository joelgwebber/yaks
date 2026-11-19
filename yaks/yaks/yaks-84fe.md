---
id: yaks-84fe
title: 'Refactor: one container lister and one resolver for files, directories and subjects (open_place, links::dest)'
status: shaving
type: task
priority: 2
created: '2026-11-19T01:32:15Z'
updated: '2026-11-19T02:33:13Z'
tags:
- links
- tui
- design
---

Step 1 of [files, directories and archives](/doks/design/files-directories-archives.md), agreed with Joel on yaks-93ee. No user-visible change except fixing yaks-1d2b. It unblocks archive browsing (yaks-93ee) and starring any subject (yaks-4381) without either touching the openers.

## Scope

- `src/entries.rs` (no TUI types): `Container::Dir`, `container_at(root, rel)`, `list`, `read`, holding the safety rules once (no `.`/`..`, no `.git`, no symlinks). The TUI directory view, `links::peek` and the attachments count list through it.
- Grammar: a farm path may end in `/` (a directory), as an attachment name already may (yaks-1d2b). `links::exists` and `peek` accept a directory.
- `locate::Place` gains `FarmDir`; `links::dest(&Link)` writes a canonical destination (the inverse of `parse` from the farm root; not `label`).
- TUI: one `App::open_place(&Place)` and `read_place` behind `follow_link`, `follow_attachment_link`, `open_external`, `open_farm_doc`, `read_doc` and `peek_target`. `farm_dir_of`, `target_dir`, `farm_path_is_dir` go. Each `DocView` reports its `Place`; history stores `dest(place)` and restores through `open_place`.

## Out of scope

Archives (yaks-93ee), star-anything (yaks-4381), `walk_dir` symlinks (yaks-8218).

## Evidence

The existing TUI and links tests unchanged and green (they are the behavior contract); new tests: `dest` round-trips `parse` for every link kind; history restores each kind of subject; `doctor`/`peek` on a dok sibling directory. A line count of the touched TUI files before and after. Headless frames of a directory link followed from a dok, and `o` back.

#### 2026-11-19T01:41:35Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-19T01:41:45Z

*herd-files*

Claim (yakherd `herd-files`): the coordinator does this one itself (shared design work everything else builds on). Running alongside: `symwalk-1` on yaks-8218, which touches only `sources::walk_dir`.

#### 2026-11-19T02:33:13Z

*herd-files*

Decision: an attachment link with an empty name, `<id>/`, is the yak's whole attachment directory, in the grammar (`links::parse`). The detail pane's `Attachments:` row has always linked `<id>/`, but the grammar read it as the farm path `<id>/`; the old TUI papered over that in `farm_dir_of`. Rejected: keeping a TUI-side special case (a second rule), or changing the row's link to the long `/<herd>/yaks/<id>/` form (the short form is what people write).

Decision: whether a link names a directory is decided once, in `locate::dir_of`, called by `Space::resolve` (giving `Place::FarmDir`), `links::exists` and `links::peek`. A farm path says it is a directory lexically with a trailing `/` (kept by `clean_farm`); an attachment or a farm path without the `/` is a directory when the disk says so, as before. Rejected: requiring the `/` everywhere (would break existing `<id>/proj` links and `/sources/<id>/files`-style paths written without it).

Decision: `doctor` now checks any farm link ending in `/` (unless it holds `{}`/`<>` placeholders), where before a slash-ended path was always `../`. Checked on this farm: `yaks doctor` output is identical before and after (8 lines). Rejected: running a slash-ended link through the extension heuristic for prose (a directory has no extension, so it would never be checked).

Decision: history's `PaneAt::Doc` keeps one `dest` string (`links::dest` of the view's link) instead of `path` + `rev`; restore is `read_place(place_of_dest(dest))`. History is in memory only, so there is no stored format to migrate. Rejected: storing a `Place` (it would need `Clone`/`PartialEq` plumbing and gives nothing a string does not; a string is also what stars will store, yaks-4381).
