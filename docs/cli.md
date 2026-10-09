# CLI reference

Run `yaks <command> --help` for full flags. Most read commands accept the shared
**filter flags** and `--json`; most note-writing and state-transition commands accept `--as <actor>`.

**Global flag: `-C <path>`** (like `git -C`; accepted before or after the
subcommand). Every command runs as if started in `<path>`: farm discovery,
`sheds`, `discover`, `status`, `commit`, `init`, `skills`, `tui`. A relative
`<path>` is resolved against the real cwd; repeated `-C` compose, each relative
to the previous (`yaks -C a -C b list` runs in `a/b`). A missing path is an
error naming it. `$YAKS_DIR` still wins over discovery (a relative one is read
from `<path>`).

**Global flag: `--shed <name>`** is `-C` by name: it runs the command in another
checkout of this repo (a shed from `yaks sheds`), picked like `yaks changes
<shed>` (path, Delta dir id, thread-title slug, branch, actor, `main`, or a
unique substring). Accepted before or after the subcommand, at most once, and
applied after any `-C` (the name is resolved over the sheds visible from that
checkout). It is FULL access, exactly `-C <that shed's path>`: a write command
writes in that shed's farm, and nothing extra is printed. No or an ambiguous
match prints `error: ...` with the list of sheds and exits 1. It never records
the Delta thread title (as with `-C`, the title in the environment is the
caller's thread). Use it only on a checkout you own.

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
`--tangled` · `--needs` (any `needs` value, `human` or `agent`). They compose, and back `list`, `search`, `log`, and the
`bulk` selector.

## Read / query

| Command | What it does |
|---|---|
| `list` | List tasks (non-dead by default; `--all` includes dead). Status renders as `[H]`/`[S]`/`[N]`/`[X]` — hairy / shaving / shor**n** / dead. |
| `show <id> [--sheds] [--json]` | Show one yak: fields, references, children, body + notes. `--sheds` then adds a `Sheds:` section for the OTHER checkouts of this repo (git worktrees and Delta clones, as `sheds` lists them): this checkout's copy is printed first, then one block per shed whose copy differs — the shed's name and path, its status, its `needs:` when set, and the note entries it has that this copy lacks (timestamp, actor, first line) — and ONE trailing line counting the rest: `same in N sheds; absent in N; no farm of its own in N sheds` (a copy with the same status, the same `needs:` and no extra notes is "same", so a shed that is merely behind counts as same; "no farm" is a shed sharing this farm, with none, or with an unreadable one). A yak that exists only in sheds is still shown (`<id> is not in this checkout.`, then each shed's copy with its title) and exits 0; without `--sheds`, or when no shed has it either, an unknown id exits 1. `--json` adds a `sheds` array to the usual object (for an id only in sheds, the object is just `{id, sheds}`): one entry per shed with a farm of its own — `shed`, `path`, `status` and `needs` (null when the shed lacks the yak / has no `needs`), `new_notes` (`[{ts, actor, text}]`), `same`, `absent`. Read-only in the other checkouts. |
| `path <id>...` / `path <filters>` | Print each yak's current file path (absolute, one per line), by id or by the shared filter flags (`--all` adds dead yaks). A transition moves a yak's file between status directories, so `git add $(yaks path <id>)` replaces hand-built `.yaks/<status>/<id>.md` paths. An unknown id goes to stderr with a non-zero exit (known ids still print). Ids or filters, not both. |
| `next` (alias `ready`) | Hairy yaks whose dependencies are all resolved — the work queue. A yak awaiting a human (`needs: human`) is left out; one that has been answered (`needs: agent`) is included, marked `✓ answered, awaiting pickup` (`yaks pickup <id>` when you take it on). |
| `tangled` (alias `blocked`) | Hairy yaks with at least one unresolved dependency. |
| `search <text>` | Substring search over id / title / description. |
| `stats` | Counts by status, type, priority. |
| `sheds` | The OTHER checkouts of this repo — plain `git worktree`s and Delta clones — and what each one's farm changed (`--json` for scripts); a shed (formerly lane) is one such checkout. Each shed's `name:` line (JSON `name`) is `main` for the primary checkout, else the slug of the Delta thread title (lowercase, runs of other characters one `-`, at most 24 characters, cut at a `-`; two clones with one slug get their dir id's first 4 characters appended, e.g. `fix-it-q7zz`), else its first `who` actor, else its Delta dir id, else its branch. The title is read from the clone's own git config `yaks.shed.title` (JSON `title`, also shown on the `name:` line), which ANY yaks command run inside a Delta clone with `DELTA_THREAD_TITLE` set records (with `yaks.shed.thread` from `DELTA_CURRENT_THREAD_ID`), only when the value changed, silently, never for `-C` or `--shed`, never in a plain git worktree; it is per-clone and never committed. Per shed: path, kind (`worktree`/`delta`), branch (or detached), short HEAD, `+N -M` (commits it has that this HEAD lacks / commits this HEAD has that it lacks; `ahead`/`behind` in JSON), or `+? -?` (JSON `null`) when this repo has no merge-base with the shed's HEAD (shallow history, unrelated histories, a shed commit unknown here) so the counts cannot be told; when this repo is shallow (`git rev-parse --is-shallow-repository`) and a shed lost its merge-base that way, ONE line at the end of the table says `note: history is shallow here, so some sheds cannot be compared exactly; git fetch --unshallow fixes it` (JSON is unchanged: still an array of sheds), dirty-file count, newest mtime under its `.yaks` (a liveness hint: finished threads' checkouts persist), and the farm delta, relative to where the SHED FORKED: the oldest entry of its HEAD reflog (`<git-dir>/logs/HEAD`, first line; for a Delta clone the coordinator's HEAD at spawn time, for a `git worktree add -b` shed the branch creation), so a worker is not credited with what its coordinator committed before spawning it, except that a shed synced with this checkout since it forked (this checkout's merge-base with it descends from the fork point: it merged this checkout, or this checkout merged it) is measured from that NEARER merge-base, because what it merged in is already in this checkout's history and is not something the shed has that this checkout lacks (exact like the fork point: no marker, JSON `farm.vs` `sync`); and a shed whose commits were squash-landed still shows only its own work (the base's committed `.yaks` is read from our own repo into a temp dir that is removed; a shed that is merely behind shows `no changes of its own`, never phantom moves). When the reflog is missing, expired or unusable (its first commit unknown here, or not an ancestor of the shed's HEAD) the baseline is the shed's MERGE-BASE with this checkout instead, and the farm line and label line say `(vs merge-base)`, because that baseline also counts whatever the shed inherited from the thread that spawned it. `ahead`/`behind` are unchanged: they describe the shed relative to this checkout — yaks only in the shed, yaks in a different status directory (`hairy -> shaving`), yaks with new notes (count; a created yak's notes count too, so the `farm:` line agrees with `yaks changes`), and yaks whose `needs:` is set in the shed and differs from the base. Under each shed's row a label line names who is working there and on what: `who: sheds-2, delta:Fix attribution · shaving: yaks-d738`. It is read only from the shed's OWN new note entries (those beyond the base's, i.e. since the shed forked, committed or not; a `moved:` transition is one): `who` is the distinct actors on them in first-seen order (entries with no actor name no one), and the yaks are those that are in `shaving` in the shed and have own entries there (a claim already in the base counts when the shed has added notes to it; a yak the shed has shorn, or never touched, does not). JSON: `who` (array) and `in_progress` (array of ids). A shed with no own entries, or one compared `vs this checkout` (no base to tell its entries from ours), has no label line and empty arrays. If neither base can be read (no usable fork point and no common history, the shed's commit unknown here, no `.yaks` committed at the base, a farm outside the shed's checkout) the shed is compared with this checkout's working tree instead and the output says `(vs this checkout)`; JSON `farm.vs` is `fork`, `sync`, `merge-base` (all with `farm.base`, the short sha) or `checkout`. A shed with no farm of its own says `no farm here (private? set YAKS_DIR)`; one that resolves to this very farm says `shares this farm`; one whose git state cannot be read is listed with its `error`. Discovery reads git data only, from inside any checkout of the repo (it never searches the filesystem): `git worktree list --porcelain` here; the HOST repo this checkout's `objects/info/alternates` names (the human's `.git` where the project was added from a folder, Delta's managed bare repo on a machine the thread was shared to; else this repo); every Delta clone pinned there as `refs/delta/<dir>/<name>/<sha>` (Delta pins a clone's starting commits when it creates it), resolved to its checkout through the clone's git dir `<store>/<dir>/<name>.git` and its `core.worktree` (a pin whose git dir is gone is a finished thread and is skipped); and, when the host is a checkout, that checkout and its own `git worktree list` (so a Delta clone sees the primary's git worktrees). A clone on a shared-thread machine cannot see the human's checkout. `yaks discover` shows each step. Strictly read-only in the other checkouts (`git --no-optional-locks`; no fetch, checkout or writes). Needs a git repo and a farm; with no siblings prints `No other checkouts found.` and exits 0. |
| `changes SHED` | What ONE shed (named by: the name `sheds` prints (the thread-title slug of a Delta clone), `main` for the primary checkout, a Delta dir id, a branch, an actor on its notes, a path, or a unique substring; the same rule as `answer <id>@<shed>`) created, moved and noted since it forked, per yak, with actors: new yaks (title, status), `from -> to` moves, each new note entry (timestamp, actor, first line) and `needs:` newly set. The per-yak expansion of the `farm:` line of `sheds`, with the same baseline (fork, sync, else `(vs merge-base)` / `(vs this checkout)`). `--json`: one object per changed yak (`id`, `title`, `status`, `added`, `moved`, `notes` with `ts`/`actor`/`text`, `needs`, `vs`, `base`). An unknown or ambiguous name lists the sheds and exits 1; a shed sharing this farm (or with none) says why there is nothing to show. Read-only |
| `discover [PATH]` | Diagnostic: the chain `sheds` uses to find the other checkouts, step by step, from `PATH` (default the cwd; any directory inside a primary checkout, a git worktree or a Delta clone; outside one it exits 1 and says so: nothing is searched for). Sections: the checkout (top-level, layout: primary checkout / git worktree of … / Delta clone whose host is a checkout or a bare repo; origin; `local` and whether it is a checkout or BARE; the farm discovery resolves); `1. GIT WORKTREE LIST (here)`; `2. HOST` (own git dir, the host git dir and how it was found, the clone store); `3. DELTA PINS in the host` (each `<dir>/<name>` pin group with its pin count and its checkout, or why not: `gone (its git dir no longer exists)`, `checkout missing`, `this checkout`); `4. GIT WORKTREE LIST (host checkout …)` when the host is a checkout reached through alternates; then `SHEDS`, exactly what `sheds` lists, with kind `worktree`/`delta`. Read-only; needs no farm. `--json` (`host`, `worktrees_here`, `worktrees_host`, `pins[]` with `checkout`/`reason`, `sheds[]`). `scripts/discover-fixture.sh` builds Delta-shaped layouts (a linked machine, a shared machine, a gone pin, an unpinned look-alike) and runs it from each kind of checkout. |
| `log` | Timestamped notes and status moves (`moved: <from> -> <to>` entries) across a filtered set, oldest first (`--since <2h\|3d\|date>`, `--by <actor>`). |
| `refs <id>` | The yaks a task points at (parent, deps, id mentions), flagging danglers. |
| `commits <id>` | Git commits linked to a yak — those naming its id, and those that touched its file across status moves. |
| `rollup` | Group yaks by the external issue they roll up to (`--keys` lists the external keys). |
| `scan-ids [file]` | Scan text/stdin for real yak-ids; prints `line:col  id`, exits non-zero if any found — a leak check for private farms (pre-commit / PR hook). |

## Create / edit

| Command | What it does |
|---|---|
| `create '<title>'` | New hairy yak. Title is positional or `--title`; `--type`/`--priority`/`--parent`/`--herd`/`--labels`/`--depends-on`/`--source`/`--description`/`--description-file`/`--verify`; `--json` prints id + file path. `--description -` reads the body from stdin (see [Multi-line text](#multi-line-text)). `--labels` (like `--add-label`/`--remove-label`/`--label` everywhere) splits on commas and whitespace — a label may contain neither, so `--labels ui,docs`, `--labels 'ui, docs'`, and `--labels ui docs` all give `[ui, docs]` (duplicates dropped). `--herd` sets the new yak's herd (id prefix; default: the `.yaks` pointer file's `herd:`, else the config `herd:`), so one `.yaks/` can hold several herds. With neither `--herd` nor a default herd, `create` fails (non-zero) rather than silently minting a `yak-` herd. |
| `update <ids…>` | Update fields/labels, replace the body (`--description`/`--description-file`) or append a `--note`/`--note-file`; the same edit (and the same text) applies to every id. `--note -` / `--description -` read stdin (see [Multi-line text](#multi-line-text)). `--as <actor>` attributes the note. `--verify '<cmd>'` sets (or, empty, clears) the yak's verification command; `--source <url>` likewise sets (or, empty, clears) its external `source:`. |
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
| `attach <id> <path>` | Copy a local file under `.yaks/artifacts/<id>/` and link it in the yak's body — artifact evidence (screenshots, TUI frames) a human or yakherd can view. `--name <name>` stores it under a different filename (sanitized; the source's extension is kept if omitted). `--note`/`--as` record an attributed note. The non-scriptable, look-at-it form of evidence: an **external file** (committed in team mode — the farm un-ignores `.yaks/artifacts/`), never inlined. |
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
| `ask <id> --note '<question>'` | Block a yak on a human: sets the `needs` field, dropping it out of `next`. On a yak that is already `needs: agent` it flips back to `human` and says so. Also `--note -` (stdin) / `--note-file PATH`. |
| `answer <id> --note '<reply>'` | Answer a question (human-reserved): records the reply and sets `needs: agent`, so the answer stays findable (in `inbox`, and in `next` when the yak is hairy) until an agent picks it up. `--done` clears `needs` instead, for an answer that needs no follow-up. A second answer on a `needs: agent` yak keeps the state and appends the note; on a yak with no `needs` it only records the note. Also `--note -` / `--note-file PATH`. `answer <id>@<shed>` answers in ANOTHER checkout's copy of the yak (`<shed>` as in `yaks changes`; the id and shed name from `inbox --sheds`): same transition and attributed note, written into that checkout's working tree under its own farm lock and left **uncommitted** there (its worker commits it with its own yak file); one line says which shed and file. Refused when the shed shares this farm (use plain `answer`), has none or an unreadable one, or lacks the yak. The only command that writes outside this checkout; nothing is staged or committed. |
| `pickup <id> [--note '<text>']` | An agent takes on an answered yak: clears `needs: agent` and records a `picked up` note attributed to the actor (`--as`). The only command that clears `needs: agent`; on a yak that is not `needs: agent` it fails, naming the state. A `needs` change is not a status move (no `moved:` entry). |
| `inbox [--for human\|agent] [--sheds] [--json]` | Every yak with a `needs` value, across all statuses, in two sections: **Awaiting a human** (`needs: human`; marked `↩ replied` when its latest note after the ask is from someone other than the asker, i.e. a plain-note reply that did not use `answer`) and **Answered, awaiting an agent** (`needs: agent`). `--for` shows one section. `--sheds` then adds, per OTHER shed (a git worktree or Delta clone of this repository, as `sheds` lists them), every yak with `needs` set in that shed's own farm: the shed's name and path, the usual row, and the question text (the newest note at or after the ask, so a reply shows instead when there is one). An ask identical in this checkout's farm (same id, same `needs`, same latest note) is not repeated; a shed whose farm is shared with this checkout or that has none shows nothing. `--for` and the filter flags apply to the shed rows; the shed farms are only read, never written. `--json` is one array of yaks, each with `needs` and a derived `replied` boolean; with `--sheds` it is instead an object `{inbox: [...that array...], sheds: [{shed, path, id, title, status, needs, replied, question}]}` (`question` is `null` for a hand-set `needs:` or an ask with no text). `ask` records an `asked: needs <who>` first line in its note: that is what `replied` is measured from, so a yak whose `needs:` was set by hand has no known asker and is never `replied`. |

## Multi-line text

Shell quoting makes multi-line markdown awkward as an inline `--note "..."` /
`--description "..."` argument (and the TUI renders a one-line blob as a single
wrapped paragraph). `create`, `update`, `ask` and `answer` therefore take the
text from stdin or a file instead:

````sh
yaks update <id> --note - <<'EOF'
## Findings
- `foo` is called from "bar" and costs $5

```sh
cargo test
```
EOF
yaks create "Title" --description-file body.md
yaks ask <id> --note-file question.md
````

| Flag | Source |
|---|---|
| `--note -`, `--description -` | stdin (`--note-file -` / `--description-file -` mean the same) |
| `--note-file PATH`, `--description-file PATH` | the file at `PATH` |

Rules, the same for all four commands:

- The text is stored exactly as given — backticks, quotes, `$`, blank lines and
  interior newlines survive — except that trailing newlines are trimmed (a
  heredoc always adds one).
- Empty (or whitespace-only) stdin or file is an error, never an empty note.
  Inline `--description ""` still clears a body.
- Stdin can be read once: `--note -` together with `--description -` is an error.
  A `-` note with a `--description-file` is fine.
- A flag given both inline and as a file (`--note x --note-file f`) is an error.
- A missing, unreadable or non-UTF-8 file is an error naming the flag and path.
- `-` with stdin attached to a terminal is refused with a message rather than
  waiting silently; pipe or redirect the text in.
- `update` with several ids applies the same text to each.
- Inline values are unchanged: the only inline value with a new meaning is the
  literal `-`.

## Farm admin & integrity

| Command | What it does |
|---|---|
| `init` | Create a `.yaks/` farm in the current directory. With no `--mode`/`--skills`/`--path` it only creates a committed (team) farm, then prints that skills are not installed and the exact command to add them. With `--mode` and/or `--skills` it does the full setup, one printed line per step (see below). Flags: `--herd`, `--type`, `--priority`, `--emacs`, `--mode team\|private\|pointer`, `--path <dir>`, `--skills none\|default\|coordination`. |
| `skills install` | Install the bundled agent skills (`yaks`, `yaks-tracker`). Target: `--dir <path>` if given; else `--user` → `~/.agents/skills`; else `./.agents/skills` in the git top-level of the cwd when inside a git repo, and `~/.agents/skills` outside one. Prints the destination it chose. `--with coordination` adds `yaks-coordinating` (+ `-team`, `-private`, `-worktrees`, `-delta`, with its `land.sh`) and `yaks-working`. A skill is installed as a unit; an outdated untouched copy is upgraded, an edited one (including an edited script) is left alone unless `--force`. Refuses to write onto yaks' own `.agents/skills/` source (even via a symlink, even with `--force`), naming `--user`/`--dir` as the way out. |
| `skills status` | Per-skill verdict — `current` / `stale` / `adoptable` / `held` / `modified` / `unmanaged` / `source` — from the provenance stamp (other files of a skill are compared by content). Same default directory as `install`; `--user` / `--dir` to inspect another; installed opt-in skills are always listed, `--with coordination` also lists absent ones. When something is stale it ends with the command to run (`yaks skills install`, plus `--with coordination` if an opt-in skill is stale). An ordinary command prints that advice itself as one stderr `note:` line, once per yaks version per checkout, for a stale project-local install (never with `--json`, nor for `skills`/`init`; `YAKS_SKILLS_AUTOSYNC=0` silences it); `doctor` repeats it in its skills advisory. See [skills.md](skills.md). |
| `doctor` | Read-only integrity check: duplicate-status ids, dangling parent/deps, malformed labels (a legacy label containing a comma or space, e.g. `ui,docs` — any label edit on that yak re-splits it). Exits non-zero on any issue (CI-usable). `--strict` also flags shorn yaks with no recorded note, and shorn yaks whose `verify:` command did not last PASS (evidence-before-shear). |
| `preflight [<id>...] [--all] [--push-main]` | Read-only landing-readiness check; run it before committing a shorn yak or merging a shed in a team farm. Checks: (1) nothing under `.yaks/` is untracked or has unstaged changes in git (staged is fine: it is the step before the commit) — a new `artifacts/<id>/` never `git add`ed fails and is named; (2) every shorn yak in scope whose `verify:` command (own, else the config default for its labels) last PASSed, same rule as `doctor --strict`; scope is the ids given, else the shorn yaks that are part of the change in git (staged, modified or new under `.yaks/`, including a new `artifacts/<id>/`), and `--all` checks every shorn yak (old ones that never ran `verify` will fail); (3) no yak in two status dirs. `ids` scopes check 2 only; 1 and 3 are farm-wide. Prints one `preflight: FAIL <check>: …` line per failure and exits non-zero, else `preflight: ok`. In a private farm (nothing under `.yaks/` tracked by git) check 1 is skipped with a printed line and check 2 covers every shorn yak; 3 still runs. `--push-main` adds (4), for the yakherd about to `git push local <branch>:main` (off by default: it depends on another checkout's state, not on the change, and a repo with no `local` remote is not a failure without it): `local` must be a path on this machine (an https/ssh URL is reported as not the human's checkout, never resolved), a non-bare git repository with `refs/heads/main`, and not checked out on `main` with uncommitted changes to tracked files (git refuses the push then). Otherwise it fails with `preflight: FAIL local-checkout: …`, naming the fallback: push `pr/<name>` to `local` and give the human `git fetch <path> pr/<name>`, or repoint `local`. Read-only (`remote get-url`, `ls-remote`, `rev-parse`, `status`); never pushes. `--json` emits `{ok, failures[{check, message, subjects}], skipped}`. |
| `brief <id> --as <name> [--yaks-dir <path>]` | Print the worker brief for one yak, read-only: the part a worker needs that the yak, the config and the farm determine. It names the yak and that it is already `shaving`; the `YAKS_ACTOR=<name>` prefix for every command (plus `YAKS_DIR=<farm>` for a private or pointer farm, or whatever `--yaks-dir` names); this binary's absolute path; the evidence (the yak's own `verify:`, else the config default for its labels and herd); the forbidden moves; how to finish, chosen by farm mode (team: `yaks shorn`, ONE commit staging `yaks path <id>`, the old status path and any `artifacts/<id>`, message `<id>: ...`; private: no yak id or the word yaks in commits); how to ask and what to do once the ask is answered (`inbox --for agent`, then `pickup`); the `Decision:` note rule; multi-line notes on stdin; and the final-message format. Mode is detected as the skills do (tracked farm in the repo of the cwd = team). An unclaimed yak still prints, with a warning on stderr; an unknown id fails with nothing on stdout; `--as` must be a shell-safe name. What a yak cannot know (spawn title, how work lands, the model, scope by symbol) stays with the coordinator, who adds it around this text. |
| `status [--check] [--json]` | What the farm has changed that git does not have yet, one line per yak, ordered by id, in yak terms: `created`, `moved hairy -> shaving`, `notes +2` (notes appended; the `moved:` entry a move writes is not counted), `edited` (a field, the description or an existing note changed), `removed`, `artifacts` (a new or changed file under `artifacts/<id>/`); a yak with several kinds of change gets one line with all of them, and `config` is `config.yaml`. A `*` in the first column marks a yak whose changes are all staged. It uses the same classification as `commit` (the verbs cannot drift: `created`/`removed`/a move read the same, `notes` and `edited` are both `updated`), and ends with the message `commit` would use. Code changed elsewhere in the repo is not listed, only counted in one line (`N other changed files outside the farm`), so you know `commit` leaves it alone. A clean farm prints `farm clean: nothing to commit`; a private farm (nothing under `.yaks/` tracked by git) prints `private farm: not tracked by git, nothing to commit`; both exit 0. While a merge or cherry-pick is in progress it says `yaks commit` will refuse (git forbids the partial commit) and why. Read-only: nothing is staged, committed or written. `--check` exits 1 when the farm is dirty (same output, for scripts and hooks; otherwise the exit code is 0). `--json`: `{clean, private, merge_in_progress, yaks: [{id, changes, staged}], other: [{path, staged}], outside_farm, message}`. |
| `commit [-m <msg>] [--dry-run]` | Commit the farm's own changes — every change under `.yaks/` (yak files, moves, `artifacts/`, config) — and nothing else, so a human's drifted edits land in one command and stop blocking a landing. Stages the farm and runs `git commit --only -- .yaks`: files you staged elsewhere stay staged and out of the commit (named in the output), and modified code is never touched. The message is generated from the changed files, one verb per yak (`created`, `shaving`/`shorn`/`dead`/`regrown` for a move, `updated`, `removed`, `artifacts for`, `farm config`), e.g. `yaks: created yaks-8c08; shaving yaks-c968`; `-m` overrides it. `--dry-run` lists the files and message without staging or committing. A normal `git commit`, so the repo's hooks run (and may fail it: the farm changes are then left staged); it never pushes. A clean farm prints `nothing to commit` and exits 0. While a merge or cherry-pick is in progress (a Delta landing leaves one pending) it refuses, stages nothing and says why: git forbids the partial commit it relies on, so finish the merge with a plain `git commit` (the farm changes are part of it) or abort it. In a private farm (nothing under `.yaks/` tracked by git) it fails with a clear error. |
| `tui` | Open the interactive terminal UI (see [tui.md](tui.md)). |

### `init`: modes and skills in one idempotent step

```sh
yaks init                                   # farm only (team); prints how to add skills
yaks init --skills default                  # team farm + project-local skills
yaks init --mode private                    # farm in .yaks/, kept out of git; default skills
yaks init --mode pointer --path ~/farms/work --herd web   # .yaks pointer file -> a farm elsewhere
```

- **Modes.** `team`: the farm in `.yaks/`, committed with the code. `private`:
  the same, plus `/.yaks` in `.git/info/exclude`. `pointer`: `--path <dir>` is the
  directory holding the farm (its `.yaks/`, or the farm itself when `<dir>` is a
  farm or named `.yaks`); it is created if absent, and `./.yaks` becomes a pointer
  file (`path:` as given, plus `herd:` from `--herd`) that discovery follows,
  also listed in `.git/info/exclude`. Excludes go to `.git/info/exclude`, never
  `.gitignore` (committed: it would leak the farm's existence). `private` and
  `pointer` need a git repo; in a linked worktree the shared exclude is edited.
- **Skills.** `--skills default|coordination` (`none` skips) installs through the
  same code as `skills install` — project-local in the git top-level, never
  overwriting an edited skill. `--mode` alone installs the default set;
  `--skills` alone means `team` for a new farm and leaves an existing farm's mode
  as it is. In `private`/`pointer` modes each installed skill directory is
  excluded too; in `team` mode they are ordinary files you may commit. Outside a
  git repo `--skills` is an error (use `yaks skills install` for `~/.agents/skills`).
- **Idempotent.** Every step prints one line, done or already-done. The same
  command again changes nothing and ends `Nothing to change`; more flags add only
  what is missing (an exclude line, a skill, the pointer). A **different** `--mode`
  on an existing farm is an error that names what would have to change; init never
  converts a farm. Settings flags (`--type`, `--priority`, `--herd`, `--emacs`) that
  contradict an existing farm's `config.yaml` are also an error; init does not rewrite it.
- **Workers.** `private`/`pointer` farms are invisible to a Delta or worktree
  checkout (discovery stops at that checkout's git top-level), so init prints the
  `YAKS_DIR=<farm>` to put in such a worker's brief.

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
