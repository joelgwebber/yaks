---
id: yaks-fe4f
title: Add :q and :wq affordances for vim-friendliness
status: shorn
type: task
priority: 3
created: '2026-08-26T12:47:52Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-fc85
labels:
- ui
needs: agent
---

Seems like a lot for such a simple thing, but muscle-memory dies hard!
Perhaps we can leave a spot for general :command affordances as part of this, even if we don't add many.

---
▸ 2026-08-27T16:41:32Z

Implemented a minimal vim command line. In a multiline editor's Normal mode (comment M + form content rows), ':' opens a command line in the status bar (App.cmdline); Enter runs, Esc dismisses, Backspace edits. Commands: :w / :wq / :x commit (a modal editor saves+closes, so all three behave alike), :q cancels (respecting the 275f dirty-discard confirm), :q! force-cancels. Unknown verbs -> 'unknown command: :X' and the editor stays open. run_command's catch-all is the extension point for future : verbs. Scope: multiline surfaces only (single-line fields don't get ':'). Help updated. Tests: :wq commits, :q! discards, :w saves the form, unknown notifies. 158 bin green, 0 warnings.

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-fe4f` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-fe4f --drop-request`, which removes this request note, so no new note is left behind.
