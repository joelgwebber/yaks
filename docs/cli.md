# CLI reference

Run `yaks <command> --help` for full flags. Most read commands accept the shared
**filter flags** and `--json`; most note-writing and state-transition commands accept `--as <actor>`.

## Which farm a command uses

Every command except `init` operates on one farm, found in this order:

1. **`$YAKS_DIR`**, if set: the `.yaks/` directory itself, a directory
   containing one, or a pointer file (relative paths are from the cwd). A value
   that names no farm is an error, not a fall-through.
2. Otherwise **walk up** from the cwd; a `.yaks/` directory, pointer file, or
   symlink at any level wins.
3. The walk **stops at the git top-level** (the first directory holding a
   `.git`, dir or file) unless that directory has a `.yaks` entry too. A
   checkout with no farm of its own (a Delta checkout under
   `<repo>/.delta/worktrees/`, an in-tree `git worktree` of a private farm)
   therefore fails with an error naming the git top-level rather than silently
   reading, and writing, another checkout's farm. Fix it with `yaks init`, a
   `.yaks` pointer file (`path:` + optional `herd:`), or `YAKS_DIR`. Outside any
   git repository the walk runs to the filesystem root.

## Filter flags (shared)

`--status <s>` · `--type <t>` · `--priority <n>` · `--label <l>` · `--herd <h>`
(id prefix) · `--search <text>` · `--parent-of <id>` (descendants at any depth) · `--ready` ·
`--tangled` · `--needs`. They compose, and back `list`, `search`, `log`, and the
`bulk` selector.

## Read / query

