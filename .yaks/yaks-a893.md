---
id: yaks-a893
title: 'Design: unify view and edit for the description and comments'
status: shaving
type: feature
priority: 2
created: '2026-10-17T04:25:39Z'
updated: '2026-10-17T17:02:15Z'
parent: yaks-faf5
labels:
- tui
---

Design pass first; implementation yaks follow from it.

Problems today:
- `M` (add a comment) opens `Overlay::Edit`, which replaces the whole detail pane: you lose the context you're replying to.
- `E` on a comment opens the edit form, whose content accordion collapses every other block to a one-line separator (yaks-127e; overflow bug yaks-c6f4). Switching between view and edit moves everything around.
- The accordion was a workaround for editing/rendering bugs that were hard to pin down (Joel's recollection; check yaks-127e's notes and git history for what they were).

Direction to evaluate: editing happens *in place* in the detail pane. The block being edited becomes an editor at the same position and the rest stays rendered around it, scrolling as usual; a new comment opens as an empty editor at the end of the comments. Header fields stay in the form, or get their own in-place editors. Questions to settle: what a key does inside vs outside the editor, how scrolling works with an editor taller than the viewport, whether the form's header rows stay a separate surface, how this interacts with the exact-line `E`.

Deliverable: a short design note on this yak with mock frames (headless renders of a prototype, or hand-drawn text frames), agreed with Joel, then implementation yaks.

---
▸ 2026-10-17T05:00:19Z [delta-lead]

moved: hairy -> shaving

[today-view-M-E](yaks-a893/today-view-M-E.txt)

---
▸ 2026-10-17T05:00:19Z [delta-lead]

Design note (delta-lead, 2026-10-17). Today's three surfaces are attached (`today-view-M-E.txt`: headless 90x34 frames of the detail pane, `M`, and `E` on a comment). Problems seen there:

- `M` replaces the whole pane with an empty editor, so the thing you are replying to disappears.
- `E` swaps the rendered body for a form: header fields shift up, the description collapses to a one-line separator, the edited comment flips to raw source in a different place.
- yaks-127e's notes and git history show the accordion was a layout choice (one `Min(0)` editor below fixed separators), not a workaround for a specific editor bug; its known bug is overflow (yaks-c6f4).

## Proposal: edit a content block in place

The detail pane stays on screen. `E` on a body row (description or a comment) turns *that block* into an editor at the same position; everything above and below stays rendered and scrolls as usual. `M` opens an empty editor under a provisional `▸ you · now` header after the last comment.

```text
│ Logging in with an expired token crashes the app.
│
│ Repro
│
│ 1. Log in, wait an hour.
│ 2. Click anything.
│
│ ▸ joel · 5m ago ──────────────────────────────────────────────────────────
│ Seen on staging too. The refresh path never runs when the token is already
│ expired.
│
│ ▸ agent-1 · 2m ago ─────────────────────────────── editing · Ctrl-S save ─
│┃Root cause: `refresh()` checks `exp` after the request, not before.
│┃
│┃- fix: check before
│┃- add a test█
│┃
│
```

- The editing block keeps its header (accented, with the save hint) and gets a left gutter bar so the edited region is obvious; it shows raw source (umbrella decision: the editor never collapses anything).
- Height: the block's source line count + 1 spare row, capped so at least ~4 rows of context stay visible above it; edtui scrolls inside beyond that. The detail scroll moves only as much as needed to show the whole editor.
- The cursor lands on the source line behind the row the line cursor was on (the row source map from yaks-649d), which delivers yaks-4fa6 for free.
- Keys: everything goes to edtui while editing. `Ctrl-S` saves this block only (re-assembled with `content::assemble`, so the other blocks round-trip byte-identical), `Ctrl-C` / `Esc` cancel with the existing dirty-confirm. After saving, the block re-renders where it was, and the line cursor stays on the same source line.
- `M` = the same editor under a new header at the end; an empty save is a no-op, as today.

## What happens to the edit form

- `c`/`C` (create) keep the form, with its single description block (nothing to edit in place yet).
- `E` on a header row (title, type, priority, labels, source, herd) keeps opening the form, but **without content blocks**: body editing moves to the pane. The accordion, `render_content_stack`, the per-comment `ContentBlock`s in edit mode and yaks-c6f4 all go away.

## Implementation sketch

A new overlay `Overlay::BlockEdit { id, block: Option<usize> /* None = new comment */, editor: Editor }`. `render_detail` swaps that block's rows for a placeholder region of the editor's height and draws the edtui view into it, reusing `editor_theme` / `set_md_highlights` / the existing edit commit path. Mouse hits and scroll arithmetic treat the editor region as N rows.

Questions for Joel are asked on this yak.

---
▸ 2026-10-17T05:00:19Z [delta-lead]

asked: needs human

View/edit design (note above, with a mock frame): OK to (1) edit the description and comments *in place* in the detail pane (`E` on a body row, `M` for a new comment at the end, `Ctrl-S` saves just that block), and (2) drop the content blocks from the `E` edit form so it holds only header fields (create keeps its description)? That removes the accordion and yaks-c6f4. If yes I'll split it into implementation yaks and fan it out.

---
▸ 2026-10-17T16:53:38Z [Joel Webber]

I dig it. That sounds like a plan to me.

---
▸ 2026-10-17T17:01:43Z [delta-lead]

picked up

Joel approved the in-place edit design ('I dig it. That sounds like a plan to me.'). Splitting into implementation yaks below.

---
▸ 2026-10-17T17:02:15Z [delta-lead]

Design approved by Joel. Implementation: yaks-597d (in-place editing, one worker: render/handlers/actions are too coupled to split), then yaks-92e5 (form trims to header fields). This design yak shears when both land.
