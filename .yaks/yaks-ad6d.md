---
id: yaks-ad6d
title: 'yaks sheds: list the current shed too (marked), and let changes/--shed/answer name it'
status: shorn
type: feature
priority: 2
created: '2026-10-10T17:21:10Z'
updated: '2026-10-10T19:43:03Z'
parent: yaks-ee0a
labels:
- cli
- delta
verify: cargo test --release
---

Joel (2026-10-10): `yaks sheds` omits the shed you are in, so you cannot see where you are, and `changes <name>`, `--shed <name>` and `answer <id>@<name>` cannot name your own shed (from inside this clone `yaks changes <its own name>` fails; from the primary `main` was mis-resolved, yaks-3344). Direction agreed (A): list the current shed as a row, marked, and let every selector accept it. The semantics of what "changes" are measured against, per shed, are the open design discussion (see the notes) and decide the code: do not start before that is settled.

---
▸ 2026-10-10T19:05:06Z [delta-lead]

moved: hairy -> shaving

---
▸ 2026-10-10T19:05:06Z [delta-lead]

Claim (coordinator). Owner: here-1. Binding: the 2026-10-10 Decisions on yaks-ee0a (read them: 1, 2, 3, 4, 7), and yaks-e9bb (landed, f03a81c: one fork-point baseline per shed from every viewer).
Why now: after yaks-e9bb the PRIMARY's row, seen from a clone, is measured from the first entry of the primary's own reflog and shows 154 new yaks, 23 moved, 172 notes (real machine, 2026-10-10). That is noise until the primary has its own baseline.

Goal (all of it, one rule: `cwd` never changes an answer, only the DEFAULT shed):
1. `yaks sheds` lists EVERY checkout including the one you are in, which is marked (text: a `(here)` tag on the `name:` line and a `*` in the row's first column or equivalent; JSON `current: true`, false elsewhere). Each row is measured like any other shed: fork point, working tree. The table rows are no longer "the others": say so in the `--help`/docs.
2. The PRIMARY's baseline is `origin/main` (Decision 4): its farm at `origin/main` (the remote-tracking ref `refs/remotes/origin/main`, falling back to `origin/HEAD`'s target) against its working tree: "what is not published". Use it wherever the primary is measured, from any viewer. When there is no usable `origin` ref (no remote, never fetched), fall back to the fork point and say `(vs fork)`-style honestly; record the choice as a Decision. A Delta clone's `local/main` is NOT used for the primary's own row (the primary IS local); it only matters to what a clone treats as "the primary's committed state", which this yak does not need.
3. Every selector accepts the current shed: `changes <name>`, `--shed <name>`, `answer <id>@<name>`, `main` (the viewer IS the primary: `main` now resolves to it; the yaks-3344 guard becomes: `main` resolves to the primary whether or not it is the current shed, and errors only when no primary is visible, e.g. from a clone on a shared-thread machine). `yaks changes` with NO argument means the current shed. `--shed <current>` is a no-op.
4. Commands that mean the OTHER sheds keep meaning that: `inbox --sheds`, `show --sheds` ("same in N sheds"), `sheds`'s ahead/behind notes. Audit every caller of `farm.sheds(...)` in src/main.rs and src/farm.rs and exclude the current shed where the command's meaning is "elsewhere"; list your audit (each caller, kept or excluded, why) in a note.
Scope: src/sheds.rs (`discover` keeps the viewer instead of `found.remove(top)`, `Shed` gets `current`, the primary baseline in `inspect`/`GitState`, `resolve`, render, `to_json`, tests), src/main.rs (`Changes { shed: Option }`, the callers above, help), src/farm.rs only for callers. Docs: docs/cli.md (sheds/changes/discover-related rows), README.md, `.agents/skills/yaks/SKILL.md` (sheds/changes rows and the `--shed` text), `Sheds`/`Changes` help in src/main.rs.
Out of scope: yak_changes and its counts (another worker, edit-2 on yaks-047a, owns them and will land first or second: do not touch the `edited` rule), `landed`/`stale` (yaks-ed5e, yaks-9277), the TUI, any write beyond what exists today.
Evidence (tests FAIL first): from three viewers (primary, a sibling git worktree, a Delta clone fixture) `yaks sheds --json` lists the same set of checkouts, the same per-shed `farm` for every shed and exactly one `current: true` that is the viewer; the primary row measured from `origin/main` shows only unpublished farm changes (a fixture primary with an origin ref one commit behind its HEAD whose farm changed); `yaks changes` with no argument equals `yaks changes <own name>`; `main` from the primary; `inbox --sheds`/`show --sheds` still exclude the viewer. Real machine: build your own binary, run `./target/release/yaks sheds` in your clone (read-only) and attach the output; the primary row must no longer show 154 new / 172 notes. Judge: coordinator.

