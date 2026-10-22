---
name: yaks-working
description: "Use when you are taking one yak from hairy to shorn, as a worker handed one by a yakherd or on your own: re-read it, claim it, keep notes, gather evidence, shear, commit. Minimal and harness-agnostic. Opt-in skill, installed by `yaks skills install --with coordination`."
---

# Working a yak

The habits that leave a trail a human or another agent can trust. The `yaks` skill has the commands; this is the discipline. If a yakherd briefed you, the brief wins where it is more specific.

**Use a binary you trust.** In a checkout of yaks itself, build your own (`cargo build --release`, then `./target/release/yaks`); otherwise the path your brief names, or `yaks` on `PATH`. Never `npx` or install one. **Name yourself on every command** (`YAKS_ACTOR=<name> yaks ...` or `--as <name>`): an `export` may not survive between terminal calls.

## Start

1. **Re-read the yak now:** `yaks show <id>`, latest note first. Someone may have moved it, redirected it or answered a question since.
2. **Check it is ready:** dependencies met, no note asking for something first.
3. **Claim it:** `yaks shave <id>` (and its parent if hairy). Skip this if your brief says it is already shaving.
4. `yaks path` (no arguments) prints the farm you are on. If it is not the one your brief names, stop and ask.

## While you work

- **Notes as you go,** one event per note; anything longer than a line on stdin (`--note - <<'EOF'`).
- **Log decisions:** a choice a reviewer could argue about (a default, a name, scope you cut) gets its own note starting `Decision:`, with the alternatives you rejected.
- **Ask, don't guess, on a real fork:** behavior a user sees, a data format, a dependency, a shipped surface. `yaks ask <id> --note "<question and your lean>"`, leave your edits in the working tree (never revert them, never park them in `$TMPDIR`), and return. Never answer your own ask. When woken, `yaks inbox --for agent` shows the answer; `yaks pickup <id> --note "what I'll do"` takes it on.
- **One writer per yak.** Touch only your yak. Shared work gets split into child yaks first.
- **Changes you didn't make are signal.** A yak that moved, a new note, a dirty file: someone is working beside you. Don't revert or report it unless it changes your task.

## Before you shear

- **Shear against the evidence contract** (in the claim note, or written by you first when solo), not against "it compiles".
- **A command proves it:** `yaks verify <id>` runs the yak's `verify:` and records PASS/FAIL.
- **Something to look at** (a UI, a rendered frame, a log): `yaks attach <id> <file>`. A UI change also updates every surface that describes it (in-app help, `--help`, docs) in the same change.
- **New snapshot tests** fail their first run by design; know your tool's accept step (`cargo insta accept`).
- **Don't grade your own subjective call.** A passing script lets you self-shear; a look-and-feel judgment goes to the human (`attach` + `ask`); otherwise hand it to the yakherd.
- **No way to verify it?** Ask, don't shear on faith.

## Finish

1. A summary note (what changed, what you learned, the evidence, any yaks spawned), then `yaks shorn <id>`.
2. **One commit of your own files.** Stage explicit paths (`git add -A -- <paths>`; one missing path stages nothing) and check `git show --stat`.
3. **Yak files, by farm:**
   - **Branch farm** (`yaks path` prints a farm outside your checkout): never stage anything under `.yaks/`; your yak edits are already in the farm and the yakherd commits it. The yak id belongs in the commit message.
   - **Committed farm** (`git ls-files .yaks` lists files): stage `.yaks/<herd>/yaks/<id>.md`, plus `.yaks/<herd>/yaks/<id>/` if you attached something, in the same commit.
   - **Private farm:** stage nothing from the farm, and keep yak ids and the word "yaks" out of the message, code and comments.
4. Repo hooks that fail your commit are the repo's: fix the code and commit again.

Coordinating several workers is `yaks-coordinating`; as a worker you need only this and your brief.
