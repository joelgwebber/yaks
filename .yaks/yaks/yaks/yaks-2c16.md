---
id: yaks-2c16
title: CLI/MCP affordance to attach to a running yaks tui
status: hairy
type: idea
priority: 3
created: '2026-08-23T02:49:59Z'
updated: '2026-10-27T01:10:50Z'
labels:
- cli
needs: agent
---

It would be *really* useful for your agent to have a way to drive the yaks tui you have open. Eg, "(user) Find me all the hairy yaks with the 'ui' label", or "(agent) Let me show you the yak herd I just created so you can review", and so forth.
We'll need to think about the mechanics of this, as it's not entirely obvious how best to approach it. Eg:
- What if you have multiple TUIs running? Are they named somehow?
- Can the agent query them all to figure out which one to connect to?
- What happens if multiple agents try to connect to the same one?
- And so on...

As we work through the mechanics and use-cases, let's make a point to identify UI affordances that would make this more effective. Some rough ideas:
- Some affordance for saved searches, allowing the agent to show you a search without interrupting whatever you were working on.
- Ephemeral agent-specific UI state -- relating to saved searches, a way for an agent to push a new UI state on the stack, so the user can just ESC out of it, back to where they were.
- A way to "indicate" to the agent that you want it to look at a particular yak/herd/search/comment/... (we handle this for individual yaks with the ability to copy an id).

---
▸ 2026-10-05T22:28:42Z [Joel Webber]

This could also be an interesting path for Delta integration. Bonus points if we can get Delta to emit some kind of in-thread UI affordance pointing to a particular place in the UI.

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-2c16` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-2c16 --drop-request`, which removes this request note, so no new note is left behind.
