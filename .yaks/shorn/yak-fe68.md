---
id: yak-fe68
title: External task reference
type: feature
priority: 2
created: '2026-04-18T19:05:46Z'
updated: '2026-04-18T20:33:18Z'
commit: 1c40806
---

Use-case: I have an external issue tracker I'm forced to use (ahem... JIRA?), but don't actually want to. It's pretty easy to get claude to read them by external MCP request, and populate a yak, but then we lose the connection.

I'm thinking we could just add a single "external task" field, which can be understood from URL context, so that the calling agent can know how to deal with it (assuming it has the right tools). Then we can create a simple skill for synchronizing them semantically (i.e., without rigid schema mapping problems).

### 2026-04-18T20:27:23Z
Design decisions:
- Field name: 'source' (single string, URL format)
- Just a URL — agent infers system from domain (github.com, atlassian.net, linear.app, etc.)
- Single value, not a list
- Simple sync skill: if yak has source, agent can fetch context from external system via MCP; on shorn, can comment back
- TUI + web UI render as clickable link

Implementation plan:
1. Add 'source' to model.py (preserve on load/save, show in 'show' output)
2. Add --source flag to create/update commands
3. Render in TUI detail pane as clickable link
4. Render in web UI detail panel as link
5. Create a sync skill
6. Update SKILL.md / docs

### 2026-04-18T20:32:56Z
Implemented: source field in frontmatter, --source flag on create/update, rendered in TUI detail pane and web UI as clickable link. Updated SKILL.md, README, command docs.
