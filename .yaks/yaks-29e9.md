---
id: yaks-29e9
title: 'An ask answered in the reference copy (needs: agent there) still reads open in the asking shed'
status: shaving
type: bug
priority: 2
created: '2026-10-13T01:51:06Z'
updated: '2026-10-13T02:04:03Z'
parent: yaks-38dc
labels:
- cli
- delta
- ui
---

Seen 2026-10-13 on yaks-7204: the coordinator asked in its shed, committed, and pushed main to Joel's checkout, so main's copy carried the same open ask. Joel answered it in main (plain `answer`, as the coordinator had told him): main's copy became `needs: agent` + his note, uncommitted. The asking shed's copy stayed `needs: human`, and every surface kept calling it open:
- `yaks sheds` / the Sheds tab in main: the shed shows `⏳` / `1 needs`, not landed: `needs_superseded` (yaks-8814) requires the reference to carry NO `needs:`, and `needs: agent` is exactly "answered, not yet picked up".
- The TUI Inbox in main (yaks-1ea3) lists it as a shed ask, and `a` there would answer it a SECOND time, in the shed.

Fix the rule in one place: a shed's `needs: human` is superseded when the reference holds the shed's ask entry followed by a later entry, whether the reference's `needs:` is cleared OR `agent` (answered). `Snapshot::open_asks` and the landed flag then agree, and the Inbox/`a` stop offering a second answer. Also consider: `inbox --sheds` (same rule?), and whether the coordinating skill should say "do not push an open ask to the primary; tell the human `answer <id>@<shed>`" (the coordinator's mistake here).
Evidence: a test with exactly this case (ref `needs: agent`, ask + answer entries) reads landed; the TUI Inbox in the primary does not list it.

---
▸ 2026-10-13T02:04:02Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-13T02:04:03Z [delta-lead]
Claim (delta-lead, yakherd): owner fix-1 (Sonnet worker). Parallel with ghost-1 (TUI only).

Goal: the rule in the description. In src/sheds.rs `needs_superseded` (beside `outstanding_ids`, from yaks-8814): a shed's `needs: human` is also superseded when the reference copy's `needs:` is `agent` (answered, not yet picked up), provided the reference holds the shed's latest ask entry followed by a later entry. A reference still at `needs: human` does not supersede (unchanged). Check `inbox --sheds` (src/farm.rs or wherever it lists other sheds' asks) and `show --sheds`: if either decides "open ask" by its own rule, make it use the same one (one rule), and say what you found in a note.

Scope: src/sheds.rs (`needs_superseded`, `outstanding_ids`, tests in its `mod tests`), src/farm.rs only if `inbox --sheds` has its own rule, docs only if a `landed` / `needs` definition's wording no longer matches (docs/cli.md, the yaks skill's sheds row, clap help in src/main.rs).
Out of scope: src/tui/ (ghost-1), everything else in src/sheds.rs.

Evidence (fail first): extend `outstanding_ids_supersedes_a_needs_the_reference_resolved` (or a sibling test) with the yaks-7204 case: the reference at `needs: agent` with the ask + an answer entry is not outstanding (so the shed reads landed); the reference at `needs: agent` WITHOUT a later entry after the shed's ask (e.g. a different, older ask) stays outstanding. Full `cargo test --release`. Judge: delta-lead at landing.
