---
id: yaks-84fe
title: 'Refactor: one container lister and one resolver for files, directories and subjects (open_place, links::dest)'
status: shorn
type: task
priority: 2
created: '2026-11-19T01:32:15Z'
updated: '2026-11-19T02:33:14Z'
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

#### 2026-11-19T02:33:13Z

*herd-files*

Done (herd-files). The shape, per [the design](/doks/design/files-directories-archives.md):

- **One lister**, `src/entries.rs` (`Container::Dir`, `container_at`, `list`, `count_files`, `listing`, `safe_rel`): the directory view, `yaks peek` of a directory and the `Attachments:`/`Files:` counts all list through it, so they agree (`.git` and symlinks left out, dot files kept, dirs first).
- **One resolver**: the grammar keeps a trailing `/` on a farm path and reads `<id>/` as all of a yak's attachments; `locate::dir_of` decides "is it a directory" once, giving `Place::FarmDir`; `links::dest` writes any link as a destination `parse` reads back.
- **One opener** in the TUI: `App::open_place` / `read_place` (`src/tui/follow.rs`) behind both follow dispatches, the directory view's Enter, history restore, `/`-URLs and `open_farm_doc`. Deleted: `farm_dir_of`, `target_dir`, `farm_path_is_dir`, `read_doc`, `open_external`, `resolve_attachment`, `follow_url`, and dirview's own lister and counter. History keeps a subject by its destination.

Evidence:

- Gate: `cargo test -p yaks` 1507 passed, 0 failed; `cargo fmt --check` clean. The existing TUI and links tests are unchanged except for renamed entry points.
- New tests: `dest_round_trips_through_parse` (every link kind, from the root and from a yak), `a_farm_path_may_name_a_directory`, `exists_and_peek_accept_a_farm_directory`, `every_kind_of_document_round_trips_through_its_destination` (dok, directory, attachment with a space, file in an attached dir, image), `entries` unit tests; the dirview climb-out test now asks the resolver.
- Real CLI (release build) on the yaks-1d2b repro farm: `yaks doctor` "All clear" (before: two `../` errors); `yaks peek /doks/x/sketches/` lists `sub/`, `a.txt` (the symlink to `/etc` left out).
- Real TUI (headless, release build), attached: [the dok's directory link followed and `o` back](yaks-84fe/dok-directory-link-and-back.txt), [a yak's Attachments row opening its tree](yaks-84fe/yak-attachments-row.txt).
- Line counts (TUI files touched): 296 added, 453 removed; `dirview.rs` 992 to 823, `handlers.rs` 3251 to 3185, `follow.rs` 999 to 1135 (it now holds the one opener).
- Docs: `docs/cli.md` (link table: farm directory row, `<id>/`; `peek` of a directory), `docs/tui.md` (the directory-link example was wrong for a dok's own sibling directory: now `x/sketches/`), `.agents/skills/yaks` Links. No CLI flag or TUI key changed.

Fixes yaks-1d2b.

#### 2026-11-19T02:33:14Z

*herd-files · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn
