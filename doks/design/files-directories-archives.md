---
type: topic
title: 'Files, directories and archives: one way to name, list and open them'
created: '2026-11-18T23:28:53Z'
description: How links, the TUI openers and listers should handle files, directories and archive members through one container and one resolver; read before adding a new kind of link target or file lister
tags:
- links
- tui
- design
---

# Files, directories and archives: one way to name, list and open them

Status: proposal, for [yaks-93ee](/yaks/yaks/yaks-93ee.md) (browse and link into archive attachments). Read this before adding a new kind of thing a link can name, or a new way to list files.

## The finding in one paragraph

The farm's files are now reached in three shapes: a single file (attachment, dok, source, code file), a directory (yaks-a57c's `yaks attach <dir>`, yaks-3329's directory view), and, wanted next, a member of an archive. Each shape was added at its call sites. The question "what does this link name, and how is it shown?" is now answered in about ten places, and they already disagree. Archives added the same way would touch every one of them again. This dok proposes one small seam first, so an archive becomes one more *kind of container* and not one more branch in every opener.

## Where the question is answered today

| Where | What it decides |
|---|---|
| `links::parse` / `clean` | the grammar. An attachment may name a directory (`<id>/proj/`); a farm path may not (`clean` refuses a trailing `/`) |
| `locate::Space::resolve` | meant to be the one resolver (the "seam" of [farm-layout-and-links](/doks/design/farm-layout-and-links.md)); knows nothing of directories |
| `links::exists`, `links::peek` | the CLI's `doctor` and `peek`; `peek` lists an attached directory with its own `read_dir` |
| `tui/dirview.rs`: `farm_dir_of`, `target_dir`, `farm_path_is_dir` | the TUI decides "is it a directory?" by asking the disk, working around the grammar |
| `tui/follow.rs`: `open_farm_doc`, `read_doc`, `peek_target` | each starts with its own directory branch |
| `tui/handlers.rs`: `open_external`, `follow_attachment_link`, `os_open` | directory branch again; archive guard twice |
| `tui/detail_nav.rs`: `follow_link` | a second follow dispatch beside the viewer's |
| `tui/detail.rs::image_slot`, `render.rs` | their own attachment lookups |
| `tui/dirview.rs`: `list`, `count_files` | the TUI's lister (skips `.git` and symlinks, dirs first) |
| `sources::walk_dir` | the write-side walk (skip list, limits; follows symlinks) |

Two disagreements this already causes (filed as bugs, see "Bugs found"):

- A dok's link to its sibling directory (`x/sketches/`, `/doks/x/sketches/`) opens in the TUI but `yaks doctor` calls it a `../` link and `yaks peek` refuses it.
- `peek` and the directory view list a directory by different rules (`peek` shows symlinks and `.git`, sorts differently).

## What archives need, if added naively

Following a link into `code.zip` would need an "is there an archive in this path?" branch in `farm_dir_of`, `target_dir`, `open_farm_doc`, `read_doc`, `open_external`, `peek_target`, `links::parse`, `links::exists`, `links::peek` and the take-out menu; a second lister; and viewers that can show bytes that are not a file on disk. That is the growth to avoid.

## Proposal

### 1. An archive reads as a read-only directory

One model for the reader: an archive attachment *is* a directory you cannot write to. The tree view shows `code.zip` with a fold marker beside `proj/`; unfolding it lists its members; `Enter` on a member opens it in the text or image viewer like any file. Nothing about it is new to learn.

### 2. One lister: a container

A small module (`src/entries.rs`, no TUI types) owns listing and reading:

```rust
enum Container { Dir(PathBuf), Zip(PathBuf), Tar { path: PathBuf, gz: bool } }
struct Entry { name: String, is_dir: bool, size: u64 }
fn container_at(root: &Path, rel: &str) -> Option<(Container, String /* inner path */)>;
fn list(c: &Container, inner: &str) -> Result<Vec<Entry>>;
fn read(c: &Container, inner: &str, max: u64) -> Result<Vec<u8>>;
```

- `container_at` is the one place that walks a farm path's segments and says "this is a directory", "this is `code.zip` and the rest is inside it", or "this is a file". It holds the safety rules once: no `.`/`..`, no `.git`, no symlinks, and for an archive no absolute or `..` member names and no symlink members.
- `DirView` lists through it, so it browses an archive with no other change. `links::peek` lists through it too, so `peek` and the TUI agree. `count_files` becomes a bounded `list` walk.
- The viewers (`AttachView`, `ImageView`) get a from-bytes constructor beside the from-file one. The file-backed path stays for editing; a member is read-only, as a pinned dok already is.

