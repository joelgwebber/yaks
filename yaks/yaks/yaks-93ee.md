---
id: yaks-93ee
title: Browse and link into archive attachments (zip, tar.gz) read-only
status: shaving
type: idea
priority: 4
created: '2026-11-17T21:41:19Z'
updated: '2026-11-18T23:29:26Z'
tags:
- tui
- doks
needs: human
---

Filed at Joel's request from yaks-0229 (not for now). Browse inside an archive attachment (`.zip`, `.tar.gz`) read-only in the TUI's directory view, and link into one (`<id>/code.zip#path/in/archive`, or a `!/` form). That needs a decision on the link form, and the `zip`/`tar`/`flate2` crates. Low priority while `yaks attach <dir>` (the sibling yak) removes the reason to attach an archive.

#### 2026-11-17T21:42:06Z

*sources-triage*

Related: yaks-0229 (where it came from), yaks-a57c (attach a directory, which removes most of the need).

#### 2026-11-18T23:10:28Z

*delta:ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-18T23:29:26Z

*delta:ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA*

Design read, no code yet. Full write-up: [files, directories and archives](/doks/design/files-directories-archives.md).

- **Finding:** "what does this link name, and how is it shown?" is answered in about ten places (`links::parse`/`exists`/`peek`, `locate::resolve`, and in the TUI `farm_dir_of`, `target_dir`, `farm_path_is_dir`, `open_farm_doc`, `read_doc`, `peek_target`, `open_external`, two follow dispatches). Directories (yaks-a57c, yaks-3329) were added at each call site; archives added the same way would touch every one again.
- **Proposal:** an archive reads as a read-only directory. First a refactor with no visible change: one container lister (`src/entries.rs`) and `Place` gaining directories, so the TUI has one `open_place`. Then archives are one more container kind plus from-bytes viewer constructors, touching no opener.
- **Link form:** the path goes through the archive, `<id>/code.zip/src/main.rs#L12`. Rejected `code.zip!/...` and `code.zip#...`.
- **Crates:** `zip` 2.x (no default features, `deflate-flate2`), `tar`, `flate2` (already in the tree).
- **Bugs filed:** yaks-1d2b (a dok's directory link is `../` to doctor and peek), yaks-8218 (`walk_dir` follows symlinks).

#### 2026-11-18T23:29:26Z

*delta:ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA · kind: ask · needs: human*

asked: needs human

Three decisions before any code:

1. **Refactor first?** Do the no-visible-change seam (one lister, `Place` knows directories, one TUI `open_place`) as its own yak before archives. Lean: yes; it also fixes yaks-1d2b.
2. **Link form:** `<id>/code.zip/src/main.rs` (path goes through the archive). Lean: this one.
3. **Archives at all, now?** The yak is p4 and `attach <dir>` removed the reason. Lean: do step 1 now (it pays for itself), leave archives at p4 until a farm actually has one.
