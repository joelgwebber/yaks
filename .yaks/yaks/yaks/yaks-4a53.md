---
id: yaks-4a53
title: 'Frontmatter-aware merge driver for farm files: concurrent edits always merge into a VALID yak'
status: shorn
type: feature
priority: 2
created: '2026-10-18T19:35:20Z'
updated: '2026-10-21T20:52:02Z'
parent: yaks-472e
labels:
- git
- cli
---

Needed once teammates push one farm branch (yaks-73f4). Observed: `merge=union` merges two concurrent notes on one yak without a conflict but leaves two `updated:` lines in the frontmatter (an invalid file). Revive yaks-74f5's design as `yaks merge-driver %O %A %B %P` registered via `.gitattributes` on the farm branch + `git config merge.yaks.driver` (yaks init sets it): union the note entries (dedupe by header, order by timestamp), max(`updated`), union labels/depends_on; a scalar both sides changed differently (status, priority, title, parent) never fails the merge: keep one value and record the other in a `conflict:` frontmatter field / an attributed note that `yaks doctor` flags and the TUI shows, so the farm is always valid and the human resolves at leisure (Joel's idea, cf. yaks-9f5c). A `yaks sync` (pull --rebase with the driver, push, retry) wraps it.

---
▸ 2026-10-18T23:24:43Z [delta-farm]

Scope note (Joel, 2026-10-18): `yaks sync` should also cover the FIRST publish of a farm branch (`push -u origin <branch>` from the farm worktree when origin has none), so `yaks migrate --to-branch`'s printed push line becomes `yaks sync`. The code-side `git push origin main` stays the user's.

---
▸ 2026-10-19T04:26:21Z [delta:herd-pass]

Carried over from yaks-78ff (Joel, 2026-10-17): should yaks commit yak changes by itself? Under the farm branch the answer is cheap to give: the farm worktree is the only writer of .yaks, so an automatic `yaks commit` (on a timer, at TUI exit, or as part of `yaks sync`) cannot collide with code commits. Decide here with sync.

---
▸ 2026-10-21T19:54:35Z [delta:herd-dirs]

Herd pass 2026-10-21: the sync lifecycle (`yaks sync`, first publish, auto-commit, pull timing) moved to its own yak, yaks-472e. This yak is now just the merge driver; yaks-472e decides whether sync ships before it.

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

moved: hairy -> shaving

---
▸ 2026-10-21T20:33:01Z [delta:herd-dirs]

Claim (delta:herd-dirs, yakherd). Owner: `driver-1`.

**Goal:** `yaks merge-driver %O %A %B %P` merges three versions of one yak file into a VALID yak, never conflict markers; registered automatically for the farm worktree.

