---
id: yak-d4f1
title: Switch task format from YAML to markdown with frontmatter
type: feature
priority: 1
created: '2026-02-20T12:00:00Z'
updated: '2026-02-21T02:34:48Z'
labels:
- format
- breaking
---

Replace .yaml task files with .md files using YAML frontmatter for metadata and
the markdown body as the description. All structured fields (id, title, type,
priority, depends_on, labels, created, updated) stay in the frontmatter. The
description field is removed from the frontmatter -- the file body IS the description.

## Scope

This is a wholesale conversion -- no dual-format support. The changes span both
yak.py (the canonical CLI) and yaks.nvim (the Neovim plugin).

### yak.py changes
- Read/write .md files instead of .yaml
- Parse frontmatter (between --- fences) as YAML for metadata
- Treat everything after the closing --- as the description
- When writing tasks, emit frontmatter + body (no description key in frontmatter)
- All file operations (create, update, shave, shorn, regrow) use .md extension
- list/show/next/tangled all read .md files

### yaks.nvim changes
- Update YAML parser to handle frontmatter format
- fs module: glob for *.md instead of *.yaml (but keep config.yaml as-is)
- Task reading: split file at --- fences, parse frontmatter as YAML, body as description
- Task writing: emit frontmatter + markdown body
- Detail view: description lines are already markdown-highlighted, no change needed there
- Edit mode: opens .md files directly, which is a much better editing experience

### Migration
- Add a startup check in both yak.py and yaks.nvim that detects .yaml task files
- Auto-convert: read the YAML, extract description, write as .md with frontmatter + body
- Delete the old .yaml file after successful conversion
- config.yaml is NOT converted (it's not a task file)
- Log/notify what was converted
