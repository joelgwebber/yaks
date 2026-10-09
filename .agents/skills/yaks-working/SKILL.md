---
name: yaks-working
description: "Use when you are taking one yak from hairy to shorn, as a worker handed one by a yakherd or on your own: re-read it, claim it, keep notes, gather evidence, shear, commit. Minimal and harness-agnostic. Opt-in skill, installed by `yaks skills install --with coordination`."
---

# Working a yak (experimental)

Repo-internal conventions for working a single yak with a trail a human or
another agent can trust later. Deliberately minimal. Prefer the smallest habit
that keeps the farm honest; do not grow this into a heavyweight flow.

Run the yaks CLI directly, with a binary you trust. In a checkout of this repo build your
own (`cargo build --release`, then `./target/release/yaks`; a `yaks` on `PATH` may be older
or absent); if your brief gives an absolute path, use that. Never `npx` or install one.

## Before you start

1. **Re-read the yak now, not from memory.** `yaks show <id>`. A human or another
   agent may have added notes, left feedback, or moved it since you last looked.
   Read the latest note first; it is the freshest signal.
2. **Confirm it is actually ready.** Dependencies resolved (`yaks next` /
   `yaks tangled`), and no note asks for something to happen first. If a note
   redirects the work, follow it or ask rather than pressing on.
3. **Claim it.** `yaks shave <id>` moves it to shaving and signals to everyone
   sharing the farm that it is taken. Shave its parent too if the parent is still
   hairy.

**When a decision needs a human, ask and hand back — don't block-and-wait.**
`yaks ask <id> --note "<question>"` records the question, sets the yak's `needs`
block, and drops it from `yaks next` until it is resolved; then return control
rather than spinning. Pending questions surface in the human's `yaks inbox`. Leave your
edits in the working tree when you return: do not revert them and do not park them in
`$TMPDIR` (deleted when your session ends), and say which you did in your final message.
Never answer your own ask. A yakherd may answer scope or mechanics, then wake you; you keep
your context. The answer may already be in your copy of the yak, written there from another
checkout (`yaks answer <id>@<your shed>`) and left uncommitted: that is expected; commit it with
your yak file. (Older briefs may instead tell you to record the yakherd's answer yourself with
`yaks answer`, quoting it; that is the yakherd answering through you, not you answering your own
ask.) A design fork
waits for the human. An answered ask is left as `needs: agent` ("answered, awaiting an agent"):
when you resume, or at the start of any session, run `yaks inbox --for agent`, read the answer,
and take it on with `yaks pickup <id> --note "what you will do"`, which clears it.

**Ask to *record* a judgment call, not only when you're stuck.** Beyond "ask when
you can't judge," raise a genuine design fork through `ask`/`answer` even when you
hold a clear lean — state the lean in the question and let the human's answer
stand as the decision. That turns a choice you'd otherwise make silently into an
attributed, durable thread on the yak: the fork, the reasoning, and who called it
— provenance for later yak archaeology. Reserve it for real forks (architecture,
semantics, scope), not trivia.

## While you work

- **Name yourself on every command.** `YAKS_ACTOR=<name> yaks ...` or `--as <name>`; an
  `export` does not survive between terminal calls in some harnesses. Without it, under Delta
  your notes are stamped `delta:<thread title>` (or `delta:<thread id>`), derived from its
  environment; elsewhere they are stamped with the git user, which is the human. An explicit
  name is still preferred: it is stable, where a thread title can be renamed.
- **Changes you did not make are signal, not an error.** A touched `updated:`, a yak that
  moved, a new file, a dirty working tree: a person or another agent is working in the same
  farm. Do not investigate, revert or announce it unless it changes your task.
- **Append progress as you go.** `yaks update <id> --note "what you found /
  decided / changed"`. Short, factual, one event per note. This running log is
  what future sessions and agents rely on. Anything longer than a line goes in on
  stdin as real markdown (`--note - <<'EOF' ... EOF`, quoted `'EOF'` so backticks and `$`
  stay literal; see the `yaks` skill, "Writing notes and descriptions"), never as one
  long argument. The TUI styles, one line at a time: `#` headings, `-`/`*`/`1.` list items,
  `>` quotes, fenced code, inline `code`, `**bold**`, `*italic*`; it does not style tables.
- **Log your decisions.** A choice a reviewer could argue about (a rule you picked, a default,
  scope you cut, a shape you rejected) goes in its own note starting `Decision:`: what you chose,
  the alternatives you rejected and why. A human audits you from these; a decision that is only
  in the code is invisible. The line between the two: a `Decision:` note is for a choice that is
  reversible and local to your yak's goal (a limit, a default, a name, which of two equivalent
  designs); `ask` is for anything that changes behaviour a user sees, a data format, a dependency
  or a shipped surface, or that you would want the human's say on (above), and it waits for them.
