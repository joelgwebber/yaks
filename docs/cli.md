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
4. **The farm as a branch** (yaks-73f4). Before that error, a top-level with
   no `.yaks` entry looks for the **farm branch**: `git config yaks.branch`
   (default `yaks`), an orphan branch whose tree is only `.yaks/`. If a
   worktree of the repository (its host, for a Delta clone: the repository its
   object store borrows from) has it checked out, that worktree's `.yaks/` is
   the farm, so every checkout on a machine (the primary, git worktrees, Delta
   clones) reads and writes ONE farm and `main` carries no yak files at all.
   If the branch exists (locally or as `origin/<branch>`, which is all a fresh
   `git clone` has) with no worktree, the first command that opens the farm
   creates one at the default farm home (`$XDG_DATA_HOME/yaks/farms/<repo>-<hash>`,
   see `migrate --to-branch`), a local branch tracking `origin/<branch>`, and
   prints one stderr line saying where; later runs are silent. It never
   fetches, and never creates one from a Delta clone (it names the main checkout
   instead). `YAKS_NO_BOOTSTRAP=1` turns it off: the command then fails with an
   error naming `git worktree add <path> <branch>`. A committed
   in-tree `.yaks/` in a repository whose farm branch has a worktree elsewhere
   (an old checkout forked before `yaks migrate --to-branch`), or one whose
   config has a `live_branch:` line, is a stale copy: discovery uses the farm
   branch's worktree instead and warns once on stderr, naming the copy to
   remove. `YAKS_DIR` naming the copy still wins. Set up:

   ```sh
   yaks migrate --to-branch                                     # move a committed in-tree farm (below)
   git -C <repo> worktree add --orphan -b yaks ../<repo>-yaks   # a new farm
   git -C <repo> worktree add <path> yaks                       # an existing one, at a path you choose (else the first yaks command makes it)
   ```

   `yaks commit` (from any checkout) commits the farm on its branch, in the
   farm worktree; no code checkout, branch or PR ever carries yak churn.
   `yaks sync` commits, pulls (rebasing with yaks's merge driver, so two
   machines' edits to one yak merge cleanly) and pushes the branch; nothing is
   synced automatically.
   `yaks path` with no arguments prints the farm a command resolved.

## Filter flags (shared)

`--status <s>` · `--type <t>` · `--priority <n>` · `--label <l>` · `--herd <h>`
(id prefix) · `--search <text>` · `--parent-of <id>` (descendants at any depth) · `--ready` ·
`--tangled` · `--needs` (any `needs` value, `human` or `agent`). They compose, and back `list`, `search`, `log`, and the
`bulk` selector.

## Read / query

| Command | What it does |
|---|---|
| `list` | List tasks (non-dead by default; `--all` includes dead). Status renders as `[H]`/`[S]`/`[N]`/`[X]` — hairy / shaving / shor**n** / dead. |
| `show <id> [--json]` | Show one yak: fields, references, children, body + notes. [Details](#show) |
| `path [<id>...]` / `path <filters>` | Print each yak's current file path (absolute, one per line), by id or by the shared filter flags (`--all` adds dead yaks). With neither, prints the farm root this command resolved. [Details](#path) |
| `next` (alias `ready`) | Hairy yaks whose dependencies are all resolved — the work queue. Leaves out `needs: human`; includes answered yaks (`needs: agent`), marked `✓ answered, awaiting pickup`. [Details](#next) |
| `tangled` (alias `blocked`) | Hairy yaks with at least one unresolved dependency. |
| `search <text>` | Substring search over id / title / description. |
| `stats` | Counts by status, type, priority. |
| `sheds` | EVERY checkout of this repo — plain `git worktree`s and Delta clones — with its git state and the yaks it has been writing to the shared farm (`--json` for scripts). Read-only. [Details](#sheds) |
| `discover [PATH]` | Diagnostic: the chain `sheds` uses to find the other checkouts, step by step, from `PATH` (default the cwd). `--json`. Read-only; needs no farm. [Details](#discover) |
| `log` | Timestamped notes and status moves (`moved: <from> -> <to>` entries) across a filtered set, oldest first (`--since <2h\|3d\|date>`, `--by <actor>`). |
| `refs <id>` | The yaks a task points at (parent, deps, id mentions), flagging danglers. |
| `refs <id> --incoming [--json]` | The reverse: every yak that points AT this one, one line per reference (`child`, `depends` or `mention`). Computed by scanning the farm on demand. `--json`. The TUI's `B` key opens the same list. [Details](#refs) |
| `commits <id>` | Git commits linked to a yak — those naming its id, and those that touched its file across status moves. |
| `rollup` | Group yaks by the external issue they roll up to (`--keys` lists the external keys). |
| `scan-ids [file]` | Scan text/stdin for real yak-ids; prints `line:col  id`, exits non-zero if any found — a leak check for private farms (pre-commit / PR hook). |

### `show`

`show <id> [--json]`

Show one yak: fields, references, children, body + notes. An unknown id exits 1.

### `path`

`path [<id>...]` / `path <filters>`

Print each yak's current file path (absolute, one per line), by id or by the shared filter flags (`--all` adds dead yaks). Each yak's file is `.yaks/<herd>/yaks/<id>.md`, the same path for its whole life (a status move edits it in place); its attachments are in `.yaks/<herd>/yaks/<id>/`.

An unknown id goes to stderr with a non-zero exit (known ids still print). Ids or filters, not both; with neither, prints the farm root this command resolved (the farm branch's worktree, when the farm is a branch): which farm am I on?

### `next`

`next` (alias `ready`)

Hairy yaks whose dependencies are all resolved — the work queue. A yak awaiting a human (`needs: human`) is left out; one that has been answered (`needs: agent`) is included, marked `✓ answered, awaiting pickup` (`yaks pickup <id>` when you take it on).

### `sheds`

`sheds [--json]`

EVERY checkout of this repo — plain `git worktree`s and Delta clones — with its git state and the yaks it has been writing; a shed is one such checkout.

Every checkout on a machine shares ONE farm (the farm branch's worktree), so there is no per-shed copy of the farm to compare. What a shed has been doing comes from the farm's **activity log**: every write of a yak file (a note, a move, an ask, an answer, an attachment, an edit) records the checkout it ran in, one line per write, in `<farm>/.activity/<hash>.log` (the hash is of the checkout's path; the file's first line names it). The log is per-machine metadata: its directory ignores itself, so it is never committed, never merged, and invisible to `yaks status` and `yaks commit`. Who wrote a note stays the note's actor, in the yak file. `YAKS_NO_ACTIVITY=1` turns the logging off.

The checkout you are in is listed too: a `*` in the first column and `(here)` on its `name:` line (JSON `current: true`, exactly one row). The farm branch's own worktree is not a shed (`yaks status` reports its state).

Per shed: kind (`worktree`/`delta`), branch (or detached), short HEAD, `+N -M` (commits it has that this checkout's HEAD lacks / commits this HEAD has that it lacks; JSON `ahead`/`behind`), or `+? -?` (JSON `null`) when there is no merge-base here (shallow history, a commit unknown here; when this repo is shallow one line at the end says `git fetch --unshallow` fixes it), the dirty-file count, its LAST WRITE to the farm, and its path. Under the row:

- `name:` (JSON `name`): `main` for the primary checkout, else the slug of the Delta thread title (lowercase, runs of other characters one `-`, at most 24 characters, cut at a `-`; two clones with one slug get their dir id's first 4 characters appended, e.g. `fix-it-q7zz`), else its Delta dir id, else its branch. The title is read from the clone's own git config `yaks.shed.title` (JSON `title`, also shown on the `name:` line), which ANY yaks command run inside a Delta clone with `DELTA_THREAD_TITLE` set records (with `yaks.shed.thread` from `DELTA_CURRENT_THREAD_ID`), only when the value changed, silently, never for `-C`, never in a plain git worktree; it is per-clone and never committed.
- the activity label, each part only when it has something: `shaving: <ids>` (the yaks it wrote that are `shaving` now; JSON `in_progress`), `N asking` (those awaiting a human), how many yaks it wrote, and `active` (it wrote within the last 2 hours) or `stale` (it holds a shaving yak but has not written for longer); `idle` is silent. JSON `liveness`, always present.
- `state:` (JSON `merged`, `untracked`): `merged, clean` when its HEAD is in the primary checkout's history and its tree is clean (safe to clean up); otherwise `merged` (when it is) and `uncommitted: N modified, M untracked`. Not shown for the primary, or for a shed that is not merged and clean.
- its five most recently written yaks, newest first: `id  status  title  (last write)`, `⏳` first on one awaiting a human, `gone` for a yak no longer in the farm. JSON `touched` lists them all (`id`, `title`, `status`, `needs`, `last`), with `last_write`.

A pinned Delta clone whose checkout cannot be resolved (most often a gone git dir: a finished thread) is not a row; the text table counts them in one `orphaned: N` line (`yaks discover` lists each).

Discovery reads git data only, from inside any checkout of the repo (it never searches the filesystem): `git worktree list --porcelain` here; the HOST repo this checkout's `objects/info/alternates` names (the human's `.git` where the project was added from a folder, Delta's managed bare repo on a machine the thread was shared to; else this repo); every Delta clone pinned there as `refs/delta/<dir>/<name>/<sha>`, resolved to its checkout through the clone's git dir `<store>/<dir>/<name>.git` and its `core.worktree`; and, when the host is a checkout, that checkout and its own `git worktree list` (so a Delta clone sees the primary's git worktrees). A clone on a shared-thread machine cannot see the human's checkout. `yaks discover` shows each step. Strictly read-only in the other checkouts (`git --no-optional-locks`; no fetch, checkout or writes).

### `discover`

`discover [PATH]`

Diagnostic: the chain `sheds` uses to find the other checkouts, step by step, from `PATH` (default the cwd; any directory inside a primary checkout, a git worktree or a Delta clone; outside one it exits 1 and says so: nothing is searched for).

Sections: the checkout (top-level, layout: primary checkout / git worktree of … / Delta clone whose host is a checkout or a bare repo; origin; `local` and whether it is a checkout or BARE; the farm discovery resolves); `1. GIT WORKTREE LIST (here)`; `2. HOST` (own git dir, the host git dir and how it was found, the clone store); `3. DELTA PINS in the host` (each `<dir>/<name>` pin group with its pin count and its checkout, or why not: `gone (its git dir no longer exists)`, `checkout missing`, `this checkout`); `4. GIT WORKTREE LIST (host checkout …)` when the host is a checkout reached through alternates; then `SHEDS`, what `sheds` lists besides this checkout, with kind `worktree`/`delta`.

Read-only; needs no farm. `--json` (`host`, `worktrees_here`, `worktrees_host`, `pins[]` with `checkout`/`reason`, `sheds[]`). `scripts/discover-fixture.sh` builds Delta-shaped layouts (a linked machine, a shared machine, a gone pin, an unpinned look-alike) and runs it from each kind of checkout.

### `refs`

`refs <id> --incoming [--json]`

The reverse: every yak that points AT this one, one line per reference saying how: `child` (its `parent:`), `depends` (its `depends_on:`), or `mention` with where (`title`, `body:L<n>` for a description line, `note:<ts> [actor]` for a note).

Mentions are whole real ids, matched exactly as `refs` and the TUI links do. Computed by scanning the farm on demand (no index).

`--json` emits `{id, title, incoming: [{id, title, status, kind, in?, line?, ts?, actor?}]}`. The TUI's `B` key opens the same list as a picker.

## Create / edit

| Command | What it does |
|---|---|
| `create '<title>'` | New hairy yak. Title is positional or `--title`; `--type`/`--priority`/`--parent`/`--herd`/`--labels`/`--depends-on`/`--source`/`--description`/`--description-file`/`--verify`; `--json` prints id + file path. `--description -` reads stdin (see [Multi-line text](#multi-line-text)). [Details](#create) |
| `update <ids…>` | Update fields/labels, replace the body (`--description`/`--description-file`) or append a `--note`/`--note-file`; the same edit applies to every id. `--note -` reads stdin (see [Multi-line text](#multi-line-text)). [Details](#update) |
| `dep add\|remove <id> <dep>` | Add / remove a dependency. |
| `reparent <ids…> --parent <id>` | Move yaks under a new parent (or `--unparent` to top-level). |
| `rename <old> <new>` | Rename a yak's id, updating every reference across the farm (its file, its `.yaks/<herd>/yaks/<id>/` attachment dir and the `](<old>/…)` links to it). |
| `rename-herd <old> <new>` | Rename a whole herd: migrate every id from one prefix to another (e.g. `yaksrs` → `yaks`). (`rename-prefix` still works as an alias.) |
| `merge <path> [--dry-run]` | Merge another farm's yaks (all statuses + attachments, byte for byte) into this one, preserving ids and status. Non-destructive; refuses on id collisions. [Details](#merge) |
| `merge-driver <base> <ours> <theirs> <path>` | **Internal** (hidden from `--help`): git's merge driver for yak files. Merges the three versions of one yak into `<ours>` as a valid yak, never conflict markers. Needs no farm. [Details](#merge-driver) |
| `bulk <filter> <mutation>` | Filter-driven field edit. **Dry-run by default**: changes nothing without `--commit`. Requires ≥1 filter flag and ≥1 mutation flag. Field edits + reparent only. [Details](#bulk) |

### `create`

`create '<title>'`

New hairy yak. Title is positional or `--title`; `--type`/`--priority`/`--parent`/`--herd`/`--labels`/`--depends-on`/`--source`/`--description`/`--description-file`/`--verify`; `--json` prints id + file path. `--description -` reads the body from stdin (see [Multi-line text](#multi-line-text)).

`--labels` (like `--add-label`/`--remove-label`/`--label` everywhere) splits on commas and whitespace — a label may contain neither, so `--labels ui,docs`, `--labels 'ui, docs'`, and `--labels ui docs` all give `[ui, docs]` (duplicates dropped).

`--herd` sets the new yak's herd (id prefix; default: the `.yaks` pointer file's `herd:`, else the config `herd:`), so one `.yaks/` can hold several herds.

With neither `--herd` nor a default herd, `create` fails (non-zero) rather than silently minting a `yak-` herd.

### `update`

`update <ids…>`

Update fields/labels, replace the body (`--description`/`--description-file`) or append a `--note`/`--note-file`; the same edit (and the same text) applies to every id.

`--note -` / `--description -` read stdin (see [Multi-line text](#multi-line-text)).

`--as <actor>` attributes the note. `--verify '<cmd>'` sets (or, empty, clears) the yak's verification command; `--source <url>` likewise sets (or, empty, clears) its external `source:`.

### `merge`

`merge <path> [--dry-run]`

Merge another farm's yaks (all statuses + attachments, copied byte for byte; a source on the old layout is refused, see `init`) into this one, preserving ids and status, and declaring any incoming herd in this farm's `herds:` config so it shows up in the pickers.

Non-destructive (source left intact); refuses on id collisions — reconcile the source's herd with `rename-herd` first.

### `merge-driver`

`merge-driver <base> <ours> <theirs> <path>`

**Internal** (hidden from `--help`): git's merge driver for yak files, run as `yaks merge-driver %O %A %B %P` (registered by `.gitattributes` `.yaks/**/*.md merge=yaks` + `git config merge.yaks.driver`).

Merges the three versions of one yak into `<ours>` as a valid yak, never conflict markers: note entries unioned (deduped, ordered by timestamp), `updated` the later, `labels`/`depends_on` merged (base + additions − removals), the description and every other field taken from the side that changed it; a field both sides changed differently keeps `<ours>` and is recorded in one `merge-driver` note (`merge conflict: status was X here, Y on the other side; kept X`).

Exits 0 on a merge (conflicts included), 1 and leaves `<ours>` untouched when an input is not a yak file (git then reports a conflict for the path), 2 on an I/O error. Needs no farm.

### `bulk`

`bulk <filter> <mutation>`

Filter-driven field edit. **Dry-run by default** — prints the matched set + the mutation and changes nothing without `--commit`.

Requires ≥1 filter flag (never the whole farm) and ≥1 mutation flag (`--add-label`/`--remove-label`/`--set-priority`/`--set-type`/`--reparent`/`--unparent`). Field edits + reparent only — no state transitions.

## Verification

| Command | What it does |
|---|---|
| `verify <ids…>` | Run each yak's verification command with live output, and record `verify: <cmd> -> PASS/FAIL (exit N)` as an attributed note. Exits non-zero if any fails. Explicit only. [Details](#verify) |
| `attach <id> <path>` | Copy a local file under `.yaks/<herd>/yaks/<id>/` and link it in the yak's body — artifact evidence a human can view. `--name` renames it; `--note`/`--as` record an attributed note. [Details](#attach) |
| `rename-attachment <id> <old> <new>` | Rename `.yaks/<herd>/yaks/<id>/<old>` and rewrite every link to it across the farm. Refuses to overwrite an existing attachment. [Details](#rename-attachment) |
| `attachment <id> [<file>]` | Print the absolute path of a yak's attachments (all, or the one named). `--json`. Read-only. [Details](#attachment) |

### `verify`

`verify <ids…>`

Run each yak's verification command with live output, and record `verify: <cmd> -> PASS/FAIL (exit N)` as an attributed note. Exits non-zero if any fails. Explicit only — never auto-run.

The command is the yak's own `verify:` field if set, else the config default resolved by the yak's labels (see below).

The scriptable form of a yak's evidence contract: a check anyone (or CI) can re-run.

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

### `attach`

`attach <id> <path>`

Copy a local file under `.yaks/<herd>/yaks/<id>/` (beside the yak's file) and link it in the yak's body as `![name](<id>/<file>)` for an image (png, jpg, jpeg, gif, webp, svg) or `[name](<id>/<file>)` for any other file (what GitHub and every CommonMark viewer expect) — artifact evidence (screenshots, TUI frames) a human or yakherd can view.

`--name <name>` stores it under a different filename (sanitized; the source's extension is kept if omitted). `--note`/`--as` record an attributed note.

The non-scriptable, look-at-it form of evidence: an **external file** (committed in team mode with the rest of `.yaks/`), never inlined.

### `rename-attachment`

`rename-attachment <id> <old> <new>`

Rename `.yaks/<herd>/yaks/<id>/<old>` (e.g. an ugly `paste-….png`) and rewrite every link to it across the farm (the `![name]` / `[name]` text follows the new stem, and the link form follows the new extension). `<new>` is sanitized and keeps `<old>`'s extension if it omits one; refuses to overwrite an existing attachment.

### `attachment`

`attachment <id> [<file>]`

With no `<file>`, lists the absolute path of each of the yak's attachments (in `<herd>/yaks/<id>/`); with `<file>`, prints that one's path (exit 1 when the yak has no such attachment). `--json`. Read-only.

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
`▸ <ts> [<actor>]`, a blank line, then `moved: <from> -> <to>` (one per yak moved, so
`slaughter --family` writes one on each), and each verb takes `--as <actor>`
like the note commands. The file alone says who moved the yak and when, and
`yaks log` lists moves with the notes. A yak moved by an older binary simply has
no entry.

## Human-in-the-loop

| Command | What it does |
|---|---|
| `ask <id> --note '<question>'` | Block a yak on a human: sets the `needs` field, dropping it out of `next`. Also `--note -` (stdin) / `--note-file PATH`. [Details](#ask) |
| `answer <id> --note '<reply>'` | Answer a question (human-reserved): records the reply and sets `needs: agent`; `--done` clears `needs` instead. [Details](#answer) |
| `pickup <id> [--note '<text>']` | An agent takes on an answered yak: clears `needs: agent` and records a `picked up` note (`--as`). Fails on a yak that is not `needs: agent`. [Details](#pickup) |
| `inbox [--for human\|agent] [--json]` | Every yak with a `needs` value, across all statuses: **Awaiting a human** and **Answered, awaiting an agent**. `--for` shows one section; `--json`. [Details](#inbox) |

### `ask`

`ask <id> --note '<question>'`

Block a yak on a human: sets the `needs` field, dropping it out of `next`. On a yak that is already `needs: agent` it flips back to `human` and says so. Also `--note -` (stdin) / `--note-file PATH`.

### `answer`

`answer <id> --note '<reply>'`

Answer a question (human-reserved): records the reply and sets `needs: agent`, so the answer stays findable (in `inbox`, and in `next` when the yak is hairy) until an agent picks it up. `--done` clears `needs` instead, for an answer that needs no follow-up.

A second answer on a `needs: agent` yak keeps the state and appends the note; on a yak with no `needs` it only records the note. Also `--note -` / `--note-file PATH`.

### `pickup`

`pickup <id> [--note '<text>']`

An agent takes on an answered yak: clears `needs: agent` and records a `picked up` note attributed to the actor (`--as`).

The only command that clears `needs: agent`; on a yak that is not `needs: agent` it fails, naming the state. A `needs` change is not a status move (no `moved:` entry).

### `inbox`

`inbox [--for human|agent] [--json]`

Every yak with a `needs` value, across all statuses, in two sections: **Awaiting a human** (`needs: human`; marked `↩ replied` when its latest note after the ask is from someone other than the asker, i.e. a plain-note reply that did not use `answer`) and **Answered, awaiting an agent** (`needs: agent`).

`--for` shows one section.

`--json` is one array of yaks, each with `needs` and a derived `replied` boolean.

`ask` records an `asked: needs <who>` first line in its note: that is what `replied` is measured from, so a yak whose `needs:` was set by hand has no known asker and is never `replied`.

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
| `migrate [--dry-run]` | Bring a farm on an older schema up to this yaks' (schema 4 or 5 → 6), under the farm lock. `--dry-run` writes nothing. Idempotent; never commits. [Details](#migrate) |
| `migrate --to-branch [--branch <name>] [--worktree <path>] [--dry-run]` | Move a repo's committed in-tree `.yaks/` farm onto an orphan-tree **farm branch** in its own worktree. `--dry-run` prints the plan. **Never pushes.** [Details](#migrate) |
| `private [--off]` | Mark this repository's farm private so it never leaves the machine: sets `git config yaks.private true` and installs a `pre-push` hook that refuses pushes of the farm branch. `--off` undoes it. [Details](#private) |
| `init` | Create a `.yaks/` farm in the current directory (team mode). With `--mode` and/or `--skills` it does the full setup, one line per step. Flags: `--herd`, `--type`, `--priority`, `--emacs`, `--mode`, `--path`, `--branch`, `--skills`. [Details](#init-modes-and-skills-in-one-idempotent-step) |
| `skills install` | Install the bundled agent skills (`yaks`, `yaks-tracker`) to `--dir <path>`, `--user` or the project's `./.agents/skills`. `--with coordination` adds `yaks-coordinating` and `yaks-working`. Edited copies are left alone unless `--force`. [Details](#skills) |
| `skills status` | Per-skill verdict (`current`, `stale`, `adoptable`, `held`, `modified`, `unmanaged`, `source`, `retired`) from the provenance stamp. `--user` / `--dir` inspect another directory. See [skills.md](skills.md). [Details](#skills) |
| `doctor` | Read-only integrity check: invalid status, duplicate id, misplaced file, dangling parent/deps, malformed labels. Exits non-zero on any issue. `--strict` also flags shorn yaks with no note or whose `verify:` did not last PASS. [Details](#doctor) |
| `preflight [<id>...] [--all] [--push-main]` | Read-only landing-readiness check: farm changes tracked in git, shorn yaks' `verify:` last PASSed, farm integrity. Prints `preflight: FAIL …` per failure, else `preflight: ok`; `--push-main` checks the `local` remote. [Details](#preflight) |
| `brief <id> --as <name> [--yaks-dir <path>]` | Print the worker brief for one yak, read-only: the part a worker needs that the yak, the config and the farm determine. `--yaks-dir` names the farm. [Details](#brief) |
| `status [--check] [--json]` | What the farm has changed that git does not have yet, one line per yak, in yak terms. Read-only. `--check` exits 1 when the farm is dirty; `--json`. [Details](#status) |
| `sync [--no-commit] [--no-pull] [--no-push] [-m <msg>]` | Sync a farm that lives on its own branch with its remote: commit, pull (rebasing with the yak merge driver), push. `--no-commit`/`--no-pull`/`--no-push` skip a step. A private farm is never pushed. [Details](#sync) |
| `commit [-m <msg>] [--dry-run]` | Commit the farm's own changes (everything under `.yaks/`) and nothing else. `-m` overrides the generated message; `--dry-run` lists files and message. Never pushes. [Details](#commit) |
| `land <sha> [--dry-run] [--force-discard]` | Land a Delta worker's commit by SHA on the current branch of the code checkout (squash, else cherry-pick). Needs no farm; never pushes. `--dry-run` changes nothing; `--force-discard` drops unexplained working-tree changes. [Details](#land) |
| `tui` | Open the interactive terminal UI (see [tui.md](tui.md)). |

### `migrate`

`migrate [--dry-run]`

Bring a farm on an older schema up to this yaks' (schema 4 or 5 → 6), under the farm lock, writing `.yaks/schema` last.

First the **`herd-directories`** layout step: every yak file (`<id>.md` at the farm root whose `id:` is its name) and its attachment dir (`<id>/`) moves to `<herd>/yaks/` (plain renames: git sees 100% renames and `git log --follow` keeps working; bodies are untouched, since attachment links are relative to the yak file); other root files stay; refuses, moving nothing, if a destination exists or a herd is named `docs`.

Then the named fixups over every yak file: **`comment-header-blank-line`** puts a blank line after every comment's `▸ <ts> [actor]` header (so the header is a paragraph of its own in any CommonMark viewer) and **`attachment-links`** relinks non-image attachments as `[stem](<id>/<file>)` instead of `![stem](<id>/<file>)` (images keep the `!`). Only the body shape changes: frontmatter, `updated:` and attachments are untouched.

`--dry-run` lists, per fixup, the counts and the yaks it would touch and writes nothing.

Idempotent: a current farm says `nothing to migrate`.

Every OTHER command refuses an older farm, naming `yaks migrate` (`yaks sheds` reports an unmigrated shed instead of reading it as empty).

Never commits: it prints the commit to make. A migration rewrites nearly every yak file, so land it as its own commit (see `RELEASING.md`), and quit every running `yaks tui` first.

`migrate --to-branch [--branch <name>] [--worktree <path>] [--dry-run]`

Move a repo's committed in-tree `.yaks/` farm onto an orphan-tree **farm branch** (default `git config yaks.branch`, else `yaks`) in its own worktree (default `$XDG_DATA_HOME/yaks/farms/<repo>-<hash>`, i.e. `~/.local/share/yaks/farms/...`; `%LOCALAPPDATA%\yaks\farms` when `HOME` is unset).

Run it in the checkout whose farm moves (a Delta clone is sent to its main checkout).

The branch starts at HEAD, so the farm's history stays reachable (`yaks commits <id>`, `git log --follow`), and its first commit keeps only the top-level `.yaks/`; the worktree is filled from that commit, so it holds only `.git` and `.yaks/` (a nested fixture `.yaks/` stays on the code branch).

Then one commit on the current branch removes `.yaks/` (`yaks: the farm moves to branch <b>`) and the ignored leftovers (`.lock`) are deleted.

A `--branch` other than the one discovery already resolves is recorded in `git config yaks.branch`.

Refuses, changing nothing: uncommitted changes (untracked files too), a merge or cherry-pick in progress, a private (untracked) farm, a detached HEAD, an existing branch (local or `origin/<b>`) or worktree path. A farm already on a branch says so (exit 0).

`--dry-run` prints the plan.

**Never pushes**: it prints `yaks sync` (which publishes the farm branch) and the code branch's push, and a note that another machine or a teammate needs no setup (the first yaks command in a clone creates the worktree from `origin/<b>`).

### `private`

`private [--off]`

Mark this repository's farm private so it never leaves the machine: sets `git config yaks.private true` and installs a `pre-push` hook (in the hooks dir git uses; `core.hooksPath` honored, shared by every worktree) that refuses any push of the farm branch (`yaks` or `yaks.branch`), including `git push --all`, while pushing `main` etc. still works. `yaks sync` reads the same flag and never pushes a private farm (it still commits and pulls).

A `pre-push` hook that is not yaks's (no `# yaks-private-guard` marker) is left untouched and the command prints the check to fold into it.

`--off` unsets the flag and removes yaks's own hook only. Needs no farm, touches no remote

### `init`: modes and skills in one idempotent step

Create a `.yaks/` farm in the current directory (`config.yaml` + `schema`; each yak is one file, `.yaks/<herd>/yaks/<id>.md`, with a `status:` field (no status directories; `docs/` and `<herd>/docs/` are reserved for non-yak documents, so `docs` is not a valid herd); a farm on the old layout, schema 3 with yaks in `hairy/ shaving/ shorn/ dead/`, is refused by EVERY command, reads included, with `<farm> uses the old yak layout (status directories, schema 3), which this yaks no longer reads: migrate it once with yaks 0.0.16 (npx -y @j15r/yaks@0.0.16 migrate), commit, then use this yaks again`, and `yaks sheds` shows such a clone as `old layout (pre-0.0.16 clone)`, never as an empty farm).

With no `--mode`/`--skills`/`--path` it only creates a committed (team) farm, then prints that skills are not installed and the exact command to add them. With `--mode` and/or `--skills` it does the full setup, one printed line per step (see below).

Flags: `--herd`, `--type`, `--priority`, `--emacs`, `--mode team|private|pointer|branch`, `--path <dir>`, `--branch <name>`, `--skills none|default|coordination`. `--mode branch` starts the farm on its own orphan branch instead (see below).

```sh
yaks init                                   # farm only (team); prints how to add skills
yaks init --skills default                  # team farm + project-local skills
yaks init --mode private                    # farm in .yaks/, kept out of git; default skills
yaks init --mode pointer --path ~/farms/work --herd web   # .yaks pointer file -> a farm elsewhere
yaks init --mode branch [--branch personal-joel]          # the farm on an orphan branch, worktree outside the checkout
```

- **Modes.** `team`: the farm in `.yaks/`, committed with the code. `private`:
  the same, plus `/.yaks` in `.git/info/exclude`. `pointer`: `--path <dir>` is the
  directory holding the farm (its `.yaks/`, or the farm itself when `<dir>` is a
  farm or named `.yaks`); it is created if absent, and `./.yaks` becomes a pointer
  file (`path:` as given, plus `herd:` from `--herd`) that discovery follows,
  also listed in `.git/info/exclude`. Excludes go to `.git/info/exclude`, never
  `.gitignore` (committed: it would leak the farm's existence). `private` and
  `pointer` need a git repo; in a linked worktree the shared exclude is edited.
  `branch`: the farm is an orphan git branch (`--branch <name>`, else
  `git config yaks.branch`, else `yaks`) checked out in its own worktree at the
  farm home (the `yaks migrate --to-branch` default), so a new repo needs no
  hand `git worktree add --orphan`. A non-default name is recorded with
  `git config yaks.branch`. If the branch already exists (locally or as
  `origin/<name>`) init only adds its worktree; the farm's `config.yaml`
  settings are never rewritten. A new branch is unborn: nothing is committed or
  pushed (`yaks sync` publishes it, `yaks private` keeps it local). Refused: a
  checkout with an in-tree `.yaks/` (use `yaks migrate --to-branch`), a Delta or
  `--shared` clone when a worktree would have to be created (run it in the main
  checkout), `--path` (the worktree location is the farm home), and a name that is
  the checked-out code branch or an existing branch holding no `.yaks/`. Skills
  go in the code checkout, as ordinary project files.
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

### `skills`

`skills install`

Install the bundled agent skills (`yaks`, `yaks-tracker`). Target: `--dir <path>` if given; else `--user` → `~/.agents/skills`; else `./.agents/skills` in the git top-level of the cwd when inside a git repo, and `~/.agents/skills` outside one.

Prints the destination it chose.

`--with coordination` adds `yaks-coordinating` and `yaks-working`; untouched installed copies of the retired `yaks-coordinating-team`, `-private`, `-worktrees` and `-delta` are removed (an edited one is kept and reported). `skills status` lists any left as `retired`.

A skill is installed as a unit; an outdated untouched copy is upgraded, an edited one (including an edited script) is left alone unless `--force`.

Refuses to write onto yaks' own `.agents/skills/` source (even via a symlink, even with `--force`), naming `--user`/`--dir` as the way out.

`skills status`

Per-skill verdict — `current` / `stale` / `adoptable` / `held` / `modified` / `unmanaged` / `source` / `retired` — from the provenance stamp (other files of a skill are compared by content).

Same default directory as `install`; `--user` / `--dir` to inspect another; installed opt-in skills are always listed, `--with coordination` also lists absent ones.

When something is stale it ends with the command to run (`yaks skills install`, plus `--with coordination` if an opt-in skill is stale).

An ordinary command prints that advice itself as one stderr `note:` line, once per yaks version per checkout, for a stale project-local install (never with `--json`, nor for `skills`/`init`; `YAKS_SKILLS_AUTOSYNC=0` silences it); `doctor` repeats it in its skills advisory. See [skills.md](skills.md).

### `doctor`

`doctor`

Read-only integrity check: a `status:` value that is not hairy/shaving/shorn/dead (`invalid-status`), one `id:` in the frontmatter of two yak files (`duplicate-id`), a yak file away from its `<herd>/yaks/<id>.md` (in another herd's directory, under another name, or loose at the farm root; not loaded from there: `misplaced-file`), dangling parent/deps, malformed labels (a legacy label containing a comma or space, e.g. `ui,docs` — any label edit on that yak re-splits it).

Exits non-zero on any issue (CI-usable).

`--strict` also flags shorn yaks with no recorded note, and shorn yaks whose `verify:` command did not last PASS (evidence-before-shear).

### `preflight`

`preflight [<id>...] [--all] [--push-main]`

Read-only landing-readiness check; run it before committing a shorn yak or merging a shed in a team farm.

Checks: (1) nothing under `.yaks/` is untracked or has unstaged changes in git (staged is fine: it is the step before the commit) — a new `<id>/` attachment dir never `git add`ed fails and is named; (2) every shorn yak in scope whose `verify:` command (own, else the config default for its labels) last PASSed, same rule as `doctor --strict`; scope is the ids given, else the shorn yaks that are part of the change in git (staged, modified or new under `.yaks/`, including a new `<id>/` attachment dir), and `--all` checks every shorn yak (old ones that never ran `verify` will fail); (3) farm integrity: no invalid `status:` value (`invalid-status`) and no `id:` in two yak files (`duplicate-id`).

`ids` scopes check 2 only; 1 and 3 are farm-wide.

Prints one `preflight: FAIL <check>: …` line per failure and exits non-zero, else `preflight: ok`.

In a private farm (nothing under `.yaks/` tracked by git) check 1 is skipped with a printed line and check 2 covers every shorn yak; 3 still runs.

`--push-main` adds (4), for the yakherd about to `git push local <branch>:main` (off by default: it depends on another checkout's state, not on the change, and a repo with no `local` remote is not a failure without it): `local` must be a path on this machine (an https/ssh URL is reported as not the human's checkout, never resolved), a non-bare git repository with `refs/heads/main`, and not checked out on `main` with uncommitted changes to tracked files (git refuses the push then). Otherwise it fails with `preflight: FAIL local-checkout: …`, naming the fallback: push `pr/<name>` to `local` and give the human `git fetch <path> pr/<name>`, or repoint `local`.

Read-only (`remote get-url`, `ls-remote`, `rev-parse`, `status`); never pushes.

When the farm is a BRANCH (its own worktree, `farm_branch`), uncommitted yak changes are that farm's normal state and the code push does not carry them, so check 1 does not fail: they become ONE line, ``preflight: warning: farm has N uncommitted change(s): run `yaks commit` `` (never "git add it"; `--push-main` is about the code branch, so it is unaffected); unresolved merge conflicts still fail. In-tree team farms are unchanged.

`--json` emits `{ok, failures[{check, message, subjects}], skipped, warnings}`.

### `brief`

`brief <id> --as <name> [--yaks-dir <path>]`

Print the worker brief for one yak, read-only: the part a worker needs that the yak, the config and the farm determine.

It names the yak and that it is already `shaving`; the `YAKS_ACTOR=<name>` prefix for every command (plus `YAKS_DIR=<farm>` for a private or pointer farm, or whatever `--yaks-dir` names); this binary's absolute path; the evidence (the yak's own `verify:`, else the config default for its labels and herd); the forbidden moves; how to finish, chosen by farm mode (team: `yaks shorn`, ONE commit staging `.yaks/<herd>/yaks/<id>.md` and any `.yaks/<herd>/yaks/<id>/`, message `<id>: ...`; private: no yak id or the word yaks in commits); how to ask and what to do once the ask is answered (`inbox --for agent`, then `pickup`); the `Decision:` note rule; multi-line notes on stdin; and the final-message format.

Mode is detected as the skills do (tracked farm in the repo of the cwd = team).

An unclaimed yak still prints, with a warning on stderr; an unknown id fails with nothing on stdout; `--as` must be a shell-safe name.

What a yak cannot know (spawn title, how work lands, the model, scope by symbol) stays with the coordinator, who adds it around this text.

### `status`

`status [--check] [--json]`

What the farm has changed that git does not have yet, one line per yak, ordered by id, in yak terms: `created`, `moved hairy -> shaving` (the `status:` field of its `<id>.md` differs between HEAD and the working tree), `notes +2` (notes appended; the `moved:` entry a move writes is not counted), `edited` (a field, the description or an existing note changed), `removed`, `artifacts` (a new or changed file under `.yaks/<herd>/yaks/<id>/`, the yak's attachments); a yak with several kinds of change gets one line with all of them, and `config` is `config.yaml`.

A `*` in the first column marks a yak whose changes are all staged.

It uses the same classification as `commit` (the verbs cannot drift: `created`/`removed`/a move read the same, `notes` and `edited` are both `updated`), and ends with the message `commit` would use.

Code changed elsewhere in the repo is not listed, only counted in one line (`N other changed files outside the farm`), so you know `commit` leaves it alone.

A clean farm prints `farm clean: nothing to commit`; a private farm (nothing under `.yaks/` tracked by git) prints `private farm: not tracked by git, nothing to commit`; both exit 0.

While a merge or cherry-pick is in progress it says `yaks commit` will refuse (git forbids the partial commit) and why.

Read-only: nothing is staged, committed or written.

`--check` exits 1 when the farm is dirty (same output, for scripts and hooks; otherwise the exit code is 0).

When the farm is a BRANCH (its own worktree on the farm branch, not an in-tree `.yaks/`), the last line says how that branch stands against its upstream, from the refs as of the last fetch (status never fetches or writes): `farm branch yaks: up to date with origin/yaks`, ``farm branch yaks: 2 ahead, 0 behind origin/yaks (as of the last fetch); `yaks sync` publishes`` (`reconciles` when it is also behind), or ``farm branch yaks: not published (no upstream); `yaks sync` publishes it``. `--check` is unaffected (dirty means uncommitted changes only).

`--json`: `{clean, private, merge_in_progress, yaks: [{id, changes, staged}], other: [{path, staged}], outside_farm, message, sync}`, where `sync` is `{branch, upstream, ahead, behind}` (`upstream` null when not published) or null for an in-tree farm.

### `sync`

`sync [--no-commit] [--no-pull] [--no-push] [-m <msg>]`

Sync a farm that lives on its own branch with its remote, in the farm worktree whatever the cwd: (1) registers the yak merge driver (`.gitattributes` rule `.yaks/**/*.md merge=yaks`, committed on the farm branch, plus `merge.yaks.driver` in the repo config); (2) commits what is dirty, as `yaks commit` (hand edits included); (3) fetches the farm branch from its remote (`branch.<b>.remote`, else `origin`) and rebases onto it, holding the farm lock, so concurrent edits to one yak on two machines merge into a valid yak (notes unioned, a field both sides changed: this side kept and the other recorded in a `merge-driver` note); (4) pushes, creating the remote branch with the same name and setting the upstream on the first publish, and pulls once more and retries if the remote moved meanwhile.

Each step can be skipped: `--no-commit` (a dirty farm then skips the pull too, leaving its edits local), `--no-pull`, `--no-push`.

A private farm (`yaks private`) commits and pulls but is never pushed.

A rebase that stops on a conflict the driver cannot settle (a non-yak file such as `config.yaml`) is aborted, leaving the branch exactly as it was, and the files are named. Refuses a farm committed beside the code (sync it with the code).

Nothing is ever synced or committed automatically: run it at session start, after landings and before a machine switch. Ends with the `yaks status` sync line.

### `commit`

`commit [-m <msg>] [--dry-run]`

Commit the farm's own changes — every change under `.yaks/` (yak files, attachments, config) — and nothing else, so a human's drifted edits land in one command and stop blocking a landing.

Stages the farm and runs `git commit --only -- .yaks`: files you staged elsewhere stay staged and out of the commit (named in the output), and modified code is never touched.

The message is generated from the changed files, one verb per yak (`created`, `shaving`/`shorn`/`dead`/`regrown` for a move, `updated`, `removed`, `artifacts for`, `farm config`), e.g. `yaks: created yaks-8c08; shaving yaks-c968`; `-m` overrides it. `--dry-run` lists the files and message without staging or committing.

A normal `git commit`, so the repo's hooks run (and may fail it: the farm changes are then left staged); it never pushes. A clean farm prints `nothing to commit` and exits 0.

While a merge or cherry-pick is in progress (a Delta landing leaves one pending) it refuses, stages nothing and says why: git forbids the partial commit it relies on, so finish the merge with a plain `git commit` (the farm changes are part of it) or abort it.

In a private farm (nothing under `.yaks/` tracked by git) it fails with a clear error. When the farm is a branch (rule 4 above) this commits in the farm's own worktree, on that branch, whatever the cwd.

### `land`

`land <sha> [--dry-run] [--force-discard]`

Land a Delta worker's commit by SHA on the current branch of the code checkout you are in (needs no farm; never pushes; git 2.38+, 2.40+ for the already-landed check).

Delta's applied state (a pending `MERGE_HEAD`, the worker's files in the working tree) is not trusted: it is reported (`worker commit:`, `pending state:`, a shallow-repository note when ancestry answers are lower bounds), then cleared.

When the worker's base is an ancestor of HEAD it computes the merged tree with `git merge-tree --write-tree` and lands it as ONE single-parent squash commit (the worker's message and author date + a `Squashed-from: <full sha>` trailer), then checks HEAD's tree equals the computed one; when the base was rewritten or the merge would conflict it says so (naming the `CONFLICT`s) and `git cherry-pick`s instead.

Refuses, changing nothing: working-tree changes the commit does not explain (exit 3; `--force-discard` drops them; an untracked file that differs from the worker's is always exit 4); a SHA that is already an ancestor of HEAD but whose change is missing or unconfirmable in HEAD's tree (exit 6, the evil merge: it names the first commit that lacks the change); a commit that touches `.yaks/` when HEAD tracks none, i.e. a branch farm (exit 7).

A SHA already in HEAD with its change present, or a worker landed before (the merged tree equals HEAD's), says `landed:` and adds no commit.

`--dry-run` reports every step and changes nothing.

Ends with `landed: <oneline>` and `doctor: <last line of yaks doctor>` when a farm resolves.

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
