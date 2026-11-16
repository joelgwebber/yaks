---
id: yaks-3da7
title: Directory sources silently skip dot-files and dot-directories
status: hairy
type: bug
priority: 2
created: '2026-11-07T15:59:10Z'
updated: '2026-11-07T15:59:10Z'
tags:
- cli
- doks
---

Found while ingesting sources in yaks-3610 (0.0.33).

`yaks source add <dir>` skips every entry whose name starts with `.` ([`src/sources.rs`](code:src/sources.rs), the directory walk, `name.starts_with('.')`), and says nothing. A repo snapshot of `donjaime/yass` (196 files in a `git archive`) saved as 168 files. The 28 missing include `.agents/`, `.claude/`, `.github/workflows/`, `.gitignore` and `.goreleaser.yaml`, and with them the six agent playbooks that are the core of that project (`kit/.agents/skills/...` was missing too, since `kit/.agents` is a dot-directory).

The source looked complete: the listing shows no count, `source.md` lists only what was kept, and I reported "196 files" in a note before I noticed.

## Fix

- Report what was skipped: `Saved sources/<id> (168 files; skipped 28 hidden)` and a line in `source.md`.
- Decide whether hidden files belong in: `.git` must stay out, but `.agents/`, `.github/` and `.gitignore` are often the point of a repo snapshot. Lean: keep them, skip only `.git`, `.DS_Store` and the existing SKIP names, and keep the 200-file and 5 MiB limits as the guard.

Related: yaks-535c (what a directory source stores).
