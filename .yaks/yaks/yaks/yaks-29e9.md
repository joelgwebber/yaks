---
id: yaks-29e9
title: 'An ask answered in the reference copy (needs: agent there) still reads open in the asking shed'
status: shorn
type: bug
priority: 2
created: '2026-10-13T01:51:06Z'
updated: '2026-10-27T01:20:35Z'
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

---
▸ 2026-10-13T02:05:11Z [fix-1]

Start: Tue Oct 13 02:05:11 UTC 2026

path: /Users/joel/src/yaks/.delta/worktrees/h106mvjvzcp9/yaks/.yaks/yaks-29e9.md

---
▸ 2026-10-13T02:06:19Z [fix-1]

Tests written first (`outstanding_ids_supersedes_a_needs_the_reference_answered`: the yaks-7204 case, needs: agent + ask + answer; needs: agent with no follow-up stays outstanding; needs: agent with only an older/different ask stays outstanding; reference still needs: human unchanged). Before the change, `cargo test --release outstanding_ids`:

```
test sheds::tests::outstanding_ids_supersedes_a_needs_the_reference_answered ... FAILED
panicked at src/sheds.rs:4168:9:
assertion failed: !outstanding_after_ask("needs-answered",
        Some(("shaving", "needs: agent\n", &["answered: yes"])))
```

---
▸ 2026-10-13T02:07:57Z [fix-1]

Decision: the one rule is `needs_superseded` in src/sheds.rs; it now accepts a reference `needs:` of None OR `agent` (`matches!(.., None | Some("agent"))`), with the ask-entry-plus-later-entry (or shorn) requirement unchanged. Rejected: a separate `Snapshot::open_asks` rule in the TUI (src/tui is ghost-1's and it already reads `y.landed`, which comes from `outstanding_ids`), and treating any reference `needs:` as superseding (a reference at `needs: human` is a live question, unchanged).

Finding: `inbox --sheds` (`Farm::inbox_sheds`, src/farm.rs) has its own, looser rule (`ask_already_seen`: the primary already holds the shed's latest ask entry and every entry after it) and does not look at the reference's `needs:`, so the yaks-7204 case (primary holds the ask + answer) is already left out of it. `show --sheds` (`Farm::show_sheds`) decides no open-ask rule: it only lists the shed's note entries this farm lacks. No change to either; unifying `inbox_sheds` onto `outstanding_ids` would change what it reports (a shed ask the primary has not seen but that is otherwise landed) and is outside scope.

Decision: docs unchanged. docs/cli.md `sheds`, the yaks skill sheds row and README define `landed` as "the reference is further along ... already has everything the shed has", which an answered `needs: agent` reference satisfies. Grep: `grep -rn landed docs/cli.md .agents/skills/yaks/SKILL.md README.md src/main.rs` found no narrower definition of when a `needs:` counts. Not done: the coordinating-skill suggestion in the description ("do not push an open ask to the primary; tell the human `answer <id>@<shed>`"); skill prose is the coordinator's, left as a follow-up.

---
▸ 2026-10-13T02:09:10Z [fix-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-13T02:11:19Z [fix-1]

Summary: `needs_superseded` (src/sheds.rs) now also supersedes a shed's `needs: human` when the reference's `needs:` is `agent` (answered, not yet picked up), given the same ask-entry + later-entry (or shorn) check. `outstanding_ids`, the `landed` flag and the TUI `open_asks`/Inbox/`a` (which read `landed`) follow. Test `outstanding_ids_supersedes_a_needs_the_reference_answered` seen failing first, passing after; full `cargo test --release` green (570 bin tests + integration), `yaks verify` PASS. `inbox --sheds` and `show --sheds` unchanged (see Finding). Skill-prose suggestion left as a follow-up. Finish: Tue Oct 13 02:11 UTC 2026.

---
▸ 2026-10-13T02:11:19Z [fix-1]

moved: shaving -> shorn
