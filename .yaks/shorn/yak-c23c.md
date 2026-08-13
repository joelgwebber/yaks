---
id: yak-c23c
title: 'Fix emoji width bug: VS16 folds onto continuation sentinel'
type: bug
priority: 2
created: '2026-08-13T00:45:51Z'
updated: '2026-08-13T00:46:28Z'
---

Zero-width combining chars (variation selector U+FE0F) fold onto c-1, which for a preceding double-width glyph is the _CONT continuation cell, corrupting it to '\x00️' so it leaks into output and detaches the selector from its base emoji. Fold onto the base glyph column instead.

---
▸ 2026-08-13T00:46:28Z
Fixed in castkit.Screen.put: track last_base (column of last real glyph) and fold zero-width combining marks (VS16) onto it, never onto c-1. For a preceding wide glyph c-1 is the _CONT sentinel; folding there corrupted it to '\x00️', which then failed the '== _CONT' skip test in render_frame and leaked a NUL + detached selector into the run — causing the emulator to size the base emoji as width-1 text and drift everything after it. Verified: scissors base cell now holds '✂️' (len 2), zero corrupted cells, tab row joins clean. Grid model was already aligned (labels all at col 54); this removes the emulator-side artifact. Regenerated docs/demo.cast.
