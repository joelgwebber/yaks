---
id: yaks-dfca
title: Rename lane to shed everywhere (yaks lanes becomes yaks sheds, no alias)
type: task
priority: 2
created: '2026-10-06T21:31:42Z'
updated: '2026-10-06T21:34:35Z'
parent: yaks-38dc
labels:
- cli
- docs
- skills
---

Decision (Joel, 2026-10-06, via the naming subthread): a lane is now called a SHED, everywhere, with no aliases. His words: "I love 'shed'. It reminds me of 'bike shed' ... Let's just bite the bullet and change everything comprehensively. No need for aliases -- I want to force anyone using these tools to deal with our wacky terminology, whether they like it or not."

Decided: lane becomes shed; `yaks lanes` becomes `yaks sheds`; `yaks lanes` is REMOVED, no alias. NOT decided (do not adopt): "clip" (what a shed has produced) and "barn" (the primary checkout). Only the word shed changes.

## What changes
- The subcommand `lanes` -> `sheds`, its `--help`/`///` text, and `Command::Lanes` -> `Command::Sheds`. No alias: `yaks lanes` must fail as an unknown subcommand.
- `src/lanes.rs` -> `src/sheds.rs` (git mv), `mod lanes` -> `mod sheds`, and every type, function, constant, test name, comment and user-visible string in it: `Lane` -> `Shed`, `LaneKind` -> `ShedKind`, `LaneFarm` -> `ShedFarm`, `lanes` -> `sheds`, `lane` -> `shed`. `Farm::lanes` -> `Farm::sheds`. JSON field names that say lane (if any), table text, error messages.
- Everywhere else the word appears in the living text: docs/cli.md, docs/skills.md, docs/tui.md (if present), README.md, AGENTS.md, `.agents/skills/**` (the yaks skill command table, yaks-coordinating core, -delta, -worktrees, -team, -private, yaks-working, the skills README), `--help` text, any TUI string. Generic prose about parallel work ("a worker's lane", "one lane per worker", "lane-cut", "lanes in flight") becomes shed too, unless a sentence then reads wrongly; keep those few as they are and list them.
- The Delta skill's example spawn title `lanes-1: build yaks lanes CLI` becomes `sheds-1: build yaks sheds CLI`.
- ONE glossary line where the yaks skill defines terms (its Terminology section) and one in docs/cli.md's `sheds` row: "shed (formerly lane)".

## What does NOT change (history)
The observation log `.yaks/artifacts/yaks-df61/**`, every other yak file under `.yaks/` (titles, descriptions, notes), `.yaks/artifacts/**`, commit messages and the git history. Old text keeps "lane"; the glossary line keeps it readable. Do not edit any other yak.

## Rules
- Mechanical and word-bounded: match `\blanes?\b`, `\bLanes?\b`, `Lane`/`lane` inside identifiers (`LaneFarm`, `lane_state`), never inside other words (plane, planet, Elane). Review every hit by eye in prose; do not run a blind global replace over prose.
- The behaviour of the command must not change at all in this yak: the same output text apart from the word, the same JSON shape. The baseline defect (yaks-37fa) is a separate yak that lands after this one.

## Evidence
- `cargo test -p yaks` green; the existing lanes tests, renamed, still pass unchanged otherwise.
- `yaks sheds` and `yaks sheds --json` print what `yaks lanes` printed before, apart from the word (compare the old binary's output with the new on the same checkout and paste the diff).
- `yaks lanes` fails with an unknown-subcommand error.
- A grep for `lane` (case-insensitive, word and identifier) over the whole tree EXCLUDING `.git`, `target/`, `.yaks/` and the allowed history, pasted, showing the few remaining hits and why each stays.
- Docs parity (the project rule): every surface that named the command now names the new one.

---
▸ 2026-10-06T21:31:51Z [delta-lead]
moved: hairy -> shaving

---
▸ 2026-10-06T21:31:51Z [delta-lead]
Claim (coordinator delta-lead). Owner: worker shd-1. Scope: the whole tree except the history listed in the description. Judge: the coordinator re-runs the gate, runs `yaks sheds` and `yaks lanes` against the real checkouts, and greps for stragglers. Forbidden: renaming anything in the allowed history, changing behaviour, any other yak, adopting "clip" or "barn".

---
▸ 2026-10-06T21:32:19Z [shd-1]
Start: Tue Oct  6 21:32:15 UTC 2026
Path: /Users/joel/src/yaks/.delta/worktrees/vk8z2vjzgdqh/yaks/.yaks/shaving/yaks-dfca.md
Old binary copied to scratch (yaks-old) before any edit.

---
▸ 2026-10-06T21:34:21Z [shd-1]
verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-06T21:34:35Z [shd-1]
Decision: mechanical `Lane`->`Shed`, `lane`->`shed` (perl, case-preserving) over every non-history text file, then read every hunk. A token census first showed no false positives (no plane/planet/Elane), so the identifier case needed no word-boundary guard. Rejected: per-file hand edits (error-prone for ~800 hits); a blind replace without the diff review.
Decision: no generic-prose sentence was kept as "lane": every one (one lane per worker, per-lane commits, a human-driven lane, ...) reads fine as "shed", so the exception list is empty.
Decision: `rustfmt --edition 2024` on the five changed src files reordered `mod sheds;` after `mod rollup;` (alphabetical, as rustfmt does); its unrelated hunks in actor.rs, tui/docshots.rs, tui/tests.rs were reverted.
Decision: glossary lines: yaks skill Terminology gets a fourth container bullet "a **shed** (formerly "lane") is one other checkout ..." and docs/cli.md's `sheds` row says "a shed (formerly lane) is one such checkout". The only two remaining "lane" hits in the tree.

Evidence:
- Old binary (copy of the pre-change one) `lanes` vs new `sheds`, same checkout: `diff` of text output IDENTICAL, `--json` IDENTICAL (output carries no occurrence of the word).
- `yaks lanes` -> `error: unrecognized subcommand 'lanes'` (exit 2).
- Straggler grep `grep -rIin lane . --exclude-dir={.git,target,.yaks,.edtui}` lists exactly: .agents/skills/yaks/SKILL.md:39 and docs/cli.md:42, the two glossary lines.
- `.yaks/` (history) untouched except this yak.
- Gate `cargo test -p yaks`: PASS (recorded above).
Summary: src/lanes.rs -> src/sheds.rs (git mv), Command::Lanes -> Sheds, Farm::lanes -> sheds, all types/tests/strings renamed; README, docs/cli.md, docs/skills.md, .agents/skills/** and the Delta skill's example title (`sheds-1: build yaks sheds CLI`) updated; no behaviour change.

---
▸ 2026-10-06T21:34:35Z [shd-1]
moved: shaving -> shorn
