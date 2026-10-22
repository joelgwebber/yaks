---
id: yaks-4f4d
title: Wider embedded font subset for svg rendering
status: hairy
type: task
priority: 4
created: '2026-10-22T22:12:46Z'
updated: '2026-10-22T22:12:53Z'
depends_on:
- yaks-4e9c
labels:
- images
- toque
---

Follow-up to yaks-4e9c. The svg spike embedded DejaVu Sans Mono (regular + bold) and Noto Color Emoji (COLRv1), each subset by `pyftsubset` to exactly the 38 non-ASCII codepoints in today's `docs/assets/*.svg`, about 10 KB, 10 KB and 14.5 KB. Any other codepoint renders blank, and toque draws arbitrary apps, so the subset must grow.

Decide and do:

- Which ranges: all of Box Drawing and Block Elements, Arrows, Geometric Shapes, Miscellaneous Symbols and Dingbats, Braille Patterns (TUIs draw charts with them), plus Powerline/Nerd Font glyphs if they are cheap (likely not in DejaVu: say so). Latin-1 and Latin Extended-A for accented names.
- Emoji: a subset covering common emoji (not all of Noto: the full COLRv1 file is 4.8 MB, the CBDT one 10.6 MB). Measure what "all emoji in Unicode 15 base, without skin-tone and ZWJ variants" costs.
- Measure the compiled-binary delta of each choice (the spike's numbers: +16.5 KB for the narrow set).
- A generating script checked into the repo (`pyftsubset` was available in pyenv here), so the subset is reproducible and its range list is reviewable, not a binary blob nobody can explain.
- Licenses stay beside the fonts (DejaVu: Bitstream Vera plus public-domain changes; Noto Color Emoji: OFL).
- A fallback story for an unsubsetted codepoint: blank is silent; consider drawing U+FFFD or a visible box so a missing glyph is noticeable in a frame.

Lives in toque with the rasterizer (see the toque yak under yaks-4e9c); yaks only consumes it.
