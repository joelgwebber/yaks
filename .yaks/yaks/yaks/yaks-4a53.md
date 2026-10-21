---
id: yaks-4a53
title: 'Frontmatter-aware merge driver for farm files: concurrent edits always merge into a VALID yak'
status: shaving
type: feature
priority: 2
created: '2026-10-18T19:35:20Z'
updated: '2026-10-21T20:33:01Z'
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
