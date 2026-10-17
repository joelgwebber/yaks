---
id: yaks-df02
title: Tighten skill guidance around completion notes & evidence
status: hairy
type: task
priority: 1
created: '2026-10-17T13:57:13Z'
updated: '2026-10-17T14:24:08Z'
labels:
- skills
---

It seems to be a pretty mixed bag as to whether we get evidence in agent comments and completion messages. Sometimes we get nothing (just "shorn"), and other times we get War and Peace. Sometimes we get attached evidence, other times not even when it would have been trivial.

Some general principles (leave a little wiggle room for the agent to ignore when it's nonsensical):
- Explain the basic shape of the fix, unless it's truly trivial.
- Attach evidence wherever it makes sense:
  - Web UI - screenshots. Use `sightmap browser` when available; it gives good tools for introspecting the DOM and component tree so that you can get focused screenshots when needed.
  - TUI - text output at least. If using a TUI library that supports it, screenshots.
  - Code - link to code when possible; small snippets if appropriate (see also: yaks-da5e).
  - Logs - focused summaries of log output when it's helpful.
- Keep completion/evidence comments reasonably terse. Design docs belong in attachments, not comments.

Also relevant: yaks-699b
