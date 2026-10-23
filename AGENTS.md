# AGENTS.md — yaks

yaks is a filesystem-native task tracker: a single self-contained Rust binary.
Tasks are markdown files with YAML frontmatter under a `.yaks/` directory; a
task's status is a frontmatter field; each yak is one file, `.yaks/<herd>/yaks/<id>.md`.

## Invariants

- **Files are authoritative.** Status is a frontmatter field (`status: hairy|
  shaving|shorn|dead`) and each yak is one file at the stable path
  `.yaks/<herd>/yaks/<id>.md` (attachments in `.yaks/<herd>/yaks/<id>/`); a status change edits the
  file in place. (A farm from before 0.0.16, with status directories, refuses
  to open; 0.0.16's `yaks migrate` converts it.) Parentage is a frontmatter `parent:`
  field; ids are flat and stable (any legacy dotted ids are inert — the `parent:`
  field is the only source of hierarchy).
- **Any index is derived.** If a lookup index is added, it must be a per-user,
  rebuildable cache — never committed, never a second source of truth.
- **Mutations hold the farm lock.** Any new `Farm` method that reads then
  writes must take `store::lock` first (`.yaks/.lock`, advisory, not
  re-entrant: lock once, at the outermost method). It keeps `.lock` in
  `.yaks/.gitignore`.
- **Don't change the on-disk format lightly.** The `.yaks/` layout is the
  contract; task files are meant to be readable, greppable, and diffable.

## Farms, herds, families

The vocabulary and the user-facing rules are in the `yaks` skill; this is where
they live in the code.

- **Herds:** per-herd config cascades one level, herd value else global
  (`store::Config::{default_type_for, default_priority_for, resolve_verify}`,
  `known_herds`). Add new per-herd settings through that cascade. A **family**
  is `view::FamilyScope`.
- **Discovery** (`store::discover`): `$YAKS_DIR`, else walk up to the git
  top-level for a `.yaks` entry (directory, symlink, or pointer file with `path:`
  and optional `herd:`), else the farm branch's worktree
  (`store::farm_branch_worktree`; `store::farm_branch(root)` says whether a root
  is one). `create` routes a herd as `--herd` > pointer `herd:` > config. A
  committed `.yaks/` beside a farm-branch worktree is a stale copy that
  `store::redirect_live` redirects with a warning. Tests use `discover_with`,
  never `set_var`.
- **This repo's farm is the branch `yaks`**, checked out at `yaks path`; `main`
  has no `.yaks/`, and no code commit ever contains it.

## Layout

A single Cargo package (the `yaks` binary). The root keeps a `[workspace]`
table only to exclude the local `.edtui` clone.

- `src/model.rs` — `Status`, `Task`.
- `src/store.rs` — `.yaks/` discovery + frontmatter parsing (hand-rolled fast path).
- `src/main.rs` — clap CLI + command dispatch.
- `src/farm.rs` — the core facade the CLI and TUI both call.
- `src/tui.rs` (+ `src/tui/`) — the interactive TUI. `tui.rs` is a thin root (the
  `App` struct, the `Focus`/`Overlay`/`PickAction`/`ConfirmAction` enums, the
  constructors, and shared form helpers/consts); behavior lives in focused
  submodules (split out under yaks-b1cc): `render` (all `render_*` + shared render
  helpers), `editor` (edtui glue), `drawer`/`create`/`fuzzy` (overlay forms +
  pickers), `viewmodel`/`detail_nav`/`actions`/`handlers` (the `impl App` split by
  concern, each an `impl App` block using `use super::*`), and `runtime`
  (`run`/event loop/terminal setup) — alongside the existing
  `detail`/`tree`/`content`/`markdown`/`view`/`views_store`/`cache` models +
  persistence. `src/tui/headless.rs` is a thin adapter implementing
  `toque::HeadlessApp` for `App`; `src/tui/docshots.rs` renders doc SVGs. The
  whole-frame integration tests + their insta snapshots are central in
  `src/tui/tests.rs` + `src/tui/snapshots/`; shared test helpers live in
  `src/tui/test_support.rs`.
- `toque` — library to drive any ratatui app headlessly (inject keys, capture
  LLM-/test-legible plain-text snapshots, and render frames to SVG for visual
  inspection). Extracted from yaks; now lives at
  <https://github.com/joelgwebber/toque> and is a git dependency (fix it
  there, then `cargo update -p toque`). `docs/research/tui-style-eval.md`
  archives the now-retired text style-encoding research.

## Build

```sh
cargo build --release
cargo test
```

## Verifying changes

Verify against the real artifact via the project's lever, not "tests pass"
alone. A TUI change is verified by driving `toque` to the relevant state and
LOOKING at the rendered frame (and via `cargo test -p yaks docshots --
--ignored`, which renders color SVG frames to `docs/assets/`, plus the `insta`
snapshots in `src/tui.rs`). If a change has no lever to see its effect,
`yaks ask` rather than declare it done on faith.

