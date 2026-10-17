---
id: yaks-bc53
title: Shared markdown style palette for read pane + editor
status: shaving
type: task
priority: 2
created: '2026-10-17T04:24:29Z'
updated: '2026-10-17T04:31:30Z'
parent: yaks-e9de
labels:
- markdown
- tui
---

Design pass, then a small code change. One table of styles used by both the read pane and the editor (today `markdown.rs` has its own private styles and the read pane layers its own link/label colors on top).

Must cover: H1/H2/H3 (three visibly distinct levels that do not read like the pane's own cyan-bold labels `Children:` / `Depends on:`), emphasis, strong, strikethrough, inline code, code block (fg + optional subtle bg), quote bar, list bullets per depth (•, ◦, ▪), rules, links (text vs dimmed URL), task boxes, alert kinds (NOTE/TIP/IMPORTANT/WARNING/CAUTION), comment headers, dimmed markers (editor).

Evidence: a docshots scene rendering every style against the default and a light terminal palette; attach the frames. Get Joel's sign-off on the frame before the renderer core consumes it.

---
▸ 2026-10-17T04:31:30Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-17T04:31:30Z [delta-lead]
Claim (yakherd delta-lead, 2026-10-17): owner **palette-1**.
Scope: a new module src/tui/palette.rs (named `Style` constructors or a struct of them) holding every markdown style listed in this yak's description; switch the private style fns in src/tui/markdown.rs to read from it (behaviour may change only where the new palette differs); a new docshots scene in src/tui/docshots.rs that renders a sample document covering every style (headings H1-H3 next to a pane label like `Children:`, emphasis, code, quote bar, bullets per depth, rule, link text + dim URL, task boxes, the five alert kinds, a comment header) to a NEW docs/assets/tui-md-palette.svg (generate only that one SVG). The renderer that will use most of these does not exist yet: render the sample by hand-building ratatui Lines from the palette, it is a swatch, not the renderer.
Constraints: 16-colour/indexed colours that work on dark and light terminals; headings must not reuse the pane-label style (cyan bold) from `render_dline`.
Evidence + judge: Joel decides by looking. Rasterize the SVG to PNG (headless Chrome recipe in docs/tui.md; on Linux try `google-chrome`/`chromium`), attach SVG (+PNG if you got one), then `yaks ask yaks-bc53` for sign-off and finish with the commit anyway (the ask stays open on the shorn-pending yak: do NOT shear; leave it shaving with the ask open, commit, return).
Do NOT regenerate docs/assets docshots (several workers would collide on the SVGs; the coordinator regenerates them after landing). Re-accept any insta snapshot your change moves (`INSTA_UPDATE=always cargo test`, or `cargo insta accept` if installed) and read every changed .snap diff before committing. `cargo fmt` reformats files nobody touched (src/actor.rs, src/tui/docshots.rs, src/tui/tests.rs): run `cargo fmt -- <your files>` and `git checkout` those three if they changed. Docs/help parity per AGENTS.md (docs/tui.md, help_content in src/tui/render.rs) where behaviour a user sees changes.