| Command | What it does |
|---|---|
| `list` | List tasks (non-dead by default; `--all` includes dead). Status renders as `[H]`/`[S]`/`[N]`/`[X]` — hairy / shaving / shor**n** / dead. |
| `show <id>` | Show one yak: fields, references, children, body + notes. |
| `path <id>...` / `path <filters>` | Print each yak's current file path (absolute, one per line), by id or by the shared filter flags (`--all` adds dead yaks). A transition moves a yak's file between status directories, so `git add $(yaks path <id>)` replaces hand-built `.yaks/<status>/<id>.md` paths. An unknown id goes to stderr with a non-zero exit (known ids still print). Ids or filters, not both. |
| `next` (alias `ready`) | Hairy yaks whose dependencies are all resolved — the work queue. |
| `tangled` (alias `blocked`) | Hairy yaks with at least one unresolved dependency. |
| `search <text>` | Substring search over id / title / description. |
| `stats` | Counts by status, type, priority. |
| `lanes` | The OTHER checkouts of this repo — plain `git worktree`s and Delta clones — and what each one's farm changed (`--json` for scripts). Per lane: path, kind (`worktree`/`delta`), branch (or detached), short HEAD, `+N -M` (commits it has that this HEAD lacks / commits this HEAD has that it lacks; `ahead`/`behind` in JSON), or `+? -?` (JSON `null`) when this repo has no merge-base with the lane's HEAD (shallow history, unrelated histories, a lane commit unknown here) so the counts cannot be told; when this repo is shallow (`git rev-parse --is-shallow-repository`) and a lane lost its merge-base that way, ONE line at the end of the table says `note: history is shallow here, so some lanes cannot be compared exactly; git fetch --unshallow fixes it` (JSON is unchanged: still an array of lanes), dirty-file count, newest mtime under its `.yaks` (a liveness hint: finished threads' checkouts persist), and the farm delta, relative to the lane's MERGE-BASE with this checkout (the base's committed `.yaks` is read from our own repo into a temp dir that is removed; a lane that is merely behind shows `no changes of its own`, never phantom moves) — yaks only in the lane, yaks in a different status directory (`hairy -> shaving`), yaks with new notes (count), and yaks whose `needs:` is set in the lane and differs from the base. Under each lane's row a label line names who is working there and on what: `who: lanes-2, delta:Fix attribution · shaving: yaks-d738`. It is read only from the lane's OWN new note entries (those beyond the merge-base's, committed or not; a `moved:` transition is one): `who` is the distinct actors on them in first-seen order (entries with no actor name no one), and the yaks are those that are in `shaving` in the lane and have own entries there (a claim already in the base counts when the lane has added notes to it; a yak the lane has shorn, or never touched, does not). JSON: `who` (array) and `in_progress` (array of ids). A lane with no own entries, or one compared `vs this checkout` (no merge-base to tell its entries from ours), has no label line and empty arrays. If the base cannot be read (no common history, the lane's commit unknown here, no `.yaks` committed at the base, a farm outside the lane's checkout) the lane is compared with this checkout's working tree instead and the output says `(vs this checkout)`; JSON `farm.vs` is `merge-base` (with `farm.base`, the short sha) or `checkout`. A lane with no farm of its own says `no farm here (private? set YAKS_DIR)`; one that resolves to this very farm says `shares this farm`; one whose git state cannot be read is listed with its `error`. Plain git plus the filesystem: `git worktree list --porcelain`, plus Delta's `<root>/<dir>/<name>` sibling clones (laptop `<repo>/.delta/worktrees/…`, managed `~/.local/share/delta/worktrees/…`) accepted when they share this repo's object-store alternates or `remote.origin.url`. Strictly read-only in the other checkouts (`git --no-optional-locks`; no fetch, checkout or writes). Needs a git repo and a farm; with no siblings prints `No other checkouts found.` and exits 0. |
| `log` | Timestamped notes and status moves (`moved: <from> -> <to>` entries) across a filtered set, oldest first (`--since <2h\|3d\|date>`, `--by <actor>`). |
| `refs <id>` | The yaks a task points at (parent, deps, id mentions), flagging danglers. |
| `commits <id>` | Git commits linked to a yak — those naming its id, and those that touched its file across status moves. |
| `rollup` | Group yaks by the external issue they roll up to (`--keys` lists the external keys). |
| `scan-ids [file]` | Scan text/stdin for real yak-ids; prints `line:col  id`, exits non-zero if any found — a leak check for private farms (pre-commit / PR hook). |

## Create / edit

| Command | What it does |
|---|---|
| `create '<title>'` | New hairy yak. Title is positional or `--title`; `--type`/`--priority`/`--parent`/`--herd`/`--labels`/`--depends-on`/`--source`/`--description`/`--verify`; `--json` prints id + file path. `--labels` (like `--add-label`/`--remove-label`/`--label` everywhere) splits on commas and whitespace — a label may contain neither, so `--labels ui,docs`, `--labels 'ui, docs'`, and `--labels ui docs` all give `[ui, docs]` (duplicates dropped). `--herd` sets the new yak's herd (id prefix; default: the `.yaks` pointer file's `herd:`, else the config `herd:`), so one `.yaks/` can hold several herds. With neither `--herd` nor a default herd, `create` fails (non-zero) rather than silently minting a `yak-` herd. |
| `update <ids…>` | Update fields/labels or append a `--note`; the same edit applies to every id. `--as <actor>` attributes the note. `--verify '<cmd>'` sets (or, empty, clears) the yak's verification command; `--source <url>` likewise sets (or, empty, clears) its external `source:`. |
| `dep add\|remove <id> <dep>` | Add / remove a dependency. |
| `reparent <ids…> --parent <id>` | Move yaks under a new parent (or `--unparent` to top-level). |
| `rename <old> <new>` | Rename a yak's id, updating every reference across the farm. |
| `rename-herd <old> <new>` | Rename a whole herd: migrate every id from one prefix to another (e.g. `yaksrs` → `yaks`). (`rename-prefix` still works as an alias.) |
| `merge <path> [--dry-run]` | Merge another farm's yaks (all statuses + artifacts) into this one, preserving ids and status, and declaring any incoming herd in this farm's `herds:` config so it shows up in the pickers. Non-destructive (source left intact); refuses on id collisions — reconcile the source's herd with `rename-herd` first. |
| `bulk <filter> <mutation>` | Filter-driven field edit. **Dry-run by default** — prints the matched set + the mutation and changes nothing without `--commit`. Requires ≥1 filter flag (never the whole farm) and ≥1 mutation flag (`--add-label`/`--remove-label`/`--set-priority`/`--set-type`/`--reparent`/`--unparent`). Field edits + reparent only — no state transitions. |

## Verification

