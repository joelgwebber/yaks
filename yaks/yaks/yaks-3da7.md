---
id: yaks-3da7
title: Directory sources silently skip dot-files and dot-directories
status: shaving
type: bug
priority: 2
created: '2026-11-07T15:59:10Z'
updated: '2026-11-17T21:56:24Z'
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

#### 2026-11-17T21:44:20Z

*srccli-1*

Start: Tue Nov 17 21:44:19 UTC 2026. Farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-3da7.md. Checkout: /Users/joel/src/yaks/.delta/worktrees/erg296cj3wps/yaks

#### 2026-11-17T21:56:24Z

*srccli-1*

Decision: keep dot files and dot directories; skip only the names in `WALK_SKIP` (`.git`, `.DS_Store`, `node_modules`, `target`, `__pycache__`, `.venv`, `venv`, `dist`, `build`), at any depth. Rejected: honouring `.gitignore` (not predictable, and a snapshot is not a checkout), and keeping a blanket "skip all dot-names" with only a warning.

Decision: the walk is `pub(crate) fn sources::walk_dir(&Path) -> Result<(Vec<(String, Vec<u8>)>, Vec<String>)>` (files sorted, skipped sorted; a skipped directory is `rel/`, a file `rel`). Limits (200 files, 5 MiB, on what is kept) are checked inside it, so `attach <dir>` gets them for free. Skipped entries are not descended into, so a skipped directory counts once.

Decision: the report is a `sources::Report {dir_files, skipped}` returned by a new `add_reporting` (and `Farm::source_add_reporting`); `add` and `Farm::source_add` keep their signatures so `promote` and every other caller is untouched. Rejected: adding a field to `Saved`, which would break every `Saved::New(dir)` pattern in the tests.

Decision: stdout is `Saved <dir>/source.md (N files; skipped M: a, b, ...)`, five names then `and K more`, only for a directory that skipped something; `source.md` gets a `Skipped M: ...` paragraph in the body (twenty names). The frontmatter and `files:` are unchanged (yaks-e83f owns the layout). Same line on `Already saved:`/`Refreshed`.

Evidence, before: the installed 0.0.37 on a tree with `.github/ci.yml` and `a.md` printed `Saved sources/2026-11-17-596dd9c9-p/source.md` and `files/` held only `a.md`.

Evidence, after: my release build on a tree with `.github/workflows/ci.yml`, `a.md`, `.git/c`, `target/t`, `.DS_Store`:

    Saved sources/2026-11-17-b3ea1126-proj/source.md (2 files; skipped 3: .DS_Store, .git/, target/)
    files: .github/workflows/ci.yml, a.md
    source.md body ends: Skipped 3: `.DS_Store`, `.git/`, `target/`.

Tests: `a_directory_keeps_dot_files_and_says_what_it_skipped` and `walk_dir_returns_files_and_skipped_and_checks_the_limits` in `src/sources.rs` (they use the new API, so they cannot compile before the change; the old binary run above is the before). Docs: `docs/cli.md`, `.agents/skills/yaks-research`, `source add --help`.