Don't keep that frame to yourself: `yaks attach` the rendered frame (or a
headless text serialization) to the yak as evidence, so a reviewer sees what you
saw — the shipped `.agents/skills/yaks` "Evidence before you shear" rule, applied here.
For a durable UI state, prefer a `docshots` scene + a `docs/` embed over a
one-off frame.

To actually *look* at an SVG frame, rasterize it to PNG with headless Chrome
(recipe in `docs/tui.md`): that is the faithful lever. The TUI's image viewer
also rasterizes an attached SVG itself (toque's `png` feature, embedded fonts,
so no system fonts; box-drawing corners are blank until the font subset is
widened), which is enough to check an attachment shows up.

**Hand the human a build to try.** After landing any significant change, the
main (coordinating) thread installs a fresh release build to `~/go/bin/yaks`:
Joel's pre-release test binary, a temporary override on his `PATH` (where `go`
puts binaries too) for trying changes before a release. That is the **only**
place a test build goes: never `~/bin`, `~/.local/bin` or anywhere else, so
there is exactly one override to keep current. Overwrite it; no backup:

```sh
cargo build --release && mkdir -p ~/go/bin && install -m 755 target/release/yaks ~/go/bin/yaks
```

Workers never do this (their clones are not what lands); the coordinator does it
once their work is merged and the gate is green, and says so in its report. A
running `yaks tui` keeps the old binary until it is restarted, so say that too
when the change is in the TUI.

## Keeping docs current

A user-facing change is not done until its docs **and its own in-app help** are.
When you add or change a CLI command or flag, a TUI key or behavior, or a
workflow, update every surface that describes it, in the **same** change — never
a follow-up:

- `docs/` (`docs/cli.md`, `docs/tui.md`, `docs/README.md`) and `README.md`.
- the bundled skills (`.agents/skills/yaks`, `.agents/skills/yaks-tracker`) and
  the opt-in `coordination` group beside them (`.agents/skills/yaks-working`,
  `.agents/skills/yaks-coordinating`).
- for a CLI change: the clap `--help` text — the `///` doc comments and
  `#[arg(...)]`/`#[command(...)]` help on the command/flag in `src/main.rs`.
- for a TUI key/behavior: the `?` help overlay (`help_content` in
  `src/tui/render.rs`) **and**, if the key is common enough to belong there, the
  compact bottom help bar (`help_hint`). A new key that lands in the handler but
  not in `help_content` is invisible to users (this is how yaks-71d1's `H`
  shipped half-done).

**Careful with the skills source.** `.agents/skills/` holds this repo's skills
as real files, the only copy (no symlinks: Delta skips symlinked skill dirs).
The explicit `BUNDLED` list in `src/skills.rs`, not the directory, decides what
is embedded and installed (`RETIRED` lists skills it removes); `cargo test -p
yaks skills` guards it. `yaks skills install` refuses to write into a yaks
checkout's `.agents/skills/`, even through a `~/.agents/skills` symlink
(yaks-d8e9). If `git status` ever shows an unexplained `metadata:` stamp in
`.agents/skills/`, `git checkout -- .agents/skills/`.

The other surfaces have no gate, so docs/help parity is part of the change's
evidence: grep for the old name or key across `docs/`, `.agents/skills/`,
`README.md` and `src/`.

## Releasing

`.github/workflows/release.yml` builds the 5-platform binaries and publishes the
`@j15r/yaks` npm packages on a `vX.Y.Z` tag (dry-run on manual
dispatch). See `RELEASING.md`.

## Task tracking

This project tracks its own work with yaks; the `yaks` skill is the workflow.
Repo specifics: never code without a shaving yak; check existing yaks before
creating one; run `yaks next` when unsure what's next. The coordinator commits
the farm (`yaks commit`); publishing it (`yaks sync`) is Joel's call.

## Delegating to cheaper models

Fan well-scoped yaks out to subagents on a cheaper model (e.g.
`claude-sonnet-5-5`) and keep design, prose, review and every landing in the
coordinating thread; the `yaks-coordinating` skill is the workflow. In yaks-a3d2
six Sonnet workers landed cleanly, but the coordinator's own run of each command
found two bugs their tests missed: the review is part of the job.

- **Delegate** a yak with one outcome, a file/function scope and a test that can
  fail first. **Keep** design passes, skill and prose rewrites, and shared
  helpers several workers build on (land those first as a prep commit).
- **Brief extras** beyond `yaks brief`: build and use your own binary; the full
  gate; the docs-and-help rule; which `src/main.rs` regions each parallel worker
  owns; the `cargo fmt` caveat below.
- **Cold-read prose with a cheap model** after a skills change: a scout that
  reads only the skill files and answers scenario questions finds the gaps.
  Trust it to find them, not to pick the fix.
- **`cargo fmt` reformats files nobody touched** (`src/actor.rs`,
  `src/tui/docshots.rs`, `src/tui/tests.rs`): run `cargo fmt -- <your files>`
  and `git checkout` those three before committing.
