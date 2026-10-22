---
id: yaks-89fc
title: Image size syntax and default fit in markdown
status: shorn
type: feature
priority: 3
created: '2026-10-22T20:35:29Z'
updated: '2026-10-22T21:03:12Z'
labels:
- tui
- images
- markdown
---

Follow-up to yaks-4305. Decide how a yak author says how big an image shows, and what the default is, once images render inline (and in the viewer). Research findings so far, so we pick a syntax that does not break other viewers:

**There is no CommonMark standard.** The spec has no image dimensions; the thread on talk.commonmark.org (2014) never settled one. What exists:

| Syntax | Where | Notes |
|---|---|---|
| `![alt](img.png){width=50%}` | pandoc `link_attributes`, kramdown, markdown-it-attrs, PHP Markdown Extra | The most widespread generic attribute form. Units: none means px, or `px`, `cm`, `in`, `%`. Renders as plain `![alt](img.png)` plus literal `{width=50%}` text in a viewer that lacks the extension (GitHub). |
| `![alt =300x200](img.png)` | markdown-it `img-size` "new syntax" | Valid CommonMark (size lives in the alt text), degrades to alt text containing `=300x200`. Either dimension optional, `%` allowed. |
| `![alt\|300x200](img.png)` | Obsidian | Same trick with a pipe. |
| `![alt](img.png =300x200)` | legacy markdown-it | Breaks image parsing on GitHub: avoid. |
| `<img src width>` | HTML | GitHub's only supported way; yaks bodies have no inline HTML, so not an option. |

**Lean:** the alt-text forms are the only ones that stay valid CommonMark and GitHub-safe (they only pollute the alt text). The `{width=50%}` attribute form is the most conventional but needs `pulldown-cmark`'s attribute extension turned on (check: it has `ENABLE_*` flags for heading attributes; confirm for images before assuming). Whichever we pick, `pulldown-cmark` gives us the image's alt text and destination, so parsing the size out of the alt text is cheap and needs no parser change.

**Units in a terminal:** px has no meaning without a cell size (we have it from `Picker::font_size()`); `%` of pane width is the natural unit; maybe also `cells`.

**Defaults to decide:**
- Fit-width, never enlarge (what the viewer does now, `Resize::Fit`)? Or natural size capped at pane width? Most viewers (GitHub) show natural size capped at the container width, i.e. exactly this.
- A height cap, so one tall screenshot cannot take a whole pane (inline only; the viewer uses the whole pane).
- Ways to ask for the other behaviours: natural size, fill width (scale up), an explicit column or row count.

Writes touch: `src/tui/mdrender.rs` (the image link row), `docs/tui.md`, the `yaks` skill's markdown notes (what authors should write), and the `yaks attach` link form only if it should emit a size by default (probably not). Sits beside the inline-image yak: that one needs the row count this one decides.

---
▸ 2026-10-22T20:47:07Z [delta:Add basic image attachment rendering]

moved: hairy -> shaving

---
▸ 2026-10-22T20:55:07Z [delta:Add basic image attachment rendering]

Decision (yakherd): syntax is a trailing size token in the image's alt text: `![alt =50%](p)`, `![alt =300x200](p)`, `![alt =x120](p)`, `![=50%](p)`. Findings behind it: CommonMark has no image sizes; `pulldown-cmark` 0.13 has NO image-attribute extension (its only `ENABLE_*attributes` flag is for headings), so the `{width=50%}` form would mean hand-parsing text after the image; the alt-text form is valid CommonMark and degrades to harmless alt text on GitHub. Rejected: `{width=..}` (hand parsing; literal text elsewhere), Obsidian `|WxH` (the pipe collides with table cells and needs escaping), `(p =WxH)` (breaks image parsing on GitHub), inline HTML (not supported in bodies).

