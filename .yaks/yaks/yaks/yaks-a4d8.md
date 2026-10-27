---
id: yaks-a4d8
title: Yaks changelog
status: hairy
type: feature
priority: 3
created: '2026-10-10T13:49:16Z'
updated: '2026-10-27T01:10:50Z'
needs: agent
---

Add tools for rolling up information about recent changes, for an agent to boil down into a changelog.
Specify the workflow in the yaks skills; make sure it's shed-aware.

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-a4d8` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-a4d8 --drop-request`, which removes this request note, so no new note is left behind.
