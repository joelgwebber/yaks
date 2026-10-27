---
id: yaks-00c4
title: Affordance for auto-creating a yak from an upstream issue
status: hairy
type: feature
priority: 3
created: '2026-08-27T16:30:57Z'
updated: '2026-10-27T01:10:50Z'
needs: agent
---

Both for CLI and in a TUI shortcut -- given an upstream issue URL, auto-create a yak from it.
As a starting point, I think it would need to just copy the title and (perhaps?) content to a yak, because we don't have (or want) an LLM in the tool.
We might consider a marker (tag, or text) signifying that it has just been copied, and could use tidying. Then the tool could signal to your agent that it should do the tidying when ready.

---
▸ 2026-10-26T23:07:03Z [delta-lead]

When this is built: create the yak, then queue `yaks request <id> --kind tidy` on it (kinds land in yaks-db07), so the user's agent tidies the imported text. No model calls in yaks (yaks-33c8).

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-00c4` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-00c4 --drop-request`, which removes this request note, so no new note is left behind.
