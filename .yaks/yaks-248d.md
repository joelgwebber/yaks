---
id: yaks-248d
title: Skill guidance for agents to link into yaks
status: hairy
type: feature
priority: 3
created: '2026-10-12T14:05:15Z'
updated: '2026-10-19T04:30:38Z'
depends_on:
- yaks-b048
labels:
- skills
---

Agent harnesses that can render links in conversation, should be able to link to yaks. Eg, in Zed/Delta, the yak markdown files already render nicely, so opening them directly is a decent experience.

Hard part: Links will drift as yaks are shorn.

---
▸ 2026-10-19T04:29:29Z [delta:herd-pass]

Herd pass 2026-10-19: the 'hard part' is gone. Since schema 5 a yak's file is `.yaks/<id>.md` for its whole life (no status directories), and the farm worktree is at a stable absolute path (`yaks path <id>`), so a link written once keeps working. What is left is the guidance itself (fits in the skills rewrite, yaks-b048).