**Scope (files/symbols):**
- `src/mergedriver.rs` (exists as a stub with `ensure_registered`): add `pub fn merge(base: &str, ours: &str, theirs: &str) -> Merged` (pure, string in, string out; `Merged { text: String, conflicts: Vec<String> }`) and implement `ensure_registered(worktree)`.
- `src/main.rs`: ONE new `Command::MergeDriver { base, ours, theirs, path }` variant (hidden from top-level help is fine: `#[command(hide = true)]`) + its dispatch arm: read the three files, write the result to `ours` (git's `%A`), exit 0 always (non-zero only on I/O error).
- Reuse `store::parse_task`, `store::write::render`, `store::parse_notes`/`NoteEntry` for parsing and serialization; do not hand-roll YAML.

**Merge rules:**
- Notes (the `---`/`▸ <ts> [actor]` entries after the description): union of both sides' entries not in base, deduped by exact entry text, ordered by timestamp (stable for ties: ours first). Entries in base and kept by both stay.
- Description (body before the first note): take the side that changed it vs base; if both changed it differently keep ours and record a conflict.
- Frontmatter: `updated` = max of the two. `labels`, `depends_on`: base ∪ additions from each side minus removals from each side. Scalars (`status`, `title`, `priority`, `type`, `parent`, `source`, `verify`, `needs`, unknown `extra` keys): the side that changed vs base wins; both changed differently -> keep ours, record conflict.
- A conflict never fails the merge: append one attributed note to the merged yak, actor `merge-driver`, text `merge conflict: <field> was <ours> here, <theirs> on the other side; kept <ours>` (one line per field), using the existing note writer shape (`store::append_note` or equivalent, current schema: blank line after header). `yaks doctor` needs nothing new; a human sees the note.
- If ANY of the three does not parse as a yak (no frontmatter), fall back: write ours unchanged, print a one-line warning to stderr, exit 1 so git reports a conflict for that path (non-yak files under `.yaks/` like config.yaml get this).

**Registration (`ensure_registered(worktree)`):** idempotent. (1) `<worktree>/.gitattributes` contains `.yaks/**/*.md merge=yaks` (create or append the one line; leave other lines). (2) `git -C <worktree> config merge.yaks.name "yaks yak-file merge"` and `merge.yaks.driver "<abs path of current exe> merge-driver %O %A %B %P"` (repo-level config; the farm worktree shares the repo config, which is fine). Do NOT call it from anywhere yet; `yaks sync` (yaks-e3b2) will.

**Out of scope:** `yaks sync`, any push/pull, TUI, docs beyond `docs/cli.md` one row for `merge-driver` (mark it internal) and the clap help.

**Evidence (tests that fail before):**
- Unit tests in `mergedriver.rs`: two different notes on one yak -> both kept, one `updated:` line, parses; same scalar changed both ways -> ours kept + conflict note; label add on one side + remove on the other; description changed one side; a non-yak input -> fallback.
- An integration test in a scratch repo: farm branch, two clones (or two branches), concurrent notes + one conflicting status change on the same yak, `ensure_registered`, `git merge` / `git rebase` succeeds with no markers and `yaks doctor` is clean.

**Judge:** the yakherd, by re-running the tests and a real two-clone merge.

---
▸ 2026-10-21T20:33:51Z [driver-1]

Start: Wed Oct 21 20:33:47 UTC 2026 (date -u). `yaks path yaks-4a53`: /Users/joel/.local/share/yaks/farms/yaks-539d3edb/.yaks/yaks/yaks/yaks-4a53.md

---
▸ 2026-10-21T20:42:48Z [driver-1]

Decision: `yaks merge-driver` is handled at the very top of `main` (before `-C`, skill auto-sync and farm open) with an `unreachable!` arm in the dispatch match.
- Rejected: a normal dispatch arm after `Farm::open`. Git runs the driver on temp files with the repo as cwd, and a farm open can fail or print notes (skill top-up) in the middle of a merge.

---
▸ 2026-10-21T20:42:48Z [driver-1]

Decision: an EMPTY base (git's `%O` for an add/add of the same path) is merged as "the empty yak" instead of taking the non-yak fallback.
- Both sides' notes and labels survive, scalars that differ become a conflict note.
- Rejected: treating it as unparsable (exit 1), which would hand git a text conflict for a perfectly valid pair of yaks.
- A non-empty base that is not a yak still falls back.

---
▸ 2026-10-21T20:42:48Z [driver-1]

Decision: when both sides changed the description differently, ours is kept AND the other side's description is quoted (`> ` lines) inside the `merge-driver` note.
- Rejected: only naming the field, which silently drops the other text from the working tree (only git history would hold it).

---
▸ 2026-10-21T20:42:49Z [driver-1]

Decision: a conflict note also bumps `updated` to the note's timestamp (`max(updated, note ts)`); the clean path is exactly `max` of the two sides.
- Rejected: leaving `updated` older than the note the driver just wrote.

---
▸ 2026-10-21T20:42:49Z [driver-1]

Decision: `merge` stays the pure API the claim asked for (non-yak input returns `ours` unchanged); `run` calls a private `try_merge -> Option<Merged>` so it can tell "not a yak" (exit 1) apart.
- `merge` carries `#[allow(dead_code)]` because only tests call it.
- Rejected: adding a field to `Merged` to flag the fallback (changes the specified shape).

---
▸ 2026-10-21T20:42:49Z [driver-1]

Decision: notes are unioned as `ours` entries then `theirs` entries not in ours, then an entry that is in base but missing from either side is dropped (a deliberate removal wins), then a stable sort by timestamp string.
- Unknown frontmatter keys merge per key (a block list is one value); `id` and `created` use the scalar rule.
- Rejected: dropping the base-removal rule (it would resurrect entries one side deleted).

---
▸ 2026-10-21T20:42:50Z [driver-1]

Decision: the integration test (`tests/merge_driver.rs`) registers the driver by hand with the same attribute line and a driver command built from `cargo_bin("yaks")`, because the crate is a binary and `ensure_registered` cannot be called from `tests/`.
- `ensure_registered` has its own unit test against a real `git init` (attribute line appended to a file with no trailing newline, config set once, idempotent).
- A control test shows plain git conflicts on the same two clones without the driver, which is the "fails before" evidence.

---
▸ 2026-10-21T20:46:48Z [driver-1]

verify: `cargo test -p yaks` -> PASS (exit 0)

---
▸ 2026-10-21T20:46:56Z [driver-1]

Summary: `yaks merge-driver %O %A %B %P` (hidden) + `mergedriver::{merge, run, ensure_registered}`.
- Notes unioned and time-ordered, `updated` max, labels/deps merged, other fields by the side that changed them; a both-changed field keeps ours and adds one `merge-driver` note.
- Non-yak input: ours untouched, exit 1.
- `ensure_registered` writes `.yaks/**/*.md merge=yaks` and `merge.yaks.{name,driver}` idempotently; not called from anywhere yet.
- Evidence: 13 unit tests in `mergedriver.rs`, 4 in `tests/merge_driver.rs` (control conflict, real `git merge`, real `git rebase` across two clones with `yaks doctor --strict` clean, non-yak conflict).
- Gate `cargo test --release`: all suites ok (760 unit passed); `yaks verify`: PASS.
- Docs: one `docs/cli.md` row; clap help on the hidden variant.

---
▸ 2026-10-21T20:46:57Z [driver-1]

moved: shaving -> shorn

---
▸ 2026-10-21T20:52:02Z [delta:herd-dirs]

Landed (delta:herd-dirs): 0b114a4 (single-parent, tree equal to the merge; Squashed-from c9f516f). Review run by the yakherd with its own build, two real clones of a farm branch on a bare origin:

- concurrent: A notes + shaves, B notes + slaughters the same yak
- without the driver: `pull --rebase` conflicts ("Could not apply ... # B")
- with the driver: rebase succeeds, stderr `both sides changed status; kept this side, the other recorded in a note`; both notes kept, ONE `updated:` line, `doctor` clean, a `[merge-driver]` note `merge conflict: status was shaving here, dead on the other side; kept shaving`.

Follow-up for yaks-e3b2: `ensure_registered` writes `.gitattributes` uncommitted; sync must commit it on the farm branch (and the rule must be on the branch so every clone gets it).
