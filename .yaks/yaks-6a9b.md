---
id: yaks-6a9b
title: 'Idea: per-herd directories in a multi-herd farm (<herd>/<id>.md)'
status: hairy
type: idea
priority: 4
created: '2026-10-21T03:30:37Z'
updated: '2026-10-21T03:30:37Z'
labels:
- design
---

Raised in yaks-b32e's question list ("Do we separate herds at the top-level, even if just for clarity?") but never decided on its own: b32e's decision (keep the flat layout) was about scale, and herd dirs were only measured as part of the hashed `<herd>/3/a/8/` layout. Joel asked again 2026-10-19 before putting his multi-herd personal farm (~/src/fs/.yaks: fullstory, sightmap, sightkick, jev, potemkin) under source control.

Lean (delta:herd-pass): not now. Costs:
- Breaks the stable-path invariant: `rename-herd` and a cross-herd move (yaks-54e9) would MOVE files, so `git log --follow`, `yaks commits` (file-follow), editor tabs and any link to the file break; today they only rewrite text.
- The herd is already in the id; the directory would be a second copy of it, so a file in the wrong dir needs a rule (which wins?) and a doctor check.
- Schema bump + migration, and every path-reading site changes (store, sheds farm-diff, preflight's path->id, status/commit messages, attachments).
- What it would buy is mostly available already: `ls .yaks/jev-*`, `git log -- '.yaks/jev-*'`, `yaks list --herd jev`; extracting a herd is `yaks merge` from a filtered copy.

Revisit if: one farm routinely holds herds owned by different people (per-dir CODEOWNERS / sparse checkout would then earn it), or a herd needs to be split out often.
