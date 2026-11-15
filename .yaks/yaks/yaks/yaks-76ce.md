---
id: yaks-76ce
title: 'Editor: search (n)ext doesn''t update location after edits'
status: hairy
type: bug
priority: 3
created: '2026-11-15T05:06:19Z'
updated: '2026-11-15T05:06:19Z'
tags:
- tui
- edtui
---

Repro:
- Search for something.
- Edit the text so that the next hit for the search target moves.
- Hit (n)ext.

The cursor will jump to the wrong location -- looks like it's where the next result *would* have been, before the edit.
