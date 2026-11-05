<!-- yass:begin (managed by `yass init`; edit outside these markers) -->
## Changes (YASS)
Planned work lives in the yass folder's `changes/`, one dated folder per change: `yass/` in the repo, or wherever `yass.yaml` points (`yass root` prints it; see its README.md). CLI: `yass`, on your PATH. Playbooks: the `yass-*` skills, in `.agents/skills/` or your user skills folder.
- `yass status` shows what's in flight. If `yass root` is outside this repo, plans and code are committed separately: see "Plans in another folder" in `yass-work`. Work only on the change you're pointed at (unless you've been put in charge of choosing), and resume it from the last **Next:** in its Log. `not started` means planned but not built yet; its plan may still need approval.
- As you work, mark boxes `[/]` when you start them and `[x]` when they're verified, and append to `## Log` and `## Decisions` alongside the code (each decision dated: `- <decision> - <why> (<who>, <YYYY-MM-DD>)`).
- Intent (`prd.md`, `design.md`, plan text, a Goal, acceptance criteria, including dropping one with `[-]`) never changes in the same commit as code (with squash merges, the same pull request), so decisions can be reviewed apart from execution. If the plan is wrong mid-build, put the fix on its own branch underneath the code and rebase onto it (see `yass-work`), or stop and say so.
- Don't commit, branch, rebase, push or open a pull request unless you've been given that latitude; otherwise show the human the git commands.
- When `yass status` notes that the `yass` binary and this repo's YASS files are out of step, or someone asks to update YASS: `yass-update`.
- A plan from plan mode can become a change: `yass-shape` for large work, `yass-plan` or `yass-work` for small.
- Never edit the archive (`archive/` in the yass folder); only `yass evict` removes from it. Follow-up work is a new change with `follows: <archived folder>`.
- Finished means every box is checked; then `yass archive <change>`, in its own commit, apart from code (with squash merges, a pull request without code).
<!-- yass:end -->