Grammar: the token is the LAST whitespace-separated word of the alt, `=` then `W`, `xH` or `WxH`. `W` is digits with an optional `%` (percent of the pane's content width) or `px`; `H` is digits with an optional `px` (no `%` for height). Bare digits are pixels. Zero, or anything else, is not a size: the word stays in the alt text. The token never displays: the `🖼 alt` row shows the alt without it.

Decision (yakherd): default and explicit sizing, as one pure function. No size given: natural size (pixels over the cell size), shrunk to fit the pane width and a caller-supplied max row count, never enlarged (what GitHub does). Width given: that width (clamped to the pane), height from the aspect ratio, no row cap (an explicit size is the author's call). Height only: likewise. Both: fit INSIDE the WxH box keeping the aspect (not distorting). Pixels use `Picker::font_size()` so the same image can take different cell counts per terminal.

Owner: `size-1`. Scope: a new `src/tui/imagesize.rs` (parse + resolve, pure, unit tests first), `src/tui/mdrender.rs` (strip the token from the shown alt, carry the parsed size on `MdRow`, see below), `docs/tui.md`. NOT the viewer, `detail.rs`, `handlers.rs`, `render.rs`: the layout that uses the cell counts is yaks-0bf0, which lands after this.

API to build (names are mine; change them if you have a reason and say why in a Decision note):
- `pub struct ImageSize { pub width: Option<Dim>, pub height: Option<Dim> }`, `pub enum Dim { Px(u32), Percent(u32) }`.
- `pub fn split_alt(alt: &str) -> (&str, Option<ImageSize>)`.
- `pub fn cells(size: Option<ImageSize>, natural_px: (u32, u32), cell_px: (u16, u16), pane_cols: u16, max_rows: u16) -> (u16, u16)` returning at least (1, 1).
- `MdRow` gains `pub images: Vec<MdImage>` where `MdImage { target: String, alt: String, size: Option<ImageSize> }`, one per image whose marker sits on that row (an image wrapped across rows belongs to its first row). It is read only by yaks-0bf0 for now: put `#[allow(dead_code)]` on the field with a comment saying so (the compiler warns on a bin-crate field nobody reads).

Evidence: unit tests that fail first for `split_alt` (each form, `px`, `%`, zero, junk, a token not in last position, `=` inside normal alt text like `a = b`) and for `cells` (default shrink, never enlarge, height cap, width-only, height-only, both, pane clamp, 1x1 floor); an mdrender test that `🖼 shot` shows without the token and `MdRow.images` carries it; the existing attachment mdrender test unchanged. A docs/tui.md paragraph with the grammar and the defaults. Gate: `cargo test --workspace`.

---
▸ 2026-10-22T20:59:51Z [size-1]

Started: 2026-10-22T20:59:50Z; yak file: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-89fc.md

---
▸ 2026-10-22T20:59:51Z [size-1]

Decision: API kept as briefed (`Dim`, `ImageSize`, `split_alt`, `cells`, `MdRow.images: Vec<MdImage>`). Additions: `cells` and `MdImage` carry `#[allow(dead_code)]` (no non-test caller until yaks-0bf0).

---
▸ 2026-10-22T20:59:51Z [size-1]

Decision: `cells` rounds UP (ceil, with a 1e-6 epsilon so exact fits stay whole), so a fractional natural size never crops; rejected round-to-nearest (a 1.4-cell image would lose 40%). Percent width is of `pane_cols`. A `Percent` height is not producible by the grammar; built by hand it means percent of the natural height (documented on `Dim`), rather than panicking or being a separate type.

---
▸ 2026-10-22T20:59:51Z [size-1]

Decision: height-only is also clamped to pane width (scale = min(height scale, pane scale)), so a tall request on a wide image cannot overflow; rejected "height only is unclamped" since the claim says width clamps to pane and nothing can draw past the pane. Zero cell size or natural size reads as 1 (no divide by zero); pane_cols 0 reads as 1; max_rows 0 reads as 1.

---
▸ 2026-10-22T20:59:51Z [size-1]

Decision: the size token is cut out of the shown alt in `Renderer::end_image` (after the alt text is in `buf`, truncating it), and the `MdImage` is parked by link index and attached to the first row that emits a cell with that link (`emit`). Rejected: parsing the alt from pulldown events up front (alt can arrive as several Text/Code events with styling; reading buf is the one source of what is shown). `split_alt` only recognises ASCII digits, so full-width digits stay alt text.

---
▸ 2026-10-22T21:00:29Z [size-1]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-22T21:00:36Z [size-1]

Summary: added src/tui/imagesize.rs (split_alt, cells + 14 unit tests written first; the cells zero-cell-size test needed its expectation corrected to (25,25) once I worked it through), MdRow.images/MdImage in mdrender.rs with the token hidden from the `🖼 alt` row (2 new mdrender tests; the existing attachment test is unchanged and passes), docs/tui.md "Image size" paragraph, one line in the yaks skill. Docs parity checked by grep of docs/ README.md .agents/skills/ src/ for 🖼/image: only docs/tui.md (:109, :246 describe the marker, still true) and imageview.rs's title (the viewer, untouched) mention it; README has nothing. Gate: cargo test --workspace green; yaks verify PASS. Nothing draws an image; yaks-0bf0 reads MdRow.images and calls imagesize::cells.

---
▸ 2026-10-22T21:00:36Z [size-1]

moved: shaving -> shorn

---
▸ 2026-10-22T21:03:12Z [delta:Add basic image attachment rendering]

Landed in the thread clone as 7462a6f (squash of size-1's 3a87135; base was HEAD, `yaks land --dry-run` clean). NOT yet pushed to the user's checkout `main`: waiting for the human's go-ahead, since this yak was asked for as "shear", not "land".

Reviewed the diff (5 files, nothing under .yaks/). Re-ran on 7462a6f myself: `cargo test --workspace` all green (724 in the bin). Reproduced with my own release build: a body with `![shot =50%](yak-0000/x.png)` and `![a = b](q.png)` shows `Look: 🖼 shot end. And 🖼 a = b.` in `yaks tui --headless`: the size token is hidden, an `=` that is not a size stays.

No follow-ups from this one. yaks-0bf0 (inline images) is now unblocked on its dependency; it still waits on two human questions.