---
▸ 2026-10-10T19:05:31Z [here-1]

Start Sat Oct 10 19:05:30 UTC 2026. yaks path: /Users/joel/src/yaks/.delta/worktrees/qt7v225tzxq0/yaks/.yaks/shaving/yaks-ad6d.md

---
▸ 2026-10-10T19:18:57Z [here-1]

Progress 1: read the code. Implemented (src/sheds.rs, farm.rs, main.rs): discover lists every checkout (checkout_paths; shed_paths still = others, used by discover.rs), Shed.current, BaseKind::Origin + origin_base for the primary, resolve main error, render '*'/(here), JSON current, changes [SHED] optional, answer <id>@<current> = plain answer, --shed <current> no-op, Farm::other_sheds for inbox/show --sheds. Next: fix old tests, add fail-first tests, docs.

---
▸ 2026-10-10T19:23:01Z [here-1]

Fail-first evidence: with discover reverted to the old 'exclude the viewer' (shed_paths) and the primary origin base disabled, the 5 new/updated unit tests fail: every_viewer_lists_every_checkout_and_marks_only_itself (left != right: only 'delta' row listed from the clone, no current row), every_selector_accepts_the_current_shed_and_main_is_the_primary_from_the_primary (Err: `main` is the primary checkout, and it is not visible from here), the_primary_is_measured_from_origin_main_whoever_looks (primary row None / no base), the_primary_falls_back_to_origin_heads_target_then_to_its_fork_point, resolve_by_path_dir_branch_actor_main_and_substring. 50 passed; 5 failed. With the fix: sheds:: 55 passed.

[yaks-ad6d-sheds-old](yaks-ad6d/yaks-ad6d-sheds-old.txt)

[yaks-ad6d-sheds-new](yaks-ad6d/yaks-ad6d-sheds-new.txt)

[yaks-ad6d-sheds-new](yaks-ad6d/yaks-ad6d-sheds-new.json)

---
▸ 2026-10-10T19:37:20Z [here-1]

