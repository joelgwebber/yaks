# Skills & workflows

Skills are prose guidance that rides *on top of* the tools — the least direction
that measurably changes an agent's behavior. yaks keeps the tools unopinionated
and puts methodology in skills, so different working styles fit the same core.

Two skills ship by default and install via `yaks skills install`; the workflow
skills for running several agents over one farm ship as the opt-in
`coordination` group (`yaks skills install --with coordination`).

## Where skills install

| Flags | Destination |
|---|---|
| `--dir <path>` | exactly that directory (wins over everything) |
| `--user` | `~/.agents/skills` |
| *(none)*, inside a git repo | `./.agents/skills` in the **git top-level** of the cwd — project-local, so agents working in the project find the skills and you can commit them |
| *(none)*, outside a git repo | `~/.agents/skills` (and it says so) |

`install` prints the destination it chose. In a yaks checkout the project-local
default *is* the skills source, so the source guard (below) refuses and names
`--user` / `--dir` as the way out. `yaks skills status` uses the same default
directory and flags.

This repo's own skills live in `.agents/skills/<name>/` — real files,
the single source of truth (no `skills/` directory, no symlinks; agent harnesses
such as Delta discover project skills there but skip symlinked skill dirs). What
ships is decided by the **explicit `BUNDLED` list** in `src/skills.rs`, *not* by
what sits in that directory. The list names each skill **and its files** (a skill
is no longer just a `SKILL.md`: `yaks-coordinating-delta` carries the executable
`land.sh`, installed with mode 0755), and a test fails if a bundled skill's
directory holds a file the list omits. The default set is `yaks` and
`yaks-tracker`; the `coordination` group adds `yaks-coordinating`,
`yaks-coordinating-team`, `-private`, `-worktrees`, `-delta` and `yaks-working`.
Any other project-local skill placed in `.agents/skills/` is never embedded or
installed unless someone adds it to that list on purpose.

## Installing, updating, and staying honest

The bundled `SKILL.md` files are baked into the binary, so the shipped `yaks`
always carries the skill matching its version. Installed copies get a
**provenance stamp** in their frontmatter, under the spec's `metadata:` field:

```yaml
metadata:
  yaks-version: "0.0.9"
  yaks-digest: "a64a55994a8ee572"
  yaks-files: "land.sh=0f3c…"   # only for a skill that ships other files
```

The stamp lives in `SKILL.md`. A skill's other files (a script) carry no stamp
of their own; the stamp records their digests as written, so an untouched script
from an older yaks upgrades while one you edited is `modified` and kept. A skill
is installed, upgraded and protected as a unit, and its verdict is its most
demanding file's (`status` names the file when it isn't `SKILL.md`).

That stamp is what makes an installed skill *identifiable* rather than an
anonymous copy, so yaks can tell "the tool moved on" from "a human edited
this" instead of just clobbering. `yaks skills status` reports the verdict:

| State | Meaning | What yaks does |
|---|---|---|
| `current` | Identical to this binary's copy | nothing |
| `stale` | Untouched since install, and this yaks is newer | upgrades it |
| `adoptable` | Unstamped but identical to ours (a pre-stamp install) | adopts it (stamps it) |
| `held` | Untouched, but installed by a yaks **not older** than this one | nothing — refuses to downgrade |
| `modified` | Edited after install | nothing without `--force` |
| `unmanaged` | Unstamped and not ours — hand-written or another tool's | nothing without `--force` |
| `source` | Resolves onto a yaks checkout's own `.agents/skills/` (often via a symlink) | nothing, **even with `--force`** |

**Ordinary `yaks` commands top this up automatically.** The common failure is an
agent running against a stale skill and nobody remembering to re-run the
installer, so a normal invocation installs what's absent and upgrades what's
cleanly `stale`. It deliberately never touches `modified`, `unmanaged`, `held`,
or `source`, only ever writes `~/.agents/skills` (never a project-local
`.agents/skills`: those are files in your working tree, so `yaks skills status`
shows them `stale` and you re-run `yaks skills install`), only the default set
(never the `coordination` group), and writes atomically so parallel agents can't
tear a file. Set `YAKS_SKILLS_AUTOSYNC=0` to disable it for CI or sandboxes.
`yaks doctor` reports anything left needing a decision.

