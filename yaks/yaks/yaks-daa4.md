---
id: yaks-daa4
title: 'Refresh every docshot: docs/assets SVGs drift from the current toque output'
status: shorn
type: chore
priority: 2
created: '2026-11-07T15:48:14Z'
updated: '2026-11-22T17:18:56Z'
tags:
- docs
- tui
---

Found while landing yaks-85a6: `cargo test -p yaks docshots -- --ignored` rewrites every SVG in `docs/assets/`, not just the frames a change touches. The checked-in frames predate two changes:

- toque's SVG output now writes `fill-opacity` for dim text and `xml:space="preserve"` on each `<text>`, with a different selection background.
- The tab row: frames still show `🛖 Sheds (2)` where the current TUI shows `📖 Doks (0)` (the Doks tab came in after they were drawn).

yaks-85a6 committed only the four `tui-doks*.svg` it changed. Regenerate the rest in one commit, look at a few rasterized (the `docs/tui.md` recipe), and check each still shows what its `docs/` caption says. Worth doing before the next release, so its docs don't show an older TUI.

#### 2026-11-22T16:40:41Z

*herd-files · kind: move · from: hairy · to: shaving · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAG9SR5IBxBCA0aLJyrhJPreUvVwLftHG*

moved: hairy -> shaving

#### 2026-11-22T16:41:01Z

*herd-files*

Claim (herd-files): worker shots-1 (Sonnet), base 6ae40a60, batch before v0.0.41. Scope: src/tui/docshots.rs, docs/assets/*.svg, the docs that embed them. Judge: herd-files at landing.

#### 2026-11-22T16:43:19Z

*shots-1*

Start 2026-11-22T16:43:18Z. Checkout /Users/joel/src/yaks/.delta/worktrees/d3wvgsm9qny3/yaks, base 6ae40a60. Farm file /Users/joel/.local/share/yaks/farms/yaks-539d3edb/yaks/yaks/yaks-daa4.md. Plan: regenerate all docshots, eyeball a sample, add archive-member + line-range scenes, check Sheds tab shots.

![tui-archive-tree](yaks-daa4/tui-archive-tree.png "New docshot tui-archive-tree: a zip attachment as a directory in the directory view (rasterized)")

![tui-line-range](yaks-daa4/tui-line-range.png "New docshot tui-line-range: #L10-L16 opens narrowed, header 'lines 10-16 · w whole file' (rasterized)")

#### 2026-11-22T17:11:38Z

*shots-1 · kind: verify · result: fail*

verify: `cargo test --workspace` -> FAIL (exit 101)

#### 2026-11-22T17:18:38Z

*shots-1 · kind: verify · result: pass*

verify: `cargo test --workspace` -> PASS (exit 0)

#### 2026-11-22T17:18:55Z

*shots-1*

Decision: regenerate all docshots in one pass (`cargo test -p yaks docshots -- --ignored`, 15 generators, second run byte-identical) and commit the drifted SVGs together. Rejected: committing only the frames whose content changed; the point of the yak is that later runs touch only what a change touches.
Decision: tui-image-viewer scene fixed, not its caption. Since the `Attachments:` row came in, Tab+Enter opens the directory view, not the image, so the frame showed a one-file directory under the caption "Enter on an image attachment: shown in the pane". The scene now presses Enter a second time to open the file. Rejected: re-captioning (the doc text about image attachments is right).
Decision: two new generators folded into one test, `docshots_archive`, writing tui-archive-tree, tui-archive-member and tui-line-range. The zip is built in the scene with `entries::fixtures::zip` (a cfg(test) helper, same as tests.rs), so no binary fixture is checked in. Range scene uses a plain run.log attachment (`yak-1a2b/run.log#L10-L16`), not `code:`, to keep the scene independent of a git checkout; the header is `lines 10–16 · w whole file`. Rejected: a code: link (needs `git init` in the scene, already covered by tui-code-viewer/tui-take-out which show `lines 4–9`).
Decision: docs/tui.md embeds: tui-line-range under "A line-range link opens narrowed", tui-archive-tree under "Archives read as read-only directories". tui-archive-member is generated and viewed but not embedded (it is a plain text viewer frame; two new embeds were asked for). Rejected: embedding all three.
Decision: fixed a stale glyph in docs/tui.md's Sheds section (🛦 -> 🛖, what the tab shows). Rejected: leaving it, it is the same doc text the Sheds shots caption.
Finding: Sheds tab shots (tui-sheds, -detail, -yak) already show the actor-based tree (actor roots, claimed/recent yak children, farm footer) from the checked-in scene; after regeneration only tui-sheds-yak changed (one line), the other two are byte-identical. Looked at all three rasterized.
Looked at rasterized: tui-sheds*, tui-detail, tui-take-out, tui-image-viewer/-inline/-caption, tui-dir-row, tui-doks, tui-archive-*, tui-line-range. All match their captions after the viewer fix. Drift confirmed as the "O:take out" hint, doks source path, peek title, line-range header, plus toque's SVG output.
Gate: cargo fmt --check clean; cargo test -p yaks all 42 binaries ok. `yaks verify` failed once on tests/release_dok (a git commit inside the test returned nothing, other workers were running), passed on rerun: PASS.

#### 2026-11-22T17:18:56Z

*shots-1 · kind: move · from: shaving · to: shorn · thread: ksQQ7I2z8nJeT6a6hDJ07WHX45LOAHgPcJIBxBCA0aLJyrhJPreUvVwLftHG*

moved: shaving -> shorn

#### 2026-11-22T17:18:56Z

*shots-1 · kind: change*

Doc screenshots refreshed for the current TUI, with new frames for browsing a zip attachment and for a narrowed line range
