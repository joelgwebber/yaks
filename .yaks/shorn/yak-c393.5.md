---
id: yak-c393.5
title: Divider break from a mismeasured glyph
type: bug
priority: 2
created: '2026-08-13T01:24:51Z'
updated: '2026-08-15T03:27:27Z'
---

A character (candidate: the U+23BF tool angle, or em-dash) mismeasures and breaks the vertical divider on some rows. Find the offending glyph and fix char_width / emission.

---
▸ 2026-08-13T01:56:51Z
Root cause: East Asian Ambiguous glyphs in agent text (em dash, curly quote) + the U+23BF tool angle render width-2 in avt but width-1 in my grid, cumulative overshoot walks into the divider. Fix: ASCII-sanitize all agent text (straight punctuation, dropped the angle -> plain '  $ cmd'), plus occlusion gives a gap. Verified: 0 non-ASCII in agent text; divider continuous at every board_x.

---
▸ 2026-08-13T03:46:03Z
Round 2 (emoji): the remaining glyph mismeasurement was char_width force-widening the whole 0x2600-0x27BF dingbat range, so scissors U+2702 was width-2 in our grid but width-1 in avt (its VS16 is width-0). Rewrote char_width to match avt's unicode-width rules exactly (VS16/ZWJ/combining=0, East Asian W/F=2, else 1 incl. narrow dingbats + Ambiguous). Added emoji_slot(): pads a status emoji to a fixed 2-col slot (scissors gets a trailing space, matching the app's double-space) so no emoji drifts next to a divider. Applied to tabs, list ghost badge, and detail parent/children/deps. Verified 0 wide-emoji-without-continuation across all frames. NOTE: bison/sheep are East Asian W (width 2) in a modern Unicode DB and in our grid; if a very old avt build still renders them width-1, we'll slot-pad those too — worth a quick in-browser check.

---
▸ 2026-08-13T04:02:56Z
Round 3 (bison): verified via raw emitted bytes that the inner separator CHA is identical on every row (ESC[41G) - my layout is correct, so the offset is avt mis-advancing on the emoji glyph itself (bison U+1F9AC / razor U+1FA92 are newer Unicode 12-13 emoji whose width avt's table may disagree with). It only breaks visibly where an emoji sits directly against a separator = the list ghost badge. Fix: use the app's single-width STATUS_CHAR (format.py, built for tight layouts) for the list badge; keep emoji in tabs + detail (not separator-adjacent).

---
▸ 2026-08-14T03:14:04Z
Resolved for now: went ALL-ASCII for status glyphs (dropped emoji from tabs + detail; list badge already ASCII). Definitive diagnosis via library teardown: asciinema-player renders lines as flowing DOM (createElement/insertBefore, ch units - no canvas), and the terminal (avt) is a Rust->WASM blob with an inlined width table we can't edit. Our layout is provably correct (separator CHA identical per row); the offset is color-emoji glyphs not advancing exactly 2 monospace cells in the DOM renderer - unfixable from the cast. Filed yak-fa3b (replace the player) and yak-f64b (shared rendering abstraction).
