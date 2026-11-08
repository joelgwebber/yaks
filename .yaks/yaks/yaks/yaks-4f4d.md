---
id: yaks-4f4d
title: Wider embedded font subset for svg rendering
status: shorn
type: task
priority: 4
created: '2026-10-22T22:12:46Z'
updated: '2026-11-08T01:19:39Z'
depends_on:
- yaks-4e9c
tags:
- images
- toque
- tui
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

---
▸ 2026-11-04T01:14:32Z [indent-links]

Findings from rasterizing a real docshot with the embedded fonts (`docs/assets/tui-image-caption.svg`, landed with yaks-0370), next to the same SVG in headless Chrome:

- **Layout matches Chrome**: text, column widths, the half-block picture and the captions all land in the same places.
- **A missing emoji is blank, not tofu**: the `📖` in the `Wiki (0)` tab is not in the 38-codepoint subset, so that tab shows no icon. Silent, as this yak already predicts. The fallback-glyph bullet below is the fix for it.
- **No italic**: the fonts are DejaVu Sans Mono regular and bold only. Our dim italic caption (and any `Modifier::ITALIC`, which includes block quotes and `*emphasis*`) draws upright. This is not in the scope above, so add it:
  - Embed an oblique face (DejaVu Sans Mono Oblique, same licence, same subsetting), or
  - say plainly that the rasterizer drops italic, so a frame is not trusted for it.
- **Box-drawing corners** still render blank (already in the list above; `docs/tui.md` names it as the known follow-up).

Why it matters beyond looks: yaks-8fd1 (a `yaks render` command) depends on this. Until the subset and a visible fallback land, a rendered frame has holes the browser version does not, so it cannot yet stand in for the Chrome recipe.

Not started; lives in toque with the rasterizer.

---
▸ 2026-11-08T00:49:16Z [herd-stars]
kind: move
from: hairy
to: shaving
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEpUNJIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: hairy -> shaving

---
▸ 2026-11-08T00:52:10Z [herd-stars]

Claim (yakherd herd-stars): worker `stars-fonts`, in the toque repo (attached by Joel 2026-11-08), with the `--scale` addition yaks-8fd1 needs.

**Scope (toque only).**

- `scripts/subset-fonts.sh` (or `.py`): downloads/locates the source fonts, runs `pyftsubset` over a reviewable range list, writes `assets/fonts/*-subset.ttf`. Source DejaVu is in texlive on this machine; Noto Color Emoji COLRv1 from googlefonts/noto-emoji.
- Ranges as the description lists (Latin-1, Latin Extended-A, General Punctuation, Arrows, Box Drawing, Block Elements, Geometric Shapes, Misc Symbols, Dingbats, Braille, plus what toque frames use today). Powerline/Nerd glyphs: say whether DejaVu has them.
- Add DejaVu Sans Mono Oblique (and Bold Oblique) so italic draws.
- Emoji: measure the full Unicode 15 base set (no skin tones, no ZWJ sequences) versus a common subset; pick, record numbers.
- A visible fallback: a codepoint no font has draws a visible box (or U+FFFD), not blank.
- `svg_to_png_scaled(svg, scale)` / `rasterize_svg_scaled` (keep the old functions).
- Measure the compiled-binary delta of each choice. Licenses stay beside the fonts. README updated.

**Evidence.** Tests in `src/raster.rs`: box-drawing corners draw, italic differs from upright, a missing glyph draws something, scale 2 doubles the size. Rasterized PNG of a yaks docshot attached (before/after). Gate `cargo test --all-features` in toque.

**Judge.** herd-stars at landing, which pushes toque `main` (Joel authorized landing, 2026-11-08) before yaks-8fd1 bumps the dependency.

---
▸ 2026-11-08T00:53:00Z [stars-fonts]

Started: 2026-11-08T00:52:52Z (date -u)

- farm file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-4f4d.md
- toque top-level: /Users/joel/src/toque/.delta/worktrees/ggfpzzt7sf82/toque
- toque HEAD: 240eb5f9fdc275d74504a7f6323ce04c2be2e581
- note: yaks commands only find the farm from the yaks worktree (toque has none).

---
▸ 2026-11-08T01:07:28Z [stars-fonts]

Progress (about 1h in): fonts, script, API and tests are working in the toque worktree (uncommitted); measuring binary deltas and writing docs next.

Findings so far:

- **Powerline/Nerd glyphs:** not in DejaVu Sans Mono (no U+E0A0-E0D4; the only PUA glyph is U+F6C5). No other font added; they draw as the `.notdef` box.
- **Braille:** also not in DejaVu Sans Mono (0/256), though the yak assumed it was. Borrowed from DejaVu Sans, shifted into the mono advance.
- **Missing glyph:** resvg draws the primary font's `.notdef`, so `--notdef-outline` in the subsetter is the whole fix (DejaVu's box). Tested with U+E0B0.
- **Gotcha (real bug found by looking at a frame):** when a run contains a char its face lacks, resvg switches to a fallback face and STAYS on it for the rest of the run, so a bold run with a `│` drew upright and light. DejaVu's bold and oblique mono lack Box Drawing and Misc Symbols. Fix in the script: every text face covers the same codepoints (missing ones copied from the regular face). Test fails without it (checked by building without the borrowing).
- **DejaVu 2.34, not 2.37:** from 2.35 the vertical box strokes are 17% shorter (-512..1921 vs -618..1992 per 2048), so at toque's 14px font / 18px row the frame borders became dashes. The texlive copy (what the first subset used) is 2.34; it is downloadable from SourceForge.
- Emoji subset sizes (Noto COLRv1, no layout tables): base Emoji_Presentation Unicode 15 without skin tones = 1200 emoji, 1.83 MB; common set ~250 emoji, 248 KB; emoticons alone (80) 69 KB.