- **One writer per yak.** If two actors need to touch the same work at once,
  split it into child yaks first so each has a single owner.

## Before you shear

- **Evidence contract first.** The definition-of-done — what will count as proof —
  is authored *before* the work: by the yakherd at claim time, or by the
  working agent itself in solo mode. You shear against it, not against vibes.
- **Evidence over assertion.** Do not shear on "it compiles", a self-report, or a
  green proxy. Verify against the *real artifact* via the project's verification
  lever — drive the TUI/browser, reproduce the state, read the actual value — and
  record it in a note: a command and its output, a test name, a file path, a
  commit SHA — or `yaks attach` the artifact itself (a screenshot, a TUI frame)
  when the evidence is something to look at. Attach keeps it an **external file**
  under `.yaks/artifacts/` (committed in team mode), so it reaches the reviewer;
  never paste a large or binary artifact (SVG included) inline into a note or
  description. Note that `yaks attach` also **modifies the tracked yak file** (it
  links the artifact in the body), so expect the yak `.md` to show as changed and
  stage it deliberately alongside the artifact. Evidence is general: the outcome achieved and
  seen, appropriate to the yak's kind (a research/decision yak's evidence is its
  finding/rationale; dead yaks are exempt).
- **A UI change's evidence is what it looks like.** For any user-facing/UI
  change, `yaks attach` a **screenshot or a text serialization** of the affected
  state (a rendered frame, a headless text dump, the rendered HTML/DOM) — not
  just a passing test — and update every surface that exposes the change (in-app
  help, `--help`, docs) in the same change. A key or flag that works but is
  undocumented or invisible in help shipped half-done (the yaks-71d1 `H` key
  landed in the handler but not the `?` overlay, and edited the wrong surface,
  precisely because no one drove the real UI or attached a frame).
- **Scriptable evidence is a `verify:` command.** When the check is a command (a
  test, a build, an artifact producer), store it on the yak (`--verify`) and run
  `yaks verify <id>`: it records the PASS/FAIL as a note and, because it's stored,
  anyone (yakherd, CI, a later agent) can re-run it. `doctor --strict` then
  enforces that a shorn yak's `verify:` last passed.
- **Authoring a *new* snapshot/golden needs your tool's accept step.** A fresh
  snapshot fails its first run by design (the tool writes a `.new` and errors),
  so know the accept path before you shear — e.g. with Rust `insta`, `cargo insta
  accept` or `INSTA_UPDATE=always cargo test` (plain `cargo test` writes
  `.snap.new` and fails; one test emitting two new snapshots can't self-bootstrap
  in a single plain run).
- **No lever? Ask, don't shear on faith.** If the project has no way to verify
  this kind of change (a TUI you can't snapshot, UI you can't drive, state you
  can't reproduce), `yaks ask <id>` the human whether to go build one rather than
  shearing on trust.
- **Don't self-shear what you can't judge.** Who judges is part of the contract.
  A script judges when your `verify:` passes — self-shear. A subjective call (look
  & feel) is the human's: `yaks attach` the artifact and `yaks ask`. Something an
  agent can eyeball but you shouldn't grade your own homework on: attach it and
  hand to the yakherd. Only self-shear when the judge is a passing script or the call
  is genuinely yours.
- Write a short shorn summary (what was done, what was learned, any yaks spawned,
  the evidence) with `yaks update <id> --note` (`shorn` takes no note), then `yaks shorn <id>`.
- **Team mode:** stage the shorn yak move together with the code that completed
  it and commit them in one commit. `yaks attach` creates a NEW directory under
  `.yaks/artifacts/<id>/` that must be added too. Stage only paths that exist: `git add`
  aborts and stages nothing if one pathspec does not match (a transient `shaving/` path after
  the move is the usual culprit), so use `git add $(yaks path <id>)` or `':(glob).yaks/*/<id>.md'` and check `git show --stat`
  lists every file you expect before you rely on the commit. That commit is also what later
  lets you trace
  the change back to this yak (provenance, `yaks-2610`).
- **Private farm** (`git ls-files .yaks` prints nothing): the yak files are not in git. Never
  put yak ids or the word yaks in a commit message, code or comments, and stage only your own
  source files.
- **The repo's own commit hooks are not yours.** A repo may run pre-commit hooks
  (husky/lint-staged, formatters, test gates) on your commit. Their output — and
  any failure they raise — is the repo's, not a yaks problem: fix the flagged
  code and re-commit; don't mistake hook noise for a yaks error.

## Many workers

When a yakherd hands yaks to several workers, `yaks-coordinating` (the core, which routes to
the farm-mode and environment skills) is the multi-worker picture. As a worker you need only this file
and your brief.
