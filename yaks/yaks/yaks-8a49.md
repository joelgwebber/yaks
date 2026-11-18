---
id: yaks-8a49
title: 'check: leftover .yaks/.activity/ after the schema 9 move gets a git mv that cannot work'
status: hairy
type: bug
priority: 3
created: '2026-11-18T14:42:38Z'
updated: '2026-11-18T14:42:38Z'
tags:
- cli
- git
source: 'feedback-to-yaks:cowpaths/joel#2026-11-16: the schema 9 move left `.yaks/.activity/` behind, and `check` asks for a `git mv` that cannot work'
---

From cowpaths/joel's feedback dok (2026-11-16 entry): after the schema 9 move, `yaks check` lists `.yaks/.activity/*.log` and `.yaks/.activity/.gitignore` as "should be at `.activity/...`", with advice to `git mv`. They are untracked (that `.gitignore` is `*`), so `git mv` fails; and `.activity/` belongs to the activity log yaks-d3ad removed, so moving them is wrong too.

Cause: `stray_layout` in `src/check.rs` walks the filesystem under `.yaks/`, skipping only `README.md`, `schema`, `.lock` and `.gitignore` at the top, and suggests a move for everything else, tracked or not.

Fix: consider only files git tracks (or skip ignored ones), and name `.activity/` as obsolete: safe to delete. The schema 9 migration could delete it in the same commit.
