---
id: yaks-735a
title: yaks land records landed shas on the yak; yaks commits reads recorded shas
status: shorn
type: feature
priority: 2
created: '2026-10-31T16:17:01Z'
updated: '2026-10-31T17:24:18Z'
parent: yaks-bdf9
depends_on:
- yaks-7a11
labels:
- git
- cli
---

Implements the decision on `yaks-bdf9`, using the note metadata core from `7f78a28`.

## Scope

1. **`yaks land <sha>` records what it landed.** After a successful landing, for each yak id named in the landed commit's message (same id rule as `yaks scan-ids`), append a note `kind: landed`, `commit: <landed sha>`, `source: <worker sha>` with the body `landed: <short sha> <subject>`. `--dry-run` writes nothing. A private farm keeps ids out of commits, so add a repeatable `--yak <id>` that names the yak explicitly.
2. **`yaks commits <id>` reads recorded shas first:** list every note with a `commit:` key (kind, sha, branch if any, subject resolved through `git log -1` in the code repo when the commit is present), then the message-grep fallback. Run the grep in the code repository (the checkout yaks was started from), never in the farm worktree; that is the `yaks-7a11` bug. The yak file's own history on the farm branch is labelled as farm history.
3. **Docs and help:** `docs/cli.md` (`land`, `commits`) and `--help`. Not `.agents/skills/`.

---
▸ 2026-10-31T16:17:15Z [yakherd]

moved: hairy -> shaving

---
▸ 2026-10-31T16:17:16Z [yakherd]

Claim (yakherd): done by `commits-1` after `yaks-7a11`. Evidence: a land test that checks the `kind: landed` note (commit, source); `commits` lists recorded shas first.

---
▸ 2026-10-31T16:45:27Z [commits-1]

Progress (commits-1): implementation and tests written; gate next.

- `Farm::record_landed` (next to `Farm::commits`): under the farm lock, appends a `kind: landed` note (`commit`, `source`, `branch`) via `append_note_meta`, sets `updated:`; skips a yak that already has a `landed` note for that commit.
- `land::record` runs after a landing: ids from the landed message (`refs::scan_text` against `store::all_ids`) plus `--yak`; unknown ids warn.
- `commits`: new "Recorded" section first (subject resolved with `git log -1` in the code repo, else `(not in this repository)`), and `--json` with `recorded`/`named`/`farm_history`.

Decision: the recorded `commit:` is the full landed sha; for a SHA already an ancestor of HEAD it is that sha and there is no `source`. Rejected: recording nothing there (the landing happened earlier, and `--yak` is the only way to link on a private farm).

Decision: a run that makes no new commit ("HEAD's tree already equals the computed merge") records nothing and now returns early after saying `landed: <HEAD oneline>`. HEAD there may be an unrelated commit, so a note would link the yak to the wrong code. The first landing already recorded it.

Decision: landing the same commit twice never writes a second note (a `landed` note with the same `commit` is the dedupe key). Rejected: always appending (noisy and the second landing is a no-op anyway).

Decision: with no farm (or an unopenable one) `land` stays silent unless `--yak` was given, then warns. Landing never needed a farm and must not start failing for want of one; a failure to record only warns, because the change is already landed.

Decision: `--dry-run` prints `dry run: would record a kind: landed note on <id> for the landed commit (worker <sha>)` and takes no farm lock.

Decision: the `branch:` metadata is HEAD's branch (omitted when detached). It is optional in the design and costs nothing; `commits` shows it as `on <branch>`.

---
▸ 2026-10-31T16:47:06Z [commits-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-31T16:47:07Z [commits-1]

Summary (commits-1): `yaks land` records what it landed; `yaks commits` reads it first.

- `Farm::record_landed` + `land::record`: a `kind: landed` note (`commit` = full landed sha, `source` = full worker sha, `branch`) on each yak the landed message names and each `--yak <id>`; unknown ids warn; `--dry-run` writes nothing and says what it would record; a re-land or a no-op land records nothing new.
- `Farm::commits` gains `recorded: Vec<RecordedCommit>` (kind, sha, source, branch, ts, actor, `found` from `git log -1` in the code repo); `yaks commits` prints "Recorded", then "Commits naming", then "Farm history"; `--json` has `recorded`, `named`, `farm_history`.
- Docs and help: `docs/cli.md` (`commits`, `land` and both table rows), `README.md`, clap help on `Commits` and `Land`/`--yak`. `.agents/skills/` untouched.
- Tests in `tests/commits.rs`: land records landed + worker sha, `commits` reads them (text and JSON), re-land adds no second note, `--yak` with an unknown id warns, dry run writes nothing, a recorded sha the repo lacks shows `(not in this repository)`.
- Real output: a demo repo `land` then `commits` printed `landed 33413da on main  t-09e7: the work  (2026-10-31T16:38:37Z [demo])`.
- Gate: `cargo test --workspace` (fsmonitor env) green; `yaks verify` PASS.

---
▸ 2026-10-31T16:47:07Z [commits-1]

moved: shaving -> shorn

---
▸ 2026-10-31T16:52:20Z [yakherd]
kind: landed
commit: 7e4a0147b919a5f23e743cfbee7c77f087aefdb3
branch: main
---

landed: 7e4a014 yaks-735a: yaks land records landed shas on the yak; yaks commits reads them first (--yak, --json) (recorded by hand: landed with a yaks binary from before `yaks-735a`)

---
▸ 2026-10-31T17:24:18Z [yakherd]
kind: landed
commit: b5abcb7acdbf03265dd0bea50bd91fcfcd460281
branch: main
---

landed (rebased onto `c2b640f`, your checkout's `main`): b5abcb7 yaks-735a: yaks land records landed shas on the yak; yaks commits reads them first (--yak, (was `7e4a014`)
