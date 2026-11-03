---
name: yaks
description: Yaks task tracking workflow. Use when the project tracks work with yaks (`yaks path` prints a farm, or a `yaks` git branch exists). Provides commands and guidance for managing tasks stored as markdown files with YAML frontmatter on their own git branch.
---

# Yaks — Task tracking workflow

This project tracks work with yaks: each task (a **yak**) is a markdown file with YAML frontmatter in a `.yaks/` **farm** that lives on its own git branch. Follow this workflow so task state stays accurate.

## Running yaks

A plain CLI, run from the shell (no slash commands, nothing to keep running): `yaks <cmd>` if it is on `PATH`, else `npx @j15r/yaks <cmd>`, else `./target/release/yaks <cmd>` in a checkout you built. Add `--json` to any query command for machine-readable output; `yaks create --json` also prints the new id and path.

**Keeping yaks current.** Run the latest release: compare `yaks --version` with `npm view @j15r/yaks version`, and when it is behind, or a command says to upgrade (`warning: this farm asks for yaks ...`, or `newer than this yaks supports`), upgrade with `npm i -g @j15r/yaks@latest` (or run every command as `npx -y @j15r/yaks@latest <cmd>` where a global install isn't possible), then carry on. A binary built from a yaks checkout is the exception: rebuild it instead.

**Never migrate a farm yourself.** When a command says the farm is on an older schema and to run `yaks migrate`, stop and ask a human (`yaks ask` on the yak you were on, or say so in chat): once a migrated farm is synced, every machine still on an older yaks is refused it, so a human picks the moment. Don't work around the refusal with an older binary either.

## Where the farm lives

The farm is an orphan branch (`yaks`) holding only `.yaks/`, checked out once per machine under `~/.local/share/yaks/farms/`. Every checkout of the repo (yours, the human's, each agent clone) reads and writes that one farm, so yak edits are live everywhere.

```sh
yaks path                  # which farm am I on?
yaks status                # what is uncommitted; is the branch ahead/behind?
yaks commit                # commit hand edits (every yaks write commits itself)
yaks undo --by <actor>     # list (then --yes: revert) an actor's farm commits
yaks sync                  # commit, pull (yak-aware merge), push
yaks conflicts             # documents a merge left for you to settle
```

- **Never stage `.yaks/` with your code.** Code commits never contain yaks.
- **Sync at session start, after landing work, and before switching machines.** Nothing syncs on its own, and nothing you write reaches another machine until you sync. On a machine nobody is sitting at (a cloud runner, a VM, a remote box: no human there reads this farm directly), also **sync after every note or state move** you make: that is the only way anyone else sees your progress, and a reset there deletes the farm worktree with every unsynced write. If a yakherd coordinates you, it syncs and you don't, unless your brief says otherwise. In `yaks tui`, `W` commits and `U` syncs.
- **Starting one:** `yaks init` (the branch, its worktree and these skills; idempotent). A clone of a repo with `origin/yaks` needs no setup.
- **Sharing another repo's farm:** `yaks init --farm <checkout> --herd <h>` sets `git config yaks.farm`/`yaks.herd`; this repo then uses that farm, and `yaks create` puts new yaks in herd `<h>`.
- **`YAKS_DIR=<dir>`** overrides discovery for one command.

**Checkout weight.** Every clone of the code repository fetches the farm branch too. `yaks weight` says what it costs; `yaks lfs` stores binary originals (images, PDFs, archives) in Git LFS from then on. A farm heavy with sources belongs in a repository of its own (`yaks init --farm <checkout>`).

**Private farms.** `yaks private` keeps the farm on this machine: a `pre-push` hook refuses any push of the farm branch, and `yaks sync` never pushes. `git config yaks.private` says whether it is set.

## Vocabulary

- **States** (adjectives): **hairy** (not started), **shaving** (in progress), **shorn** (done), and the hidden **dead**.
- **Verbs** (the commands): **shave** (hairy → shaving), **shear** / `yaks shorn` (shaving → shorn), **regrow** (shorn → hairy), **slaughter** (→ dead), **revive** (dead → hairy).
- **Farm**: one `.yaks/` store, on its own branch. **Herd**: the yaks sharing one id prefix (a farm can hold several; `yaks create --herd <h>`). **Family**: a parent yak and its descendants. **Shed**: someone working in the farm (a human, a worker, a thread), known by the actor name on their notes; `yaks sheds` lists them, on any machine once the farm is synced.

## Hard rules

1. **Never write code without a shaving yak.** Even a one-line fix: `yaks shave <id>` first (create the yak if needed). A claim is how everyone else, on any machine, sees what you are working on; unclaimed work is invisible.
2. **Shear as soon as the work is done:** evidence and a summary note, then `yaks shorn <id>`.
3. **Keep yak ids out of external surfaces:** PR titles and bodies and external trackers. Ids are fine in commit messages, unless the farm is private (`yaks private`): then keep ids, and the word "yaks", out of commits, code and comments, since their readers cannot see the farm. Use the external key instead (`yaks rollup --keys`; the **yaks-tracker** skill). `yaks scan-ids [FILE]` exits non-zero if text holds a real yak id; wire it into a hook.

## Workflow

1. **Session start:** check yaks is current ("Keeping yaks current", above). Name yourself once, if you are an agent: `yaks whoami --set <name>` stores it for this checkout, so every later command uses it even in a fresh shell (`export YAKS_ACTOR` does not survive harnesses whose terminal calls are separate shells). Choose a name nobody else in the farm uses, specific to you (`door-2`, `herd-interact`, not `herd`): others will read your trail by it. Without one, your notes carry a name derived from the harness, or the human's git name. Then `yaks sync` (unless a yakherd coordinates); then `yaks inbox --for agent` (answers and requests nobody has acted on: handle these first); then `yaks next`.
2. **Before code:** `yaks shave <id>` (it shaves hairy ancestors too). Re-read it with `yaks show <id>`; the latest note is the freshest signal.
3. **While working:** `yaks update <id> --note "..."`, one event per note, at each milestone and at least every 30 minutes of work: your latest note is how others tell a busy worker from a dead one.
4. **Code others can't see** (you are on another machine, or in a clone nobody reads): at each checkpoint, commit, push your work branch to `origin` (any branch name, your harness's own pattern is fine, but never `main` and never under `yaks/`: the farm branch `yaks` blocks that prefix, so such a branch cannot be pushed), note it (`--meta kind=checkpoint --meta branch=<branch> --meta commit=$(git rev-parse HEAD)`), then sync. In that order, so the farm never names a commit nobody can fetch.
5. **Stopping without finishing:** to resume later, a note saying where you stopped and what's next (the claim stays yours and reads `stale` in `yaks sheds` after 2 hours); to give it up, that note and then `yaks regrow <id>`, so nobody mistakes it for live work.
6. **Done:** gather evidence, add a short summary note (see "Evidence before you shear"), `yaks shorn <id>`. Then the family: shear the parent if that was its last open child, and file any follow-up where "Families" says.

**You share the farm with a human.** They edit yaks in `yaks tui` while you work, so a yak may move, gain a note or appear without you. That is signal, not an error: don't revert, restage or report it; follow it if it redirects your work.

## Families

A parent is the sum of its children: open (hairy or shaving) while any child is, finished (shorn or dead) only when every child is. The commands keep it so: `shave` takes hairy ancestors along; `shorn` and `slaughter` refuse a yak with open children; `create --parent`, `reparent`, `regrow`, `revive` and `shave` refuse to open a yak under a finished one; `yaks doctor` flags a farm that breaks it anyway. When a command refuses, take the fix it names.

- **Closing a family.** One `yaks shorn <parent> <children...>` shears a whole family, in any order. A parent that is done except for some children: graduate each first, under the nearest open ancestor it fits (`yaks reparent <child> --parent <id>`) or to the top level (`--unparent`), with a note on it saying where it came from; then shear.
- **Follow-ups you find while working.** A child of your yak only if your yak isn't done without it, and then you finish it before you shear. Anything else goes under the nearest open ancestor it belongs to, or at the top level; name it in your summary note. If the parent's own description promises that work, the parent stays open until it is done.
- **Abandoning.** `yaks slaughter --family <id>` takes the open descendants with it; shorn ones stay shorn, since that work was done.

## The inbox: work waiting on someone

A yak's `needs` field says who acts next, and `yaks inbox` lists every yak that has one: **awaiting a human** (`needs: human`) or **awaiting an agent** (`needs: agent`, marked `answered` or `requested`).

**Asking a human.** For a decision only a human should make (an ambiguous requirement, a risky tradeoff, a missing credential), don't guess:

- `yaks ask <id> --note "the question"` sets `needs: human` and drops the yak out of `yaks next`. Then return control.
- The human runs `yaks answer <id> --note "..."` (or `a` in the TUI), which sets `needs: agent`. Never answer your own ask.
- `yaks inbox` marks a yak `replied` when the human answered with a plain note instead of `answer`.
- Also ask to *record* a real design fork (behavior a user sees, a data format, a dependency) even when you have a lean: state the lean in the question.
- **Design questions belong on yaks, not only in chat.** When a design pass raises questions for the human, ask each on the yak it concerns, with the options and your lean, and in chat link the asks rather than restating them. The question and its answer then stay with the yak for whoever picks it up later; record the outcome as a `Decision:` note when you pick it up. A yak holds one open ask at a time: put related questions in one numbered ask, or split the yak first.

**Requests.** A human (or a yakherd) hands you work with `yaks request <id> --note "..."` (`a` in the TUI on a yak that isn't awaiting them): a chore on that yak, such as tidying its markdown, summarizing its notes or checking something. Treat it as an instruction from the human, scoped to that yak, and do it now unless it needs a decision (then `ask`). A canned one (`--kind tidy|summarize`) arrives as a `Tidy request:` / `Summarize request:` note carrying its whole instruction: follow it literally. It ends with `yaks pickup <id> --drop-request`, which removes the request note so the yak shows only the result; use that for any request whose result speaks for itself.

**Taking it on.** For either kind, `yaks pickup <id> --note "what I'll do"` is the only thing that clears `needs: agent`; then do the work and note the result. An inbox row you leave stays there for the next agent.

**Waiting.** `yaks wait --for agent --timeout 10m` blocks until something is awaiting an agent, then prints it like `inbox` (exit 0; exit 2 on timeout). If your harness can run a command in the background, run it there after an `ask` to be woken by the answer; otherwise return and check `yaks inbox` next session. `--new` ignores rows already there when it started.

## Notes and descriptions

Bodies are GitHub-flavoured CommonMark. The TUI renders each note as its own document, and a human reads it there, so write for a reader scanning a pane, not a log file.

**Shape.**

- **A blank line between every block.** A single newline is only a space: lines with no blank line between them join into one paragraph. Separate each paragraph, list, heading, code fence and table from its neighbours by a blank line. Don't hard-wrap prose.
- **Short blocks.** A paragraph makes one point in a few sentences. Parallel facts (results, files, per-case numbers, next steps, options) are a list, one item each, never enumerated inline (`(1) ... (2) ...`, or `X (option D), or Y (option A)` strung across clauses). A long body with distinct parts gets `##` headings.
- Fence commands, code and pasted output, with a language (```` ```sh ````). Put ids, paths, flags and symbols in backticks.
- Also rendered: tables, task lists (`- [ ]`), footnotes, strikethrough and alerts (`> [!NOTE]`, `TIP`, `IMPORTANT`, `WARNING`, `CAUTION`). Not rendered: inline HTML (shows as literal text), math, emoji shortcodes; mermaid shows as a code block.
- Keep a `Decision:` note (the choice and the alternatives rejected) separate from progress notes.

**Metadata.** A note can carry `key: value` fields, which the TUI shows in its header and `yaks show --json` lists under `meta`. The commands stamp their own (`kind: move`, `ask`, `answer`, `request`, `pickup`, `verify`; `yaks land` writes `kind: landed`); add yours with `--meta key=value` on `yaks update`, which needs a note: `yaks update <id> --note "..." --meta kind=checkpoint --meta branch=<branch> --meta commit=<sha>`. Reserved keys: `kind`, `for`, `commit`, `branch`, `remote`, `thread`. In the file they sit under the note's `▸` line, closed by `---`, like a yak's frontmatter; never write that by hand.

Not this (one run-on paragraph, because single newlines join):

```text
G1 landed (d84aa65). Splits families into tokens; fullstory joins 161.
NO FIXTURE yet: the hand-built one didn't reproduce the chain.
Next: G3's question, so it belongs with yak-7af5.
```

This:

```markdown
G1 landed (d84aa65): families are now split into their tokens.

- fullstory: 161 joins
- no fixture yet: the hand-built one didn't reproduce the chain

Next: G3's question, which belongs with yak-7af5.
```

**Links.**

- Another yak: its **full id** (`yak-0af1`, never the bare `0af1`); the TUI makes it a link and `yaks refs <id> [--incoming]` lists the references.
- Repo code: `code:` and a path from the repo root, with GitHub's line fragment: `[fn parse](code:src/foo.rs#L12-L40)`; `@<sha>` before the fragment pins it to a commit (`code:src/foo.rs@3f2a9c1#L12`). The older `../src/foo.rs` and `../blob/<sha>/src/foo.rs` still work in yaks.
- A page: from the farm root, `[link grammar](/wiki/design/link-grammar.md#heading)`; a relative link only goes down (`notes/detail.md`), never `../`. `@<sha>` pins a page too.
- `yaks peek <link>` prints just what a link names (the lines, the section, or a page's frontmatter and first paragraph): read that rather than a whole file. `yaks doctor` reports links that lead nowhere.
- Attachments: `yaks attach` writes the link (`[name](<id>/<file>)`, `![name](...)` for an image). Attach images and long output rather than pasting them.
- A markdown document you attach that embeds images (an evidence write-up with screenshots): attach it and its images together with `--no-link` (`yaks attach <id> doc.md a.png b.png --no-link`), then link only the document in a note (`[evidence](<id>/doc.md)`). Inside an attachment, a bare file name (`![a](a.png)`) is a sibling attachment of the same yak, so keep the document's own links bare and don't put each image on the yak.
- Size an image with a trailing alt-text token: `![shot =50%](<id>/shot.png)`, `=300x200` or `=x120` (px, or `%` of width).
- **A yak from your conversation with the human:** when your harness renders links to local files (Zed, Delta, most IDE chats), link the yak's file by its absolute path, as printed by `yaks path yak-0af1` (`[yak-0af1](/home/me/.local/share/yaks/farms/.../yak-0af1.md)`), so they can open it in one click. A yak's file never moves, so the link stays good. (Hard rule 3 still keeps ids out of PRs and trackers.)

**Writing it.** Any note with a newline, a backtick or a `$` goes on stdin: `yaks update <id> --note - <<'EOF'` … `EOF` (quoted `'EOF'` keeps backticks and `$` literal). Never inside `$( )` or a double-quoted argument, where backticks run as commands. `--note-file` / `--description-file` read a file; the same `-` works for `create --description`, `ask --note` and `answer --note`.

## Evidence before you shear

Shear against evidence, not vibes: "it compiles" or a self-report is not enough.

- **A command proves it:** store it with `yaks update <id> --verify '<cmd>'` and run `yaks verify <id>`, which records PASS/FAIL as a note anyone can re-run. `yaks doctor --strict` flags a shorn yak whose `verify:` did not last pass.
- **A web UI change:** a screenshot of the changed state, attached. `sightmap browser`, when available, can find the component and capture just it.
- **A TUI change:** a text frame of the changed state at least; a rendered screenshot if the TUI library can produce one.
- **Code:** link the code that matters (`[fn parse](code:src/foo.rs#L12-L40)`); a short snippet only when it is the point.
- **Logs and long output:** the few lines that show the result, fenced; attach the rest.
- Update every surface that describes a user-facing change (in-app help, `--help`, docs) in the same change.
- **No way to verify it?** `yaks ask` whether to build one rather than shearing on faith.

Skip what makes no sense for the change; never skip it because it is effort.

**The summary note** is short: what changed and, unless that already says it all (a typo fix), the shape of the fix in a sentence or two; the evidence (linked or attached, per the list above), and any yaks spawned. Not a bare "done", and not a design doc: a long write-up goes in an attachment the note links to.

`yaks attach` stores the file beside the yak in `.yaks/<herd>/yaks/<id>/` and links it in the body.

## Tags

Few and broad (`ui`, `docs`, `cli`): reuse an existing tag before inventing one (`yaks stats` lists them; `yaks page types` shows the farm's known tags and what each means, from `schemas/yak.md`), at most two or three per yak, one word each (`--tags ui,docs` is two tags). A tracker tag (`jira`, `github`) marks yaks that map to an external issue.

## Pages

A farm also holds **pages**: markdown documents that are not yaks, under `wiki/` (farm-wide) or `<herd>/wiki/` (one herd's), named by their path from the farm root (`wiki/topics/vector-search.md`). Use a page for working knowledge that should outlive the yak that found it: research findings, a decision record, a design note. Product docs that ship with the code stay in the code repo's `docs/`; findings scoped to one yak stay in its notes and attachments.

- Search before creating (`yaks page list`, `yaks page query --where tags=<tag>`); extend an existing page rather than starting a second one.
- Create with `yaks page new <path> --type topic --set 'tags=[a, b]' --body-file notes.md` (kinds: `note`, the default; `topic`; `howto`; `summary`). Lowercase-kebab paths.
- Edit fields with `yaks page set <path> --set key=value`, or the body with your editor. `yaks doctor` checks pages.
- Facts you would list, filter or sort (appliances, vendors, platforms compared) go in **typed pages**: `yaks page types` shows the kinds and their fields; give a page `type: <kind>` and the fields its `schemas/<kind>.md` lists, and add or extend the schema when a new kind earns one. Warnings are advice, never refusals. `schemas/yak.md` lists the farm's known tags: pick from it before inventing one.
- Research yaks: the `yaks-research` skill says where each finding goes, and how to save, cite and split.
- Link the page from the yak that produced it, so it stays in context. Move a page with `yaks mv <from> <to>`, which rewrites every link to it.
- Outside material you rely on (a paper, a downloaded page, a screenshot, a small code directory) is a **source**: `yaks source add <file> [--url <page it came from>]` snapshots it once under `sources/<id>/`; cite its `source.md` in the page's `sources:` list. A finding first kept as a yak attachment moves with `yaks promote <id>/<file>` (a source) or `--page wiki/<path>.md` (a page); links to it follow.

## Commands

Every command has `--help`; this is the map. Global flags: `-C <path>` (run as if in `<path>`, like `git -C`), `--as <actor>` on any write (else `$YAKS_ACTOR`, else the checkout's `yaks whoami --set` name, else a Delta thread name, else the git user).

| Command | What it does |
|---|---|
| `create "title"` | New hairy yak: `--type --priority --parent --herd --tags --depends-on --source --pr --verify --description` |
| `list` / `next` / `tangled` / `search` | Query (shared filters, below); `next` = hairy with deps met, `tangled` = blocked on deps |
| `find <pattern>` | Full-text search over yak bodies and notes, pages, sources' text and sessions (`--scope <prefix>`, `--kind yaks\|pages\|sources\|sessions`, `--regex`, `-i`, `--json`); `path:line: text` per match, exit 1 when none. `search` is the yak-only id/title/description lookup |
| `show <id>` | Full details, parent and children |
| `path [<id>...]` | Absolute file path of yaks (by id or filters); no arguments: the farm in use |
| `update <id>...` | Fields, `--add-tag`/`--remove-tag`, `--note` (with `--meta key=value`), `--verify`, `--source`, `--pr`/`--remove-pr`, `--set key=value`/`--unset key` (any other field) |
| `shave` / `shorn` / `regrow` / `slaughter` / `revive` | State moves; several ids at once; each appends an attributed `moved:` entry. `slaughter --family` takes open descendants too |
| `ask` / `answer` / `request` / `pickup` / `inbox` / `wait` | The inbox (above); `inbox --for human\|agent`, `wait --timeout 10m` |
| `verify <id>` | Run the yak's `verify:` (else the config default) and record PASS/FAIL |
| `attach <id> <file>...` | Store and link evidence (`--no-link`: store only, for the parts of a document that links them itself); `rename-attachment` renames one |
| `whoami [--set <name>]` | The name your writes are stamped with, and why; `--set` names this checkout once |
| `log` | Notes and moves over a filtered set, oldest first; `--since 2h`, `--by <actor>` |
| `refs <id>` | What it points at; `--incoming` for what points at it |
| `commits <id>` | Commits recorded on the yak (`yaks land` writes them) and the branches its checkpoints name (`--fetch` fetches them), then those naming it in the code repo you run it from, then the yak file's farm history |
| `page list` / `show` / `new` / `set` / `query` / `types` | The farm's pages (`wiki/**.md`, `<herd>/wiki/**.md`): see "Pages" below |
| `dep` / `reparent` | Add/remove a dependency; move under `--parent <id>` or `--unparent` |
| `bulk` | One field edit or reparent over a filtered set; dry-run unless `--commit`; refuses without a filter |
| `rename` / `rename-herd` | Rename a yak or a whole herd, rewriting every reference; `--dry-run` first |
| `undo` | Revert farm commits: named, or `--by <actor> [--since 2h]` (lists; `--yes` reverts) |
| `status` / `commit` / `sync` | The farm's uncommitted changes; commit them; commit, pull and push it |
| `conflicts` | Files a merge left to settle: prose holding `<<<<<<< here` … `>>>>>>> other side` markers, or a page's `merge_conflicts:` list. Fields always merge on their own; edit the text to one version (and drop the settled `merge_conflicts:` entry) |
| `weight` / `lfs` | What the farm costs a code clone; store binary originals in Git LFS |
| `private [--off]` | Keep the farm on this machine |
| `import-hearth <repo>` | Copy a Hearth-layout repo's wiki, sources, schemas and sessions into the farm (links rewritten, one commit; `--dry-run` first) |
| `init` / `migrate` / `merge` | Start a farm (or `--farm` to share one); upgrade an old schema (a human's call, never an agent's); consolidate another farm |
| `doctor [--strict]` / `preflight` | Farm integrity; pre-landing check |
| `check [--json]` | Upkeep of pages and sources: stale pages (`stale_after`), uncited sources, kinds that look alike, broken links; exits 1 on a finding |
| `stats` / `rollup [--keys]` | Counts; yaks grouped by external issue |
| `skills install` / `skills status` | Install or check these skills |
| `tui` | The interactive UI (`?` lists its keys) |

Coordinating several agents (`brief`, `land`, `sheds`) is the **yaks-coordinating** skill.

## Filtering

`list`, `next`, `tangled`, `search`, `path`, `log` and `bulk` share filters: `--status`, `--type`, `--priority`, `--tag`, `--herd` (each repeatable, OR within a flag), `--search Q`, `--ready`, `--tangled`, `--needs`, `--run-on` (a body whose lines join into one paragraph), `--parent-of <id>` (its descendants); AND across flags. `--all` includes dead yaks.

```sh
yaks list --type bug --priority 1
yaks next --tag auth
yaks bulk --parent-of yak-c3d4 --add-tag spike --commit
```

**Large farms.** Don't summarize yak files through `ls .yaks/... | xargs`: past a few thousand paths `xargs` splits the list and a total like `xargs wc -l | tail -1` is silently wrong, and an absolute glob fails with "argument list too long". List with `yaks list --json` or `yaks path <filters>` (one path per line); to total, feed one stream to one counter (`yaks path --status shorn | xargs cat | wc -l`, never `xargs wc` or `find -exec wc {} +`, which print a total per batch).

## Task format

Each yak is one file, `.yaks/<herd>/yaks/<id>.md`, at the same path for life (a state move edits `status:` in place):

```markdown
---
id: yak-a1b2              # flat {prefix}-{4hex}; never encodes hierarchy
title: Fix the login crash
status: shaving           # hairy | shaving | shorn | dead
type: bug
priority: 2
created: "2026-02-16T10:00:00Z"
updated: "2026-02-16T10:30:00Z"
parent: yak-c3d4          # optional; the only source of hierarchy
depends_on: [yak-e5f6]
tags: [auth]
source: https://jira.example.com/browse/PROJ-123   # optional external issue
prs:                      # optional: the yak's open pull requests
- https://github.com/org/repo/pull/42
---

Details go here.
```

Other fields are allowed (`due:`, `every:`, `commit:`, ...): set them with `yaks update <id> --set key=value` rather than by hand; they round-trip untouched and appear under `extra` in `show --json`.

### Due dates and recurring yaks

`yaks create`/`update --due 2026-11-15` sets a deadline (a date or an RFC 3339
datetime); a hairy or shaving yak past it shows `overdue` in `list`/`next`, and
`yaks list --due [--before DATE]` lists the dated yaks soonest first. `--every
P1W` (an ISO 8601 duration: `P1D`, `P2W`, `P1M`, `P1Y`; no words) makes a yak
recur: shear it as usual, and `yaks regrow-due` reopens it (back to hairy, `due:`
moved to the next deadline, a `kind: regrow` note, `regrown:` stamped) once the
next occurrence has arrived, one regrow however many were missed. Nothing runs
by itself: cron, an agent harness or the TUI calls `regrow-due`, which is
idempotent. The yak stays one yak, so each round's shear note and links
accumulate in it. Don't hand-edit `regrown:`.

Don't parse ids for hierarchy (old dotted ids like `yak-a1b2.1` are opaque). `.yaks/config.yaml` sets the default `herd`, `default_type`, `default_priority` and a `verify:` map (tag → command, plus `default`); a `herds:` map overrides those per herd, one level deep; `min_version:` makes an older yaks warn that it should upgrade. A farm on an older schema is refused by every command but `yaks migrate`, and one on a newer schema by every command.
