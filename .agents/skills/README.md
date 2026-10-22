# .agents/skills — this repo's skills (the only copy)

These are the real `SKILL.md` files for this repo: no `skills/` directory, no
symlinks. Agent harnesses (Delta, and anything else that scans
`.agents/skills/<name>/SKILL.md`) load them from here, and the binary embeds the
shipped ones from here.

- **Repo-local, never shipped:** `land` (see the warning below).
- **Shipped by default:** `yaks` and `yaks-tracker`. The explicit `BUNDLED` list in
  `src/skills.rs` — skills and, per skill, files; not the contents of this
  directory — decides what is embedded in the binary and installed by
  `yaks skills install`.
- **Shipped, opt-in** (`yaks skills install --with coordination`): the workflow
  skills below, which we also use to develop yaks itself. Nothing here that
  isn't in `BUNDLED` is ever embedded or installed; the same goes for any other
  project-local skill dropped into this directory. A skill's extra files (none
  today; the landing script became `yaks land`) must be listed in `BUNDLED` too;
  a test fails if one is missing.
  - `yaks-working` is for a worker taking ONE yak from hairy to shorn.
  - `yaks-coordinating` is for a yakherd handing yaks to other agents, with a
    section each for Delta threads and plain git worktrees. (It replaced four
    companion skills, `-team`, `-private`, `-worktrees` and `-delta`; `yaks skills
    install` removes untouched installed copies of those.)
  A guard test (`repo_skills_are_well_formed` in `src/skills.rs`) keeps the names,
  descriptions and cross-references consistent.

> [!WARNING]
> **`land` is NOT a yaks skill and must NEVER be bundled.** `land/` is this
> repo's own Delta "Land Changes" action (Delta insists on a skill named `land`):
> it hard-codes this repo's gate, `main` and farm branch, and means nothing in
> anyone else's project. It stays out of `BUNDLED` in `src/skills.rs` (so it is
> never embedded or installed by `yaks skills install`), and the test
> `land_is_repo_local_and_never_bundled` fails if that changes.

`yaks skills install` refuses to write into this directory (the source-tree
guard, yaks-d8e9), with or without `--force`.

Keep each skill minimal. The goal is the least guidance that measurably changes
behavior — judged by whether it cuts turns and yak-herding drudgery — not a
heavyweight framework. Promote a convention into the tool (a CLI flag, a lint, a
field) once it has earned its place; until then it lives here as prose.

## Changing these skills

A skill is only as good as a cold reader's reading of it, and a flaw can keep a skill from loading at all. For every change to a skill's text or description:

1. `cargo test -p yaks skills` (the guards: frontmatter quoting, the explicit `BUNDLED` file lists, source protection; the landing scenarios are `yaks land`'s own tests, `cargo test -p yaks land::`).
2. Load each changed skill through the real `skill` tool, with the worktree argument. A description containing `: ` must be quoted, or the skill will not load.
3. Cold-read it: a fresh scout that sees only the skill files answers scenario questions that probe what you changed. The reusable brief is attached to yaks-5c9f (`yaks attachment yaks-5c9f cold-read-brief.md`). Fix the real gaps, then re-run only the questions it missed with a second scout.
