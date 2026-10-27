---
id: yaks-d695
title: Add pull-request(s) field to frontmatter
status: hairy
type: feature
priority: 3
created: '2026-08-26T12:58:19Z'
updated: '2026-10-27T01:10:50Z'
labels:
- schema
needs: agent
---

Alongside 'source' for upstream issues, we need a place to track active PR(s) as well.
I think this needs to handle multiple PRs, as that's a pretty common pattern.
Source can be singular, because we want to discourage a single yak from representing multiple issues (that's what herds are for).

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-d695` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-d695 --drop-request`, which removes this request note, so no new note is left behind.
