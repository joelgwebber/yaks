# yaks

**yaks** is a task tracker made of markdown files: a single self-contained Rust
binary. Each task (a "yak") is one file with YAML frontmatter; its **status is a
frontmatter field**. There is no database and no separate state file. The
files live on the repo's **farm branch** (`yaks`, an orphan branch holding only
`.yaks/`), checked out once per machine and shared by every checkout.

```sh
.yaks/            # the farm branch's worktree: ~/.local/share/yaks/farms/<repo>-<hash>/
  config.yaml     # farm settings
  schema          # the layout version (6; `yaks migrate` upgrades a 4 or 5)
  yaks/           # one directory per herd (the id prefix)
    yaks/
      yaks-1a2b.md  # one file per yak; frontmatter status: hairy | shaving | shorn | dead
      yaks-1a2b/    # its attachments, beside it
    docs/         # reserved for the herd's non-yak documents (not built yet)
  docs/           # reserved for farm-wide documents (not built yet)
```

## The model

- **Files are authoritative; any index is derived.** Status is a frontmatter
  `status:` field, and a yak's file never moves. Hierarchy is a frontmatter `parent:` field. Ids are flat and stable.
- **Tiny state machine.** Four statuses, five transitions (`shave`, `shorn`,
  `regrow`, `slaughter`, `revive`). Everything else — priority, type, labels,
  dependencies, a `needs` block, notes — is metadata that pushes nuance out of
  the state machine and into fields you can filter on.
- **Notes are the work-trail.** Each yak accumulates timestamped, optionally
  attributed notes (`▸ <ts> [actor]`, then a blank line, then the text); every status move adds a
  `moved: <from> -> <to>` entry in the same shape. The notes are the durable
  record of what happened and why; git history is the record of what *changed*.

## Quick start

```sh
yaks init                      # the farm on the `yaks` branch + the agent skills
yaks create 'Wire up auth'     # new hairy yak (prints its id)
yaks next                      # what's ready to work (deps met)
yaks shave [id]                # start it (→ shaving)
yaks update [id] --note '...'  # record progress
yaks shorn [id]                # finish it (→ shorn)
yaks commit                    # commit the farm on its branch
yaks sync                      # pull + push it
yaks tui                       # or drive it all interactively
```

## Where to go next

- **[CLI reference](cli.md)** — every command, grouped by what it's for.
- **[TUI guide](tui.md)** — the interactive terminal UI and its keys.
- **[Skills & workflows](skills.md)** — the agent skills that ride on top of the
  tools, and the workflows they encode (solo, and careful multi-agent
  parallelism).

## Where the farm lives

One farm per repo per machine, on the farm branch. Code branches and PRs
never carry yak files.

```sh
yaks init                              # start it (or add the worktree for an existing `yaks` branch)
yaks path                              # where it is
yaks commit; yaks sync                 # commit it; pull + push it
yaks private                           # keep it on this machine
yaks init --farm ../api --herd web     # use ../api's farm instead (git config yaks.farm, yaks.herd)
YAKS_DIR=/some/.yaks yaks list         # one-off: any farm directory
```

Discovery: `YAKS_DIR`, else `git config yaks.farm` (another repo's farm
branch), else this repo's farm branch. Both git settings are read from a Delta
clone's host repo too, so agent clones need no setup. See the **yaks** skill
for the habits that go with it.

**Concurrent writers.** Every command that changes the farm holds an exclusive
advisory lock on `.yaks/.lock` for its whole read-modify-write, so parallel
processes (several agents, a TUI and a CLI) never drop each other's edits, even
on one yak. Readers don't take it. The first write (and `yaks init`) adds a
`.lock` line to `.yaks/.gitignore`, creating that file if needed and keeping any
other lines, so the lock is never shown or committed.

## Farms, herds, and families

A **farm** is one `.yaks/` store; a **herd** is the yaks sharing an id prefix
within it (`yaks create --herd web`); a **family** is a parent yak and its
descendants. Several repos share one farm by pointing at it with
`yaks init --farm` (each repo routes new yaks into its own `yaks.herd`); fold
an old farm in with `yaks merge`. See [cli.md](cli.md#which-farm-a-command-uses).
