# CLI reference

Run `yaks <command> --help` for full flags. Most read commands accept the shared
**filter flags** and `--json`; most note-writing and state-transition commands accept `--as <actor>`.

**Global flag: `-C <path>`** (like `git -C`; accepted before or after the
subcommand). Every command runs as if started in `<path>`: farm discovery,
`status`, `commit`, `init`, `skills`, `tui`. A relative
`<path>` is resolved against the real cwd; repeated `-C` compose, each relative
to the previous (`yaks -C a -C b list` runs in `a/b`). A missing path is an
error naming it. `$YAKS_DIR` still wins over discovery (a relative one is read
from `<path>`).

## Which farm a command uses

Every farm is a **farm branch**: an orphan branch (`yaks`, or `git config
yaks.branch`) whose tree is only `.yaks/`, checked out once per machine at
`$XDG_DATA_HOME/yaks/farms/<repo>-<hash>` (`~/.local/share/...`; see "One
worktree per repository" below). Every command
except `init` uses the first of:

1. **`$YAKS_DIR`**: a `.yaks/` directory, or a directory containing one
   (relative to the cwd). A value that names no farm is an error.
2. **`git config yaks.farm <checkout>`**: another repository's farm branch
   (any checkout of it). `git config yaks.herd <prefix>` is this repo's default
   herd for `create`. Set both with `yaks init --farm`.
3. **This repository's farm branch.**

Both git settings, and `yaks.branch`, are read from a Delta clone's host
repository too, so agent clones need no setup. Nothing in the checkout's own
tree is read: a leftover `.yaks/` directory or file there is ignored.

```sh
yaks path                              # the farm this command resolves
yaks init --farm ~/src/api --herd web  # use api's farm from this repo
git config --unset yaks.farm           # back to this repo's own
YAKS_DIR=~/scratch/.yaks yaks list     # one-off: any farm directory
```

A clone whose `origin/yaks` exists but has no worktree gets one on the first
command (one stderr line says where). A Delta clone whose host is a checkout
never makes one itself: it names that checkout instead. A Delta clone whose host
is a BARE repo (a cloud runner's managed repo, which nobody works in) adds the
worktree to the host, so every clone of it shares the farm; when the host lacks
the farm branch it is fetched from the host's `origin` first. `YAKS_NO_BOOTSTRAP=1`
turns all of that off.

**One worktree per repository.** The farm home is named after the
repository's `origin` URL (`schism-<hash>` for
`github.com/rocketsurgery-games/schism`, whatever the scheme or `.git`
suffix), so separate git dirs holding one repository, such as your own
checkout and the bare repository Delta manages for it, share one farm
worktree: a repository with no worktree of its own uses the one at its farm
home when that is on the farm branch and has the same `origin`. Writes from
your checkout and from Delta's clones meet at once, with no `yaks sync` between
them. A repository with no `origin` is named by its git dir, as before.

A machine that already has two worktrees for one repository (made before
this) keeps using both until you merge them. Get them in step first: `yaks
sync` in each until both are committed and at the same commit (`git -C
<worktree> log -1`). Then remove both worktrees (the `yaks` branch and its
commits stay in each git dir) and run any yaks command in your checkout, which
makes the one shared worktree at the new farm home; Delta's clones find it from
then on:

```sh
git -C <other git dir> worktree remove <its farm worktree>
git -C <checkout> worktree remove <its farm worktree>
yaks -C <checkout> path   # prints "created the farm worktree ..."
```

Restart a running `yaks tui` afterwards.

## Filter flags (shared)

`--status <s>` · `--type <t>` · `--priority <n>` · `--tag <l>` · `--herd <h>`
(id prefix) · `--search <text>` · `--source <url>` (the yak whose own `source:` is exactly that URL) · `--parent-of <id>` (descendants at any depth) · `--ready` ·
`--tangled` · `--needs` (any `needs` value, `human` or `agent`) · `--run-on` (the description or a note has
run-on lines: adjacent lines that markdown joins into one paragraph, which a tidy splits into blocks; a hard
break, a list or a code fence is not run-on). They compose, and back `list`, `search`, `log`, and the
`bulk` selector.

## Read / query

| Command | What it does |
|---|---|
| `list` | List tasks (non-dead by default; `--all` includes dead). Status renders as `[H]`/`[S]`/`[N]`/`[X]` — hairy / shaving / shor**n** / dead. `--due [--before <date>]` keeps the yaks with a `due:`, soonest first; a hairy or shaving one past it reads `overdue`. [Due dates](#due-dates-and-recurring-yaks) |
| `show <id> [--json]` | Show one yak: fields, references, children, body + notes. [Details](#show) |
| `path [<id>...]` / `path <filters>` | Print each yak's current file path (absolute, one per line), by id or by the shared filter flags (`--all` adds dead yaks). With neither, prints the farm root this command resolved. [Details](#path) |
| `next` (alias `ready`) | Hairy yaks whose dependencies are all resolved — the work queue. Leaves out `needs: human`; includes answered yaks (`needs: agent`), marked `✓ answered, awaiting pickup`. [Details](#next) |
| `tangled` (alias `blocked`) | Hairy yaks with at least one unresolved dependency. |
| `search <text>` | Substring search over id / title / description. |
| `stats` | Counts by status, type, priority. |
| `sheds [--all]` | Who is working on what: one row per actor (the stamp on notes and moves), read from the farm alone: what it is `shaving`, `asking`, and wrote `recent`ly, `active`/`stale`/`idle`, and how fresh the farm is (`--json` for scripts). Read-only. [Details](#sheds) |
| `log` | Timestamped notes and status moves (`moved: <from> -> <to>` entries) across a filtered set, oldest first (`--since <2h\|3d\|date>`, `--by <actor>`). |
| `refs <id>` | The yaks a task points at (parent, deps, id mentions), flagging danglers. |
| `refs <id> --incoming [--json]` | The reverse: every yak that points AT this one, one line per reference (`child`, `depends` or `mention`), then the doks and session notes that link it. Computed by scanning the farm on demand. `--json`. The TUI's `B` key opens the same list. [Details](#refs) |
| `commits <id> [--fetch] [--json]` | Git commits linked to a yak — shas recorded in its notes (`yaks land` writes them), the branches they record (`--fetch` fetches first), those naming its id in the code repository (all branches), then the yak file's own history on the farm. [Details](#commits) |
| `rollup` | Group yaks by the external issue they roll up to (`--keys` lists the external keys). |
| `scan-ids [file]` | Scan text/stdin for real yak-ids; prints `line:col  id`, exits non-zero if any found — a leak check for a private farm's commits (pre-commit / PR hook). |

### `show`

`show <id> [--json]`

Show one yak: fields, references, children, body + notes. An unknown id exits 1.

The text header lists `prs:` (comma-separated) after `source:`. `--json` is the task object (with `prs: [...]` when the yak has any, like `source`) plus, when the yak has notes, `notes: [{ts, actor, meta, text}]` in file order (`meta` is an object of the note's [metadata](#note-metadata), `{}` for a plain note; `actor` is null for a bare note). `description` still holds the whole body, notes included. `yaks log --json` entries carry the same `meta`.

### `path`

`path [<id>...]` / `path <filters>`

Print each yak's current file path (absolute, one per line), by id or by the shared filter flags (`--all` adds dead yaks). Each yak's file is `.yaks/<herd>/yaks/<id>.md`, the same path for its whole life (a status move edits it in place); its attachments are in `.yaks/<herd>/yaks/<id>/`.

An unknown id goes to stderr with a non-zero exit (known ids still print). Ids or filters, not both; with neither, prints the farm root this command resolved (the farm branch's worktree, when the farm is a branch): which farm am I on?

### `next`

`next` (alias `ready`)

Hairy yaks whose dependencies are all resolved — the work queue. A yak awaiting a human (`needs: human`) is left out; one that has been answered (`needs: agent`) is included, marked `✓ answered, awaiting pickup` (`yaks pickup <id>` when you take it on).

### `sheds`

`sheds [--json] [--all]`

Who is working on what, read from the farm alone. A **shed** is a worker in the farm: one row per **actor**, the `[actor]` stamp on notes and moves (`▸ <ts> [actor]`; set it with `YAKS_ACTOR` or `--as`). Nothing here looks at git checkouts, Delta or the machine, so a synced farm reads the same on every machine. Notes with no actor belong to no row.

Per actor, a header `<actor>  <liveness> · <last write>`, then, when the actor has one, `branch <b> @ <short sha> (<ago>)` (yaks-4728): where its code is, from the actor's latest `checkpoint` note naming a branch, on ANY yak (so one worker branch spanning several yaks shows once). It is read from the note alone, never fetched; `yaks commits <id> --fetch` is how to look at the branch. Then its yaks:

- `shaving  <id>  <title>`: **working on**. The yak is `shaving` now and this actor holds the claim: the claimant of the latest move into `shaving` is the actor named by that move's `for:` metadata when it has one, else the actor of the most recent note since that move (move included). So a coordinator that moves a yak into `shaving` for a worker, and the worker's notes after it, put the yak under the worker. A yak is under exactly one actor's `shaving`. `asking you` ends the line when it also waits on a human.
- `asking   <id>  <status>  <title>  <ago>`: the actor's other yaks with `needs: human`.
- `recent   <id>  <status>  <title>  <ago>`: every other yak it wrote, newest first, with its status now (five, then `(+N more; --json lists them all)`).
- Liveness: `active` (last write within 2 hours), `stale` (it holds a `shaving` claim but has not written for longer), else `idle`.

Order: an actor with an open ask first, then active, stale, idle, then the most recent writer. Idle actors with no claim and no ask are only counted (`(3 idle actors; --all lists them)`); `--all` lists them. A first line gives the farm's freshness, `farm: up to date with origin/yaks` or `farm: 2 ahead, 0 behind origin/yaks (as of the last fetch)`: the farm branch against its upstream from the refs as of the last fetch (never fetches; the same state `yaks status` reports), then `· origin/yaks changed 37m ago` (the upstream tip's commit time: the last time ANY machine published farm changes) and `· fetched just now` (this farm worktree's last fetch). Together they tell "nobody wrote" from "nobody synced": workers on a cloud runner that look quiet while `origin/yaks changed` is old may just be waiting for their coordinator's next `yaks sync`. JSON adds `upstream_changed` and `fetched` to `farm`. The line is left out for a farm with no upstream.

`--json`: `{farm, sheds}`. `farm` is `{branch, upstream, ahead, behind}` or `null`. `sheds` lists EVERY actor in the same order, idle ones too: `{name, liveness, last_write, checkpoint: {branch, commit, at} or null, working: [ids], asking: [ids], touched: [{id, title, status, needs, last, working}]}` (`touched` is every yak it wrote, newest first). Read-only.

### `refs`

`refs <id> --incoming [--json]`

The reverse: every yak that points AT this one, one line per reference saying how: `child` (its `parent:`), `depends` (its `depends_on:`), or `mention` with where (`title`, `body:L<n>` for a description line, `note:<ts> [actor]` for a note). Then the doks and session notes that link it, one line per link: `dok` or `session`, the path and `:L<line>`, and `<id>#<ts>` when the link names one of the yak's notes. Only markdown links count there (`[x](<id>)`, `[x](<id>#<ts>)`), not a bare id in a dok's prose.

Mentions are whole real ids, matched exactly as `refs` and the TUI links do. Computed by scanning the farm on demand (no index).

`--json` emits `{id, title, incoming: [{id, title, status, kind, in?, line?, ts?, actor?}], docs: [{path, kind, line, note}]}` (`docs` are the doks and session notes, `kind` `dok` or `session`, `note` the linked note's timestamp or null). The TUI's `B` key opens the same list as a picker.

## Create / edit

| Command | What it does |
|---|---|
| `create '<title>'` | New hairy yak. Title is positional or `--title`; `--type`/`--priority`/`--parent`/`--herd`/`--tags`/`--depends-on`/`--source`/`--key`/`--pr`/`--description`/`--description-file`/`--verify`; `--json` prints id + file path. `--description -` reads stdin (see [Multi-line text](#multi-line-text)). [Details](#create) |
| `update <ids…>` | Update fields/tags, replace the body (`--description`/`--description-file`) or append a `--note`/`--note-file` (with `--meta key=value`, repeatable, for the note's [metadata](#note-metadata)); `--set key=value` / `--unset key` for any other frontmatter field; the same edit applies to every id. `--note -` reads stdin (see [Multi-line text](#multi-line-text)). [Details](#update) |
| `dep add\|remove <id> <dep>` | Add / remove a dependency. |
| `peek <link>` | Print only what a link names: lines, a section, a dok's head. [Details](#links) |
| `conflicts [--json]` | List farm documents holding a merge conflict to settle: conflict markers in prose, or a dok's `merge_conflicts:`. [Details](#merge-driver) |
| `undo <commit>... \| --by <actor> [--since <when>]` | Revert farm commits as one new commit. [Details](#undo) |
| `mv <from> <to>` | Move a dok and rewrite every link to it. [Details](#links) |
| `find <pattern> [--scope P] [--kind K] [--regex] [--json]` | Full-text search over yaks, doks, sources' text and sessions. [Details](#find) |
| `source add <path> \| list \| mark <id>` | Save outside material as a write-once source; list them; mark one kept on purpose. [Details](#source) |
| `session note\|touched\|list\|promote` | Session notes: save one at the end of a working session, see what you wrote, list the unreviewed, mark them reviewed. [Details](#session) |
| `promote <id>/<file> [--dok <path>]` | Move a yak's attachment into a source or a dok, rewriting links. [Details](#source) |
| `dok list\|show\|new\|set\|query\|row\|types\|define` | The farm's doks: markdown documents under `doks/` or `<herd>/doks/` that are not yaks. [Details](#dok) |
| `reparent <ids…> --parent <id>` | Move yaks under a new parent (or `--unparent` to top-level). |
| `rename <old> <new>` | Rename a yak's id, updating every reference across the farm (its file, its `.yaks/<herd>/yaks/<id>/` attachment dir and the `](<old>/…)` links to it). |
| `rename-herd <old> <new>` | Rename a whole herd: migrate every id from one prefix to another (e.g. `yaksrs` → `yaks`). (`rename-prefix` still works as an alias.) |
| `merge <path> [--dry-run]` | Merge another farm's yaks (all statuses + attachments, byte for byte) into this one, preserving ids and status. Non-destructive; refuses on id collisions. [Details](#merge) |
| `merge-driver <base> <ours> <theirs> <path>` | **Internal** (hidden from `--help`): git's merge driver for farm documents. Merges the three versions of a yak or dok into `<ours>` with fields merged by rule, never conflict markers in frontmatter. Needs no farm. [Details](#merge-driver) |
| `bulk <filter> <mutation>` | Filter-driven field edit. **Dry-run by default**: changes nothing without `--commit`. Requires ≥1 filter flag and ≥1 mutation flag. Field edits, reparent, and `--request <kind>` only. [Details](#bulk) |

### `create`

`create '<title>'`

New hairy yak. Title is positional or `--title`; `--type`/`--priority`/`--parent`/`--herd`/`--tags`/`--depends-on`/`--source`/`--pr`/`--description`/`--description-file`/`--verify`; `--json` prints id + file path. `--description -` reads the body from stdin (see [Multi-line text](#multi-line-text)).

`--key <url>` is find-or-create by external key, for a script that mirrors an outside queue (pull requests, issues) into yaks and must run twice safely. If any yak, in **any state, dead included**, already has that `source:`, nothing is created: it prints `Exists <id>: <title>` and exits 0, leaving the yak as it is (a dead one stays dead and a shorn one stays shorn, so the caller decides whether to revive or regrow). Otherwise it creates the yak with `--key` as its `source:` and prints `Created`. The check and the create happen under one farm lock, so concurrent callers make one yak. With `--json` the output gains `created: true|false`. Not with `--source` (the key is the source), and an empty key is refused. `list --source <url>` is the read side.

`--tags` (like `--add-tag`/`--remove-tag`/`--tag` everywhere) splits on commas and whitespace — a tag may contain neither, so `--tags ui,docs`, `--tags 'ui, docs'`, and `--tags ui docs` all give `[ui, docs]` (duplicates dropped).

`--pr <url>` (repeatable) lists a pull request in the yak's `prs:` (see [`update`](#update)).

`--due <date>` and `--every <duration>` set the yak's `due:` and `every:` fields ([Due dates and recurring yaks](#due-dates-and-recurring-yaks)).

`--herd` sets the new yak's herd (id prefix; default: `git config yaks.herd`, else the farm config's `herd:`), so one farm can hold several herds.

With neither `--herd` nor a default herd, `create` fails (non-zero) rather than silently minting a `yak-` herd.

### `update`

`update <ids…>`

Update fields/tags, replace the body (`--description`/`--description-file`) or append a `--note`/`--note-file`; the same edit (and the same text) applies to every id.

`--note -` / `--description -` read stdin (see [Multi-line text](#multi-line-text)).

`--meta key=value` (repeatable) puts `key: value` lines under the note's marker (see [Note metadata](#note-metadata)). It needs a note (`--note`, `--note-file` or `--note -`); a missing `=`, a key that is not `[A-Za-z_][A-Za-z0-9_-]*`, or no note is an error and nothing is written. The value is everything after the first `=`.

`--as <actor>` attributes the note. `--verify '<cmd>'` sets (or, empty, clears) the yak's verification command; `--source <url>` likewise sets (or, empty, clears) its external `source:`.

`--pr <url>` adds a pull-request URL to the yak's `prs:` frontmatter list and `--remove-pr <url>` removes one; both repeat, and adding a URL already listed (or removing one that is not) changes nothing. `prs:` is the yak's current PRs, a list (one yak often has several), where `source:` is a single external issue; it is written as a block list after `source:` and omitted when empty. The merge driver merges it like `tags`.

`--due <date>` / `--every <duration>` set (an empty string clears) the yak's `due:` and `every:`; an unreadable value is refused before anything is written (`--set due=...` writes whatever it is given, and `doctor` flags it against a `date` schema) ([Due dates and recurring yaks](#due-dates-and-recurring-yaks)).

`--set key=value` (repeatable) sets any other frontmatter field: `--set due=2026-11-05`, `--set 'commit=[3f2a9c1, e99754d]'` (a list), `--set 'owner={team: web}'` (a map). `--unset key`, or an empty value, removes it. A field already in the file is rewritten in place; a new one goes after the built-in fields. Built-in fields (`title`, `tags`, `status`, ...) are refused: they have their own flags. Every other field round-trips exactly as written, hand edits and comments included, and `show --json` lists them parsed under `extra`.

### `merge`

`merge <path> [--dry-run]`

Merge another farm's yaks (all statuses + attachments, copied byte for byte; a source on the old layout is refused, see `init`) into this one, preserving ids and status, and declaring any incoming herd in this farm's `herds:` config so it shows up in the pickers.

Non-destructive (source left intact); refuses on id collisions — reconcile the source's herd with `rename-herd` first.

### `merge-driver`

`merge-driver <base> <ours> <theirs> <path>`

**Internal** (hidden from `--help`): git's merge driver for farm documents, run as `yaks merge-driver %O %A %B %P` (registered by `.gitattributes` `.yaks/**/*.md merge=yaks` + `git config merge.yaks.driver`). The rule: **fields merge by rule and never get conflict markers; prose may keep them** for a person or agent to resolve.

- **A yak** (a file under `<herd>/yaks/`): note entries unioned (deduped, ordered by timestamp), `updated` the later, `tags`/`depends_on`/`prs` and any other list merged (base + additions − removals), every other field taken from the side that changed it. A field both sides changed differently keeps `<ours>` and is recorded in one `merge-driver` note (`merge conflict: status was X here, Y on the other side; kept X`).
- **A dok or any other markdown file:** frontmatter merged by the same field rule; a field both sides set differently keeps `<ours>` and adds `{field, theirs}` to a `merge_conflicts:` list in the frontmatter (a dok has no notes log). Remove the entry once settled.
- **Prose** (a yak's description, a dok body, a file without frontmatter) merges line by line with `git merge-file`. Edits to different lines both land; where both sides changed the same lines, both versions stay in the text between `<<<<<<< here`, `=======` and `>>>>>>> other side`, and the merge still succeeds (a yak also gets a `merge-driver` note saying so). During `yaks sync`, which rebases, git's "ours" is the remote's version: `here` and a kept field are then what was already pushed, and `other side` is your own commit being replayed.

`yaks conflicts` lists every farm markdown file holding conflict markers (outside code fences) or a `merge_conflicts:` list, with the marker lines, and exits 1 when there is any; `--json` gives `[{path, marker_lines, fields}]`. `doctor` reports the same as `merge-conflict`. Edit the file to settle it.

Exits 0 on a merge (conflicts included), 1 and leaves `<ours>` untouched when it cannot merge (a file in a `yaks/` directory that is not a yak, binary content; git then reports a conflict for the path), 2 on an I/O error. Needs no farm.

### `bulk`

`bulk <filter> <mutation>`

Filter-driven field edit. **Dry-run by default** — prints the matched set + the mutation and changes nothing without `--commit`.

Requires ≥1 filter flag (never the whole farm) and ≥1 mutation flag (`--add-tag`/`--remove-tag`/`--set-priority`/`--set-type`/`--reparent`/`--unparent`/`--request`). Field edits, reparent, and requests only — no state transitions.

`--request tidy|summarize [--note '<extra>']` queues that canned request on every matched yak, exactly as [`request --kind`](#request) does (one `request` per yak). It combines with the field edits or stands alone. A yak awaiting a human is skipped and listed as skipped (answer it instead); the dry run marks it `(awaiting a human: request skipped)`. Example: `yaks bulk --tag docs --request tidy --commit`.

### `dok`

`dok list [<prefix>]` · `dok show <path>` · `dok new <path>` · `dok set <path>` · `dok query` · `dok row set|delete` · `dok types` · `dok define <kind>`

Doks are the farm's other documents: research, decision records, design notes, anything that should outlive one yak and stay beside the work. A dok lives under `doks/` (farm-wide) or `<herd>/doks/` (one herd's), at any depth, and is named by its path from the farm root: `doks/topics/vector-search.md`. A leading `/` is accepted (`/doks/x.md`, the form links use). Product docs that ship with the code stay in the code repo's `docs/`.

A dok is markdown with YAML frontmatter in OKF's shape: `type:` (`note`, the default; `topic`, a living dok about one subject; `howto`; `summary`, a digest of one source; or a typed kind), `title:`, and any other field (`description`, `tags`, `sources`, `status`, `stale_after`, ...). Every field round-trips exactly as written. Doks have no notes log and no `updated:` field: git says when one changed.

- `dok list [<prefix>]` prints path, type and title, sorted by path; `<prefix>` narrows it (`doks/topics`, `yaks/doks`).
- `dok show <path>` prints the file; `--json` gives `path`, `hash`, `type`, `title`, `herd` (null for the farm-wide doks), `fields` parsed, and `body`. The file's **hash** (the first 12 hex digits of the sha256 of its bytes) goes to stderr as `hash: <h>` in text mode, so stdout stays the file.
- `dok new <path> [--type T] [--title T] [--set key=value ...] [--body TEXT | --body-file PATH]` creates a dok. The path must be `doks/<parts>.md` or `<herd>/doks/<parts>.md`, every part lowercase-kebab (`a-z`, `0-9`, `-`); an existing file is refused. The title defaults to the file name (`heat-pump.md` → `Heat pump`). It is stamped `created:` with the time it was made (UTC, as a yak's), unless `--set created=...` gives one; nothing stamps an `updated:`, which hand edits would leave stale.
- `dok set <path> [--set key=value ...] [--unset key ...] [--body ... | --body-file ... | --old TEXT --new TEXT] [--if-hash H]` edits fields (rewriting only those keys), replaces the body, or replaces one exact piece of it. An empty value removes a field. It prints `Updated <path>` (or `No changes to <path>`) and `hash: <h>`, the file's new hash.
  - `--old`/`--new` (or `--old-file`/`--new-file`; `-` reads stdin for one of them, trailing newlines are dropped, `--new ''` deletes) replace `--old` in the body, which must match exactly once: no match, or several, is refused saying how many, and nothing is written. An agent edits one passage without rewriting the body.
  - `--if-hash H` refuses (exit 1, nothing written) unless the file's hash is still `H`, naming the current hash: an edit built from a stale `dok show` never overwrites a newer body. Chain edits by the hash `dok set` prints. It combines with any other flag.
- `dok query [--type T] [--where key=value ...] [--prefix P]` lists doks whose fields match, all tests ANDed; a list field (`tags`) matches when any item does. A plain value matches exactly; an operator in front of it changes the test: `key=~text` contains (ignoring case), `key=<x` `<=x` `>x` `>=x` compare (numbers numerically, otherwise as text, so `YYYY-MM-DD` dates order correctly: `--where "renews=<2026-12-01"`), `key=!x` is not equal (a missing field, or a list without `x`, matches), and `key=` is missing or empty. The same tests filter `--rows` on a column.
- `dok row set <dok>#<key> [--set column=value ...]` creates or updates the row of a **table dok** whose first column is `<key>`; `dok row delete <dok>#<key>` removes it. Only the table's lines change: the rest of the dok is kept byte for byte, untouched rows keep their text, a `|` in a value is written `\|` (and read back), and a column the table lacks is appended to the header, the separator and every row (empty cells). A new row goes at the end, or in place when the rows are already sorted by key (ignoring case). Refused, with what to do, when the dok has no table (the first line starting with `|`, followed by a `| --- |` line) or two rows share the key; `--set` cannot change the key column (delete the row and set a new one); a value is one line.

`--set` values parse like `update --set`: `[a, b]` is a list, `{a: b}` a map. `--body -` reads stdin. Editing a dok by hand is fine; `yaks doctor` checks doks (below). `list` and `query` take `--json`.

**Typed doks.** A kind other than the untyped four can have a schema, `schemas/<type>.md`, saying which fields its doks have, in Hearth's format:

```markdown
---
type: Dok Type
title: Appliance
fields:
  name: {type: text, required: true}
  purchased: date
  warranty: {type: text, default: none, description: Until when}
---
```

A field is `{type, required, default, description}` or the shorthand `name: <type>`; types are `text`, `number`, `bool`, `date`, `datetime`, `money` (`649.00 USD`), `link`, `list` (anything else reads as text, with a warning). Schemas only advise: `dok new` fills in defaults, `dok new`/`dok set` print warnings (a usually-set field missing, a value of the wrong type, a field the schema does not list), and `doctor` reports them as `schema-mismatch`. No write is ever refused. `dok types` lists the untyped kinds and every schema with its fields. A small list whose rows aren't doks of their own is a **table dok**: a dok of that type whose body holds a markdown table; `dok query --rows [--type T] [--where column=value]` lists its rows, and `dok row set|delete <dok>#<key>` writes one.

`dok define <kind> --field name=type[!] ... [--title T] [--description D] [--stale-after P1Y] [--replace]` writes the schema, so you need not write it by hand:

```sh
yaks dok define appliance --field name=text! --field purchased=date --stale-after P1Y
```

- `--field name=type` is repeatable; a trailing `!` (`name=text!`) marks the field required. The types are those above; any other type, a field name that is not letters, digits, `_` and `-`, or a field given twice is refused. A new file gets `type: Dok Type`, a `title` (`--title`, else the kind's name capitalised), and the fields in the shape `dok types` reads (`name: date`, or `{type: text, required: true}`).
- **It merges into an existing schema.** Fields the file has are kept as written, and so is every other key (`tags:`, comments) and its prose; new fields are appended. `--title`, `--description` and `--stale-after` (a duration such as `P6M`, or a date) are written only when given. A field that already exists with a different type or required flag is refused, with nothing written, unless you pass `--replace`; its `default` and `description` then stay. A field's `default` and `description` have no flag: edit the file for those.
- The kind is lowercase letters, digits and `-`. The untyped kinds (`note`, `topic`, `howto`, `summary`) are refused. Saying nothing to define (no field, title, description or `--stale-after`) is refused. It prints `Created`, `Updated` (what was added, changed or set) or `No changes to` the file. One commit.

`schemas/yak.md` extends yaks the same way: its `fields:` describe extra yak fields (`due: date`), checked by `doctor` (the built-in fields are never reported), and its `tags:` map lists the farm's known tags with a description each (`ui: The TUI`), shown by `dok types`.

### `find`

`find <pattern> [--regex] [--scope <prefix>]... [--kind yaks|doks|sources|sessions]... [--case-sensitive] [--json]`

One search over the farm's prose. It reads yak files (title, description and notes: the whole file), doks (frontmatter and body), each source's `source.md` and `extracted.md` (never the original file) and `sessions/**.md`. Binary files and files that are not UTF-8 are skipped. It scans the files each time, with no index: on this farm's ~700 yaks (about 10 MB) a search takes about 0.2 s, and on a 160-file Hearth import (1.5 MB of sources) about the same.

- The match ignores case (`--case-sensitive` turns that off; `-i` is accepted and changes nothing). The pattern is a literal substring; `--regex` makes it a regular expression: `. [a-z] [^x] \d \w \s \b ^ $ ( ) (?: ) | * + ? {n,m}`. A pattern with a lookaround, backreference or inline flag is refused. Matching is linear in the line, so a pattern cannot hang.
- `--scope` keeps paths under a prefix of the farm, repeatable: `doks`, `sources`, a herd name (its yaks and its doks), or a longer prefix such as `doks/topics/vec`. A name without a `/` means that directory (`web` does not match `webby/`).
- `--kind` keeps one kind of file, repeatable.
- Output is a header per file, `path  title` (the yak's, dok's or source's title), then `path:line: text` per matching line (`line` counts the file's own lines, frontmatter included), text trimmed to about 160 characters around the match. Files come yaks, doks, sources, sessions, each sorted by path.
- `--json` gives `[{path, kind, title, line, text}]`, `kind` being a `--kind` value; `[]` when nothing matched.
- Exit 1 when nothing matches, like `grep`; 1 with `error: invalid pattern: ...` when a regex does not parse.

`yaks search` is unchanged: the yak-only lookup over id, title and description, with the shared filters.

### `source`

`source add <path> [--title T] [--url URL] [--text FILE] [--part-of sources/<id>/source.md] [--refresh] [--text-only]` · `source list [--json]` · `source mark <id> (--uncited WHY | --superseded-by <id> | --clear)` · `promote <id>/<file> [--dok <path>]`

A **source** is a snapshot of material from outside the farm, written once and cited from doks (`sources:`) and yaks (a link). It lives in `sources/<YYYY-MM-DD>-<first 8 hex of its sha256>/`: the original file (a directory's files under `files/`, its layout kept), `extracted.md` with its text: a copy of a markdown, plain-text or code original (a directory's text files joined under `## <path>` headings), or your own conversion passed with `--text` (HTML and PDF are not text: yaks never treats markup as extracted), and `source.md`: frontmatter with `type: source`, `title`, `description`, `file` (or `files:`), `media_type`, `size`, `sha256`, `previous_sha256` (after a refresh), `part_of`, a `sources:` entry saying where it came from (`resource`, `host`, `last_modified`, `retrieved`; a file saved from a temp directory records its name, not the scratch path) and `generated: {by, at}`. That is Hearth's format, so a Hearth repo's `raw/sources/` moves over as is.

- `source add <path>` saves a file (up to 20 MiB) or a small directory (up to 200 files and 5 MiB; `.git`, `node_modules`, `target` and dot files are left out). Saving the same content again, on any day, reports `Already saved:` with the existing source. A path already inside the farm is refused: link it instead.
- yaks fetches nothing and converts nothing. To save a web page, download it (or have your agent do so), convert it to markdown, and save it **once**: `source add page.html --url <page> --title "<what it is>" --text page.md`. The URL becomes the resource, the original sits beside `extracted.md`, which is your conversion. `--text` takes a file and is refused for a directory or a file that is not UTF-8. `--part-of` records which source a part (a screenshot of that page) belongs to, never a conversion of the same thing.
- The description is built from the URL or the file's name, never from a path on your machine.
- **One file for text.** A markdown or plain-text original is its own text, so by default it is saved twice: as itself and as `extracted.md` (git stores one blob, but the checkout shows two files). `--text-only` saves just `extracted.md`, and `source.md` says `file: extracted.md`; use it when the file you hold is already the text to read (a document exported as markdown) and its file name means nothing. The original's name is not kept. It is refused for a directory (which keeps its layout under `files/` so relative links work), for a file that is not markdown or text or not UTF-8, and together with `--text`; it works with `--refresh`. Sources saved without it keep both files.
- **Living sources.** A source is a snapshot, so by default a changed page is a second source. For material that changes under a stable URL (a shared document, a wiki page), `source add <file> --url <URL> --refresh` updates in place: the source whose resource is `<URL>` keeps its directory and id, takes the new original, `extracted.md`, `sha256` and `size`, and records the hash it replaced as `previous_sha256`; its title and `part_of` are kept unless you give new ones. Unchanged content reports `Already saved:` and writes nothing; a new URL is saved as a new source (so one command creates and refreshes), and `--refresh` without `--url`, or with a URL that more than one source names, is refused. Citations of the source stay valid, and the farm's git history keeps the earlier versions. The id's `-<sha8>` is the hash it was first saved with, so saving the same content again finds a source by the `sha256:` in its `source.md`, not by its id. yaks still fetches and converts nothing: a skill (or you) fetches the document and hands yaks the file, and nothing refreshes by itself.
- `source list` prints id, media type, title and resource, oldest first.
- `source mark <id> (--uncited "<why>" | --superseded-by <id> | --clear)` keeps a source off `yaks check`'s list of uncited sources, by writing the field it honours in `source.md` (`uncited: <why>` or `superseded_by: sources/<id>/source.md`) as one commit. `<id>` is the source's id, `sources/<id>`, or its `source.md` path; `--superseded-by` takes the same forms for the newer source. One mark at a time: setting one replaces the other, and `--clear` removes either. A missing source, a `--superseded-by` that does not exist or names the source itself, and an empty reason are refused, with nothing written. Marking one source of a `--part-of` family keeps the whole family off the list, as a citation would. It is only a mark: the source stays, and so does its history.
- `promote <id>/<file>` turns a yak's attachment into a source (resource `<id>/<file>`); `--dok doks/<path>.md` turns a markdown attachment into a dok instead (given `type: note` and a title when it has no frontmatter). The attachment is removed and every link to it, in any yak, dok or markdown attachment, now points at the new place; a markdown attachment promoted to a dok has its bare sibling links (`shot.png`) rewritten to `<id>/<file>` so they keep naming the yak's attachments.

### `session`

`session note --title T (--body - | --body-file F) [--since WHEN] [--harness H]` · `session touched [--since WHEN] [--by ACTOR] [--json]` · `session list [--unreviewed] [--json]` · `session promote <path>...`

A **session note** is a short account of one working session, kept so a later review can move what it learned into doks and a weekly reflection can learn from how sessions go. The `yaks-upkeep` skill says when to write one and what goes in it; this is the mechanics.

- `session note` saves `sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md` as one commit and prints its path. The body is markdown with `## Summary`, `## Turns` (one `- <verdict>: <reason>` per request; verdict `good`, `partial`, `failed` or `unclear`), `## Lessons` (`- none` when none), and optionally `## Discussion` and `## Saved`. A turn without a verdict, a missing section or a body over 12,000 characters is refused, and nothing is written.
- yaks writes the frontmatter: `type: session-review`, `title`, `source: session-note`, `actor`, `person` (git `user.name`), `host`, `harness` (`--harness`, else `delta` or `claude-code` when the environment says so), `thread` (the Delta thread id, the way back to the transcript), `day`, `reviewed` (when it was saved) and a count of each verdict. It is the shape of Hearth's `session-review` files, so notes copied in from a Hearth repo list alongside.
- Without a `## Saved` section, yaks writes one before `## Lessons` from `session touched`.
- `session touched` lists what an actor (you, else `--by`) wrote in the farm since `--since` (`2h`, a date or an RFC 3339 time; default: its latest session note, at most 12 hours back): each yak it created, moved or noted, with a [link to each note](#links), then the doks, sources and schemas its commits created or changed. A hand edit committed with a plain `yaks commit` carries no actor, so it is not listed; `yaks commit --as <actor>` stamps it.
- `session list` prints path, day, actor, verdict counts and title, oldest first; `--unreviewed` keeps the agents' own notes (`source: session-note`) without `promoted:`.
- `session promote <path>...` stamps `promoted: <today>` on each, in one commit, once the review has kept what they learned; one already stamped is reported and left alone.
- `doctor` checks the links in session notes, and `find --kind sessions` searches them.

### Links

Yaks and doks link with ordinary markdown links, in one grammar: `<target>[@<rev>][#<anchor>]`.

| Target | Form | Example |
|---|---|---|
| a yak | its full id, bare in prose | `yaks-1a2b` |
| one note of a yak | the id, `#` and the note's timestamp | `[the answer](yaks-1a2b#2026-11-04T22:13:03Z)` |
| a farm document | path from the farm root, leading `/` | `/doks/topics/x.md`, `/sources/<id>/source.md` |
| below the linking dok | relative, downward only | `notes/detail.md` (from `doks/topics/x.md`) |
| a yak's attachment | `<id>/<file>`, from anywhere; inside an attachment, a bare sibling `<file>` | `yaks-1a2b/log.txt`, `shot.png` |
| the code repository | `code:` and a path from the repo root | `code:src/foo.rs#L12-L40` |
| a URL | as is | `https://...` |

- `@<rev>` (7 to 40 hex digits) pins a path to a commit: a farm path to a farm-branch commit, a `code:` path to a code commit (`code:src/foo.rs@3f2a9c1#L12`, `/doks/x.md@91be0d2#setup`).
- `#L12` or `#L12-L40` names lines; any other `#anchor` a heading, by GitHub's slug rule (lowercase, spaces to `-`, punctuation dropped; a repeated heading gets `-1`, `-2`).
- **A note** is named by its timestamp, which it keeps for life (removing another note never shifts it). Notes written in the same second (a shear note and its move) share one: the first is the plain timestamp, later ones take `-1`, `-2`. `yaks update --note` prints the new note's link (`Updated yaks-1a2b: note yaks-1a2b#2026-11-04T22:13:03Z`), so copy it rather than building one; `peek` prints just that note and `doctor` reports a note link that leads nowhere. The TUI follows one to its yak, cursor on that note.
- **In a file attached to a yak**, a bare file name is another attachment of the same yak (`![crop](shot.png)` in `yaks-1a2b/evidence.md` is `yaks-1a2b/shot.png`), because that is where any other markdown viewer looks for it too; an attachment directory is flat, so only a bare name counts, and `<id>/<file>`, `/farm/path`, `code:` and URLs mean what they do anywhere. `doctor` checks the links inside markdown attachments, and `rename-attachment` and `promote` keep them right.
- A relative link never climbs with `../`: link from the farm root instead. In yaks the older code forms stay valid: `../src/foo.rs` and the pinned `../blob/<sha>/src/foo.rs`.
- `code:` links resolve in the code checkout: the git top-level of the directory yaks runs in, else the repository's main checkout.

`peek <link> [--context N]` prints only what a link names: the lines (numbered; `--context` widens them), the section under a heading (up to the next heading of the same or a higher level), or with no anchor a dok's frontmatter and first paragraph (the first 20 lines of anything else). A pinned link reads the file as it was at that commit. Exits non-zero, saying why, for a missing file, heading or line.

`refs <dok path> [--incoming] [--json]` lists the links a dok holds (marking any that lead nowhere), or with `--incoming` every yak and dok that links to it, with the line.

`mv <from> <to> [--dry-run]` moves a dok and rewrites every link to it, in yaks and doks, in the farm-root form, keeping each link's `@rev` and `#anchor`; the moved dok's own relative links are rewritten to keep their targets. The destination follows `dok new`'s naming rule.

`doctor` reports `broken-link`: a farm path or attachment that does not exist, or a `../` link outside the legacy code forms. Placeholder links in prose (`[x](path)`, `[x](<id>/<file>)`) are not checked.

## Verification

| Command | What it does |
|---|---|
| `verify <ids…>` | Run each yak's verification command with live output, and record `verify: <cmd> -> PASS/FAIL (exit N)` as an attributed note. Exits non-zero if any fails. Explicit only. [Details](#verify) |
| `attach <id> <path>...` | Copy local files under `.yaks/<herd>/yaks/<id>/` and link each in the yak's body — artifact evidence a human can view. `--no-link` stores them without a body link (the parts of a document that links them itself); `--caption` captions the image it links; `--name` renames a single file; `--note`/`--as` record an attributed note. [Details](#attach) |
| `rename-attachment <id> <old> <new>` | Rename `.yaks/<herd>/yaks/<id>/<old>` and rewrite every link to it across the farm. Refuses to overwrite an existing attachment. [Details](#rename-attachment) |
| `attachment <id> [<file>]` | Print the absolute path of a yak's attachments (all, or the one named). `--json`. Read-only. [Details](#attachment) |

### `verify`

`verify <ids…>`

Run each yak's verification command with live output, and record `verify: <cmd> -> PASS/FAIL (exit N)` as an attributed note. Exits non-zero if any fails. Explicit only — never auto-run.

The command is the yak's own `verify:` field if set, else the config default resolved by the yak's tags (see below).

The scriptable form of a yak's evidence contract: a check anyone (or CI) can re-run.

Default verify commands live in `.yaks/config.yaml` under a nested `verify:` map (tag → command, plus an optional `default`); a yak with no explicit `verify:` field resolves its command from the first of its tags with an entry, else `default`. Name the project's levers once:

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

A lever is the pass/fail **gate** for its tag — what `doctor --strict` enforces at shear. For UI yaks that's the tui snapshot tests (`cargo test -p yaks`); the SVG docshots are a separate *visual* channel, generated on demand (`cargo test -p yaks docshots -- --ignored`) and attached for human review, not a gate.

### `attach`

`attach <id> <path>...`

Copy local files (several attach in one write, one farm commit) under `.yaks/<herd>/yaks/<id>/` (beside the yak's file) and link each in the yak's body as `![name](<id>/<file>)` for an image (png, jpg, jpeg, gif, webp, svg) or `[name](<id>/<file>)` for any other file (what GitHub and every CommonMark viewer expect; a space or parenthesis in the file name is percent-encoded, `my%20shot.png`) — artifact evidence (screenshots, TUI frames) a human or yakherd can view.

`--caption <text>` writes the image's markdown title, `![name](<id>/<file> "text")`, which `yaks tui` shows as a caption under the picture (one file, with its link: not with `--no-link`).

`--name <name>` stores a single file under a different filename (sanitized; the source's extension is kept if omitted). `--note`/`--as` record an attributed note.

`--no-link` stores the files and leaves the yak's body alone. Use it for a **document and the images it embeds**: attach them all together with `--no-link`, then link only the document from the yak (`[evidence](<id>/evidence.md)`). Its `![crop](crop.png)` links resolve to the sibling attachments (see [Links](#links)), so the yak shows one link, not one per image. A file the yak itself shows (a screenshot, a log) is attached plainly.

The non-scriptable, look-at-it form of evidence: an **external file** (committed with the farm by `yaks commit`), never inlined.

### `rename-attachment`

`rename-attachment <id> <old> <new>`

Rename `.yaks/<herd>/yaks/<id>/<old>` (e.g. an ugly `paste-….png`) and rewrite every link to it across the farm (the `![name]` / `[name]` text follows the new stem, and the link form follows the new extension). `<new>` is sanitized and keeps `<old>`'s extension if it omits one; refuses to overwrite an existing attachment. A markdown attachment of any yak that links the file (by bare name or `<id>/<file>`) is rewritten too, keeping the form it used.

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
| `snooze` | move an open yak's `due:` (`--to +2d`, `+1w`, a date; default `+1d`), with a note ([below](#due-dates-and-recurring-yaks)) |
| `tick` | the scheduling round you kick off by hand: `regrow-due`, then one report of what needs you ([below](#due-dates-and-recurring-yaks)) |
| `regrow-due` | every shorn yak with `every:` whose next occurrence has arrived → hairy ([below](#due-dates-and-recurring-yaks)) |

### Due dates and recurring yaks

Three extra yak fields, no new file format (they are written like any `--set` field and listed under `extra` in `show --json`):

- `due:` — a date (`2026-11-15`) or an RFC 3339 datetime: when the yak should be done. `--due <when>` sets it; besides a date or datetime it takes `+Nd` or `+Nw` (N days or weeks from today, stored as that date; no months, no `-`, no weekday words). A **hairy or shaving** yak past it is **overdue**: `list` and `next` print ` due 2026-11-15 overdue`, `show` prints `due ... (overdue)`, `--json` adds `"overdue": true`, and the TUI shows the date in red. A date is overdue once its day has passed; a datetime the moment it does. A shorn or dead yak is never overdue.
- `every:` — an ISO 8601 duration only: `P1D`, `P2W`, `P1M`, `P3M`, `P1Y`, or a run of them in Y M W D order (`P1Y2M`). `--every <duration>` sets it; zero (`P0D`) and words (`monday`) are refused. It makes the yak recur.
- `wake:` — `agent` or `human`: who a recurring yak goes to when it regrows. `--wake <who>` sets it (an empty string clears it). Without it the yak only becomes hairy, as before. See [`regrow-due`](#due-dates-and-recurring-yaks) below.
- `regrown:` — a datetime yaks writes when `regrow-due` reopens the yak. Not for hand editing.

`list --due [--before <date>]` keeps the yaks that have a `due:` (before the date, exclusive, when given), sorted by it, then by id.

`snooze <id>... [--to <when>] [--as <actor>]` puts off open yaks: it moves `due:` to `<when>` (a date, a datetime, or `+Nd`/`+Nw` from today; default `+1d`) and appends a `kind: snooze` note (`snoozed until 2026-11-11 (was due 2026-11-06)`), so the history shows each put-off. A shorn or dead yak is refused (nothing to put off; `regrow` it first). Every id is tried; the exit is non-zero if any was refused or missing.

`regrow-due [--dry-run] [--json] [--as <actor>]` — the clock lives outside yaks; `yaks tick` (below) calls this by hand, as does the TUI (on load and each sync tick) while it is open. For each **shorn** yak with `every:` it finds the schedule's latest occurrence that is not in the future. If there is one it moves the yak to hairy (like `regrow`), sets `due:` to that occurrence plus one period (the next deadline) and `regrown:` to now, and adds a `kind: regrow` note (`regrew for the 2026-11-15 occurrence (1 skipped)`, with `occurrence` in its metadata). Occurrences are `start + k × every`, counted from the start so a month-end keeps its day (Jan 31 + `P1M` is Feb 28, then Mar 31). The start is `due:` (k from 1) for a yak never regrown, else its shear time. A regrown yak already carries `due:` = its next occurrence, so that is the start and k counts from 0: a late run never pushes the schedule later. Occurrences missed while the yak stayed shorn don't pile up: one regrow covers them all, and the note counts the skipped. It is idempotent (the new `due:` is in the future, and a hairy yak is not a candidate), so a second run does nothing.

**Waking someone.** A yak with `wake: agent` or `wake: human` goes, when it regrows, to the inbox like any other work: `needs:` is set to that value and a note is written under the marker the same command uses (`requested: needs agent`, the one `request` writes, or `asked: needs human`, the one `ask` writes), so `yaks inbox --for agent|human`, `wait` and the TUI Inbox list it and say why, and `doctor` sees no drift. The note text says it is the recurring yak's standing instruction: **the yak's description is the task**. An agent runs `pickup` (a human, `answer --done`) to clear it, and shears the yak when done; the next round wakes it again, because `wake:` stays. A question or request from an earlier round that nobody has cleared is **kept, not stacked** (the yak keeps its `needs:` and gets no second note; the report says `already awaiting`). `wake:` is checked: anything but `agent` or `human` holds the yak, reported like an unreadable `every:`. `--json` adds `wake` and `awaiting` (`who`, `already`) to each `regrew` entry.

**`tick`: a round by hand.** `tick [--dry-run] [--within <span>] [--json] [--as <actor>]` is the one command for "do whatever is due, then show me what needs me". It runs `regrow-due` (the same code the TUI's tick calls), then prints a summary line and a section for each of: what regrew and what was held; **overdue** yaks (open, past `due:`, most overdue first); yaks **due within** `--within` (default `3d`; a number then `m`/`h`/`d`/`w`; a yak already in the inbox is listed there, not twice); **awaiting you** and **awaiting an agent** (the inbox, as `yaks inbox` prints it, so a woken recurring yak shows up here); **stale claims** (a `shaving` yak whose holder has written nothing for two hours, as `sheds` reads them); and a count of `yaks check` findings. A farm with nothing to say prints `tick: nothing needs you.` It is idempotent (a second tick regrows nothing and writes nothing), and it **never syncs, runs a command or posts a notification**: the clock is yours, and so is `yaks sync`. `--dry-run` reports what would regrow and writes nothing. `--json` is the shape for a harness: the `regrow-due --json` keys (`dry_run`, `regrew`, `held`), plus `within`, `overdue` and `due_soon` (task objects as `list --json`), `awaiting_human` and `awaiting_agent` (as `inbox --json`), `stale` (`actor`, `id`, `title`, `last`), `findings` (a count, or null when `check` could not run) and `quiet`. It exits 0 unless a write failed; a bad `--within` is refused before anything is written.

A recurring yak under a shorn parent reopens the parent first (the move note carries `child:`, like `shave`'s lifts), so the family rule holds. One under a dead parent, or with an unreadable `every:`/`due:`/`regrown:`, is **held**: reported, left shorn. Dead yaks never regrow. Each yak is one commit. `--dry-run` writes nothing; `--json` gives `dry_run`, `regrew` (`id`, `title`, `occurrence`, `skipped`, `due`, `reopened`) and `held`. Word schedules (`monday`, `1st`) are not supported.

The notes keep every round, so a recurring chore accumulates one shear note per occurrence, with its links.

`yaks doctor` never reports `due`, `every`, `wake` or `regrown` as missing from `schemas/yak.md`.

### Families stay consistent

A finished yak (shorn or dead) never has an open (hairy or shaving) child.
The moves keep it that way, in the CLI and the TUI alike, and `yaks doctor`
flags a farm that holds it anyway (`unfinished-children`):

- `shorn` refuses a yak with open children. It names them and the fix, and the
  batch exits non-zero after the other ids. Shear the children in the same call
  (a batch takes children before their parents, whatever order the ids are
  given in), or graduate them out of the family first with `yaks reparent
  <child> --parent <other>` or `--unparent`.
- `slaughter` refuses a yak with open descendants the same way.
  `slaughter <id> --family` takes them along instead: every open descendant,
  deepest first, then the yak itself. Shorn descendants are finished work: they
  never block a slaughter and stay shorn.
- `shave` takes hairy ancestors along: shaving a yak also shaves its parent if
  hairy, that one's parent if hairy, and so on (`Shaving <parent> (parent of
  <id>)`; the parent's move entry carries `child: <id>`). Starting part of the
  work starts the whole.
- Nothing opens under a finished parent: `create --parent`, `reparent
  --parent` (and `bulk --reparent`) onto a shorn or dead yak refuse an open
  yak, and so do `regrow`, `revive` and `shave` of a child of one. The message
  names the parent and the fixes: reopen it first (`yaks regrow` or `revive`;
  a batch takes parents before children), or put the yak under an open
  ancestor or at the top level.

The TUI's moves refuse the same things (a bulk move of marked yaks reports
them as skipped), and a refused shear leaves its comment unwritten.

Every move appends an entry to the yak's body in the note shape,
`▸ <ts> [<actor>]`, its `kind: move` / `from` / `to` [metadata](#note-metadata), a blank line, then `moved: <from> -> <to>` (one per yak moved, so
`slaughter --family` writes one on each), and each verb takes `--as <actor>`
like the note commands. The file alone says who moved the yak and when, and
`yaks log` lists moves with the notes. A yak moved by an older binary simply has
no entry.

See [Note metadata](#note-metadata) for the `kind: move` fields each entry also carries.

### Note metadata

A note can carry `key: value` lines directly under its `▸ <ts> [actor]` marker, closed by a `---` line, like a yak's own frontmatter:

```markdown
---
▸ 2026-10-31T15:20:00Z [tex-1]
kind: checkpoint
commit: 47076b0
---

Atlas and UVs done.
```

A note without them is still valid, and unknown keys are kept. The commands that write a conventional note stamp its `kind`, and keep the body text they always wrote (`moved: a -> b`, `asked: needs human`, …) so anything matching on text keeps working:

| Command | `kind` | Other keys |
|---|---|---|
| a move (`shave`, `shear`, `regrow`, `slaughter`, the TUI) | `move` | `from`, `to` (statuses); `thread` (the harness thread id) when made inside one; `child` (the yak it moved for) on an ancestor `shave` took along |
| `ask` | `ask` | `needs` (who it waits on) |
| `answer --note` | `answer` | `needs` (what it leaves: `agent`, or `none` for `--done` or a yak with no `needs`) |
| `request` | `request` | |
| `snooze` | `snooze` | |
| `pickup` | `pickup` | |
| `verify` | `verify` | `result` (`pass` or `fail`) |

Add your own with `update --meta` (`kind=checkpoint`, `commit=…`). The TUI shows a comment's metadata after `actor · when` in its header, `kind` first (`checkpoint · commit 47076b0`), and editing the comment keeps it. `doctor` checks the notes against `needs:` (see [`doctor`](#doctor)).

## Human-in-the-loop

| Command | What it does |
|---|---|
| `ask <id> --note '<question>'` | Block a yak on a human: sets the `needs` field, dropping it out of `next`. Also `--note -` (stdin) / `--note-file PATH`. [Details](#ask) |
| `answer <id> --note '<reply>'` | Answer a question (human-reserved): records the reply and sets `needs: agent`; `--done` clears `needs` instead. [Details](#answer) |
| `request <id> [--kind tidy\|summarize] [--note '<what to do>']` | Hand a yak to an agent with no prior question: sets `needs: agent` and records a `requested: needs agent` note (a `--kind` writes its canned instruction). Refuses a yak awaiting a human (answer it instead). [Details](#request) |
| `pickup <id> [--note '<text>' \| --drop-request]` | An agent takes on a yak awaiting it (answered or requested): clears `needs: agent` and records a `picked up` note (`--as`); `--drop-request` removes a done request's note instead. Fails on a yak that is not `needs: agent`. [Details](#pickup) |
| `inbox [--for human\|agent] [--json]` | Every yak with a `needs` value, across all statuses: **Awaiting a human** and **Awaiting an agent** (each marked `answered` or `requested`). `--for` shows one section; `--json`. [Details](#inbox) |
| `wait [--for human\|agent] [--timeout DUR] [--new] [--json]` | Block until the inbox has something for that audience, then print it as `inbox` does (exit 0); a timeout exits 2 with nothing on stdout. [Details](#wait) |

### `ask`

`ask <id> --note '<question>'`

Block a yak on a human: sets the `needs` field, dropping it out of `next`. On a yak that is already `needs: agent` it flips back to `human` and says so. Also `--note -` (stdin) / `--note-file PATH`.

### `answer`

`answer <id> --note '<reply>'`

Answer a question (human-reserved): records the reply and sets `needs: agent`, so the answer stays findable (in `inbox`, and in `next` when the yak is hairy) until an agent picks it up. `--done` clears `needs` instead, for an answer that needs no follow-up.

A second answer on a `needs: agent` yak keeps the state and appends the note; on a yak with no `needs` it only records the note. Also `--note -` / `--note-file PATH`.

### `request`

`request <id> [--kind tidy|summarize] [--note '<what to do>']`

Hand a yak to an agent when no question came first: a human (or a yakherd) asks for work on it, such as tidying the body or summarizing the notes. Sets `needs: agent` and records the request under a `requested: needs agent` first line, so the yak shows in `inbox --for agent` marked `requested` (and in `next`, when hairy, as `requested, awaiting pickup`) until an agent runs `pickup`.

A second request on a `needs: agent` yak keeps the state and appends the note. On a yak awaiting a human it fails and writes nothing: that yak needs an `answer`. Also `--note -` / `--note-file PATH`, and `--as`.

`--kind` records a canned chore whose whole instruction is written into the note, so the agent needs no skill to know what to do. The note is the kind's instruction (its first line names the kind), then `--note` (if any) after a blank line. Give `--kind`, `--note`, or both.

- `tidy` (`Tidy request: ...`): reshape the description and every note so they render as CommonMark and scan easily (blank lines between blocks, lists for run-on facts, missing code formatting added: fences with a language for commands, code and pasted output, backticks for inline commands, paths, flags, ids and symbols; and an inline enumeration or a paragraph of several points split into a lead-in and a list); only the connecting words that needs change, never facts, names, note headers or frontmatter; edit the file at `yaks path <id>` directly; then `pickup --drop-request`, which removes the request note, so the yak shows only the tidied body.
- `summarize` (`Summarize request: ...`): append one summary note (current state, decisions made, what is left), change nothing else, then `pickup --drop-request`.

To queue a kind on many yaks at once, use [`bulk --request`](#bulk).

### `pickup`

`pickup <id> [--note '<text>' | --drop-request]`

An agent takes on a yak awaiting it, answered or requested: clears `needs: agent` and records a `picked up` note attributed to the actor (`--as`).

`--drop-request` is the clean finish of a done request (what both canned kinds end with): it clears `needs: agent` and removes the latest `requested` note instead of adding a `picked up` one, so the yak shows only the result, and git keeps the record. It refuses a yak awaiting an agent because of an answer: an answer is never deleted.

The only command that clears `needs: agent`; on a yak that is not `needs: agent` it fails, naming the state. A `needs` change is not a status move (no `moved:` entry).

### `inbox`

`inbox [--for human|agent] [--json]`

Every yak with a `needs` value, across all statuses, in two sections: **Awaiting a human** (`needs: human`; marked `↩ replied` when its latest note after the ask is from someone other than the asker, i.e. a plain-note reply that did not use `answer`) and **Awaiting an agent** (`needs: agent`), each row marked `answered` or `requested`: whichever of an `ask` or a `request` note came last (a hand-set `needs: agent` with neither counts as `answered`).

`--for` shows one section.

`--json` is one array of yaks, each with `needs`, a derived `replied` boolean, and a derived `why`: `"answered"` or `"requested"` for a yak awaiting an agent, else `null`.

`ask` records an `asked: needs <who>` first line in its note: that is what `replied` is measured from, so a yak whose `needs:` was set by hand has no known asker and is never `replied`.

### `wait`

`wait [--for human|agent] [filters] [--timeout DUR] [--new] [--json] [--interval DUR]`

Block until the inbox for that audience (both sections when `--for` is omitted), narrowed by the usual filter flags, is non-empty; then print exactly what `inbox` prints for the same flags and exit 0. An agent runs `yaks wait --for agent --timeout 10m` as a background task to be woken when a question is answered; a human-side tool can `wait --for human` the same way.

- `--timeout DUR` gives up after `30s`, `10m`, `2h` (the duration form `log --since` takes; `0s` checks once). On timeout nothing goes to stdout, one line goes to stderr, and the exit code is 2. Without it, `wait` waits forever.
- `--new` ignores yaks already in the inbox when the wait starts. A row is a yak plus its `needs` value, so a yak answered during the wait (human to agent) counts as new, and so does one that is picked up and later comes back.
- `--json` prints the hit as `inbox --json` does.
- `--interval DUR` (default `2s`) is how often it re-reads the farm. It polls the files and takes no lock; there is no daemon or watcher.

## Multi-line text

Shell quoting makes multi-line markdown awkward as an inline `--note "..."` /
`--description "..."` argument (and the TUI renders a one-line blob as a single
wrapped paragraph). `create`, `update`, `ask`, `answer` and `request` therefore take the
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
| `migrate [--dry-run]` | Bring a farm on an older schema up to this yaks' (schema 4 to 7 → 8), under the farm lock. `--dry-run` writes nothing. Idempotent; never commits. [Details](#migrate) |
| `weight [--json]` | What the farm costs a clone of this repository: commits, farm-only bytes, the tree by kind, the largest blobs. [Details](#weight) |
| `lfs` | Store the farm's binary originals in Git LFS from now on. [Details](#weight) |
| `private [--off]` | Mark this repository's farm private so it never leaves the machine: sets `git config yaks.private true` and installs a `pre-push` hook that refuses pushes of the farm branch. `--off` undoes it. [Details](#private) |
| `init [--farm <checkout>]` | Start this repo's farm branch, or share another repo's (`--farm`), and install the skills. Idempotent. [Details](#init) |
| `skills install` | Install all four bundled agent skills to `--dir <path>`, `--user` or the project's `./.agents/skills`. Edited copies are left alone unless `--force`. [Details](#skills) |
| `skills status` | Per-skill verdict (`current`, `stale`, `adoptable`, `held`, `modified`, `unmanaged`, `source`, `retired`) from the provenance stamp. `--user` / `--dir` inspect another directory. See [skills.md](skills.md). [Details](#skills) |
| `doctor` | Read-only integrity check: invalid status, duplicate id, misplaced file, dangling parent/deps, malformed tags, `needs:` that disagrees with the latest kinded ask/answer/request/pickup note, a shorn or dead yak over unfinished children. Exits non-zero on any issue. `--strict` also flags shorn yaks with no note or whose `verify:` did not last PASS. [Details](#doctor) |
| `check [--json]` | Read-only upkeep check on doks and sources: doks past their `stale_after`, sources no dok or yak cites, dok kinds that differ only by plural or hyphens, plus `doctor`'s broken links, schema mismatches, merge conflicts and dok problems. Exits 1 on any finding. [Details](#check) |
| `preflight [<id>...] [--all] [--push-main]` | Read-only landing check: shorn yaks' `verify:` last PASSed, farm integrity, no merge conflict in the farm. `--push-main` checks the `local` remote. [Details](#preflight) |
| `whoami [--set <name>]` | Show the actor this checkout's writes are stamped with, and why; `--set` stores one for this checkout (`git config yaks.actor`), which outlasts fresh shells. Warns on a name another thread uses. [Details](#whoami) |
| `brief <id> --as <name> [--yaks-dir <path>]` | Print the worker brief for one yak, read-only: the part a worker needs that the yak, the config and the farm determine. `--yaks-dir` names the farm. [Details](#brief) |
| `status [--check] [--json]` | What the farm has changed that git does not have yet, one line per yak, in yak terms. Read-only. `--check` exits 1 when the farm is dirty; `--json`. [Details](#status) |
| `sync [--no-commit] [--no-pull] [--no-push] [-m <msg>]` | Commit, pull (rebasing with the yak merge driver) and push the farm branch. A private farm is never pushed. [Details](#sync) |
| `commit [-m <msg>] [--dry-run]` | Commit the farm on its branch, from any checkout. `-m` overrides the generated message. Never pushes. [Details](#commit) |
| `land <sha> [--dry-run] [--force-discard] [--yak <id>]...` | Land a Delta worker's commit by SHA on the current branch of the code checkout (squash, else cherry-pick), and note the landed sha on the yaks it names. Needs no farm; never pushes. `--dry-run` changes nothing; `--force-discard` drops unexplained working-tree changes; `--yak` names a yak explicitly. [Details](#land) |
| `tui` | Open the interactive terminal UI (see [tui.md](tui.md)). |

### `migrate`

`migrate [--dry-run]`

Bring a farm on an older schema up to this yaks' (schema 4, 5, 6 or 7 → 8), under the farm lock, writing `.yaks/schema` last.

First the **`herd-directories`** layout step: every yak file (`<id>.md` at the farm root whose `id:` is its name) and its attachment dir (`<id>/`) moves to `<herd>/yaks/` (plain renames: git sees 100% renames and `git log --follow` keeps working; bodies are untouched, since attachment links are relative to the yak file); other root files stay; refuses, moving nothing, if a destination exists or a herd is named like a reserved directory (`docs`, `doks`, `sources`, `schemas`, `sessions`).

Schema 8 renamed the wiki to **doks** (yaks-cc40), with two more steps. The **`doks-directories`** layout step moves `wiki/` to `doks/` and each herd's `<herd>/wiki/` to `<herd>/doks/` (plain renames, so `git log --follow` keeps working); it refuses, moving nothing, if `doks/` already exists. **`wiki-links-in-doks`** rewrites farm-root `/wiki/...` links to `/doks/...`, keeping any `@rev` and `#anchor`, in doks, schemas and sessions, and `type: Page Type` in a schema to `Dok Type`; the **`wiki-links-become-doks-links`** fixup does the same for the links in yak files. Sources are never rewritten (a source is a snapshot), and a link pinned to a commit from before the migration still reads the dok at its old `wiki/` path. `code:` links and relative links are untouched.

Then the named fixups over every yak file: **`labels-become-tags`** (schema 7) renames the top-level `labels:` frontmatter key to `tags:`, value untouched; **`comment-header-blank-line`** puts a blank line after every comment's `▸ <ts> [actor]` header (so the header is a paragraph of its own in any CommonMark viewer) and **`attachment-links`** relinks non-image attachments as `[stem](<id>/<file>)` instead of `![stem](<id>/<file>)` (images keep the `!`); **`attachment-link-encoding`** (schema 7) percent-encodes an attachment link whose file name holds a space (`](<id>/my%20shot.png)`), which markdown does not read as a link otherwise. Apart from that one key, only the body shape changes: other frontmatter, `updated:` and attachments are untouched.

`--dry-run` lists, per fixup, the counts and the yaks it would touch and writes nothing.

Idempotent: a current farm says `nothing to migrate`.

Every OTHER command refuses an older farm, naming `yaks migrate` (`yaks sheds` is no exception). A farm branch's worktree whose upstream (as last fetched) is already on the newer schema is only behind, because another machine migrated it: the refusal then says so and names `git -C <farm worktree> pull --ff-only` instead of `yaks migrate`. Otherwise it reminds you to fetch before migrating, so a farm is never migrated twice.

Never commits: it prints the `yaks commit` to run.

**Versions.** A farm on a newer schema than the binary is refused by every command, naming the upgrade (`npm i -g @j15r/yaks@latest`), and every write is refused too, so a long-running `yaks tui` that pulls a migrated farm goes read-only rather than writing the old shape into it. `min_version: 0.0.31` in `.yaks/config.yaml` makes an older yaks print `warning: this farm asks for yaks 0.0.31 or newer ...` on every command (not with `--json`) and in the TUI, but still run.

Once synced, a migration locks out every older yaks, so the bundled skills tell agents never to run it: a human does, in this order. Set `min_version:` to the new release and sync; upgrade every machine; `yaks migrate --dry-run`, `yaks migrate`, commit and sync, once; restart every running `yaks tui`.

### `weight`

`weight [--json]` measures what the farm costs: a plain `git clone` of a code repository fetches every branch, so each code clone pays for the farm branch's whole history (every attachment, source and per-change commit). It prints the farm branch's commit count, the bytes a clone fetches for it (objects no other branch has), the current tree by kind (yaks, attachments, doks, sources, other), the ten largest blobs only the farm branch holds, and whether LFS is on. Read-only.

`lfs` stores the farm's binary originals in Git LFS: images, PDFs, office documents, audio, video and archives under `.yaks/` (any case of the extension). It installs LFS's hooks for this repository (`git lfs install --local`), writes the rules into the farm branch's `.gitattributes` (other lines kept), and commits them together with the binaries already in the tree, converted to pointers; that commit holds only those paths. Older history keeps the plain bytes. Afterwards a clone fetches small pointers, and the bytes come down only where the farm is checked out. Markdown, `source.md` and `extracted.md` stay plain text, so search, diffs and merges work without the originals. Needs `git-lfs` installed and a remote that accepts LFS (GitHub does). Idempotent.

When a farm grows heavy with sources (many PDFs, several herds, or one farm shared by several repos), keep it in a repository of its own: `yaks init --farm <checkout>` points this repository at it, and code clones never see it.

### `private`

`private [--off]`

Mark this repository's farm private so it never leaves the machine: sets `git config yaks.private true` and installs a `pre-push` hook (in the hooks dir git uses; `core.hooksPath` honored, shared by every worktree) that refuses any push of the farm branch (`yaks` or `yaks.branch`), including `git push --all`, while pushing `main` etc. still works. `yaks sync` reads the same flag and never pushes a private farm (it still commits and pulls).

A `pre-push` hook that is not yaks's (no `# yaks-private-guard` marker) is left untouched and the command prints the check to fold into it.

`--off` unsets the flag and removes yaks's own hook only. Needs no farm, touches no remote.

### `init`

```sh
yaks init                                  # this repo's farm on the `yaks` branch + the skills
yaks init --herd api --type feature        # ...with the new farm's defaults
yaks init --branch yaks/team-a             # ...on another branch name (saved as yaks.branch)
yaks init --no-skills                      # ...without installing the skills
yaks init --farm ~/src/api --herd web      # share api's farm; new yaks here go in herd `web`
```

- **Own farm.** Creates the orphan branch and its worktree at the farm home.
  If the branch exists (locally or on `origin`) it only adds the worktree. Run
  inside a worktree you made with `git worktree add --orphan -b yaks <dir>`, it
  fills that in. Nothing is committed or pushed: `yaks sync` publishes,
  `yaks private` keeps it local.
- **`--farm <checkout>`.** Sets `git config yaks.farm` (and `yaks.herd` from
  `--herd`, declaring that herd in the farm's config). The other repo must
  already have a farm. Nothing is written in this repo's tree, and it gets no
  farm branch of its own.
- **Skills**: all of them, into the git top-level's `.agents/skills/` as
  ordinary project files, through the same install as `yaks skills install`.
  `--no-skills` leaves them out.
- **Idempotent.** One line per step, done or already-done; a re-run ends
  `Nothing to change`. A setting that contradicts what is there (another
  `--herd`, another `--farm`) is an error naming it; init never rewrites.
- **Refused**, changing nothing: outside a git repo; a Delta or `--shared` clone
  when a worktree would have to be created (run it in the main checkout); a
  `--branch` that is the code branch you are on; `--farm` naming this same repo
  or one with no farm; plain `init` in a repo whose `yaks.farm` is set.

The farm layout: `config.yaml`, `schema`, and one file per yak at
`<herd>/yaks/<id>.md` with a `status:` field (`docs/` and `<herd>/docs/` are
reserved, and so are the farm-root `doks`, `sources`, `schemas` and `sessions`
directories: none of those is a valid herd name). A farm on the old layout (schema 3,
status directories) is refused by every command, naming
`npx -y @j15r/yaks@0.0.16 migrate`.

### `skills`

`skills install`

Install the bundled agent skills (`yaks`, `yaks-tracker`, `yaks-coordinating`, `yaks-working`, `yaks-research`, `yaks-scheduling`, `yaks-upkeep`): always the whole set. Target: `--dir <path>` if given; else `--user` → `~/.agents/skills`; else `./.agents/skills` in the git top-level of the cwd when inside a git repo, and `~/.agents/skills` outside one.

Prints the destination it chose.

Untouched installed copies of the retired `yaks-coordinating-team`, `-private`, `-worktrees` and `-delta` are removed (an edited one is kept and reported). `skills status` lists any left as `retired`.

A skill is installed as a unit; an outdated untouched copy is upgraded, an edited one (including an edited script) is left alone unless `--force`.

Refuses to write onto yaks' own `.agents/skills/` source (even via a symlink, even with `--force`), naming `--user`/`--dir` as the way out.

`skills status`

Per-skill verdict — `current` / `stale` / `adoptable` / `held` / `modified` / `unmanaged` / `source` / `retired` — from the provenance stamp (other files of a skill are compared by content).

Same default directory as `install`; `--user` / `--dir` to inspect another. Every bundled skill is listed, absent ones included.

When something is stale it ends with the command to run (`yaks skills install`, with the same `--user` / `--dir`).

An ordinary command prints that advice itself as one stderr `note:` line, once per yaks version per checkout, for a stale project-local install (never with `--json`, nor for `skills`/`init`; `YAKS_SKILLS_AUTOSYNC=0` silences it); `doctor` repeats it in its skills advisory. See [skills.md](skills.md).

### `doctor`

`doctor`

Read-only integrity check: a `status:` value that is not hairy/shaving/shorn/dead (`invalid-status`), one `id:` in the frontmatter of two yak files (`duplicate-id`), a yak file away from its `<herd>/yaks/<id>.md` (in another herd's directory, under another name, or loose at the farm root; not loaded from there: `misplaced-file`), dangling parent/deps, malformed tags (a legacy tag containing a comma or space, e.g. `ui,docs` — any tag edit on that yak re-splits it), and `needs-drift`: the latest `ask`, `answer`, `request` or `pickup` note *with* `kind` [metadata](#note-metadata) leaves a state that `needs:` disagrees with (`ask` leaves its `needs` key, default `human`; `answer`, its `needs` key, default `agent`; `request`, `agent`; `pickup`, none). A yak with no such note is never flagged. Two commands change `needs:` without writing a note, so the check allows what they leave: after an `ask` note, `agent` or no `needs` (an `answer` with no `--note`); after an `answer` note, no `needs` (`answer --done` with no note, or `pickup --drop-request`, which removes the request note). And `unfinished-children`: a shorn or dead yak with hairy or shaving children, which the moves refuse ([Families stay consistent](#families-stay-consistent)) but an older farm or a hand edit can still hold; finish the children, regrow (or revive) the parent, or graduate them. And `dok-problem`: a dok with no frontmatter, no `type:` or `title:`, a name that is not lowercase-kebab, or a field value in YAML yaks does not read; the issue's `ids` holds the dok's path. And `broken-link` ([Links](#links)), `schema-mismatch` ([typed doks](#dok)), and `merge-conflict` ([merge driver](#merge-driver)).

Exits non-zero on any issue (CI-usable).

`--strict` also flags shorn yaks with no recorded note, and shorn yaks whose `verify:` command did not last PASS (evidence-before-shear).

### `check`

`check [--json]`

Hearth's nightly check minus what needs a model: what a person or an agent should look at in the farm's doks and sources. Read-only, never edits; exits 0 when clean and 1 when there is any finding. Each finding is `{kind, message, ids}` (`--json` gives the array; the text report groups by kind). Seven kinds:

- `stale`: a dok past its `stale_after`, the dok's own else its `schemas/<type>.md`'s. The value is an ISO 8601 duration (`P1Y`, `P6M`, `P2W`, `P30D`, or a run of them in that order) or a date (`2026-12-01`: stale once that day has passed). A duration counts from the dok's `verified:` date, else from the dok file's last commit on the farm branch; a dok with neither is skipped, and so is a value that is neither a duration nor a date. `doks/x.md was verified 2026-01-02; its type asks for a re-check by 2026-07-02`. To clear it, check the facts again and set `verified:` (`yaks dok set <path> --set verified=<date>`).
- `uncited`: a source under `sources/<id>/` that no dok or yak cites. A citation is a link or mention of `sources/<id>/` anywhere in a yak or dok, frontmatter `sources:` included; the source's own files do not count. A source and the sources saved with `--part-of` it count as one: cite any, and none is listed. A source whose `source.md` has `superseded_by:` or `uncited:` (any text) counts as cited: that is how to keep one on purpose, and [`source mark`](#source) sets it. Sources are written once, so this is a list to look at, not something to delete.
- `similar-kinds`: dok kinds, among the `type:` values in use and the names of `schemas/*.md`, that differ only by case, hyphens or a plural (`project`, `projects`, `work-project`). Pick one and `dok set` the others.
- `broken-link`, `schema-mismatch`, `merge-conflict` and `dok-problem`: the same issues, in the same words, as [`doctor`](#doctor) reports them; `check` does not check anything about yaks themselves.

### `preflight`

`preflight [<id>...] [--all] [--push-main]`

Read-only landing-readiness check, run before landing work.

Checks: (1) the farm has no unresolved merge conflict; its other uncommitted
changes are normal and print one ``preflight: warning: farm has N uncommitted
change(s): run `yaks commit` `` line; (2) every shorn yak in scope whose
`verify:` (own, else the config default for its tags) last PASSed, as
`doctor --strict`; scope is the ids given, else the shorn yaks with
uncommitted farm changes, `--all` every shorn yak; (3) integrity: no invalid
`status:`, no `id:` in two yak files.

Prints one `preflight: FAIL <check>: …` line per failure and exits non-zero,
else `preflight: ok`. A farm that is not a farm branch (`YAKS_DIR`) skips
check 1 and checks every shorn yak.

`--push-main` adds (4), for the yakherd about to `git push local <branch>:main` (off by default: it depends on another checkout's state, not on the change, and a repo with no `local` remote is not a failure without it): `local` must be a path on this machine (an https/ssh URL is reported as not the human's checkout, never resolved), a non-bare git repository with `refs/heads/main`, and not checked out on `main` with uncommitted changes to tracked files (git refuses the push then). Otherwise it fails with `preflight: FAIL local-checkout: …`, naming the fallback: push `pr/<name>` to `local` and give the human `git fetch <path> pr/<name>`, or repoint `local`.

Read-only (`remote get-url`, `ls-remote`, `rev-parse`, `status`); never pushes.

`--json` emits `{ok, failures[{check, message, subjects}], skipped, warnings}`.

### `whoami`

`whoami [--set <name>]`

Prints the actor this checkout's writes are stamped with and where it came from
(`--as`, `$YAKS_ACTOR`, `git config yaks.actor`, the harness environment or git
`user.name`; see [Attribution](#attribution)), plus `thread: <id>` inside a
harness thread. Needs no farm.

`--set <name>` stores the name as this checkout's `git config yaks.actor`: the
repository's own config for a primary checkout or a Delta clone, the worktree's
config (enabling `extensions.worktreeConfig`) for a linked `git worktree`, so it
never leaks into the other checkouts. Every later command, in any shell, then
uses it. It notes when `$YAKS_ACTOR` still overrides it.

Names must not overlap: pick one nobody else in the farm uses (`door-2`,
`herd-interact`, not `herd`). When a farm is found, `whoami` warns if another
thread's moves are recorded under the same name (`thread:` metadata). `brief
--as <name>` warns whenever the farm already has notes under that name.

### `brief`

`brief <id> --as <name> [--yaks-dir <path>]`

Print the worker brief for one yak, read-only: the part a worker needs that the yak, the config and the farm determine.

It names the yak and that it is already `shaving`; a first step that has the worker confirm its own checkout (`git rev-parse --show-toplevel`) and name itself with `yaks whoami --set <name>`; the `YAKS_ACTOR=<name>` prefix for every command (plus `YAKS_DIR` from `--yaks-dir`, or for a farm that is not a farm branch); this binary's absolute path; the evidence (the yak's own `verify:`, else the config default for its tags and herd); the forbidden moves; how to finish (`yaks shorn`, ONE commit of the worker's own files, never `.yaks/`; message `<id>: ...`, or, after `yaks private`, no yak id or the word yaks); how to ask and what to do once the ask is answered (`inbox --for agent`, then `pickup`); the `Decision:` note rule; multi-line notes on stdin; and the final-message format.

An unclaimed yak still prints, with a warning on stderr, as does a name the farm already has notes under (brief a fresh one); an unknown id fails with nothing on stdout; `--as` must be a shell-safe name.

What a yak cannot know (spawn title, how work lands, the model, scope by symbol) stays with the coordinator, who adds it around this text.

### `status`

`status [--check] [--json]`

What the farm has changed that git does not have yet, one line per yak, ordered by id, in yak terms: `created`, `moved hairy -> shaving` (the `status:` field of its `<id>.md` differs between HEAD and the working tree), `notes +2` (notes appended; the `moved:` entry a move writes is not counted), `edited` (a field, the description or an existing note changed), `removed`, `artifacts` (a new or changed file under `.yaks/<herd>/yaks/<id>/`, the yak's attachments); a yak with several kinds of change gets one line with all of them, and `config` is `config.yaml`.

A `*` in the first column marks a yak whose changes are all staged.

It uses the same classification as `commit` (the verbs cannot drift: `created`/`removed`/a move read the same, `notes` and `edited` are both `updated`), and ends with the message `commit` would use.

Code changed elsewhere in the repo is not listed, only counted in one line (`N other changed files outside the farm`), so you know `commit` leaves it alone.

A clean farm prints `farm clean: nothing to commit` and exits 0.

While a merge or cherry-pick is in progress it says `yaks commit` will refuse (git forbids the partial commit) and why.

Read-only: nothing is staged, committed or written.

`--check` exits 1 when the farm is dirty (same output, for scripts and hooks; otherwise the exit code is 0).

The last line says how that branch stands against its upstream, from the refs as of the last fetch (status never fetches or writes): `farm branch yaks: up to date with origin/yaks`, ``farm branch yaks: 2 ahead, 0 behind origin/yaks (as of the last fetch); `yaks sync` publishes`` (`reconciles` when it is also behind), or ``farm branch yaks: not published (no upstream); `yaks sync` publishes it``. `--check` is unaffected (dirty means uncommitted changes only).

`--json`: `{clean, untracked, merge_in_progress, yaks: [{id, changes, staged}], other: [{path, staged}], outside_farm, message, sync}`, where `sync` is `{branch, upstream, ahead, behind}` (`upstream` null when not published), or null for a `YAKS_DIR` farm.

### `sync`

`sync [--no-commit] [--no-pull] [--no-push] [-m <msg>]`

Sync the farm branch with its remote, in the farm worktree whatever the cwd: (1) registers the yak merge driver (`.gitattributes` rule `.yaks/**/*.md merge=yaks`, committed on the farm branch, plus `merge.yaks.driver` in the repo config); (2) commits what is dirty, as `yaks commit` (hand edits included); (3) fetches the farm branch from its remote (`branch.<b>.remote`, else `origin`) and rebases onto it, holding the farm lock, so concurrent edits to one yak on two machines merge into a valid yak (notes unioned, a field both sides changed: this side kept and the other recorded in a `merge-driver` note); (4) pushes, creating the remote branch with the same name and setting the upstream on the first publish, and pulls once more and retries if the remote moved meanwhile.

Each step can be skipped: `--no-commit` (a dirty farm then skips the pull too, leaving its edits local), `--no-pull`, `--no-push`.

A private farm (`yaks private`) commits and pulls but is never pushed.

A rebase that stops on a conflict the driver cannot settle (a non-yak file such as `config.yaml`) is aborted, leaving the branch exactly as it was, and the files are named. Refuses a `YAKS_DIR` farm that is not a farm branch.

The CLI never syncs by itself (only `yaks tui` does, every `sync_interval` in `.yaks/config.yaml`: seconds or `30s`/`5m`, default `5m`, `0`/`off` to disable, `YAKS_NO_AUTOSYNC=1` to skip it for a run): run it at session start, after landings and before a machine switch. Ends with the `yaks status` sync line.

### Every change commits

On a farm branch, every command that writes the farm commits what it wrote as one commit on that branch, with a `Yaks-Actor: <actor>` trailer naming who made it (yaks-a838). Nothing is pushed: `yaks sync` publishes.

- The commit holds only the paths the command wrote. Anything else dirty in the farm (a dok you are editing by hand, a scratch file) is left exactly as it was, staged or not.
- If the command rewrites a file that held an uncommitted hand edit, that edit is committed first, by itself (`yaks: hand edits to ...`), so undoing the command never takes the hand edit with it.
- In `yaks tui`, a quick burst of saves to the same files by the same actor (within 5 seconds, not yet pushed) folds into one commit.
- A commit that fails never fails the command: the change is written, the reason is printed (in the TUI, the status line), and `yaks commit` commits it later. Nothing is committed while a merge or cherry-pick is in progress in the farm worktree.
- `YAKS_NO_AUTOCOMMIT=1` turns it off for one command. A `YAKS_DIR` farm outside git never commits.

`yaks commit` remains for hand edits (a dok edited in your editor) and anything else left dirty. A hand edit is the person's by design, so that commit carries no `Yaks-Actor`; an agent that edited a dok by hand commits it with `yaks commit --as <actor>` to own it (see [`commit`](#commit)).

### `undo`

`undo <commit>...` · `undo --by <actor> [--since <when>] [--yes]`

Revert farm commits as one new commit (nothing is rewritten), merged through the yak merge driver so every yak stays valid. Name the commits (as `git log` shows them in the farm worktree), or select every commit whose `Yaks-Actor` is `--by <actor>`, optionally `--since 2h` (or `1d`, an ISO date). With a selection it only lists the commits; `--yes` reverts them. Refuses while the farm has uncommitted changes, and when a later change touched the same lines (the revert is abandoned and nothing changes).

### `commit`

`commit [-m <msg>] [--dry-run] [--as <actor>]`

Commit the farm's uncommitted changes (yak files, attachments, doks, config) on its branch, in the farm worktree, from any cwd: hand edits, mostly, since every yaks write commits itself ([above](#every-change-commits)). Stages the farm and runs `git commit --only -- .yaks`.

The message is generated from the changed files, one verb per yak (`created`, `shaving`/`shorn`/`dead`/`regrown` for a move, `updated`, `removed`, `artifacts for`), then `doks <paths>` for doks, sources and schemas, then `farm config`, e.g. `yaks: created yaks-8c08; shaving yaks-c968`; `-m` overrides it. `--dry-run` lists the files and message without staging or committing.

`--as <actor>` stamps the commit as that actor's: a `Yaks-Actor: <actor>` trailer (and, on a machine with no git identity, that name as the committer), so the commit shows up in `yaks session touched --by <actor>` and `yaks undo --by <actor>` and in the actor's generated `## Saved`. Without `--as` nothing changes: a plain `yaks commit` stays unstamped, because the checkout's `whoami` name may be a person's and the commit would claim their edits for whoever ran it. The stamp covers everything the commit holds, so use it for edits that are yours. An empty `--as` is an error.

A normal `git commit`, so the repo's hooks run (and may fail it: the farm changes are then left staged and the hook's output is in the error); git's and the hooks' output is captured, so a successful commit prints only yaks' own summary (and never garbles the TUI); it never pushes. A clean farm prints `nothing to commit` and exits 0.

While a merge or cherry-pick is in progress in the farm worktree it refuses, stages nothing and says why. A `YAKS_DIR` farm outside git is an error.

### `commits`

`commits <id> [--fetch] [--json]`

Four sections, in this order (the second only when a note records a branch):

1. **Recorded** lists every note on the yak with a `commit:` key (`yaks land` writes them, see below): the note's `kind`, the short sha, `on <branch>` when it has one, the subject as the code repository has it (`(not in this repository)` when the commit is not there: a clone that lacks it, a rewritten history), and the note's timestamp and actor. A recorded sha is the only link that works on a private farm, whose commits never name a yak.
2. **Branch `<remote>/<branch>`** (yaks-4728), one per branch the yak's notes record with a `branch:` key (a `checkpoint` note: `--meta kind=checkpoint --meta branch=<b> --meta commit=<sha>`; `landed` notes do not count, their branch is where the work landed). Never name a branch under the farm branch's name (`yaks/...` beside a farm branch `yaks`): git cannot hold both refs, so it cannot be pushed, and `update` warns when a `branch=` value does. Optional `remote:` (default `origin`) and `base:` (the newest one wins; default the remote's default branch, `origin/HEAD`, else a local `main`). It lists `git log --oneline <base>..<remote>/<branch>` from the code repository, newest first, and marks each recorded checkpoint commit with `← checkpoint`. A checkpoint sha that is not reachable from the branch gets a line saying so (`checkpoint abc1234 (…) is NOT on origin/<b>: the branch was rewritten, or the commit was never pushed`); one already in the base says it landed or merged.

   **Fetching is explicit.** Without `--fetch` nothing touches the network: it reads the remote-tracking ref already there and says `as of the last fetch of <remote>` (or that the branch has never been fetched here). `--fetch` first runs `git fetch <remote> +refs/heads/<branch>:refs/remotes/<remote>/<branch>` for each branch, with prompts off. A fetch that fails (no such remote branch, offline) prints `fetch failed: <git's message>` under that branch and goes on to the next, falling back to whatever tracking ref exists; the command still exits 0 when the yak exists.
3. **Commits naming `<id>`** greps commit messages in the *code repository*: the git top-level of the directory yaks was started from (so `-C <path>` picks it), across every local branch, remote branch and tag except the farm branch, newest first. A branch farm lives on its own branch, so searching the farm's worktree would find only `yaks: ...` farm commits and never the code; when yaks is started outside any code checkout (not in git, or in the farm's own worktree) the section says so instead of listing farm commits.
4. **Farm history** is the yak file's own edit history on the farm (`git log --follow`), labelled as such: it shows when the yak changed, not what code did.

`--json` prints the same as `{id, code_repo, recorded, fetch, branches, named, farm_history}`: `branches` entries are `{branch, remote, tracking, base, fetched, fetch_error, error, commits: [{sha, short, subject, checkpoint}], checkpoints: [{commit, ts, actor, state}]}` with `state` one of `on_branch`, `in_base`, `not_on_branch` (`error` says why there is no log: no tracking ref, no base); `recorded` entries carry `kind`, `commit`, `source`, `branch`, `ts`, `actor` and `found` (`{sha, short, subject}` or `null`); `named` and `farm_history` are lists of `{sha, short, subject}`.

### `land`

`land <sha> [--dry-run] [--force-discard] [--yak <id>]...`

Land a Delta worker's commit by SHA on the current branch of the code checkout you are in (needs no farm; never pushes; git 2.38+, 2.40+ for the already-landed check).

Delta's applied state (a pending `MERGE_HEAD`, the worker's files in the working tree) is not trusted: it is reported (`worker commit:`, `pending state:`, a shallow-repository note when ancestry answers are lower bounds), then cleared.

When the worker's base is an ancestor of HEAD it computes the merged tree with `git merge-tree --write-tree` and lands it as ONE single-parent squash commit (the worker's message and author date + a `Squashed-from: <full sha>` trailer), then checks HEAD's tree equals the computed one; when the base was rewritten or the merge would conflict it says so (naming the `CONFLICT`s) and `git cherry-pick`s instead.

Refuses, changing nothing: working-tree changes the commit does not explain (exit 3; `--force-discard` drops them; an untracked file that differs from the worker's is always exit 4); a SHA that is already an ancestor of HEAD but whose change is missing or unconfirmable in HEAD's tree (exit 6, the evil merge: it names the first commit that lacks the change); a commit that touches `.yaks/` when HEAD tracks none (a worker staged yak files; exit 7).

A SHA already in HEAD with its change present, or a worker landed before (the merged tree equals HEAD's), says `landed:` and adds no commit.

**Records the landing on the yak.** After a landing that made a commit, `land` opens the farm of the checkout (if one resolves) and appends a note to every yak id the landed commit's message names (the same scanner as `yaks scan-ids`, against the farm's real ids) and to each `--yak <id>` (repeatable; a private farm keeps ids out of messages, so name the yak explicitly). The note carries metadata `kind: landed`, `commit: <full landed sha>`, `source: <full worker sha>` and `branch: <branch HEAD is on>`, with the body `landed: <short sha> <subject>`, stamped with the usual actor, under the farm lock. Only the landed sha is durable (a worker's sha dies on squash or rebase), which is why both are kept. An id the farm does not have is warned about and skipped; landing is never undone by a recording problem. A SHA that was already in HEAD records itself (`commit` is that sha, no `source`); landing the same commit twice records once, and a run that adds no commit records nothing. `yaks commits <id>` reads these notes back first.

`--dry-run` reports every step and changes nothing; it says which yaks it would record on.

Ends with `landed: <oneline>` and `doctor: <last line of yaks doctor>` when a farm resolves.

## Attribution

`--as <actor>` on note-writing commands stamps the note `▸ <ts> [actor]`, and on
the state-transition verbs stamps the `moved: <from> -> <to>` entry the same
way. The
actor resolves `--as` → `$YAKS_ACTOR` → this checkout's `git config yaks.actor`
(set by [`whoami --set`](#whoami)) → the harness identity → git `user.name`.
Under Delta the harness identity is derived from its terminal environment: a
thread title that starts with a worker name (`door-2: add doors`: one token of
letters, digits, `.`, `_`, `-`, then `: `) gives `delta:door-2`; any other title
gives `delta:<thread title>` (`$DELTA_THREAD_TITLE`, whitespace collapsed, cut
to 40 characters); with no title, `delta:<thread id>` (`$DELTA_CURRENT_THREAD_ID`).
The `delta:` prefix marks a derived, non-human actor, and the CLI says so on
stderr, once per command, when a write's actor was derived. A chosen name is
still preferred: it is stable, where a thread title can be renamed. In a
harness where every terminal call is a fresh shell, `export YAKS_ACTOR` does not
last; `yaks whoami --set <name>` does. Inside a harness thread, every state move
also records `thread: <id>` metadata, so a name can be traced back to the thread
that used it. (The git author, the human under Delta, is not what attributes
a move: the entry in the yak file is.) Attribution, never
ownership — a yak belongs to no one.
