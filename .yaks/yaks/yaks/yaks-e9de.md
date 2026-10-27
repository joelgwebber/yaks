---
id: yaks-e9de
title: Readable markdown + detail/edit QoL
status: shorn
type: feature
priority: 2
created: '2026-10-17T04:23:51Z'
updated: '2026-10-27T01:18:53Z'
labels:
- markdown
- tui
---

Umbrella for the markdown rendering, markdown editing and detail-pane QoL work planned in the Delta thread "Markdown rendering / editing QoL" (2026-10-17). Children carry the work; this yak carries the decisions every child must follow.

## Decisions (agreed with Joel, 2026-10-17)

- **CommonMark line breaks (option B).** A single newline is a soft break and renders as a space; only a blank line starts a paragraph. A hard break is two trailing spaces or a trailing backslash. Legacy bodies written one-item-per-line will run together; that is accepted (they get fixed by hand, or later by yaks-33c8), and the skills must teach agents to use lists or blank lines.
- **Parser: `pulldown-cmark`** (pure Rust, keeps the C-free release matrix). GFM tables, task lists, strikethrough, footnotes and alerts on. Its offset iterator is the source map.
- **Split comments before parsing.** The body is split with `content::parse` first; the description and each comment render as separate markdown documents (no state leaks between them, e.g. an unclosed fence).
- **Rendering may collapse, the editor never does.** The read pane may hide markers, collapse links to their text, reflow paragraphs and draw tables. The editor keeps display width == text width: styling, dimming and `Modifier::HIDDEN` only. Anything that needs a display mapping in the editor (collapsed links, wrap hanging indents, rendered tables) is past the edtui breakpoint and out of scope.
- **Every rendered row knows its source lines.** Yank copies source markdown, find searches displayed text, `E` opens the editor at the exact source line.
- **No-wrap blocks (code, tables) scroll horizontally**, per block, when the line cursor is on them; nothing else in the pane scrolls sideways.
- **One style table** for the read pane and the editor; body headings (3 distinct levels) must not look like the pane's own cyan-bold labels.
- **DRY:** one comment-header design (shared with the edit form's `render_block_separator`), one parser, one palette.
- **Skipped:** math (revisit only with Kitty-graphics images), emoji shortcodes, definition lists. Mermaid renders as a labelled code block. Inline HTML renders as literal text.

## Evidence convention

Every UI child attaches before/after headless frames (`yaks tui --headless`, plus a docshots SVG for durable states). yaks-ea13's body is the torture-test fixture.

## Order

1. Quick wins (independent): editor background, pinned-header rule + status icon, dates, link `)` bug, comment headers from `content::parse`.
2. Shared style palette (design pass).
3. Renderer core (pulldown-cmark → row model with source map, prefix-stack layout).
4. Features on the core: headings/inline, code + tables with h-scroll, links + attachments, GFM extras, exact-line `E`.
5. Editor: hidden markers, quieter links, highlights from the shared parser, auto-continue (in the edtui fork).
6. View/edit unification (design pass first).
7. Skills guidance (yaks-382b) once the core lands; each feature updates skills/help/docs in its own change.

---
▸ 2026-10-17T04:31:30Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-17T04:33:16Z [delta-lead]

Batch 1 spawned 2026-10-17 from base ce4aeba, all on claude-sonnet-5-5 (worker profile, isolated clones):
- bg-1 → yaks-5408, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiKcJIBxBCg_8DSgeNASrgoPHtnOl5O
- hdr-1 → yaks-66a3, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiLPZIBxBCg_8DSgeNASrgoPHtnOl5O
- links-1 → yaks-f67e, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiMf5IBxBCg_8DSgeNASrgoPHtnOl5O
- dates-1 → yaks-ef82, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiOt5IBxBCg_8DSgeNASrgoPHtnOl5O
- palette-1 → yaks-bc53, thread ksQQqt3mx3gRRTWsmE1GMTX1T5LOABiTKpIBxBCg_8DSgeNASrgoPHtnOl5O

The lead (delta-lead) builds yaks-649d (renderer core) in parallel, in a new module.

---
▸ 2026-10-17T04:37:18Z [delta-lead]

Joel answered yaks-2362 and yaks-3b8d (both yes, both as a schema migration in the next release): new yak yaks-a870 (schema 5 migration) carries both fixups; migrate-1 spawned on it.

---
▸ 2026-10-17T04:43:37Z [delta-lead]

Batch 1 landed 2026-10-17 by the yakherd (land.sh, merge by SHA, each identical to the computed merge; `cargo test --release` 623+ pass, doctor clean):
- links-1 e7f2d34 (yaks-f67e) → d430c17
- dates-1 7684791 (yaks-ef82) → cfa4bd1
- hdr-1 2e9ba38 (yaks-66a3) → da1a2d2
- bg-1 d9f3493 (yaks-5408) → bc1deaf
- palette-1 2a6faf4 (yaks-bc53, sign-off ask open to Joel) → b7c9cea

Checked on the real build: headless detail shows `Status: 🪒 shaving`, `Created: 19m ago`, the pinned header with its rule row. Lead's mdrender module (yaks-649d, not yet wired) committed 9e986e6.

---
▸ 2026-10-17T04:59:10Z [delta-lead]

Batch 2 spawned from 3c47d0f on claude-sonnet-5-5: hscroll-1 → yaks-c733 (thread ...jrf5I...), edmarks-1 → yaks-9425 (thread ...jtwJI...). migrate-1 (yaks-a870) woken after an early silent exit.

---
▸ 2026-10-17T05:00:47Z [delta-lead]

Status 2026-10-17 ~05:05 (delta-lead):

- Shorn: yaks-f67e, yaks-ef82, yaks-66a3, yaks-5408, yaks-649d (core), yaks-e30d and yaks-93cd (delivered by the core).
- Waiting on Joel: yaks-bc53 (palette sign-off), yaks-a893 (in-place view/edit design), yaks-1f84 (OK to push to the edtui fork).
- Running: migrate-1 (yaks-a870), hscroll-1 (yaks-c733), edmarks-1 (yaks-9425).
- Held until hscroll-1 lands (both touch mdrender.rs): yaks-1bbb (footnote jumps; task lists and alerts already render), yaks-3020 (code syntax colours), and yaks-3b8d's attachment glyphs (also waits on yaks-a870).
- yaks-382b: the CommonMark rule is already in the yaks skill (47d5d5e); the `$( )` rule in brief.rs and the coordinating skill remain, after migrate-1 lands (it edits the skills too).

---
▸ 2026-10-17T05:04:18Z [delta-lead]

migrate-1 landed 2026-10-17: 80659ed (yaks-a870, schema 5 + `yaks migrate`) → 2d80a0f, identical to the computed merge. Gate re-run in a separate target dir (`CARGO_TARGET_DIR=target/s5 cargo test --release`: 17 result lines ok). Reviewed on a fresh copy of this farm with that build:

- dry run: 1336 comment headers in 402 yaks, 84 non-image links in 57 yaks
- after the real run: the diff is exactly 1336 added blank lines and 84 dropped `!`
- `doctor` clean, and a second `migrate` reports nothing to do
- a migrated yak renders its comments correctly in the TUI

Hold: `target/release/yaks` stays the schema-4 build, and this repo's `.yaks/` stays schema 4, until hscroll-1 (yaks-c733) and edmarks-1 (yaks-9425) land. They do their bookkeeping with that binary on schema-4 clones, and their yak edits would conflict with a migration commit. After they land: rebuild, `yaks migrate`, commit it alone, regenerate every docshot (the comment shape changed), then shear yaks-2362.

Worker note: migrate-1 says its new tests were written after the code and never ran red; the review above is the independent check.

---
▸ 2026-10-17T05:05:57Z [delta-lead]

edmarks-1 landed: 41e6e05 (yaks-9425) → 1b5e0b6, identical to the computed merge; gate re-run green (target/s5). Looked at the attached frame: markers are dim on the cursor line and blank elsewhere. A hidden `(url)` leaves a gap as wide as the URL (width-preserving by decision); flagged to Joel as a possible follow-up (e.g. dim instead of hide for link URLs).

---
▸ 2026-10-17T05:18:09Z [delta-lead]

Progress 2026-10-17 ~05:20 (delta-lead):

- Landed: hscroll-1 6a00b3b (yaks-c733) → 4b0a142; farm migrated to schema 5 (6dd38ad); every docshot regenerated (07df082); yaks-382b and yaks-2362 shorn (ea64a36).
- Batch 3 spawned from 4c586e9 on claude-sonnet-5-5: syntax-1 → yaks-3020 (thread …kcz5I…), links-2 → yaks-1bbb + yaks-3b8d (thread …ke_5I…).
- Incident: syntax-1's first note landed in the coordinator's checkout; reverted, worker told to stay in its own clone.
- Still waiting on Joel: yaks-bc53 (palette sign-off), yaks-a893 (in-place edit design), yaks-1f84 (edtui fork push).

---
▸ 2026-10-17T05:30:45Z [delta-lead]

syntax-1 landed: da680bb (yaks-3020) → beb8e45, identical to the computed merge, 16 files; gate re-run green; looked at the attached frame (rust/sh/json coloured, unknown language flat). Its commit carries no trace of the stray note in the coordinator's checkout.

---
▸ 2026-10-17T05:34:27Z [delta-lead]

links-2 landed: 6a74341 (yaks-1bbb + yaks-3b8d) → eb268a6, identical to the computed merge; gate re-run green (17 ok). Checked on the real build against a scratch farm with a real `yaks attach`: `[the docs](url)` shows as text with `→ https://example.com/docs` in the status bar on Tab, the attachment shows as `📎 build-log` with `→ yak-…/build-log.txt`, the footnote renders as `claim[1]` with `[1] Because.` collected at the end.

Every yak in the family that can be done without a decision is done. Remaining: yaks-bc53 (palette sign-off), yaks-a893 (in-place edit design, blocks yaks-4fa6 and yaks-c6f4), yaks-1f84 (edtui fork push), yaks-33c8 (parked idea).

---
▸ 2026-10-17T17:03:17Z [delta-lead]

2026-10-17 ~17:05 (delta-lead):

- Merged the yaks-48d6 perf subthread (fae62f3; it also merged local/main and Joel's own farm migration); gate 18/18 green; `~/go/bin/yaks` rebuilt from 7f5cc98.
- Joel signed off yaks-bc53 (shorn) and approved yaks-a893. That design is split into yaks-597d (in-place editing) and yaks-92e5 (trim the form). yaks-4fa6 now depends on 597d, yaks-c6f4 on 92e5.
- Joel OK'd edtui fork commits (yaks-1f84).
- Spawned from 2d25402 on claude-sonnet-5-5: inplace-1 → yaks-597d (thread …mRp5I…), autocont-1 → yaks-1f84 (thread …mUspI…).

---
▸ 2026-10-17T17:12:04Z [delta-lead]

autocont-1 landed: 67abbe5 (yaks-1f84) → 1145a08. Fork review by the yakherd:

- the diff is one commit, `ff5c1a9`, behind an off-by-default `EditorState` option inside `LineBreak`;
- the fork's tests pass (175), including the worker's key-handler test, which asserts that a whole insert session with continuations undoes in one step (my ad-hoc probe with a hand-built key sequence failed on the probe itself, not on the feature);
- merged into fork `yaks-integration` as c49afd9 and pushed; yaks Cargo.toml back on `branch = "yaks-integration"`, Cargo.lock pinned to c49afd9;
- gate 18/18 green;
- headless comment editor on the real build: `- a⏎b⏎⏎> q⏎` gives `- a / - b / > q / > `.

---
▸ 2026-10-17T17:52:29Z [delta-lead]

inplace-1 landed: 4fbf85c (yaks-597d) → 41394a2, identical to the computed merge. The worker ran only the workspace gate; the yakherd re-ran `cargo test --release` (18/18 ok). Driven on the real build against a scratch farm:

- `E` on a comment: the block becomes an editor in place (accented header with the save hint, `┃` gutter, raw source), everything above stays rendered.
- An `o`-new-line edit + Ctrl-S: the file diff is exactly the added line + `updated:`; all other blocks byte-identical.
- `M` + text + Ctrl-S: a new attributed comment appended, the editor closes.

Next: yaks-92e5 (drop the form's content blocks), then shear yaks-4fa6 / slaughter yaks-c6f4 / shear yaks-a893.

---
▸ 2026-10-17T18:01:31Z [delta-lead]

formtrim-1 landed: 9381adf (yaks-92e5) → c2b20a0; gate 18/18; real build: `E` on the list opens a header-only form. Closed: yaks-4fa6 shorn (delivered by 597d), yaks-a893 shorn, yaks-c6f4 slaughtered. Docshots refreshed for the `M:comment` help bar. Open in the family: yaks-bc30 (perf cache), yaks-da5e and yaks-62cf (Joel's new ideas), yaks-33c8 (parked).

---
▸ 2026-10-17T18:14:58Z [delta-lead]

Joel: finish the rest, land the arc, cut a release; leave yaks-33c8 and yaks-62cf out. Spawning cache-1 (yaks-bc30) and srclinks-1 (yaks-da5e, format agreed).

---
▸ 2026-10-17T19:43:26Z [delta-lead]

Landed 2026-10-17:

- srclinks-1 684800b (yaks-da5e, source links incl. pinned commits) → 5e3e5f1
- cache-1 f8e5c10 (yaks-bc30, memoized rows()/detail lines; held `j` ~22 ms → ~0.7 ms per key on this farm) → 90d22ee

Both identical to the computed merge; gate 18/18 green; doctor clean; `~/go/bin/yaks` rebuilt. My own pty drive of the `$EDITOR` hand-off timed out in this sandbox (no keystrokes reached the TUI), so that path rests on srclinks-1's pty run plus its unit tests (`editor_arguments_by_editor`, `following_a_source_link_queues_the_editor_at_the_line`).

The arc is complete apart from the release (0.0.20), which waits on Joel's go-ahead.

---
▸ 2026-10-17T19:55:43Z [delta-lead]

Arc complete: every child shorn (or moved out / slaughtered), released in 0.0.20 (yaks-ade2).

---
▸ 2026-10-17T19:55:57Z [delta-lead]

moved: shaving -> shorn
