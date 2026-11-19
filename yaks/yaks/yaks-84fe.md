---
id: yaks-84fe
title: 'Refactor: one container lister and one resolver for files, directories and subjects (open_place, links::dest)'
status: hairy
type: task
priority: 2
created: '2026-11-19T01:32:15Z'
updated: '2026-11-19T01:32:36Z'
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