![caption-before](yaks-4f4d/caption-before.png "BEFORE: tui-image-caption.svg rasterized with the 38-codepoint subset: blank icon on the Doks tab, upright captions")

![caption-after](yaks-4f4d/caption-after.png "AFTER: same SVG, new subset: the book icon draws on Doks, captions are italic")

![showcase-before](yaks-4f4d/showcase-before.png "BEFORE: showcase frame (box corners, italic, braille, arrows, latin, emoji, missing glyphs) with the old subset: mostly blank, no italic")

![showcase-after](yaks-4f4d/showcase-after.png "AFTER (3x via svg_to_png_scaled): box corners/borders, bold-italic, braille, arrows, accents, emoji; Powerline and CJK draw as boxes, not blank")

---
▸ 2026-11-08T01:18:32Z [stars-fonts]

Summary (toque commits f2100c5 fonts+script, 4fdb374 API+tests+README; parent 240eb5f; not pushed):

**What changed**

- `scripts/subset_fonts.py`: reviewable range list, pinned sources (DejaVu 2.34 from SourceForge, Noto COLRv1 at noto-emoji commit 99701e0, Unicode 15 emoji-data), runs the `pyftsubset` subsetter in-process. Needs only `pip install fonttools`; no texlive. Output is byte-reproducible (ran twice, same sha).
- Four DejaVu Sans Mono faces (regular, bold, oblique, bold oblique); resvg picks the oblique face for `font-style:italic` (toque's svg.rs writes it as a CSS class), asserted by a test.
- Missing glyph = DejaVu `.notdef` box (`--notdef-outline`). Test uses U+E0B0.
- `rasterize_svg_scaled` / `svg_to_png_scaled`, exported from lib.rs; the old two are scale 1 (identical output asserted). Bad scale (0, negative, NaN, inf) is `None`.
- README png section and `assets/fonts/README.md` (provenance, licenses; the one DejaVu license covers all four faces).

**Sizes (bytes)**

| set | fonts | yaks release binary |
| --- | --- | --- |
| before (38 codepoints) | 34,284 | 11,114,464 |
| now: 4 DejaVu faces + common emoji (252) | 882,212 (4 faces 633,812 + emoji 248,400) | 11,973,088 (+858,624, +7.7%) |
| alternative: base Unicode 15 emoji (1200) | 2,469,368 (emoji 1,835,556) | 13,574,768 (+2,460,304, +22%) |

Builds: `cargo build --release` in the yaks worktree with a temporary `[patch]` to a copy of toque, then `git checkout -- Cargo.toml Cargo.lock` (yaks tree is clean).

**Evidence**

- `cargo test` (13 + doc) and `cargo test --all-features` (21 + doc) pass; clippy `--all-features --all-targets` clean; `cargo fmt --check` clean.
- Four PNGs attached (caption before/after, showcase before/after). Looked at them: Doks icon, italic captions, box corners/borders, braille, accents, emoji, boxes for Powerline/CJK.
- Mutation check: building without the borrow step makes `a_wide_charset_keeps_bold_and_italic_after_symbols` fail.

**Known limits (follow-ups, not done)**

- Box-drawing verticals in the showcase show 1px gaps between rows in italic/oblique? No: borders are continuous with 2.34 at 14px/18px. Other unexplored gaps: no Greek/Cyrillic/Latin Ext-B (DejaVu has them; cost not measured), no combining marks (no mark positioning), no flags/skin tones/ZWJ (draw as parts).
- Oblique box/symbol glyphs are the upright ones (a terminal draws them upright too).
- DejaVu 2.34 comes from SourceForge: if that URL rots, set `TOQUE_DEJAVU_DIR`.

---
▸ 2026-11-08T01:18:33Z [stars-fonts]
kind: move
from: shaving
to: shorn
thread: ksQQcvZn6b-aQ7OxtwNwU3-8iJLOAEy6vZIBxBCA0aLJyrhJPreUvVwLftHG
---

moved: shaving -> shorn

---
▸ 2026-11-08T01:18:38Z [stars-fonts]

Correction to the summary's first "Known limits" bullet (garbled): the box-drawing borders are continuous with DejaVu 2.34 at 14px font / 18px row (verified in the attached showcase). The real gaps are: no Greek/Cyrillic/Latin Extended-B (DejaVu has them; cost not measured), no combining marks, and no flags/skin tones/ZWJ (they draw as their parts).

---
▸ 2026-11-08T01:19:39Z [herd-stars]

herd-stars: landed in toque as `f2100c5` + `4fdb374`, pushed to toque's `origin/main` (Joel authorized landing, 2026-11-08). Gate in toque: `cargo test --all-features` 21 passed, `cargo test` 13 passed. Looked at the showcase PNG: box corners, italic and bold italic, braille, Latin accents and emoji draw; Powerline and CJK draw as visible boxes. yaks picks it up with yaks-8fd1's `cargo update -p toque`.
