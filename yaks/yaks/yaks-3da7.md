---
id: yaks-3da7
title: Directory sources silently skip dot-files and dot-directories
status: shaving
type: bug
priority: 2
created: '2026-11-07T15:59:10Z'
updated: '2026-11-17T21:42:57Z'
parent: yaks-4cfe
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

#### 2026-11-17T21:42:25Z

*sources-triage · kind: move · from: hairy · to: shaving · thread: ksQQgIRwKupoSJu4WL5a-Y3Ss5LOAGwQr5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-17T21:42:57Z

*sources-triage*

Claim (yakherd `sources-triage`). Owner: worker `srccli-1`, who also owns yaks-4dd3, yaks-06aa and yaks-fb15. All four live in `src/sources.rs` and the `source` arms of `src/main.rs`, so they go to one worker as four commits.

- Scope: `sources::add_on`'s directory walk. Keep dot-files and dot-directories, skip only `.git`, `.DS_Store` and the existing skip names, and report what was skipped (the stdout line and one line in `source.md`'s body).
- Out of scope: the source layout (yaks-e83f is deciding a schema-10 format). Change nothing about `files/`, `extracted.md` or the `files:` list.
- Evidence: a test with a dot-dir in the tree that fails before and passes after; the gate; a real `source add` of a tree with `.github/` in a scratch farm.
