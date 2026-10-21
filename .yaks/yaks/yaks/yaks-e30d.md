---
id: yaks-e30d
title: Render headings, rules, emphasis and strikethrough
status: shorn
type: task
priority: 3
created: '2026-10-17T04:25:16Z'
updated: '2026-10-17T04:58:08Z'
parent: yaks-ea13
depends_on:
- yaks-649d
- yaks-bc53
labels:
- markdown
- tui
---

On the renderer core: hide `*`/`_`/`**`/`~~` markers; H1/H2/H3 styled from the shared palette (H1 maybe underlined with a rule, H2 bold-colored, H3 bold), H4-H6 as H3; thematic breaks as a full-width dim `─` rule. Depends on the palette and the core.

---
▸ 2026-10-17T04:58:04Z [delta-lead]

moved: hairy -> shaving

[headings-rules-frame](yaks-e30d/headings-rules-frame.txt)

---
▸ 2026-10-17T04:58:04Z [delta-lead]

Done as part of the renderer core (yaks-649d, commit 47d5d5e) by delta-lead: markers hidden (`*`, `_`, `**`, `~~`), headings H1-H6 from `palette::heading` (H1 magenta bold underlined, H2 blue bold, H3 bold italic, H4-6 grey bold: palette-1's choice, sign-off pending on yaks-bc53), thematic breaks as a full-width dim `─` rule. Evidence: attached headless frame of a fixture with every heading level, strikethrough, a rule; the styled view is docs/assets/tui-md-render*.svg (attached as PNGs on yaks-649d). Tests: mdrender `rule_heading_and_table`, `emphasis_markers_are_hidden_and_styled_across_a_soft_break`.

---
▸ 2026-10-17T04:58:08Z [delta-lead]

verify: `cargo test --workspace` -> PASS (exit 0)

---
▸ 2026-10-17T04:58:08Z [delta-lead]

moved: shaving -> shorn
