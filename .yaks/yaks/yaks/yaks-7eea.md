---
id: yaks-7eea
title: 'Open the TUI on another shed: a shed picker (the CLI selector, -C and --shed, shipped)'
status: shorn
type: idea
priority: 2
created: '2026-10-07T01:00:44Z'
updated: '2026-10-27T01:10:50Z'
parent: yaks-ee0a
labels:
- cli
- ui
- delta
needs: agent
---

Raised by Joel (2026-10-06): to look at a Delta thread's farm in `yaks tui` you must run it from that thread's worktree, and copying the path out of the Delta UI by hand is "kind of a PITA". How do we make opening the TUI on the right shed ergonomic?

## Status (2026-10-10)
Built and released in v0.0.14, so this yak no longer needs a CLI answer:
- Discovery from inside any sibling checkout, git data only (yaks-c635); no scan of Delta's roots, no per-user registry or symlink directory (Joel's decision, in the notes; not to be revived without new evidence, and see the yak below this one for the case that is new evidence).
- NAMES: a shed is named by the slug of its Delta thread title (yaks-886a).
- The selector: global `-C <path>` (yaks-4e39) and `--shed <name>` (yaks-440d), full access (Joel's decision, tighten later only if it proves problematic). Every command takes it, `yaks tui` included.

## Still open
- A shed picker inside the TUI (the unified view, yaks-7204): open `yaks tui` in the main checkout, press a key, choose a shed, and the TUI re-roots to it, with a banner naming the shed because the TUI there has full write access. This is the one remaining option of the original brainstorm; it goes with the TUI design.
- A ready-to-paste `open:` line / `--paths` output in `yaks sheds` (option C of the brainstorm): not built; `--shed` covers most of it, decide with the TUI design whether it is still wanted.
- Showing which thread spawned which (parked 2026-10-10, see the last note).

The original options A to F and the refinements are in the notes below (they keep the word "lane" in places: read it as "shed").

---
▸ 2026-10-07T03:47:37Z [delta-lead]

Refinement (coordinator delta-lead, 2026-10-06, after Joel's two questions and the investigation in yaks-df61 O63).

Anchors that do not depend on the current directory, and what each can find:
1. The current repo (today): its `git worktree`s and the sibling / `.delta/worktrees` clones.
2. Known Delta roots, scanned and matched on `remote.origin.url`: Linux `${XDG_DATA_HOME:-~/.local/share}/delta/worktrees/*/*`, macOS `~/Library/Application Support/delta/worktrees/*/*`. A convention baked into yaks, so it is isolated, documented as a convention, and overridable by a config list of extra roots. Finds managed checkouts from anywhere.
3. A per-user registry written as a side effect of any yaks command run inside a Delta thread (DELTA_THREAD_TITLE and DELTA_CURRENT_THREAD_ID are in its environment): the title as the shed's name, and the path. Finds a shed wherever yaks has run in it.

Realize the registry as a SYMLINK DIRECTORY, not (only) a file: for example `~/.yaks/sheds/<sanitized-title>` pointing at the shed's checkout (a short id appended on a clash; refreshed on each run, keyed by thread id, so a renamed thread keeps one link). Then `cd ~/.yaks/sheds/<tab>` works from any terminal with tab completion, Zed and other tools can open the folder, and nothing needs a yaks invocation to find a thread. This also answers Joel's original goal in yaks-df61 O50: work locally in Zed and the CLI without chasing where Delta put the worktrees.

Delta CLI: no thread-to-path lookup exists (`thread status` and `thread data` carry no paths); "open a terminal at the worktree" is a Delta feature request, not something we can build.

Still open: `--shed` as a visit (read plus answer/ask) or full access; and whether to try, on a fresh unused thread, Delta's "existing local checkout" / "existing Git worktree" modes through the UI (untested, and per the app the choice cannot change once an agent has used the thread).

---
▸ 2026-10-09T03:14:28Z [delta-lead]

Joel's model of discovery (2026-10-06), checked by the coordinator; adopted as the starting point for the core.

1. git worktrees: point yaks at any one checkout (main or a worktree, any subdirectory); `git worktree list` from there sees every worktree of that repo. Correct. Team farm: each worktree has its own `.yaks/` copy, so the view is relative to the one you are in. Private farm: worktrees do not hold the farm (YAKS_DIR or a pointer).
2. Delta (a), inside a Delta worktree: the sibling scan (same root, matching alternates or origin URL) finds the other Delta clones of that repo, and on the linked layout the primary checkout too. Gap: a primary checkout's plain git worktrees are not seen from inside a Delta clone.
3. Delta (b), anywhere: needs a starting point: the platform Delta root (Linux XDG_DATA_HOME or ~/.local/share, macOS ~/Library/Application Support) gives every Delta checkout of every repo; group them by origin URL and pick one repo.
Refinements: (i) the root scan belongs in discovery ALWAYS, not only in case (b): from the primary checkout on a machine where a thread is managed (not linked), the managed clone is only found by the root scan; (ii) a coherent view needs a reference shed, because "own changes" are relative to the viewer: in (b) that means choosing the repo AND which shed is "here" (default: the primary checkout when one is found).
Brief: discovery = git worktree list + sibling scan + Delta root scan, filtered to one repo (origin URL); a selector (`-C <path>` / `--shed <name>`) sets the viewer. brf-1 (the first, stalled brief worker) reported stopped; nothing of it to land (its draft was finished as yaks-2c22).

---
▸ 2026-10-09T20:12:18Z [delta:Delta :Yaks (cont'd)]

Decision (Joel, 2026-10-09): no "magical" discovery from outside a checkout. Sheds are found from inside any sibling using git data only (host via alternates, refs/delta pins, each pin's git dir core.worktree, and the host checkout's git worktrees). Built and wired into `yaks sheds` as yaks-c635; `yaks discover` shows the chain.
Dropped from this design, and not to be revived without new evidence: (a) scanning Delta's platform roots (~/Library/Application Support, XDG_DATA_HOME); it found only managed checkouts, never the linked ones under each project folder, and nothing on disk links the two (Delta's sqlite blobs carry no paths); (b) the per-user registry and the ~/.yaks/sheds symlink directory (option B), whose only purpose was discovery from anywhere; (c) `yaks discover --root` and its outside-a-repo mode (removed).
Still open here: NAMES (the thread title rather than the opaque dir id; `who:` labels are the current proxy) and the `-C <path>` / `--shed <name>` selector over the sheds a checkout can see, plus whether `--shed` is a visit (read plus answer/ask) or full access. These are the CLI and UI affordances that come next.

---
▸ 2026-10-10T13:40:07Z [delta:Delta :Yaks (cont'd)]

Parked (Joel, 2026-10-10): showing which thread spawned which in sheds (the shared prefix of yaks.shed.thread ids, e.g. ksQQI7UGw66zRMm9tnfRIaYEapLOAA... across a yakherd and its workers, and 'delta thread list --subagents --json' lists exactly that thread's subagents) is deferred until Delta adds CLI affordances for thread hierarchy. Not built; the undocumented id format is the reason.

---
▸ 2026-10-10T18:55:34Z [delta-lead]

Two items of the original brainstorm were still open here; both belong to the TUI design, so they move to yaks-7204 (copied there as a note, not lost):
- a shed picker inside the TUI that re-roots it, with a banner naming the shed;
- a ready-to-paste `open:` line / `--paths` in `yaks sheds` (option C), only if the picker leaves a need.
Everything else is built: discovery (yaks-c635), names (yaks-886a), `-C` (yaks-4e39) and `--shed` (yaks-440d, full access). The managed-repo discovery gap moved out as yaks-6beb. Shorn as done-and-subsumed (Joel, 2026-10-10).

---
▸ 2026-10-10T18:55:34Z [delta-lead]

moved: hairy -> shorn

---
▸ 2026-10-27T01:10:50Z [Joel Webber]

requested: needs agent

Tidy request: change only the line structure of this yak's description and every note, so each renders as intended under CommonMark.

Put a blank line between blocks, make run-on facts a list, and fence pasted output. Never change a word, a note header (`▸ <ts> [actor]`) or the frontmatter. Edit the file at `yaks path yaks-7eea` directly: `update --description` replaces the whole body, notes included. Then `yaks pickup yaks-7eea --drop-request`, which removes this request note, so no new note is left behind.
