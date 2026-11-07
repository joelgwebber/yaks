---
name: yass-status
description: Report where YASS work stands. Summarize progress across active changes, what's blocked and why, decisions made recently, stale changes, and changes ready to archive, then list what needs a human decision. Read-only. Use when someone asks where things stand, what's next, or what needs them.
---

# yass-status

`yass` is the YASS CLI, on your PATH. This playbook only reads; it changes no files. Be brief: the human wants decisions, not narration.

1. **Gather** (quietly):
   - `yass status`, then `yass status <change>` for every change that isn't `not started`: its open boxes show which items are `[/]` (in progress), and it's where blocked and nearly done changes show their detail.
   - What happened recently (ask since when, or use a week): `git log --since=<date> -- <folder>` for each folder `yass root` prints (for one outside the repo, `git -C <folder> log --since=<date>`, if it's in a git repo) and the Log entries dated since then.
   - Decisions recorded in that time (the `yass-log` playbook does this well).
   - With plans kept apart from the code (`yass root -v` says `separate`), pull them first, and note what `yass status` says about them being behind or uncommitted. Run in a code repo, `yass status <change>` lists the change's merged code from commits' `yass:` trailers.
2. **One screen:**
   1. Progress per change (boxes ticked, pieces done), one line each, in the order `yass status` lists them: a yass folder's `queue.md` ranks its changes, top first. Keep changes in progress apart from ones that are planned but `not started`, and name the items marked `[/]`: that's what's actively being worked on.
   2. Blocked changes and the reason, and changes `waiting on:` others (say whether those are moving).
   3. Decisions made recently, with who made them, so the human can object.
   4. Changes `yass status` marks `done`: ready to archive.
   5. Stale changes: no activity in a while.
   6. Intent edits waiting underneath work in progress (a plan or PRD revision on its own branch), so they get reviewed first.
3. **What needs a human,** one line each, with the move that handles it:
   - **Blocked:** the question to answer. If the answer changes intent, `yass-shape` or `yass-plan` makes the edit.
   - **Disagreeing with a decision:** the reversal is recorded as a new decision, not by rewriting the old one.
   - **Ready to archive:** `yass archive <change>`.
   - **Versions out of step** (`yass status` ends with a note about the `yass` binary and the repo's YASS files): `yass-update` sorts out both.
   - **Stale or abandoned:** resume it with `yass-work`, or drop what's left (`[-]`, with a reason in Decisions) and archive it.
4. **Offer one next move.** Asked "what's next?", it's the first change in `yass status` order that isn't `done` or blocked; without a `queue.md`, say that nothing ranks the changes and offer to start one. Plan a shaped change, point `yass-work` at a specific change or piece, or shape something new. Don't make it until the human says so.
