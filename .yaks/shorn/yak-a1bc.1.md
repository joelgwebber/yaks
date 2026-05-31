---
id: yak-a1bc.1
title: Update SKILL.md frontmatter for Agent Skills spec
type: task
priority: 2
created: '2026-05-31T16:26:02Z'
updated: '2026-05-31T16:26:30Z'
---

Add 'name' and 'description' fields to skills/yak/SKILL.md and skills/yak-sync/SKILL.md frontmatter alongside existing 'activation' field. This makes the skills compatible with Zed (~/.agents/skills/), Codex plugins, and skills.sh without breaking Claude Code plugin behavior. Folder name must match the 'name' field (already correct: 'yak' and 'yak-sync').
