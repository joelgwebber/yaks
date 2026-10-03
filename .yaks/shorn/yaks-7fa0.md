---
id: yaks-7fa0
title: Line-join in vim mode should leave whitespace
type: bug
priority: 3
created: '2026-08-29T17:28:43Z'
updated: '2026-10-03T14:44:19Z'
---

The vim 'J' affordance tacks one line onto the previous, but doesn't leave whitespace, as is the default in vim, and the appropriate norm for this kind of text.

---
▸ 2026-10-02T21:07:57Z [Joel Webber]
Found that the join-lines logic is in the edtui-jagged dependency (Lines::join_lines method). Need to check that implementation for the whitespace handling.

---
▸ 2026-10-02T21:08:38Z [Joel Webber]
Found the bug! In edtui-jagged src/jagged.rs, join_lines() appends without space. Line 159-164 in the jagged.rs file. Need to insert a space before append: self.data[row_index].push(' '); before the append().

---
▸ 2026-10-02T21:10:35Z [Joel Webber]
Successfully fixed! Added join_lines_with_space() helper to edtui/src/helper.rs that inserts a space before joining lines. Updated all 4 calls to join_lines in edtui/src/actions/delete.rs to use the new helper. Build successful.

---
▸ 2026-10-02T21:12:21Z [Joel Webber]
All tests pass including new join_lines_vim_adds_space test. Fix complete and verified. Ready to shear.

---
▸ 2026-10-02T21:13:28Z [Joel Webber]
Merged to parent thread. Cargo.toml points to local .edtui (gitignored), which is in the worker worktree but not replicated here. Decision needed: (1) Commit the edtui changes upstream to github.com/joelgwebber/edtui, merge the PR, then revert Cargo.toml to git remote + update branch/SHA in Cargo.lock, OR (2) keep the path dependency for now, document the workflow, and coordinate with edtui upstream. Current state passes all tests in the worker's worktree.

![evidence-7fa0](artifacts/yaks-7fa0/evidence-7fa0.txt)

---
▸ 2026-10-03T14:44:19Z [delta-agent]
CORRECTION to the notes above: the earlier commit (b14a9a3) pointed Cargo.toml at a gitignored local .edtui path and dropped the lockfile source, so it could not build on CI or a fresh clone; its edtui change was never pushed anywhere, and it changed all four join_lines callers (so Backspace/Delete/delete-word would also have gained a space). Redone properly: (1) edtui fork branch yaks/join-lines (19100c3) changes ONLY JoinLineWithLineBelow, modelled on vim J: drop next line's leading whitespace, insert one space unless current line is empty/ends in whitespace or next line is empty/starts with ')'; cursor lands on the join point. Merged into yaks-integration (6ba0dbf) and pushed. (2) yaks: Cargo.toml/.gitignore restored to the git dependency, Cargo.lock bumped via cargo update -p edtui (the other lockfile lines are cargo's own re-resolution). (3) Rewrote the yaks-side test to press Enter (the old one typed a literal newline char) and cover indent stripping. Evidence attached: test fails before (hello    world) and passes after, 321+28 yaks tests green on the real pinned git dep, 164+22 edtui tests green. Not covered: vim count prefix (3J) — the fork's J is not count-aware; separate yak if wanted.
