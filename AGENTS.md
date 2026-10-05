# AGENTS.md — yaks

yaks is a filesystem-native task tracker: a single self-contained Rust binary.
Tasks are markdown files with YAML frontmatter under a `.yaks/` directory; a
task's status is implicit in which subdirectory it lives in.

## Invariants

- **Files are authoritative.** Status is implicit from the directory a task file
  lives in (`hairy/ shaving/ shorn/ dead/`). Parentage is a frontmatter `parent:`
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

- A **farm** is one `.yaks/` directory (what `Farm` opens). A **herd** is a group
  of yaks sharing an id prefix within a farm; a farm may hold several (config
  `herds:` map; `create --herd`), so one private farm can track several
  projects. A **family** is a parent yak + its descendants (the TUI tree /
  `view::FamilyScope`).
- **Per-herd config cascades one level:** `default_type` / `default_priority` /
  `verify` resolve herd-value-else-global (`store::Config::{default_type_for,
  default_priority_for, resolve_verify}`, `known_herds`). Add new per-herd
  settings through that same cascade.
- **A repo can point at an out-of-tree farm** via a `.yaks` pointer file
  (`path:` + optional `herd:`) or a symlink, resolved in `store::discover`, so
  several repos share one farm. `create` routes a new yak's
  herd as: explicit `--herd` > pointer `herd:` > config default.
- **Discovery is bounded by the git top-level.** `store::discover` uses
  `$YAKS_DIR` first (the `.yaks/` dir, a dir containing one, or a pointer file),
  else walks up from the cwd and stops at the first directory with a `.git`
  unless it also has a `.yaks` entry; outside a git repo it walks to the
  filesystem root. So a nested checkout (Delta's `<repo>/.delta/worktrees/…`, an
  in-tree `git worktree`) never reads another checkout's farm by accident; it
  needs its own pointer or `YAKS_DIR`. Tests use `discover_with` (the env var
  passed in), never `set_var`.
- **Consolidate** separate farms with `Farm::merge` (`yaks merge`), which is
  collision-checked and non-destructive.

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
(recipe in `docs/tui.md`) — yaks stays out of rasterization, and a pure-Rust
rasterizer botches the color emoji, so a browser engine is the lever.

## Keeping docs current

A user-facing change is not done until its docs **and its own in-app help** are.
When you add or change a CLI command or flag, a TUI key or behavior, or a
workflow, update every surface that describes it, in the **same** change — never
a follow-up:

- `docs/` (`docs/cli.md`, `docs/tui.md`, `docs/README.md`) and `README.md`.
- the bundled skills (`.agents/skills/yaks`, `.agents/skills/yaks-tracker`) and
  the opt-in `coordination` group beside them (`.agents/skills/yaks-working`,
  `.agents/skills/yaks-coordinating*`).
- for a CLI change: the clap `--help` text — the `///` doc comments and
  `#[arg(...)]`/`#[command(...)]` help on the command/flag in `src/main.rs`.
- for a TUI key/behavior: the `?` help overlay (`help_content` in
  `src/tui/render.rs`) **and**, if the key is common enough to belong there, the
  compact bottom help bar (`help_hint`). A new key that lands in the handler but
  not in `help_content` is invisible to users (this is how yaks-71d1's `H`
  shipped half-done).

**Careful with the skills source.** This repo's skills live in `.agents/skills/`
— real files, the only copy (no `skills/` dir, no symlinks: Delta discovers
project skills there but skips symlinked skill dirs; yaks-0576).
The skills in `BUNDLED` are embedded via `include_str!`. `BUNDLED` in
`src/skills.rs` is the explicit list — skills **and, per skill, files** (a skill
like `yaks-coordinating-delta` carries `land.sh`) — and not the directory
contents; it alone decides what is embedded and installed. The default set is
`yaks` + `yaks-tracker`; `yaks-working` and `yaks-coordinating*` ship only as
the opt-in `coordination` group (`yaks skills install --with coordination`), and
any other project-local skill is never shipped. A test fails if a bundled
skill's directory holds a file the list omits. `yaks skills install` writes
project-local (`./.agents/skills` in the git top-level) by default — in this
checkout that *is* the source, so it refuses; use `--user` or `--dir`.
`~/.agents/skills/<name>`
is sometimes a symlink back at the source — so a `yaks skills install` (even
with no `--dir`) used to write *through the link* and revert your edits to the
binary's baked-in copy, looking exactly like an authored change in `git status`.
That's yaks-d8e9; it bit twice. The installer now refuses any target that
resolves into a yaks checkout's `.agents/skills/` (or the pre-0576 `skills/`;
`source` state, not overridable by `--force`), the test suite sets
`YAKS_SKILLS_AUTOSYNC=0` so `cargo test` can't touch your real skills, and a test
asserts the embedded source is never itself stamped. If `git status` ever shows
an unexplained `metadata:` stamp in `.agents/skills/`, that's the symptom —
`git checkout -- .agents/skills/`.

The bundled skills are embedded in the binary (`src/skills.rs`), so
`cargo test -p yaks skills` guards them; the other surfaces have no gate, so
treat docs/help↔reality parity as part of the change's evidence — grep for the
old name/key across `docs/`, `.agents/skills/`, `README.md`, and `src/` and confirm every
description matches the shipped behavior.

## Releasing

`.github/workflows/release.yml` builds the 5-platform binaries and publishes the
`@j15r/yaks` npm packages on a `vX.Y.Z` tag (dry-run on manual
dispatch). See `RELEASING.md`.

## Task tracking

This project uses yaks to track its own work. The yaks skill has the full
workflow.

1. Never start coding without a shaving yak. No exceptions.
2. Shear a yak as soon as its work is done, and commit the shorn yak file
   alongside the code that completed it (`.yaks/` is committed here — team mode).
3. Check existing yaks before creating new ones.
4. Append progress notes to yak descriptions as you work.
5. When unsure what's next, run `yaks next` — don't freelance.