### 3. One resolver: `Place` knows directories and members

`locate::Space::resolve` (already the designated seam) answers through `container_at`: `Place::FarmFile`/`Attachment` stays for a file, and a new `Place::FarmDir { rel }` and `Place::Member { archive, inner, anchor }` cover the rest. Then the TUI's openers collapse into one `App::open_place(&Place)` (and `read_place` for history): `follow_link`, `follow_attachment_link`, `open_external`, `open_farm_doc`, `read_doc` and `peek_target` each lose their directory branch, and `farm_dir_of`, `target_dir` and `farm_path_is_dir` go away. That is the "one `follow(Place)`" the farm-layout dok already planned and nobody finished.

The grammar change is the same one the trailing-slash bug needs anyway: a farm path may end in `/` (a directory), as an attachment name already may.

### 4. The link form

Recommended: **the path just goes through the archive**, `<id>/code.zip/src/main.rs#L12`, and `<id>/code.zip/` for its top. Resolution finds the archive by walking segments (`container_at`), which is what the TUI already does for directories.

- For: no new syntax; `#L12` and headings keep working on the member; readers learn one rule (an archive behaves like a directory); and resolution reuses the segment walk the TUI already does for directories.
- Rejected: `code.zip!/src/main.rs` (the jar-URL convention). Lexical and explicit, but a second path syntax, and `!` is legal in file names.
- Rejected: `code.zip#src/main.rs`. `#` is already the anchor.
- Neither form works on a markdown host (GitHub shows the archive, not the member). That is the same for both, so it does not decide.

### 5. Scope and limits for archives

- Formats: `.zip`, `.tar`, `.tar.gz`/`.tgz`. `.7z .bz2 .xz .gz` (bare) stay as today: guarded, "copy its path".
- Crates: `zip` with default features off and `deflate-flate2` (zip 2.x adds `crc32fast`, `indexmap`, `memchr`, all already in the lockfile or tiny; zip 9.x pulls `getrandom`, `nix`, `zeroize`, `typed-path`, so pin 2.x unless there is a reason), `tar`, and `flate2` (already in the tree through `png`).
- A tar has no index: listing reads the whole stream. Cap the archive size that is browsed (say 50 MiB) and the entries listed (the view's existing 2000), and cache an archive's index per (path, mtime) for the life of the view.
- Reads are capped as files are (`VIEW_MAX_BYTES`), read through `take()`, so a compression bomb costs a bounded read.
- Take-out (`O`) on a member: `p`/`r` give the archive's path plus the member's (`code.zip/src/main.rs`); `e` and `o` extract the member to a read-only temp copy, the rule `locate::pinned_file` already uses for a pinned file. An archive inside an archive is still never system-opened.
- `doctor` checks a member link exists (reads the index). `yaks attach` is unchanged: attaching a directory is still what the skill recommends.

## The order

1. **Prep refactor (no user-visible change, except the bug fix):** `src/entries.rs` with `Container::Dir` only; `DirView`, `peek` and the count list through it; `Place` gains `FarmDir`; the grammar allows a farm path ending in `/`; the TUI openers go through one `open_place`. This fixes the sibling-directory link bug and deletes the TUI's disk-sniffing helpers. Measured by: fewer lines in `dirview.rs`/`follow.rs`/`handlers.rs`, and one place per question.
2. **Archives (yaks-93ee):** add `Container::Zip`/`Tar`, the from-bytes viewer constructors, `Place::Member`, take-out's temp copy, docs and help. With step 1 done this touches `entries.rs`, `locate.rs`, the viewers' constructors and the take-out menu, and no opener.

Doing 2 without 1 is possible, and is exactly the growth this dok is about.

## Bugs found while reading

- **A link to a dok's or source's directory is broken for the CLI.** `links::clean` refuses a trailing `/`, so `doctor` reports `[sketches](x/sketches/)` and `/doks/x/sketches/` as climbing out with `../`, and `peek` refuses them. The TUI opens them (`farm_dir_of` asks the disk). Only attachment names were fixed in yaks-a57c.
- **`walk_dir` follows symlinks.** `p.is_dir()`/`p.is_file()` follow links, so `yaks attach <dir>` (and `yaks source add <dir>`) with a symlink inside walks its target: a link to `/etc` failed with `Permission denied`; one to a readable directory would copy files from outside the tree into the farm. The TUI's lister already refuses symlinks; the walk should too, and report them as skipped.
