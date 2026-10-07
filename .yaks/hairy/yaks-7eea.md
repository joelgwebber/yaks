---
id: yaks-7eea
title: 'Open the TUI on a shed from anywhere: discovery, names and a --shed selector'
type: idea
priority: 2
created: '2026-10-07T01:00:44Z'
updated: '2026-10-07T03:47:37Z'
parent: yaks-ee0a
labels:
- cli
- ui
- delta
---

Raised by Joel (2026-10-06): to look at a Delta thread's farm in `yaks tui` you must run it from that thread's worktree. When the thread was cloned from the local checkout, `yaks sheds` run from ~/src/yaks lists it and its path is easy to find; when it was NOT cloned from there (a thread shared from another machine, a managed-layout mount), the sibling scan does not find it and he copies the path out of the Delta UI by hand, "kind of a PITA". How do we make opening the TUI on the right shed ergonomic?

This is a brainstorm; nothing is decided.

## What is actually missing
1. DISCOVERY from outside: a shed that is not a sibling of the viewer (different alternates, different root) is invisible to `yaks sheds`.
2. NAMES: a shed's directory id is opaque, and `who:` is a set of actors, not the thread title the human sees in Delta.
3. A way to say "open the TUI THERE" without cd'ing.

## Options
A. `yaks tui --shed <name>` (and the same selector on read commands, like `git -C`): resolves a shed by directory id, branch, actor label or a unique substring, lists candidates when ambiguous, then runs the TUI rooted at that shed's farm.
B. A per-user registry: every `yaks` command run inside a Delta thread (DELTA_THREAD_TITLE and DELTA_CURRENT_THREAD_ID are in its environment) records {path, thread title, thread id, last seen} in a small per-user file (a derived, rebuildable cache, never committed: the AGENTS.md rule for any index). Then `yaks sheds` and `--shed` can find a shed ANYWHERE yaks has run in it, and can show the real thread title as its name, with no Delta database and no Delta CLI.
C. `yaks sheds` prints a ready-to-paste `open:` line per shed (the cd and the tui command), and `--paths` for scripting (`cd "$(yaks sheds --paths <name>)"`).
D. A shed picker inside the TUI (the unified view, yaks-7204): open `yaks tui` in the main checkout, press a key, choose a shed, and the TUI re-roots to it. The long-term answer; A and B are its building blocks.
E. A Delta-side request ("open terminal at this thread's worktree"). Useful and out of our hands.
F. Today, from the agent: it can end its messages with the exact command for its own worktree.

## The question that decides the rest
Opening the TUI on another shed's farm gives full write access there (create, move, edit), which cuts across the guardrail Joel agreed in this design: only `answer` (and maybe `ask`) may write into another checkout. Today he gets that by cd'ing in by hand, with no guardrail. Should `--shed` be a VISIT (read everything, `answer`/`ask` only, a banner naming the shed) with a deliberate way to get full access (cd there), or full access?

## Lean
Build A and B together (small, read-mostly, and B fixes both discovery and the names), plus C for free; make `--shed` a visit; D comes with the unified view later.

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