| Command | What it does |
|---|---|
| `verify <ids…>` | Run each yak's verification command with live output, and record `verify: <cmd> -> PASS/FAIL (exit N)` as an attributed note. Exits non-zero if any fails. Explicit only — never auto-run. The command is the yak's own `verify:` field if set, else the config default resolved by the yak's labels (see below). The scriptable form of a yak's evidence contract: a check anyone (or CI) can re-run. |
| `attach <id> <path>` | Copy a local file under `.yaks/artifacts/<id>/` and link it in the yak's body — artifact evidence (screenshots, TUI frames) a human or coordinator can view. `--name <name>` stores it under a different filename (sanitized; the source's extension is kept if omitted). `--note`/`--as` record an attributed note. The non-scriptable, look-at-it form of evidence: an **external file** (committed in team mode — the farm un-ignores `.yaks/artifacts/`), never inlined. |
| `rename-attachment <id> <old> <new>` | Rename `.yaks/artifacts/<id>/<old>` (e.g. an ugly `paste-….png`) and rewrite every link to it across the farm. `<new>` is sanitized and keeps `<old>`'s extension if it omits one; refuses to overwrite an existing attachment. |

Default verify commands live in `.yaks/config.yaml` under a nested `verify:` map (label → command, plus an optional `default`); a yak with no explicit `verify:` field resolves its command from the first of its labels with an entry, else `default`. Name the project's levers once:

```yaml
verify:
  ui: cargo test -p yaks
  cli: cargo test -p yaks
  default: cargo test --workspace
```

Per-herd overrides live under a `herds:` map (one entry per id prefix), each optionally setting `default_type`, `default_priority`, or its own `verify:` map. Any setting cascades a single level — the herd's value if present, else the farm-global — and the `herds:` keys are the known-herd set the create/TUI picker offers:

```yaml
herd: core
verify:
  default: cargo test
herds:
  core:
    verify: { default: cargo test --workspace }
  web:
    default_type: feature
    verify: { default: npm test }
```

A lever is the pass/fail **gate** for its label — what `doctor --strict` enforces at shear. For UI yaks that's the tui snapshot tests (`cargo test -p yaks`); the SVG docshots are a separate *visual* channel, generated on demand (`cargo test -p yaks docshots -- --ignored`) and attached for human review, not a gate.

## State transitions (all accept multiple ids)

| Command | Move |
|---|---|
| `shave` (alias `work`) | → shaving (start) |
| `shorn` (alias `close`) | → shorn (done) |
| `regrow` (alias `reopen`) | shorn → hairy |
| `slaughter` | → dead (abandon) |
| `revive` | dead → hairy |

`slaughter` refuses a yak that still has live (non-dead) descendants, since that
would orphan them (it names them and exits non-zero). `slaughter <id> --family`
takes the whole family instead: every live descendant, deepest first, then the
yak itself.

Every move appends an entry to the yak's body in the note shape,
`▸ <ts> [<actor>]` then `moved: <from> -> <to>` (one per yak moved, so
`slaughter --family` writes one on each), and each verb takes `--as <actor>`
like the note commands. The file alone says who moved the yak and when, and
`yaks log` lists moves with the notes. A yak moved by an older binary simply has
no entry.

## Human-in-the-loop

| Command | What it does |
|---|---|
| `ask <id> --note '<question>'` | Block a yak on a human: sets the `needs` field, dropping it out of `next`. |
| `answer <id> --note '<reply>'` | Clear the `needs` block (human-reserved). |
| `inbox` | Yaks awaiting a human — equivalent to `list --needs` across all statuses. |

## Farm admin & integrity

