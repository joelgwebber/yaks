---
name: yaks-coordinating
description: Coordinate several agents and a human over one yaks farm (claim, fan out, land, reconcile). Start here when you will hand yaks to other agents or land their work; it tells you which farm-mode and environment skills to load next. Opt-in skill, installed by `yaks skills install --with coordination`.
---

# Coordinating yaks (core)

You are the **coordinator**: the one agent that claims yaks, briefs workers, lands
their work, and talks to the human. A **worker** owns exactly one yak and follows
`yaks-working`. This skill holds what is true in every setup; two short companions
hold the rest.

## 1. Pick your layers
1. **Farm mode.** Run `git ls-files .yaks`.
   - It lists files: the farm is committed (**team**). Load `yaks-coordinating-team`.
   - It prints nothing: the farm is **private** (gitignored or stealth) or
     **out-of-tree** (a `.yaks` pointer file or symlink). Load `yaks-coordinating-private`.
   - Unsure: treat it as private. Leaking an id is worse than not using one.
2. **Environment.** Running in a Delta thread (`$DELTA_CURRENT_THREAD_ID` is set and
   you have `spawn_subagent`): load `yaks-coordinating-delta`. Using `git worktree`
   with your own checkouts: load `yaks-coordinating-worktrees`. One agent, no helpers:
   `yaks-working` is enough; stop reading.
3. Workers load only `yaks-working`. Your brief tells them which mode applies.

## 2. The design test
Keep the yaks tool unopinionated. If something is a query or mutation over the task
files, it belongs in the tool. If it assumes subagents, worktrees or an orchestration,
it belongs in a skill. The files and the CLI are the contract; coordination is prose.
Scale down gracefully: with one agent yaks is memory and a log; with many it is a
shared blackboard. The many-agent case adds writers, nothing else.

## 3. The shape: claim, fan out, land, reconcile
1. **Claim.** Before any worker starts, create or pick the leaf yaks, `yaks shave` each,
   and add a note: owner name, file scope, evidence contract and who judges (section 5).
   Claiming is the coordinator's job; the mode skill says how a claim becomes visible.
2. **Fan out.** One worker per yak, from the freshest state. Every brief meets the
   brief contract (section 6).
3. **Land.** Only you land work, one worker at a time, from the SHA in the worker's
   final message, after the checks in section 7. The environment skill has the mechanics.
4. **Reconcile.** A yak still `shaving` after the batch is stalled: `yaks regrow` it or
   re-run it. Run `yaks doctor`.

## 4. Scope work so it cannot collide
One writer per yak; if work is shared, split it into child yaks first. Separate before
you serialise.
- **Types, not only files.** A change to a shared type (a struct with exhaustive
  constructions) breaks the other lane's files at merge. Put it in a coordinator prep
  commit first, or keep it in one lane. Scan before fanning out: `grep 'TypeName {'`.
- **One big file is unavoidable?** Scope each lane to a disjoint function or region, prefer
  yaks that add no field to a shared struct, anchor briefs by symbol (`fn name`), never by
  line number, and warn lanes that a shared render path can ripple into snapshots they must
  re-accept and eyeball.
- This is technique, not architecture advice; whether the file should be split is its own yak.

## 5. Evidence contract and judge
State in the claim note what counts as done and who decides:
- a script (`verify:` on the yak, or the label default in `.yaks/config.yaml`): the
  worker self-shears when it passes and you re-run it;
- a subjective call (look and feel): a human, via `yaks ask`;
- otherwise you, at landing.
If you cannot say what acceptable evidence is, the lever is missing: `yaks ask` the human
about building one before you fan out.

## 6. The brief contract
Every worker brief states: the yak id and that it is already `shaving`; the worker's name
and that EVERY yaks command is prefixed `YAKS_ACTOR=<name>` (each terminal call may be a
fresh shell; an `export` does not carry over); how to get a working `yaks` binary (build in
the checkout, or an absolute path; never `npx` or an install); scope by file and symbol;
the evidence contract and judge; forbidden moves (no dependency, config or `.gitignore`
edits; no path dependencies; no push); how to finish (one commit, which paths to stage);
what to do when blocked (`yaks ask`, leave edits in the working tree, never revert and never
park work in `$TMPDIR`); and the final-message format (SHA, gate output, timings, surprises).
The environment skill carries a fill-in template.

## 7. Landing checks (every mode, every environment)
- Re-run the worker's gate yourself, in your checkout. If you cannot reproduce a claim, the
  work is rejected, not annotated.
- Reproduce the key scenario with your own build; read the diff for quality (duplicated
  logic, missed docs). Make your changes as follow-up commits so the worker's commit stays
  intact.
- Update-every-surface check: grep for the old behaviour across docs, skills, README and
  `--help` text before accepting.
- `yaks doctor` after every landing. It checks farm integrity (for example a yak present in two
  status directories, or a broken reference). "All clear" is a pass; any issue it lists is yours
  to fix before you move on.
- Never write a claim into a yak that you did not observe.

## 8. Ask and answer
- A worker that needs a human decision runs `yaks ask <id> --note "..."`, leaves its edits
  in place, and returns. It does not block.
- You answer **scope and mechanics** yourself (`yaks answer`), and say in the note that you
  are the coordinator. For a **real design fork** you do NOT run `yaks answer`: leave the ask
  open (it sits in the human's `yaks inbox`), add your lean as a note, and tell the human. The
  answer is the record; a message to the worker is only the wake-up.
- **Where the line is.** *Scope and mechanics* are reversible, local to the yak's stated goal,
  and nothing a reviewer would argue about later: may a worker touch one more file, which
  command stages a path, which of two equivalent test shapes to use. A *design fork* changes
  behaviour or semantics a user sees, a data format, a dependency, a shipped surface, or
  anything a reasonable reviewer would want a say in. If you are unsure, treat it as a fork.
- Raise a genuine design fork through `ask` even when you hold a clear lean: the decision
  becomes an attributed thread on the yak. Do not settle it in chat only.

## 9. Attribution
Notes carry the actor: `--as <name>` or an inline `YAKS_ACTOR=<name>` on each command. With
neither, a note written under Delta is stamped `delta:<thread title>` (else
`delta:<thread id>`) from Delta's environment, and elsewhere with the git user, which is the
human; an explicit name is still preferred (stable, where a thread title can be renamed). Status moves (`shave`,
`shorn`, `regrow`, ...) take the same `--as`/`YAKS_ACTOR` and append a `moved: <from> -> <to>` entry to the
yak, so the file says who moved it and when; `yaks log` lists them. They are not evidence: a shorn yak still
needs a real note (`yaks doctor --strict`). Git authorship is the human in most setups. Because a move now
edits the yak file, never edit a yak a worker owns while it runs: its landing would conflict with you.
Attribution is never ownership.

## 10. Drift is signal
The human edits yaks while you work: a touched `updated:`, a yak that moved, a new file.
That is a person working beside you, not an error. Do not revert, restage, investigate or
report it unless it changes your task. Stage only the yak files you touched.

## 11. Pre-flight before fanning out
- Scanned shared types; one writer per yak; briefs anchored by symbol.
- Workers start from the freshest committed state (mode skill: what "freshest" means).
- Evidence contract and judge written in each claim note.
- Coordinator's own tree clean enough that a landing cannot be mistaken for your edits.
