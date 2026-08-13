---
id: yak-c393.5
title: Divider break from a mismeasured glyph
type: bug
priority: 2
created: '2026-08-13T01:24:51Z'
updated: '2026-08-13T01:56:51Z'
---

A character (candidate: the U+23BF tool angle, or em-dash) mismeasures and breaks the vertical divider on some rows. Find the offending glyph and fix char_width / emission.

---
▸ 2026-08-13T01:56:51Z
Root cause: East Asian Ambiguous glyphs in agent text (em dash, curly quote) + the U+23BF tool angle render width-2 in avt but width-1 in my grid, cumulative overshoot walks into the divider. Fix: ASCII-sanitize all agent text (straight punctuation, dropped the angle -> plain '  $ cmd'), plus occlusion gives a gap. Verified: 0 non-ASCII in agent text; divider continuous at every board_x.