| Command | What it does |
|---|---|
| `init` | Create a `.yaks/` farm in the current directory. |
| `skills install` | Install the bundled agent skills (`yaks`, `yaks-tracker`). Target: `--dir <path>` if given; else `--user` → `~/.agents/skills`; else `./.agents/skills` in the git top-level of the cwd when inside a git repo, and `~/.agents/skills` outside one. Prints the destination it chose. `--with coordination` adds `yaks-coordinating` (+ `-team`, `-private`, `-worktrees`, `-delta`, with its `land.sh`) and `yaks-working`. A skill is installed as a unit; an outdated untouched copy is upgraded, an edited one (including an edited script) is left alone unless `--force`. Refuses to write onto yaks' own `.agents/skills/` source (even via a symlink, even with `--force`), naming `--user`/`--dir` as the way out. |
| `skills status` | Per-skill verdict — `current` / `stale` / `adoptable` / `held` / `modified` / `unmanaged` / `source` — from the provenance stamp (other files of a skill are compared by content). Same default directory as `install`; `--user` / `--dir` to inspect another; installed opt-in skills are always listed, `--with coordination` also lists absent ones. See [skills.md](skills.md). |
| `doctor` | Read-only integrity check: duplicate-status ids, dangling parent/deps, malformed labels (a legacy label containing a comma or space, e.g. `ui,docs` — any label edit on that yak re-splits it). Exits non-zero on any issue (CI-usable). `--strict` also flags shorn yaks with no recorded note, and shorn yaks whose `verify:` command did not last PASS (evidence-before-shear). |
| `preflight [<id>...] [--all] [--push-main]` | Read-only landing-readiness check; run it before committing a shorn yak or merging a lane in a team farm. Checks: (1) nothing under `.yaks/` is untracked or has unstaged changes in git (staged is fine: it is the step before the commit) — a new `artifacts/<id>/` never `git add`ed fails and is named; (2) every shorn yak in scope whose `verify:` command (own, else the config default for its labels) last PASSed, same rule as `doctor --strict`; scope is the ids given, else the shorn yaks that are part of the change in git (staged, modified or new under `.yaks/`, including a new `artifacts/<id>/`), and `--all` checks every shorn yak (old ones that never ran `verify` will fail); (3) no yak in two status dirs. `ids` scopes check 2 only; 1 and 3 are farm-wide. Prints one `preflight: FAIL <check>: …` line per failure and exits non-zero, else `preflight: ok`. In a private farm (nothing under `.yaks/` tracked by git) check 1 is skipped with a printed line and check 2 covers every shorn yak; 3 still runs. `--push-main` adds (4), for the coordinator about to `git push local <branch>:main` (off by default: it depends on another checkout's state, not on the change, and a repo with no `local` remote is not a failure without it): `local` must be a path on this machine (an https/ssh URL is reported as not the human's checkout, never resolved), a non-bare git repository with `refs/heads/main`, and not checked out on `main` with uncommitted changes to tracked files (git refuses the push then). Otherwise it fails with `preflight: FAIL local-checkout: …`, naming the fallback: push `pr/<name>` to `local` and give the human `git fetch <path> pr/<name>`, or repoint `local`. Read-only (`remote get-url`, `ls-remote`, `rev-parse`, `status`); never pushes. `--json` emits `{ok, failures[{check, message, subjects}], skipped}`. |
| `commit [-m <msg>] [--dry-run]` | Commit the farm's own changes — every change under `.yaks/` (yak files, moves, `artifacts/`, config) — and nothing else, so a human's drifted edits land in one command and stop blocking a landing. Stages the farm and runs `git commit --only -- .yaks`: files you staged elsewhere stay staged and out of the commit (named in the output), and modified code is never touched. The message is generated from the changed files, one verb per yak (`created`, `shaving`/`shorn`/`dead`/`regrown` for a move, `updated`, `removed`, `artifacts for`, `farm config`), e.g. `yaks: created yaks-8c08; shaving yaks-c968`; `-m` overrides it. `--dry-run` lists the files and message without staging or committing. A normal `git commit`, so the repo's hooks run (and may fail it: the farm changes are then left staged); it never pushes. A clean farm prints `nothing to commit` and exits 0. While a merge or cherry-pick is in progress (a Delta landing leaves one pending) it refuses, stages nothing and says why: git forbids the partial commit it relies on, so finish the merge with a plain `git commit` (the farm changes are part of it) or abort it. In a private farm (nothing under `.yaks/` tracked by git) it fails with a clear error. |
| `tui` | Open the interactive terminal UI (see [tui.md](tui.md)). |

## Attribution

`--as <actor>` on note-writing commands stamps the note `▸ <ts> [actor]`, and on
the state-transition verbs stamps the `moved: <from> -> <to>` entry the same
way. The
actor resolves `--as` → `$YAKS_ACTOR` → the harness identity → git `user.name`.
Under Delta the harness identity is derived from its terminal environment:
`delta:<thread title>` (`$DELTA_THREAD_TITLE`, whitespace collapsed, cut to 40
characters), else `delta:<thread id>` (`$DELTA_CURRENT_THREAD_ID`); the `delta:`
prefix marks a derived, non-human actor. An explicit name (`--as` or
`$YAKS_ACTOR`) is still preferred: it is stable and chosen, where a thread title
can be renamed. (The git author, the human under Delta, is not what attributes
a move: the entry in the yak file is.) Attribution, never
ownership — a yak belongs to no one.