Real machine (old = PATH yaks 0.0.14, new = ./target/release/yaks; `sheds` run read-only in this clone; attached above as yaks-ad6d-sheds-old.txt / -new.txt / -new.json). Old: the primary row, seen from this clone after yaks-e9bb, was measured from its own first reflog entry (154 new / 23 moved / 172 notes). New: the primary row is `vs origin/main` (refs/remotes/origin/main = dc9da87): `6 new, 2 moved, 35 new notes`, i.e. what the primary has not pushed (its HEAD is 4+ commits ahead of the clone's fork, plus a working tree). This clone is now the `*` row with `(here)`, measured from its fork like any other; JSON has exactly one `current: true`. Its `+0 -0` is by construction (own HEAD against itself).

Audit of every farm.sheds(...) caller (src/main.rs, src/farm.rs):
- main.rs enter_shed (`--shed <q>`): KEEP the current shed; naming the shed you are in is a no-op (returns without chdir, so a subdirectory cwd is kept).
- main.rs Command::Sheds: KEEP, lists everything, the current row marked.
- main.rs Command::Changes: KEEP; no SHED means the current shed; `changes <own name>` works.
- main.rs Command::Answer (`<id>@<shed>`): KEEP for resolution; when the resolved shed is the current one it is a plain `answer` (answer_in_shed would refuse it as "shares this checkout's farm", and there is no "another checkout" to write into).
- main.rs Command::Show --sheds -> farm.rs show_sheds: EXCLUDE the current shed via the new private Farm::other_sheds (sheds() minus current): "same in N sheds" counts the others, this checkout's copy is the block above.
- main.rs Command::Inbox --sheds -> farm.rs inbox_sheds: EXCLUDE the current shed (the plain inbox is this checkout's).
- farm.rs Farm::sheds / shed_changes: thin wrappers; Farm::sheds is now documented as including the current one.
- src/discover.rs (`yaks discover`) calls shed_paths (others only, prints "this checkout excluded"): unchanged on purpose, it is a discovery diagnostic, not the shed list.
- ahead/behind and the shallow note: the current row shows +0 -0; the shallow note ignores the current row.

---
▸ 2026-10-10T19:37:20Z [here-1]

Decision: the primary's baseline is refs/remotes/origin/main, else the target of refs/remotes/origin/HEAD; with no usable ref it falls back to the fork point and says so: text ` (vs fork)` (only for the primary), JSON farm.vs "fork". New farm.vs value "origin", text ` (vs origin/main)`. Rejected: (a) a bare `origin/main` rev lookup (it also matches a LOCAL branch of that name; the full refs/remotes path cannot be fooled); (b) silently using the fork point for a primary with no origin ref (the number would pass for "unpublished"; it is the flood we are fixing); (c) merge-base / `this checkout` fallback (viewer-relative).
Decision: the primary is measured from origin from EVERY viewer (found.primary), not only when the viewer is the primary. Rejected: viewer-only (that is the cwd-dependence Decision 1 forbids).
Decision: `yaks changes` uses the same marker rule (Changes gets a `primary` field to say `(vs fork)` vs the plain unmarked fork case); changes --json vs/base follow farm.vs (so "origin").
Decision: `main` resolves to the primary whether or not it is current; the yaks-3344 guard stays but fires only when no primary is visible, reworded "`main` is the primary checkout, and it is not visible from here". Rejected: dropping the guard (from a primary-less Delta clone `main` would fall through to the branch rule and pick a clone).
Decision: the current row is measured from ITS fork point like any other, ahead/behind included (+0 -0, not hidden). Its farm state is never `Shared` (that now means ANOTHER shed uses the viewer's farm: root == our_farm && !current). Rejected: a special "current" farm state (a second rule for one measurement).
Decision: marking: JSON `current` (true on exactly one row) and, in the table, a `*` in a new first column on every row (blank elsewhere) plus a ` (here)` tag on the `name:` line. Rejected: tag only (not scannable) or `*` only (invisible in name:-oriented output).
Decision: `changes` with no SHED defaults to the current shed (shed: Option<String>); `--shed <current>` is a no-op and `answer <id>@<current>` is a plain answer, not errors. Rejected: an error "that is this shed" (breaks cwd-never-changes-the-answer).
Decision: the viewer is always listed even if no source records it (checkout_paths inserts `top`), so exactly one row is current from any checkout; shed_paths (others only) stays for `yaks discover`.
Decision: in the unit tests `sheds_of` now means "the OTHER checkouts" (filters current) so the ~50 existing tests keep their meaning; new tests use all_sheds_of. tests/shed_title.rs `names` filters the current row for the same reason.

---
▸ 2026-10-10T19:40:38Z [here-1]

Docs parity: grepped "OTHER checkouts|other checkouts|other sheds|No other checkouts|(or not visible" across docs/, README.md, .agents/skills/yaks, .agents/skills/yaks-tracker and src/. Every remaining hit is deliberately "elsewhere" (show/inbox --sheds, `yaks discover`, "read-only in the other checkouts", the "No open asks in other sheds." line); the sheds/changes/answer/--shed wording in docs/cli.md, README.md, .agents/skills/yaks/SKILL.md and the Sheds/Changes/Show/Answer/--shed clap help in src/main.rs now says: every checkout listed, current marked (* / (here) / JSON current), changes [SHED] defaults to the current shed, main = primary whether or not current, primary measured vs origin/main (else vs fork), no-op for naming your own shed. Real binary check (this clone): `changes` == `changes <own name>` == `--shed <own name> changes` (byte-identical); `changes main` = the primary (vs origin/main); `show --sheds`/`inbox --sheds` do not list this clone. Pre-existing unrelated warning: unused mut in src/tui/docshots.rs:259 (untouched).
Summary: yaks sheds lists every checkout with the viewer marked; the primary is measured from origin/main from every viewer; every selector accepts the current shed (changes with no argument, --shed, answer <id>@<current>, main); show/inbox --sheds still exclude the viewer. Tests: 5 new/rewritten unit tests + 3 CLI tests fail-first then pass; full `cargo test --release` all suites ok after the last code edit.

---
▸ 2026-10-10T19:41:13Z [here-1]

verify: `cargo test --release` -> PASS (exit 0)

---
▸ 2026-10-10T19:41:37Z [here-1]

Correction to the summary above: of the 3 new CLI tests in tests/changes.rs, 2 fail on the old src/ (no_argument_means_the_current_shed_and_every_selector_accepts_it: left 0 right 1 current row; answering_names_a_shed_and_the_current_one_is_a_plain_answer: no current row, Option::unwrap on None) and pass now; inbox_and_show_sheds_still_mean_the_other_sheds is a regression guard that passes before and after, by design (show/inbox --sheds must keep excluding the viewer). Checked by stashing src/ and running `cargo test --release --test changes`.

---
▸ 2026-10-10T19:43:03Z [here-1]

moved: shaving -> shorn
