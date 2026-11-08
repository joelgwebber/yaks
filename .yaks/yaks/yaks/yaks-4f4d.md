---
id: yaks-4f4d
title: Wider embedded font subset for svg rendering
status: shaving
type: task
priority: 4
created: '2026-10-22T22:12:46Z'
updated: '2026-11-08T00:53:00Z'
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