Because auto-upgrade keys on the **version**, a skills-only edit doesn't reach
users until the next release — two binaries at the same version never fight over
an install (that's the `held` rule). When iterating locally, use
`yaks skills install --force` (in a git repo add `--user` to target
`~/.agents/skills`).

> **If you develop yaks:** this repo's `.agents/skills/` *is* the source of the
> skills (and what agents working in the repo load). Symlinking
> `~/.agents/skills/yaks` at `.agents/skills/yaks` is a handy way to run the live
> skill everywhere — but it means the "installed" skill *is* the source. yaks
> detects both cases (`source`) and refuses to write into `.agents/skills/`
> directly or through a link, with or without `--force`. Without that guard, an
> older `yaks` on your `PATH` would silently revert your edits to its baked-in
> copy, which looks exactly like an authored change in `git status`.

| Skill | Installed by | Use it when… |
|---|---|---|
| **`yaks`** | default | Managing tasks in a `.yaks/` farm — the commands, the workflow, solo vs team/private modes. |
| **`yaks-tracker`** | default | Relating yaks to external issue trackers (Jira/Linear/GitHub) as a one-way roll-up projection. |
| **`yaks-working`** | `--with coordination` | Taking one yak from hairy to shorn with a legible trail. |
| **`yaks-coordinating`** (+ `-team`, `-private`, `-worktrees`, `-delta`) | `--with coordination` | Coordinating a shared farm across parallel agents and humans. |

## The design test (what is a tool vs a skill)

If a thing is expressible as a CLI query or mutation over the task files, it
belongs in the **tool**. If it assumes sub-agents, worktrees, or a particular
orchestration, it belongs in a **skill**. The files and the CLI are the
contract; coordination is prose on top. A convention graduates from skill to
tool (a flag, a field, a `doctor` check) once it has earned its place.

## `yaks-working` — one yak, honestly

The per-yak discipline the other skills lean on:

- **Notes first.** Re-read a yak (freshest note first) before starting — a human
  or another agent may have added feedback or moved it.
- **One writer per yak.** Split shared work into child yaks so each has a single
  owner and a clean file scope.
- **Evidence before shear.** Don't move a yak to `shorn` without a note that
  records what was done and how it was verified (`doctor --strict` checks this).
- **Ask, don't guess.** On a decision that needs a human, `yaks ask` and hand
  back — never clear your own `needs` block.

## `yaks-coordinating` — careful parallelism

Conventions for running a small number (2–4) of agents over one farm reliably.
The load-bearing ideas, all validated by dogfooding:

- **Run shape: claim → fan out → merge → reconcile.** The yakherd makes one
  commit that moves the batch's yaks to `shaving` with per-yak assignment notes
  ("licks the cookie", so `main`'s `shaving` set reflects what's in flight),
  cuts a git worktree per lane, spawns one worker each (workers skip the shave —
  their yak is already shaving), squash-merges each lane back (the yak id in the
  message), and regrows any yak left stranded in `shaving`.
- **Disjoint scope is about *types*, not just files.** A change to a shared type
  breaks the other lane at merge even across disjoint files; put it in a
  yakherd prep-commit first, or keep it in one lane.
- **Human-in-the-loop routes through the yakherd.** Workers `ask` and hand
  back; the human answers on `main` (`yaks inbox`); the next spawn starts fresh
  from `main`. No live cross-worktree feedback.
- **Human-driven interactive lane.** A thorny, iterative design problem can run
  as its own worktree lane *in parallel* without a fan-out: a peer lane the human
  drives, landed through the yakherd (or the human is the yakherd). It
  starts yak-less, emits code or a fresh farm, and — because the human opens the
  worktree as the harness root — dodges the spawned-worker file-tool pitfall by
  construction.
- **Recovery.** A worker's work lives in its worktree, so a lost session isn't
  lost work — recover from the worktree rather than restarting.
- **Integrity.** `yaks doctor` after a batch catches merge damage; across many
  runs the disjoint-leaf model has kept farms corruption-free.

## Workflows at a glance

- **Solo:** `create` → `next` → `shave` → `update --note` → `shorn`. The farm is
  durable memory; notes are how you (or an agent) remember across sessions. Keep
  it private with one of the hiding options in [README](README.md#solo-vs-team-mode).
- **Team:** commit `.yaks/` with the code. Yak surgery lands alongside the change
  it describes; `yaks commits <id>` recovers provenance from git; ids may appear
  in commit messages but never in PR titles or external trackers (`scan-ids`
  guards that).
- **Parallel agents:** the yakherd drives the claim → fan-out → squash →
  reconcile shape above, with `ask`/`answer`/`inbox` for human decisions and
  `doctor` for integrity.
- **Interactive lane alongside parallel work:** a human-driven worktree lane for
  design problems that aren't fire-and-forget — same worktree mechanics, landed
  through the yakherd; in private mode its emitted yaks are shared live, so
  only the code has to land.
