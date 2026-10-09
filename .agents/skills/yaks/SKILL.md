---
name: yaks
description: Yaks task tracking workflow. Use when a .yaks/ directory exists in the project. Provides commands and guidance for managing filesystem-native tasks stored as markdown files with YAML frontmatter.
---

# Yaks — Task tracking workflow

This project tracks work with Yaks. Tasks are markdown files with YAML frontmatter in `.yaks/`. You MUST follow this workflow to keep task state accurate.

## Running yaks

Yaks is a single self-contained binary — a plain command-line tool. Run it directly from your shell; there are **no slash commands**. Use the first invocation that works in your environment:

1. **`yaks <cmd>`** — if `yaks` is on `PATH` (installed via `npm i -g @j15r/yaks`, or a cargo-dist shell/Homebrew installer). Prefer this.
2. **`npx @j15r/yaks <cmd>`** — zero-install, if Node is available.
3. **`./target/release/yaks <cmd>`** — when working inside a checkout you've built with `cargo build --release`.

The npm package is `@j15r/yaks` (the unscoped `yaks` was taken, so it's published under the `j15r` scope); the command it installs is `yaks`. Every example below is written as `yaks <cmd>` — substitute whichever invocation works for you. The CLI is stateless: each call is independent, there's nothing to keep running.

**Starting a fresh farm.** If there's no `.yaks/` directory yet, create one with `yaks init` (run in the repo root). It scaffolds `.yaks/{hairy,shaving,shorn,dead}/` plus a `config.yaml`; tune the defaults with `--herd`, `--type`, and `--priority`. Plain `yaks init` only creates a committed (team) farm, refuses to clobber an existing one, and prints how to install the skills. **Choose the mode and install the skills in the same step:** `yaks init --mode team|private|pointer [--path <dir> --herd <prefix>] [--skills default|coordination]` creates the farm, edits `.git/info/exclude` for `private`/`pointer` (never `.gitignore`), installs the skills project-local (their directories excluded too in `private`/`pointer`), and prints one line per step. It is idempotent: re-run it with more flags and it adds only what is missing, never overwrites an edited skill, and errors (naming what would change) if `--mode` differs from the existing farm's. For `private`/`pointer` it prints the `YAKS_DIR=<farm>` a Delta or worktree worker needs in its brief.

Add `--json` to any query command (`list`, `show`, `next`, `tangled`, `search`, `stats`, `rollup`, `inbox`, `log`, `sheds`, `doctor`) for machine-readable output. `yaks create --json` also prints the new yak's id and file path, which is handy when you create a yak and immediately act on it.

## Terminology (say it right)

The three states are **adjectives** describing a yak: **hairy** (not started), **shaving** (in progress), **shorn** (done). The dead state (**slaughtered**) is hidden.

The **verbs** are the transitions, and they don't all match their state's spelling — say them the way the commands are named:
- **shave** a yak: hairy → shaving (start work).
- **shear** a yak / mark it **shorn**: shaving → shorn (finish). (Grammatically "shorn" is the past participle of *shear*, not *shave* — that's fine, it's a deliberate bit of archaic flavor.)
- **regrow**: shorn → hairy. **slaughter**: → dead. **revive**: dead → hairy.

"Yak shaving" is the background meme (endless incidental tasks); the tool's states and verbs are the precise vocabulary — prefer them over loose phrasing so humans and agents stay unconfused.

Three **containers** name where yaks live and how they group:
- a **farm** is one `.yaks/` directory — the store the CLI and TUI operate on.
- a **herd** is a group of yaks sharing an id prefix within a farm. A farm usually has one, but can hold several — create into a specific herd with `yaks create --herd <herd>`. This lets one (typically private) farm track several projects at once, each with its own prefix.
- a **family** is a yak-tree: a parent yak together with its descendants.
- a **shed** (formerly "lane") is one other checkout of the repo that a worker or the human is working in; `yaks sheds` lists them.

## Hard rules

1. **NEVER write code without an active shaving yak.** Before touching any code — even a one-line fix — you must have a yak in shaving state. If you don't, stop and `yaks shave <id>` one first (create it if needed). No exceptions.
2. **ALWAYS shear when a yak's work is done.** Run `yaks shorn <id>` as soon as the task is complete. In **team mode** (below), stage the shorn yak file alongside the code that completed it and commit them together. In **local-only mode**, never commit yak files at all.
3. **NEVER leak yak IDs into external-facing surfaces** (PR titles/descriptions, external issue trackers, and — in local-only mode — commit messages). See "Keep yaks private," below.

## Two workflows: local or team

Yaks runs in one of two modes, with different habits. A farm set up with `yaks init --mode …` already has its mode (and its git excludes) chosen. For any other farm, **figure out which mode you're in before you commit anything** — the signal is whether `.yaks/` is tracked by git:

- `.yaks/` is gitignored or otherwise untracked → **local-only** (a private scratchpad).
- `.yaks/` is committed alongside the code → **team** (a shared tracker).

To check, **use `git ls-files .yaks`** — it lists files in **team** mode and prints nothing when the farm is untracked (**local-only**). Prefer this signal: it is reliable across *every* hiding method. Do **not** rely on `git check-ignore .yaks` alone — the self-contained `.yaks/.gitignore` = `*` method ignores the directory's *contents*, not the `.yaks` entry itself, so `git check-ignore .yaks` reports "not ignored" for a fully-private farm. If you do use `check-ignore`, test a path *inside* it (`git check-ignore .yaks/config.yaml`). If a fresh checkout is genuinely ambiguous, default to local-only — the safer assumption.

**Local-only.** The yak files live only on this machine; they're planning memory, not shared history.
- Never `git add` yak files or include them in commits.
- Keep yaks invisible to everyone else: don't mention them — or their IDs — in commit messages, PR titles/descriptions, code comments, or external trackers. Describe the change in plain terms ("add retry logic"), not "shorn yak-1234".

The quickest route is `yaks init --mode private` (the farm plus a `.git/info/exclude` line, and the skills). Otherwise pick the hiding method that fits — they differ in blast radius:
- **Root `.gitignore`** (add a `.yaks/` line): simplest, but the ignore rule is itself committed, so the team sees that a farm exists.
- **`.yaks/.gitignore` containing `*`**: self-contained — the farm hides itself with no edit to the repo root. Use this **only** for a plain, non-nested local-only farm; the `*` also blinds any git repo *inside* `.yaks/`, so it's the wrong tool for the multi-machine pattern below.
- **`.git/info/exclude`**: per-repo and untracked, so nothing about the farm touches the committed tree. This is the right choice when `.yaks/` is itself a nested repo.
- **Global `core.excludesFile`**: ignore `.yaks/` across every repo on the machine at once — handy when you keep private farms in many projects.

> **Footgun:** `git clean -fdx` in the outer repo deletes an ignored/excluded `.yaks/` (and, in the nested case, its git history) — `-x` sweeps ignored files too. A gitignored `.yaks/` is also **not** carried into fresh clones or other git worktrees, so it's simply absent there. Push a private farm often, or you can lose it.

**Local-only across machines.** To sync a private farm between machines without committing it to the code repo, give `.yaks/` its **own** git repo on a private remote, nested inside the project:
- `cd .yaks && git init`, add a private remote, and commit the farm there. Run all farm git ops from inside `.yaks/`.
- Hide the nested repo from the **outer** repo with `.git/info/exclude` — never the `*` trick, which would also blind the farm's own repo. The outer repo then ignores `.yaks/` cleanly instead of flagging it as an embedded repo.
- Habit: **pull before, push after** a work session so machines stay in sync. yaks needs no configuration for this — discovery finds `.yaks/` exactly as always (from the repo root or any directory below it).

**Several repos, one farm (out-of-tree).** To track several projects in one private farm that lives *outside* their repos, point each repo at it rather than giving each its own `.yaks/`:
- **Pointer file (recommended):** `yaks init --mode pointer --path <dir> [--herd <herd>]` creates the farm there if absent, writes the pointer and excludes it. By hand: put a `.yaks` *file* (not a directory) at the repo root with `path: <path to the shared .yaks>` and, optionally, `herd: <this repo's herd>`. `path` may be absolute, `~/`-relative, or relative to the repo. Discovery follows it, so every `yaks` command in that repo operates on the shared farm — and `yaks create` (with no `--herd`) routes new yaks into that repo's herd automatically.
- **Symlink (zero-config):** alternatively symlink `.yaks` → the shared farm. Discovery follows it too, but every repo then shares one config herd, so pass `yaks create --herd <herd>` per repo.
- Keep the pointer/symlink out of the shared repo (`.git/info/exclude`), exactly like a private farm. Consolidate existing separate farms into the shared one with `yaks merge`.
- **Discovery stops at the git top-level.** `yaks` walks up from the cwd but not past the first directory with a `.git` unless that directory has a `.yaks` entry, so a nested checkout (a Delta checkout, an in-tree `git worktree`) does not reach another checkout's farm by accident; it errors, naming the fixes: `yaks init`, a `.yaks` pointer file, or `YAKS_DIR=<farm>` (the `.yaks/` dir, a directory containing it, or a pointer file; wins over the walk).

**Team.** The yak files are part of the repo — treat them like code.
- Commit the shorn yak move together with the code that completed it (hard rule 2).
- Yak IDs are fine in commit messages and other in-repo references — collaborators can resolve them from the committed files.

> This repo is **team mode**: `.yaks/` is committed, and yak IDs in commit messages are expected.

## Keep yaks private (external surfaces)

Whichever mode you're in, yaks are a **private, fine-grained layer**. Keep yak IDs and `[yaks:…]` markers out of anything a broader audience reads — **pull-request titles/descriptions and external issue trackers** (Jira, Linear, GitHub Issues). This one keeps going wrong under light guidance, so treat it as firm: leaking a yak ID upstream is almost never right.

Enforce it mechanically with **`yaks scan-ids`**: it reads a file and/or piped stdin and exits non-zero if any token is a real yak-id in this farm (printing each as `line:col  id`). Wire it into a pre-commit or PR hook to catch a leaking id before it ships — especially valuable in local-only mode, where yak IDs must stay out of commit messages too.

Most projects that use an external tracker don't use yaks team-wide; yaks roll **up** to those issues. When a PR or issue needs a reference, use the **external** key, not the yak ID: run `yaks rollup --keys` over the shipping set and paste that — the forge links the PR to the issue natively. The **yaks-tracker** skill covers this projection in full.

## You are working alongside a human

The farm is shared. A human may be editing yaks in the `yaks tui` (or by hand) **while you work** — creating yaks, moving them between states, jotting notes. So expect **working-tree drift in `.yaks/`** that you didn't cause: a touched `updated:` timestamp, a yak moved `hairy ↔ shaving`, a new file you didn't create. This is **normal and expected**, not an error to flag or fix.

- Don't revert, restage, or "clean up" `.yaks/` changes you didn't make.
- When committing (team mode), stage **only** the specific yak files your work touched — never `git add .yaks` wholesale.
- Treat such drift as signal, not noise: a yak the human just moved to shaving, or a note they added, often tells you what they care about right now. If it seems to redirect the work, follow it or ask.

## Asking a human (ask / answer / inbox)

When you hit a decision only a human should make — an ambiguous requirement, a risky tradeoff, a missing credential — **don't guess**. `yaks ask <id> --note "the question"` blocks the yak on a human: it sets the `needs` field, records your question as an attributed note, and drops the yak out of `yaks next` so you (or another agent) won't pick it back up while it waits. The human replies with `yaks answer <id> --note "the decision"`, which records the reply and sets `needs: agent` ("answered, awaiting an agent"): the yak is no longer blocked (a hairy one is back in `yaks next`, marked answered) and, whatever its status, it stays in `yaks inbox` until an agent takes it on with `yaks pickup <id> [--note "what you'll do"]`, the only command that clears `needs: agent`. (`yaks answer --done` clears `needs` outright, for an answer that needs no follow-up.) `yaks inbox` has two sections, "awaiting a human" (`needs: human`; a yak whose latest note after the ask is from someone other than the asker is marked `↩ replied`: a plain-note reply that skipped `answer`) and "answered, awaiting an agent"; `--for human|agent` narrows it and `--json` carries each yak's `needs` and `replied`. `yaks inbox --sheds` also lists the open asks in the *other* sheds (worktrees and Delta clones) of the repository, each with the shed's name and the question text, read-only; use it to see what your workers are waiting on. **At the start of a session, run `yaks inbox --for agent`**: anything there is a decision a human made that nobody has acted on yet. Asking a yak that is already `needs: agent` is allowed (it flips back to `human`). Attribute the note with `--as <actor>` (or set `$YAKS_ACTOR`) so the log shows who asked; under Delta an unnamed note is stamped `delta:<thread title>`, but an explicit name is still preferred.

In `yaks tui` the same flow is one key: `a` asks or answers the selected yak in place, an **Inbox** view lists the `needs` queue (answered yaks carry a ✅ badge, blocked ones a ⏳ badge with a warning accent; `a` on a yak awaiting a human answers it, which sets `needs: agent`), and `m` marks rows for a bulk state change over the selection.

## Workflow

1. **Session start** — run `yaks list` and `yaks next` to see current state.
2. **Before writing code** — `yaks shave <id>` (create the yak first if needed).
3. **While working** — append progress notes with `yaks update <id> --note "what you found / decided / changed"` (one short line), or pipe a multi-line note in on stdin (see **Writing notes and descriptions**). This builds a running log in the markdown body so future sessions have context.
4. **When the work is done** — gather evidence (see **Evidence before you shear**), append a brief shorn summary (what was done, what was learned, the evidence, any yaks spawned), then `yaks shorn <id>`. In team mode, stage the shorn yak move together with the code and commit them in one commit whenever practical.

## Writing notes and descriptions (markdown)

Yak bodies are markdown, and the TUI (detail pane and editor) styles them **one line at a time**, so structure has to be on its own lines. A heading, list items and a code fence crammed into one long line render as a single wrapped paragraph, which is nearly unreadable.

- **Pass multi-line text on stdin or from a file, not as one inline argument.** `yaks update <id> --note - <<'EOF'` followed by the text and a closing `EOF` (a quoted `'EOF'` keeps backticks and `$` literal). The same works for `yaks create --description -`, `yaks ask --note -` and `yaks answer --note -`; `--note-file PATH` and `--description-file PATH` read a file. Inline `--note "..."` is for one short line.
- **What the TUI styles:** headings (`#` to `######` plus a space, alone on a line), list items (`-`, `*`, `+` or `1.` / `1)`, one per line), `>` quotes, fenced code blocks (``` or `~~~`, each fence alone on its line), inline `code`, `**bold**` and `*italic*`. A blank line separates paragraphs. Yak ids and URLs in the body become followable links.
- **What it does not style:** tables (they show as plain text), HTML, nested constructs beyond list indentation. Prefer a list to a table, and `yaks attach` for images and long output.
- **One event per note, the fact first.** Quote a command and its real output in a fenced block, and keep a `Decision:` note (the choice and the alternatives rejected) separate from progress notes.

## Evidence before you shear

A yak is shorn against **evidence, not vibes**. "It compiles", a green proxy, or a self-report is not enough — verify against the **real artifact** and record the proof on the yak so a human or a later agent can trust it (and re-run it).

- **Scriptable check → a `verify:` command.** When the proof is a command (a test, a build, a linter), store it (`yaks update <id> --verify '<cmd>'`) and run `yaks verify <id>` — it records the PASS/FAIL as a note, and because it's stored, a reviewer, CI, or a later agent can re-run it. `yaks doctor --strict` enforces that a shorn yak's `verify:` last passed.
- **User-facing / UI change → attach what it looks like.** Drive the real UI (don't infer it from the diff) and `yaks attach` the evidence: a **screenshot** of the rendered state, or a **text serialization** of it (a headless frame, the rendered HTML/DOM, a snapshot dump). "The tests pass" does not prove a UI actually renders or behaves. And update **every surface the change is exposed through** — in-app help, `--help`, and docs — in the *same* change: a key or flag that works but is undocumented or invisible in help shipped half-done.
- **No lever to verify this kind of change?** `yaks ask <id>` the human whether to build one, rather than shearing on faith.

`yaks attach` keeps evidence as an external file under `.yaks/artifacts/<id>/` (committed in team mode) and links it in the body — so never paste a large or binary artifact (an SVG included) inline into a note. (It also edits the tracked yak `.md`, so stage that alongside the artifact.)

## Parent/child state rules

A parent yak's state should reflect its children:
- When you shave a child, shave the parent too (if it's still hairy).
- When you shear the last unshorn child, shear the parent too.
- NEVER leave a hairy parent with shorn children — that means work was done but the parent doesn't reflect it.

## Labels — keep them few and purposeful

Labels are for slicing the farm later (`yaks list --label ui`), not for elaborate classification. Absent discipline, agents invent sprawling taxonomies that help no one.

- **Reuse before inventing.** Check what already exists (`yaks list`, `yaks stats`) and prefer an existing label over a near-synonym (`ui` vs `interface` vs `frontend` — pick one).
- **Prefer broad, durable areas** (`ui`, `search`, `docs`, `skills`, `rust`) over hyper-specific one-offs. A label earns its keep only if you'd plausibly filter on it.
- **A few is plenty.** Zero labels is fine; more than ~2–3 on one yak is usually a smell.
- **One word each.** A label can't contain a comma or space: `--labels ui,docs` (or `ui docs`) means two labels, `ui` and `docs`.
- **Tracker labels:** when a yak maps to an external issue, a label named for the tracker (`jira`, `github`, `linear`) makes the upstreamed yaks easy to find. The **yaks-tracker** skill covers this.

## Commands

Run these directly from the shell (see **Running yaks** above for the exact invocation).

| Command | What it does |
|---------|-------------|
| `yaks init` | Scaffold a new `.yaks/` farm in the current directory. `--herd`, `--type`, `--priority`, `--emacs`; `--mode team\|private\|pointer` (+ `--path`) and `--skills none\|default\|coordination` for the full, idempotent setup. Works without an existing farm |
| `yaks create` | Create a new task (in hairy). Title is positional (`yaks create "Fix the login crash"`; `--title` still works for back-compat); `--type`, `--priority`, `--parent`, `--herd` (herd / id prefix for this yak; defaults to the config herd — lets one `.yaks/` hold several herds), `--labels`, `--depends-on`, `--source`, `--description` (`-` reads stdin; `--description-file PATH`), `--verify`, `--json` (print the new id + path) |
| `yaks list` | List tasks with optional filters (`--all` also includes dead) |
| `yaks show` | Show full details of a task |
| `yaks path` | Print the current absolute file path of yaks, by id or by the filter flags. A state move relocates the file, so stage a yak precisely with `git add $(yaks path <id>)` instead of hand-building `.yaks/<status>/<id>.md` |
| `yaks refs` | List what a task points at (parent, deps, id mentions), flagging danglers |
| `yaks commits` | Show the git commits linked to a yak — those naming its id and those that touched its file across status moves |
| `yaks update` | Update fields, labels (`--add-label`/`--remove-label`), or append a `--note` (`--note -` reads stdin, `--note-file PATH` a file: use these for multi-line markdown; same on `ask`/`answer`). Accepts multiple ids (same edit to each); `--as <actor>` attributes a note; `--verify '<cmd>'` sets/clears the yak's verification command; `--source <url>` sets/clears the external source |
| `yaks verify` | Run a yak's verification command (its own `verify:` field, else the config `verify:` default resolved by label) and record the PASS/FAIL as an attributed note (exits non-zero on failure). Explicit — never auto-run. Gives a yak a rerunnable proof-of-correctness |
| `yaks attach` | Attach a local file (screenshot, TUI frame) as evidence: stores it under `.yaks/artifacts/<id>/` and links it in the body. `--name` sets the stored filename; `--note`/`--as` record an attributed note. `yaks rename-attachment <id> <old> <new>` renames one later (links rewritten). The look-at-it form of evidence, for a human or yakherd to validate |
| `yaks ask` | Block a yak on a human decision (sets `needs`, drops it from `next`); records the question as an attributed `--note` |
| `yaks answer` | Answer a question: record the reply and set `needs: agent` (answered, awaiting pickup); `--done` clears `needs` instead |
| `yaks pickup` | Take on an answered yak: clear `needs: agent` with a `picked up` note (only that clears it; errors on any other state) |
| `yaks inbox` | The `needs` inbox across all statuses, in two sections (awaiting a human, marked `replied` after a plain-note reply; answered, awaiting an agent); `--for human\|agent`, `--sheds` (also the open asks in other sheds, each with its question text), `--json` |
| `yaks shave` | Start shaving a yak (hairy → shaving) [alias: `work`] |
| `yaks shorn` | Mark a yak shorn (shaving → shorn) [alias: `close`] |
| `yaks regrow` | Regrow a shorn yak (shorn → hairy) [alias: `reopen`] |
| `yaks slaughter` | Slaughter a yak (move to hidden `dead/`) — for ideas you won't pursue or tasks that got obviated. Refuses a yak with live descendants; `--family` slaughters the whole family |
| `yaks revive` | Revive a dead yak back to hairy |
| `yaks next` | Hairy tasks whose deps are all resolved [alias: `ready`] |
| `yaks tangled` | Hairy tasks with at least one unresolved dep [alias: `blocked`] |
| `yaks search` | Substring search over id/title/description |
| `yaks sheds` | List the other checkouts of this repo (git worktrees and Delta clones) and what each one's farm changed since that shed forked (the oldest entry of its HEAD reflog, so a worker is not credited with what its coordinator did first, and a shed synced with this checkout since then counts only what came after the sync; a shed that is only behind shows no changes; with no usable reflog it falls back to the merge-base and says `(vs merge-base)`): yaks only there, yaks in a different status, new notes, `needs:` newly set; plus commits ahead/behind (`+? -?` / JSON `null` when there is no merge-base here, e.g. shallow history; the table then adds one `git fetch --unshallow` note), and a label line per shed with its own entries: `who:` (the distinct actors on its new notes, e.g. a `YAKS_ACTOR` worker or `delta:<thread title>`) and `shaving:` (yaks in `shaving` there that it wrote to). Read-only everywhere; `--json` (`who`, `in_progress`). A shed with no farm of its own reads `no farm here (private? set YAKS_DIR)` |
| `yaks discover [PATH]` | Diagnostic: how `yaks sheds` finds the other checkouts, step by step, from inside any checkout (`git worktree list`; the host repo `objects/info/alternates` names; every Delta clone pinned there as `refs/delta/<dir>/<name>/*`, live or gone; the host checkout's git worktrees). Use it when `yaks sheds` misses or lists a checkout you did not expect. Git data only; read-only, no farm needed; `--json` |
| `yaks log` | Timestamped notes and status moves across a filtered set, oldest first (an activity log); `--since` and `--by` narrow it |
| `yaks dep` | Add/remove a dependency between tasks |
| `yaks reparent` | Move a task under a new `--parent` (or `--unparent` to top-level) |
| `yaks bulk` | Apply one field edit (and/or reparent) to every yak matching a filter. **Dry-run by default**; pass `--commit` to apply. Refuses to run without at least one filter flag *and* at least one mutation flag |
| `yaks rename` | Rename a yak + rewrite every reference to it across the farm |
| `yaks rename-herd` | Rename a whole herd: migrate all yaks from one id prefix to another; `--dry-run` to preview (`rename-prefix` still works as an alias) |
| `yaks merge` | Merge another farm into this one — copies every yak (all statuses) + artifacts, preserving ids and status, and declares any incoming herd in this farm's `herds:` config (so it appears in the pickers); **refuses on id collisions** (reconcile the source's herd with `rename-herd` first). Non-destructive: the source is left intact. `--dry-run` previews |
| `yaks stats` | Show task statistics |
| `yaks rollup` | Group yaks by the external issue they roll up to (`--keys` for just the keys) |
| `yaks doctor` | Read-only farm-integrity check (duplicate-status ids, dangling parent/dep refs, malformed labels containing a comma or space); exits non-zero on issues, so it's CI-usable. `--json` emits issues as JSON |
| `yaks doctor --strict` | Also flags shorn yaks with no recorded note, and shorn yaks whose `verify:` command did not last PASS — a shear without evidence (the evidence-before-shear rule) |
| `yaks preflight [<id>...]` | Read-only check to run before landing (committing a shorn yak / merging a shed) in a team farm: nothing under `.yaks/` untracked or with unstaged changes (a new `artifacts/<id>/` you never `git add`ed fails, naming it), each shorn yak in scope (the ids; else those in the change in git; `--all` for every one) whose `verify:` (own or config default) last PASSed, no yak in two status dirs. One line per failure, non-zero exit; else `preflight: ok`. A private farm skips the git check and says so. `--push-main` adds a check that the `local` remote is the human's checkout (non-bare, has `main`, not dirty on `main`), for the one about to push `main`. `--json`. |
| `yaks brief` | Print the worker brief for a yak (`yaks brief <id> --as <name> [--yaks-dir <path>]`), read-only: the actor prefix and `YAKS_DIR`, the gate, how to finish in a team or private farm, how to ask and pick up an answer, the `Decision:` rule. Warns on stderr if the yak is not yet `shaving` |
| `yaks status` | What the farm has changed that git does not have yet, one line per yak (`created`, `moved hairy -> shaving`, `notes +2`, `edited`, `removed`, `artifacts`; `*` = staged), then the message `yaks commit` would use. Read-only; run it before a landing or a handoff. `--check` exits 1 when dirty, `--json` for scripts. Clean: `farm clean: nothing to commit`. |
| `yaks commit` | Commit the farm's own changes (all of `.yaks/`, never code) in one command, with a generated `yaks: shorn …; updated …` message (`-m` overrides, `--dry-run` previews). Other staged files stay staged and out of it; hooks run; never pushes; fails in a private farm. |
| `yaks skills status` | Report whether the installed copies of these skills are current, stale, or locally edited (from a provenance stamp in their frontmatter). Ordinary `yaks` commands already upgrade a cleanly-stale *user-level* (`~/.agents/skills`) skill; a project-local install (`yaks skills install` writes `./.agents/skills` in a git repo) is never touched behind your back: an ordinary `yaks` command prints one `note:` line on stderr, once per yaks release per checkout, when it is `stale` (`status` and `doctor` say it any time) and you re-run `yaks skills install` |
| `yaks scan-ids` | Scan a file and/or stdin for tokens that are real yak-ids in this farm — a private-mode leak check; exits non-zero if any are found |
| `yaks tui` | Open the interactive terminal UI |

The state-transition verbs (`shave`, `shorn`, `regrow`, `slaughter`, `revive`) and `reparent` accept **multiple ids** in one call, applying the same move to each. Every move appends a `▸ <ts> [<actor>]` entry reading `moved: <from> -> <to>` to the
yak, so the file says who moved it and when (`yaks log` lists these). Attribute any note
or move with `--as <actor>` (else `$YAKS_ACTOR`, else `delta:<thread title>`/`delta:<thread id>` derived from Delta's environment, else the git user; an explicit name is still preferred); attribution never implies ownership.

## Task format

Tasks live in `.yaks/hairy/`, `.yaks/shaving/`, or `.yaks/shorn/` as `.md` files. Slaughtered tasks live in `.yaks/dead/` and are excluded from every default query — pass `--all` (or `--status dead`) to `list` to find them. Status is implicit from the directory. Metadata is YAML frontmatter; the markdown body is the description.

```markdown
---
id: yak-a1b2              # flat, opaque, and stable — never encodes hierarchy
title: Fix the login crash
type: bug
priority: 2
created: "2026-02-16T10:00:00Z"
updated: "2026-02-16T10:30:00Z"
parent: yak-c3d4         # optional; present only on child tasks
depends_on:
  - yak-e5f6
labels:
  - auth
source: https://jira.example.com/browse/PROJ-123  # optional external issue URL
---

Details go here.
```

Child tasks use `--parent <id>` on create. Every ID is flat (`{prefix}-{4hex}`) and stable for the task's whole life; the parent/child relationship lives in the `parent:` frontmatter field, not in the ID. Move a task with `yaks reparent <id> --parent <new>` (or `--unparent`), which just rewrites that one field. `yaks show` displays parent and children automatically.

> Older farms may still contain dotted IDs (e.g. `yak-a1b2.1`) created before this change. Those dots are now just opaque characters — the `parent:` field is authoritative — so don't parse IDs to infer hierarchy.

The default herd (id prefix), default type, and default priority come from `.yaks/config.yaml` (falling back to `yak` / `task` / `3`). A nested `verify:` map there (label → command, plus an optional `default`) supplies the default verification command for a yak that has no explicit `verify:` field, resolved by the yak's labels — so the project's levers are named once (e.g. `ui: cargo test -p yaks`).

A farm can also declare a `herds:` map — one entry per id prefix — giving that herd its own `default_type`, `default_priority`, or `verify` overrides. Each setting **cascades a single level**: the herd's value if set, else the farm-global. The `herds:` keys are the *known-herd set* the create/TUI picker offers, so you don't retype a prefix; route a new yak into a herd with `yaks create --herd <herd>` (or a repo's `.yaks` pointer `herd:`). Example:

```yaml
herd: core              # the default herd
default_priority: 3
verify:
  default: cargo test
herds:
  core:
    verify: { default: cargo test --workspace }
  web:
    default_type: feature
    verify: { default: npm test }
```

### External source linking

Use `--source <url>` on create or update to link a yak to an external issue (Jira, GitHub Issues, Linear, etc.); `--source ''` clears it, and the TUI create/edit form has a `source` row for the same. The URL is stored in the `source` frontmatter field. The relationship is a **one-way projection**: the yak points at the external issue, never the reverse, and the external tracker stays unaware of yaks.

Many yaks can roll up to one external issue. `yaks rollup` groups yaks by their source (a yak with no `source:` inherits its nearest ancestor's, so one stamp on an umbrella yak covers the subtree); `yaks rollup --keys` lists the external keys to paste into a PR body. For seeding a yak from an external issue or drafting a status update back to one, see the **yaks-tracker** skill.

## Referencing other yaks

Beyond the structural links (`parent:`, `depends_on:`), you can mention one yak from another's title, description, or a note just by writing its **full id** — `{prefix}-{4hex}`, using **this farm's configured prefix** (in `.yaks/config.yaml`; don't hardcode a prefix you saw in another repo). A mention is recognized by matching the token against real yak ids, so:

- **Always write the full id** (`yak-0af1`), never the bare 4-hex shorthand (`0af1`). Only the full form is detected and linked; a bare tail reads as ordinary prose.
- `[[yak-0af1]]` wiki-brackets work too and render as a bare link.
- Because matching is against real ids (not a prefix regex), mentions keep resolving even in a farm mid-migration with mixed prefixes.

In `yaks tui`, a mention is highlighted and followable (Tab / `[` / `]` to cycle, Enter to follow).

Use **`yaks refs <id>`** to see everything a yak points at — parent, dependencies, and id mentions in its text — with any dangling formal reference flagged. It's a quick integrity check before or after edits.

### Renaming safely

`yaks rename <old> <new>` renames a yak and rewrites **every** reference to it (parent, `depends_on`, and id mentions in bodies/titles) across the farm, matching whole ids only so lookalike prose is left untouched. `yaks rename-herd <old> <new>` does the same for a whole herd at once (e.g. migrating `yaksrs` → `yaks`) and updates `.yaks/config.yaml`. Both take **`--dry-run`** — always preview a bulk rename first.

## Filtering

Every query command (`list`, `search`, `next`, `tangled`) shares the same filter flags. AND across dimensions; within a repeatable flag, OR:

- `--status S` / `--type T` / `--priority P` / `--label L` / `--herd H` (all repeatable; `--herd` scopes to an id prefix)
- `--search Q` — substring match on title/description/id
- `--ready` / `--tangled` — dep-state filters
- `--parent-of ID` — only descendants of ID

Examples:
- `yaks list --type bug --type feature --priority 1` — urgent bugs or features
- `yaks list --label auth --search retry` — auth-labeled tasks mentioning "retry"
- `yaks next --type bug` — ready bugs only

## Bulk field edits

`yaks bulk` applies the **same** field edit (and/or reparent) to every yak matching a filter. It reuses the same filter flags as the query commands above (`--status`/`--type`/`--priority`/`--label`/`--search`/`--ready`/`--tangled`/`--needs`/`--parent-of`) to *select* the set, then one or more mutation flags apply the change:

- `--add-label L` / `--remove-label L` — add/remove labels on every matched yak
- `--set-priority P` / `--set-type T` — set the field on every matched yak
- `--reparent ID` / `--unparent` — move every matched yak under `ID` / to top-level

Safety rails:

- **Dry-run by default.** Without `--commit`, `bulk` only prints the matched set and the intended mutation — it changes nothing. Pass `--commit` to actually apply.
- **Never operates on the whole farm.** It refuses to run without at least one filter flag, and refuses without at least one mutation flag.
- **Field edits + reparent only** — no state transitions (use `shave`/`shorn`/`regrow`/`slaughter`/`revive` for those).

Examples:
- `yaks bulk --label auth --set-priority 1` — preview bumping every auth yak to P1 (dry run)
- `yaks bulk --label auth --set-priority 1 --commit` — actually apply it
- `yaks bulk --parent-of yak-c3d4 --add-label spike --commit` — label a whole subtree
