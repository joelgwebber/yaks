# yaks-9714 audit: dropping the `.yaks/` root, and changing stored link forms

Audited at `main` e4d0ff5a (v0.0.36). All line numbers are in that tree. I checked
behaviour against the live farm (`yaks peek`, `yaks doctor`) and a scratch git repo,
where noted. No code was changed.

## 1. Places that depend on `.yaks/`, or on the farm root being `<worktree>/.yaks`

There are two kinds of dependency, and each kind needs its own fix:

- **(P) parent-is-top.** The code takes the farm root `root` and uses `root.parent()` as
  the git worktree top, or checks `top == root.parent()`. If the farm root *is* the
  worktree top, `parent()` gives the farms directory, so the farm stops being treated
  as a farm branch. Nothing errors.
- **(S) string prefix.** The code finds or builds repo-relative paths with a literal
  `.yaks/` component, either in git output (paths relative to the repo top) or in
  pathspecs and attribute globs.

### 1a. Discovery and identity (core: everything else depends on these)

| Where | Kind | What it does today | If the farm root were the branch root |
|---|---|---|---|
| `src/store.rs:377-391` `farm_branch_worktree` | S | `wt.join(".yaks")` is the root. If missing, it bails with "has no .yaks/ there; restore it (`git checkout -- .yaks`)" | Return `wt` when `wt/config.yaml` or `wt/schema` exists, else `wt/.yaks` (an old layout, sent to the migration refusal). **This is where an old binary fails on a new branch** (see §5). |
| `src/store.rs:335-347` `shared_farm_worktree` | S | Requires `home.join(".yaks").is_dir()` before it adopts a shared farm home | Same dual test. Today a new-layout home would be silently ignored, and bootstrap would then try to `git worktree add` over it and fail ("already exists"). |
| `src/store.rs:522-541` `farm_git` | **P** | `dir = root.parent()`, and returns `Some` only if `top == dir` and HEAD is the configured branch | **This is the switch every other check reads.** With root == top, `farm_git` returns `None`, so `farm_branch()` returns `None`, and then per-change commits (`autocommit.rs:112`), `changes::read` (`changes.rs:88`), `pending_operation` (`changes.rs:237`), preflight, sync, weight, brief, `code_base` and `migration_refusal` all behave as if the farm were a `YAKS_DIR` farm. Fix: compute `top` from `root` and accept `top == root` **or** `top == root.parent()`. `FarmGit` gains `prefix` (`""` or `".yaks/"`) and every (S) site uses it. |
| `src/store.rs:439-441` `farm_branch` | P | Thin wrapper over `farm_git` | Fixed by the line above. |
| `src/store.rs:543-549` `from_yaks_dir`, `:567-579` `resolve_farm_root` | S | `$YAKS_DIR` names the farm dir or a dir containing `.yaks` | Keep for scratch and legacy farms. A `YAKS_DIR` that points at a new-layout worktree already works, because `config.yaml` is at its root. |
| `src/farm.rs:3662-3676` `resolve_farm_root` (for `yaks merge <source>`), message `src/main.rs:2195` | S | The same rule, plus "has any yaks" | Keep. Reword the messages. |
| `src/store.rs:2659-2694` `migration_refusal` | S | Reads `git show {upstream}:.yaks/schema` to tell "behind" from "unmigrated" | Read `{upstream}:schema`, falling back to `{upstream}:.yaks/schema`. Without this, a machine that is behind a migrated upstream is told to migrate a second time. |
| `src/store.rs:1486-1505` `store::init` | — | `if root.exists() { return AlreadyExists }` | **Breaks init in the new layout**, because the root (the worktree) always exists, so config and schema would never be written. Test for `config.yaml`/`schema` instead. |
| `src/store.rs:1551,1598-1634` `.lock`, `ensure_lock_ignored` | — | `root/.lock`, and adds `.lock` to `root/.gitignore` | Moves to the branch root unchanged (`/.lock`, `/.gitignore`). Fine. |
| `src/store.rs:2745` `schema_status`, `:1498` | — | `root/schema` marker | Becomes `/schema` at the branch root. Fine. |
| `src/store.rs:616-633` `herd_dirs`, `src/doks.rs:177-200` `doks_roots`, `src/migrate.rs:267`/`:344`/`:379` | — | Walk the root's top-level dirs, skipping dot-dirs and `RESERVED_HERD_NAMES` (`store.rs:588`) | Safe at the worktree root: `.git` is a file in a linked worktree and dot-entries are skipped. A future top-level non-farm *directory* would be read as a herd only if it contains `yaks/`. |
| `src/init.rs:286-294`, `:330-341` | S | `git cat-file -e {branch}:.yaks` decides whether an existing branch "is a farm" | Test `{branch}:schema` or `{branch}:config.yaml`, then fall back to `:.yaks`. Today `yaks init` refuses a new-layout branch as "not a farm". |
| `src/init.rs:380-398` | S | `root = worktree.join(".yaks")`, then the first commit | `root = worktree`. Depends on the `store::init` fix above. |
| `src/branchfarm.rs:53-165` `bootstrap` | — | Adds a worktree for the branch. Layout-agnostic | No change. |

### 1b. Users of the worktree top that call `root.parent()` (P)

| Where | Use | Fix |
|---|---|---|
| `src/status.rs:131-133` `sync_state` | `top = root.parent()` for `rev-list` | Use `farm_git(root).top`. |
| `src/sync.rs:85-99` `farm_worktree` | `top = root.parent()` (pull, rebase, push, `.gitattributes`) | Same. |
| `src/weight.rs:38-51` `farm_top` | `top = root.parent()` | Same. |
| `src/sheds.rs:226-229` `Freshness::read` | `top = root.parent()` | Same. |
| `src/tui/farmsync.rs:140`, `:216` | `has_remote`, `no_push` from `root.parent()` | Same. |
| `src/links.rs:856-876` `code_base` | `own = farm_root.parent()`: for a non-branch farm that is the code checkout. For a branch farm it compares `cwd`'s top with the farm worktree | Compare against `farm_git.top`. Keep `parent()` only for the `YAKS_DIR` in-tree legacy case. |
| `src/farm.rs:1847-1861` `code_repo` | `root.parent() == top` means "cwd is the farm worktree" | Use `farm_git.top`. |
| `src/autocommit.rs:390-396` (`undo`) | `rev-parse --show-toplevel` from root, then `ensure_registered(top)` | Already layout-agnostic. |
| `src/autocommit.rs:108-131` `snapshot` | Uses `farm.top` | Already agnostic, once `farm_git` returns `Some`. |

### 1c. Literal `.yaks/` in repo-relative paths, pathspecs and globs (S)

| Where | What | If root == top |
|---|---|---|
| `src/changes.rs:385-394` `farm_rel` | Strips everything up to the last `.yaks` component of a `git status` path. Falls back to the whole path | Works by accident (the fallback). Should use `FarmGit.prefix`. |
| `src/preflight.rs:169-177` `yak_id_of` | `rposition(".yaks")?`, so it returns `None` when there is no `.yaks` component | **Silent breakage.** Every yak change is classified as `other` (`changes.rs:327` `classify`). `yaks commit` messages read "farm config", `yaks status` shows no per-yak moves, and preflight's default verify scope (`preflight.rs:146-150`) is empty. Also used by `changes.rs:9`. |
| `src/commit.rs:216-247` `generate_message` | `split_once(".yaks/")` with a fallback | Works by fallback. Clean it up. |
| `src/weight.rs:90-110` `kind_of` | `strip_prefix(".yaks/")`, otherwise `"other"` | `yaks weight` puts everything in "other". |
| `src/weight.rs:308` `LFS_MARKER`, `:324-333` `lfs_rules` (`.yaks/**/*.{ext}`), `:396` `ls-files -- .yaks` | LFS rules on the branch's `.gitattributes` | Globs become `**/*.{ext}`. **Existing farms' `.gitattributes` must be rewritten by the migration**, otherwise new binaries are committed as plain blobs (the old patterns no longer match). |
| `src/mergedriver.rs:37` `ATTR_LINE = ".yaks/**/*.md merge=yaks"`, `:622-640` `ensure_registered` | The merge-driver rule | It becomes `*.md merge=yaks` (or `**/*.md`). `ensure_registered` only *appends* when the exact line is missing, so a migrated farm would carry both lines. The migration should replace the line. `is_yak_path` (`:500`) is suffix-based and already agnostic. |
| `src/sync.rs:251-255` | `git status --porcelain -- .yaks .gitattributes` is the "dirty" test before a rebase | With no `.yaks` the pathspec matches nothing, so the farm reads as clean, the rebase is attempted on a dirty tree, and git refuses (**hard failure** in `yaks sync`). Use `-- .` from the top. |
| `src/sync.rs:307-329` `commit_gitattributes` | Commits `/.gitattributes` | Unchanged (it is already at the branch root). |
| `src/land.rs:606-622` | Guard (yaks-7bfd): the worker commit touches `.yaks/…` while HEAD tracks no `.yaks` | Without `.yaks` it can never fire. A worker staging `yaks/yaks/x.md` would land. Needs a new rule: a path matching `store::path_owner` (`<herd>/yaks/<id>.md`) **and** the code repo has a farm branch. Watch for false positives on a code repo that legitimately has a `*/yaks/*.md`. |
| `src/sessions.rs:533-561` `yak_file_id`, `changed_paths` | `git show --relative` from root, exactly `<herd>/yaks/<id>.md` | Works in both layouts for new commits. For **pre-move commits** `--relative` from top yields `.yaks/h/yaks/x.md`, so `yaks session touched` misses them. Soft. |
| `src/check.rs:162-180` `last_change` | `git log -1 -- rel` with cwd root | Works for both layouts, but the dok's history stops at the move commit unless `--follow` is added, so `yaks check` staleness resets for every dok on migration day. |
| `src/links.rs:664-693` `read_at`, `:698` `pre_doks_path` | `git show {rev}:./{path}` with cwd root | **Every `@sha` pinned farm link to a pre-move commit breaks** (the file was at `.yaks/{path}`). Add a fallback next to `pre_doks_path`: try `{rev}:.yaks/{path}` (and `.yaks/wiki/...`). |
| `src/main.rs:2515-2551` | Output text: `Path::new(".yaks").join(...)` in `attach` and `rename-attachment` messages | Cosmetic. Print the farm-relative path. |
| `src/render.rs:23`, `src/tui/detail.rs:72`, `:718`, `src/tui/handlers.rs:123`, `:139`, `:1149`, `:1606`, `src/tui/mdrender.rs:518-525` | `strip_prefix(".yaks/")` on an attachment target (legacy `.yaks/<id>/x.png` links from schema ≤5) | Harmless in both layouts. Fold into the resolver (§3) as one legacy alias. |
| `src/brief.rs:167`, `:239-240` | Worker brief: "never create or stage a `.yaks/`", "nothing under `.yaks/`" | Reword ("never stage farm files; the farm is on its own branch"). |
| `src/tui/runtime.rs:32-36`, `:106-131` | Recursive watcher on `root` | With root == top it also watches `/.gitattributes` and `/.gitignore` (fine). The linked worktree's `.git` is a file, so git's own writes are not seen. No change needed. Fix the comment. |
| `src/tui/cache.rs:28`, `src/tui/views_store.rs:20`/`:42`, `src/tui/drafts.rs:189` | Per-user UI state, views, working set and **drafts** keyed by `slug(canonical root)` | The root path changes (`…/farm/.yaks` becomes `…/farm`), so each user **loses saved views, the working set and unsaved drafts** on upgrade. Soft, but drafts are user text. Migrate the cache dir, or key by the worktree. |
| Doc comments with `.yaks/` | `store.rs:3,28,41,111,331,370,1000,1048,1480,1614,2743`, `farm.rs:983,1000,1048,2057,2823,2908`, `model.rs:4,109`, `zone.rs:4`, `links.rs:826,853`, `tui/source.rs:3`, `tui/editor.rs:53`, `tui/drafts.rs:5`, `status.rs:73` | Wording only. |

### 1d. What sits at the branch root after the change, and the edge cases

- **Branch root contents:** `config.yaml`, `schema`, `.gitignore` (`.lock`; also the
  existing untracked `.yaks/.activity/`, a leftover of the logger removed in 3468c778,
  which has its own `.gitignore`), `.lock`, `.gitattributes` (already at the root
  today), `doks/`, `sessions/`, `sources/`, `schemas/`, `<herd>/yaks/`, `<herd>/doks/`.
  Today the branch tracks exactly one non-farm file, `.gitattributes` (checked:
  `git ls-files | grep -v '^\.yaks/'`). There is no `README` today. A root
  `README.md` would be the GitHub landing page and is harmless to every walker: it is
  not under a doks root, and `migrate::plan_layout` ignores a root `.md` whose
  frontmatter `id:` differs from its stem.
- **`yaks init --farm <checkout>` / `git config yaks.farm`** (`store.rs:177`): this
  routes to the other repo's farm branch worktree, so it inherits the
  `farm_branch_worktree` fix. No other change.
- **`$YAKS_DIR` and `Farm::at` scratch dirs:** these are plain directories, not
  branches. `farm_git` returns `None` for them and must keep doing so. Their root may
  stay a `.yaks/` dir (tests use `<tmp>/.yaks`, and `tests/fixtures/farm/.yaks` via
  `YAKS_DIR`, `tests/cli.rs:11-17`). For a `YAKS_DIR` farm inside a code checkout,
  `links::code_base` keeps the `parent()` rule. In-tree farm discovery from a checkout
  is already gone (yaks-a99f).

## 2. Link parsing, generation, rewriting and resolution

### 2a. The grammar and the farm graph: `src/links.rs`

- `parse` (`:102-184`) understands URLs/`mailto:`; `#anchor` (`Local`); `code:path[@rev]`;
  legacy `../path` and `../blob/<sha>/path` (only from yaks and attachments, `:126-138`);
  a bare yak id (+ `#<ts>` note anchor); `/farm/path[@rev][#a]`; `<id>/<file>`
  (Attachment); a bare sibling inside an attachment; and otherwise a path relative to
  the farm root (yak/cli) or the dok's directory (Dok). `..` anywhere gives `Upward`.
  **The grammar has no notion of directories or aliases.**
- `exists` (`:600-625`): checks `root.join(path).exists()`. `looks_like_file` (`:604`)
  skips any path whose last segment has no `.`, so `/doks/sources/2026-11-03-81364278`
  and `/sources/<id>` are **never reported** by `doctor` (confirmed: `yaks doctor`
  reports only the yaks-bede session link).
- `edges` (`:539-595`): yak bodies (From::Yak), markdown attachments (From::Attachment),
  doks and sessions (From::Dok). Sources are not scanned.
- `read_at`, `peek`, `code_base` (`:664-876`). There is **no `Resolved` type**, even though
  the module doc mentions one (`:25`, stale).
- `label` (`:344`), `rewrite` (`:420`, inline links only: reference-style links are
  never rewritten), `extract`, `sections`, `slug`, `percent_encode_path` / `decode`.

### 2b. Why `/doks/sessions/…` and `/doks/sources/<id>` fail (yaks-bede)

Trace for the TUI: `mdrender::link_target` (`src/tui/mdrender.rs:1012`) calls
`links::parse("/doks/sessions/2026/11/x.md", From::Yak)`, which gives
`Link::Farm{path:"doks/sessions/2026/11/x.md"}` (`links.rs:146-150`), which becomes
`Target::Farm`. Enter then runs `detail_nav.rs:401` → `App::open_farm_doc` (`follow.rs:225`)
→ `read_farm_doc` (`follow.rs:87-119`), and `root.join(path).is_file()` is false, so the
TUI shows "…: not in this farm". The session file is at `root/sessions/…`
(`sessions.rs:28` `DIR="sessions"`, a sibling of `doks/`, not inside it), so
`/sessions/2026/11/x.md` resolves (checked with `yaks peek`).

`/doks/sources/2026-11-03-81364278` fails the same way, and so does
**`/sources/2026-11-03-81364278`**: it names a *directory*, and `read_farm_doc` needs
`is_file` (`follow.rs:97`) and `read_at` reads a file (`links.rs:666-671`). Only
`/sources/<id>/source.md` resolves. `follow.rs:108-111` then also loads
`extracted.md`. CLI `yaks peek` gives the same results (checked).

Root cause: the namespace is `{doks, sessions, sources, schemas, <herd>/{yaks,doks}}`
under the farm root, but people (and the skill text "a dok: from the farm root,
`/doks/...`") treat `/doks/` as the knowledge root. The fix is one aliasing rule in the
resolver, not in each caller: `doks/{sessions,sources,schemas}/…` → `{…}/…`,
`sources/<id>` (dir) → `sources/<id>/source.md`, and any dir with an `index.md` /
`README.md` → that file. Plus a `doctor` message suggesting the canonical form. A
schema-9 fixup could also rewrite these.

### 2c. Places that write link strings

| Writer | Where | Emits |
|---|---|---|
| `yaks attach <id>` | `store.rs:2066-2090` `attachment_link[_titled]`, via `farm.rs:2828-2868` | `[stem](<id>/<enc name> "caption")`, `![…]` for images |
| `yaks attach <dok-or-session>` | `farm.rs:2869-2906` (`link_dir`, `:2893`) | `![x](<stem>/<file>)`: relative, down from the doc |
| `yaks promote` | `farm.rs:2511-2605` → `rewrite_links_to` (`:2610-2681`) | Every link to the attachment becomes `/<new_path>[#frag]` (farm-root form). Sibling links inside a promoted dok become `<id>/<file>` (`:2571-2580`) |
| `yaks mv` (dok move) | `farm.rs:2697-2810` | `/<to>[@rev][#frag]` (farm root). Links into the moved attachment dir stay relative (`<name>/rest`) inside the moved dok, else `/to_dir/rest` |
| `yaks rename-attachment` | `farm.rs:2914-3016` | `<id>/<enc new>`, the bare sibling in attachments, and link text follows the stem |
| `yaks rename` / `rename-prefix` | `farm.rs:1953-2090` via `refs::rewrite` (`refs.rs:174`) | Whole-token id rewrite (catches `[x](old)`, `old/file`, `old#ts`). **Doks, sessions and markdown attachments are not rewritten** (only yak titles and bodies, `:1996-2044`) |
| `yaks source add` | `sources.rs:305-610` (`:600`) | `[extracted.md](extracted.md)` inside `source.md` (dok-relative). `--part-of` and `superseded_by` take `sources/<id>/source.md` (no leading `/`) |
| `yaks session note` (`## Saved`) | `sessions.rs:564-608` `render_saved` | `[label](<id>#<ts>)` and `[/p](/p)` (farm root) |
| `yaks update --note` | `main.rs:3752` `latest_note_link` (`links.rs:251`) | Prints `<id>#<ts>` |
| TUI dok completion | `tui/editor.rs:296-330` (triggers on `/doks/…`, `/<herd>/doks/…`), `tui/handlers.rs:2393-2399` | Inserts `/<path>` |
| `yaks migrate` fixups | `migrate.rs:82-98` (`/wiki/` → `/doks/`), `:104-143`, `:191-258` (attachment shapes), `:379-444` (doks, schemas, sessions; **not sources, not markdown attachments**) | Farm-root and `<id>/<file>` |
| TUI `y` (copy) | `handlers.rs:133-145` `copy_path`, `doks_tab.rs:273` `copy_doks_path` | Farm-relative path **without** a leading `/`, a source by its dir (`sources/<id>`), an attachment as `<id>/<file>`. This is not a link that resolves: `sources/<id>` fails everywhere (see §2b) |

### 2d. Readers, resolvers and openers

- **CLI:** `yaks peek` (`main.rs:2255-2270`: `From::Cli`, `code_base(cwd)`); `yaks refs --incoming`
  for a dok (`main.rs:5039-5082`, `farm.rs:2685` `links_to`, exact `farm_path` equality, so
  aliases don't count); `yaks doctor` (`farm.rs:1722-1735`); `yaks render` (`render.rs:16-34`:
  cwd file, else farm path with `/` and `.yaks/` stripped, else attachment); `check.rs:245-270`
  `source_refs` (a textual `sources/<id>/` scan, which already matches `/doks/sources/<id>/`
  *by accident*: the preceding char is `/`). `find.rs:80-131` walks files and does not touch links.
- **TUI classification:** `mdrender::link_target` (`mdrender.rs:1012-1040`) maps `Link` to
  `detail::Target` (`detail.rs:~22-55`: `Task, Note, Url, Artifact(String), Source{path,lines,rev},
  Footnote, Farm{path,rev,anchor}, Local`). `Target::as_link` (`detail.rs:58-87`) maps it back.
  Images: `mdrender.rs:511-525` (farm image only if it starts with `/` or is in a dok).
  `detail.rs:710-720` `image_slot`.
- **TUI dispatch**, three copies of the same match: `detail_nav.rs:387-410` `follow_link`,
  `handlers.rs:1455-1470` (viewer Enter), `handlers.rs:1615-1631` `open_current_external` (`O`).
  Then:
  - `Artifact` → `open_external` (`handlers.rs:1191-1218`) → `resolve_attachment`
    (`:1604-1612`) → image viewer, text viewer, else `os_open`.
  - `Farm` → `open_farm_doc` / `read_farm_doc` (`follow.rs:87-248`), and `read_doc`
    (`:201-221`) for history restore.
  - `Source` → `open_source` (`tui/source.rs:162-202`) → `resolve_source_file` (`:90-144`,
    a temp read-only copy for a pinned file) → `$VISUAL`/`$EDITOR`, else `os_open`.
    **This is what yaks-c3d0 wants to replace with a viewer first.**
  - `FileOpenExternal` (`O` in viewer and image: `handlers.rs:1398-1402`,
    `imageview.rs:430`).
  - Peek: `follow.rs:311-349`.
  - History stores `PaneAt::Doc{path, rev}` (`history.rs:76-89`), with `path` either
    `<id>/<file>` or a farm path, told apart by `read_doc` checking `root.join(path).is_file()`.
- **Keymap entries involved:** `PaneEnter`, `LinkOpenExternal` (`keymap.rs:124`, `:747`),
  `FileOpenExternal` (`:125`, `:788`, `:808`), `RowCopy` (`:115`, `:824`), `SelectYank`
  (`:141`, `:777`). The help text at `keymap.rs:1174-1178` names link forms.

## 3. Proposed seam: one resolver for "link → place → {local path, content, web URL}"

Today the grammar is shared, but resolution is spread across `links::{exists,read_at,peek,
code_base}`, `tui/follow.rs::read_farm_doc`, `tui/source.rs::resolve_source_file`,
`handlers.rs::resolve_attachment`, `render.rs::resolve` and `detail.rs::image_slot`, with
`.yaks/` strip hacks in five places and three copies of the dispatch match.

```text
links.rs        (pure grammar, unchanged API)  parse, Link, From, Anchor, label, rewrite, extract
locate.rs (new) Space    = what a link resolves against, built once per Farm (+cwd):
                  farm_root, farm_git: Option<FarmGit{top, prefix, branch}>, code: Option<PathBuf>
                Place    = Yak{id,note} | FarmFile{rel,rev,anchor} | Attachment{id,name,anchor}
                           | CodeFile{rel,rev,anchor} | Url | Missing{why, suggest}
                Space::resolve(&Link) -> Place     // ONE place for aliases: /doks/sessions→/sessions,
                                                   // sources/<id>→…/source.md, legacy .yaks/<id>/x,
                                                   // pre-move @rev fallback
                Space::local_path(&Place) -> Option<(PathBuf, ReadOnly)>   // absolute; pinned → temp copy
                                                   // (moves resolve_source_file here)
                Space::content(&Place) -> Result<Content{text|binary,label}> // read_at + extracted.md;
                                                   // yaks-c3d0 read-only viewer and peek use it
                Space::web_url(&Place, Pin) -> Option<String>  // via hosting.rs
                Space::repo_path(&Place) -> Option<(Repo, String)>  // repo-relative path (prefix applied)
hosting.rs (new) enum Forge { GitHub, GitLab, Gitea/Forgejo, Bitbucket, SourceHut, Unknown }
                Forge::detect(normalized_remote) // reuse store::normalize_remote_url (store.rs:286)
                fn blob_url(forge, base, rev_or_branch, path, Option<(a,b)>) -> String
                fn remote_for(top, branch) -> Option<String>  // sync::remote_of rule (sync.rs:103),
                                                              // moved to gitrepo.rs and shared with farmsync.rs:139
```

- `links::exists`, `links::peek` and `links::code_base` become thin wrappers over
  `Space`, or move into it. `peek` = `content` + `span`.
- The TUI's `Target::{Farm,Source,Artifact}` collapse into `Target::Link(Link)` (keeping
  `Task/Note/Footnote/Local/Url` for render-only cases), and the three dispatch matches
  become one `App::follow(Place)`. The yaks-c3d0 "view first" behaviour is then one rule:
  `CodeFile` opens in the text viewer via `content` (syntax highlighting from
  `tui/syntax.rs`, already used by `mdrender`); `O` opens it in the editor; a new
  `link.copy_path` / `file.copy_path` uses `local_path`; a new `link.open_web` /
  `file.open_web` uses `web_url`. The same verbs apply to a yak row (`Place::Yak` →
  `task_path`), a Doks row, or the open viewer.
- Pinned hashes (yaks-c3d0's question): `local_path` gives the read-only temp copy (as
  today), and `web_url` gives `…/blob/<sha>/<prefix><path>#L12-L40`. That is
  exact on every forge with no local work, so it is the natural fallback when the local
  copy can't be produced. For a farm file at branch tip, use `…/blob/<branch>/…` (a
  stable link), or the HEAD sha for a permalink (a choice for the human).
- CLI: `yaks peek` stays. Add `yaks url <link|id|path> [--pin]` (and perhaps
  `yaks path <link>`) on the same `Space`. `Farm` gains `fn space(&self, cwd)`.
- Where `prefix` is used: `FarmGit.prefix` is the one source of truth for "farm-relative
  → repo-relative" (web URLs, `git show rev:` fallbacks, changes, weight, land). It is
  `""` after the layout change and `".yaks/"` on an unmigrated farm, so the same code
  serves both during the transition.

## 4. Test and documentation surface

- **Unit tests in `src/` that mention `.yaks`** (test regions only): status 60, store 39,
  commit 26, preflight 22, farm 13, weight 11, tui/tests 9, brief 7, mergedriver 7,
  land 6, tui/source 4, plus about 15 single digits. **About 220 lines.** Most build a
  repo with a committed `.yaks/` (for example `commit.rs:318-327`, `status.rs`,
  `preflight.rs`) and assert repo-relative `.yaks/…` paths.
  `tui/test_support.rs:129` `temp_farm` makes `proj/.yaks`, and many TUI tests assert
  `proj.join(".yaks/t0/yaks/...")` (`tui/tests.rs:3125-6015`).
- **Integration tests in `tests/`:** merge_driver 15, init_modes 11, cli 8 (plus the
  `tests/fixtures/farm/.yaks` fixture via `YAKS_DIR`), sync 5, autocommit 3, dash_c 2,
  weight, sheds and bare_host 1 each. `tests/common/mod.rs` already goes through
  `yaks init`.
- **Insta snapshots:** none of the 54 under `src/tui/snapshots/` or the 22 under
  `tests/snapshots/` contain `.yaks`. 3 contain link forms. Docshots:
  `docs/assets/tui-doks.svg` has 1 link-form string.
- **Docs mentioning `.yaks`:** `docs/cli.md` 21, `AGENTS.md` 10, `docs/tui.md` 7,
  `docs/README.md` 6, `README.md` 3, `docs/skills.md` 1. Link forms: `docs/tui.md` 14,
  `docs/cli.md` 12.
- **Bundled skills:** `.agents/skills/yaks/SKILL.md` (8 `.yaks`, 8 link-form lines,
  including the "Links." section at about lines 125-136, which is the agent-facing
  grammar), `yaks-research` (8 link-form), `yaks-upkeep` (9 link-form),
  `yaks-coordinating` 3, `land` 2, `yaks-working` 1, `yaks-scheduling` 1. Also the
  notes line in `src/brief.rs` (AGENTS.md rule), and the farm dok
  `/doks/design/knowledge-store.md` (layout table at lines 25-27, grammar at line 34).
- **Clap help** (`src/main.rs`): lines 5, 185, 266, 313, 320, 332, 482-484, 1027, 1231
  ("sets `.yaks/schema` to 8"), 1271, 1280, 1421-1428, 1461, 1534.

## 5. Risks

**Old binary, new-layout branch: hard failure, with the wrong advice.** Discovery
fails before the schema gate ever runs (`store.rs:382-389`): "has no .yaks/ there;
restore it (`git checkout -- .yaks`)", and that command then fails too. `min_version`
(in `config.yaml`, `store.rs:1089`, `:2719`) can't help, because the old binary never
finds the config.

Mitigation, which costs nothing in new code: **leave a stub `.yaks/schema` containing `9`
on the branch.** Old binaries then open `<wt>/.yaks`, `schema_status` reports Newer, and
they refuse with "upgrade yaks" (`farm.rs:1019-1023`). A long-running old TUI refuses its
next write the same way (`store.rs:1585`). The stub must contain nothing else. New
discovery must look at `wt/config.yaml` first, and the reserved names must include
`.yaks` (already skipped as a dot-dir). Also: set `min_version` in the release before the
migration, as `doks/howto/migrate-a-farm.md` already recommends.

**New binary, old-layout branch:** handled if discovery falls back to `wt/.yaks`. The
schema-8 farm then hits `migration_refusal` ("run `yaks migrate`"), as today.
`migration_refusal`'s upstream probe must read both `{upstream}:schema` and
`:.yaks/schema` (§1a), or a behind-but-migrated checkout is told to migrate again.

**Half-migrated working tree:** `migrate::run` writes the schema marker last
(`migrate.rs:619`). For the layout step, do the root move first, then the
`.gitattributes` rewrite and link fixups, then `/schema`=9 and the `.yaks/schema` stub
last. An interrupted run then leaves `<wt>/.yaks/config.yaml` absent but
`<wt>/config.yaml` present, and the plan is re-runnable only if `plan_*` is idempotent
over a partially moved tree. Use `git mv` (or a plain rename plus `yaks commit`, which
runs `git add -A` and records renames, `commit.rs:125`). Make it **one commit**.

**Shared branch and other machines' unpushed commits:** I reproduced this in a scratch
repo with git 2.56. Machine B had old-layout commits on top of the pre-move base; it
rebased onto the moved branch, as `yaks sync` does (`sync.rs:134-148`):

- Edits to existing yak files were carried over cleanly by rename detection
  (`h/yaks/h-0001.md` got the edit).
- A **new yak file in an existing renamed dir** came out as a `UA` conflict with the
  default `merge.directoryRenames=conflict`. `yaks sync` would abort the rebase and
  report.
- A **new file in a directory that did not exist at the base** (`.yaks/sessions/n.md`)
  was **stranded at the old path**, even with `merge.directoryRenames=true`. Git's
  directory-rename detection does not apply to a rename into the top level.

So unpushed old-layout work must be pushed and merged *before* the migration (the
howto's "fetch, migrate, publish" rule), or the migrate step must sweep stray
`.yaks/**` paths (other than the stub) into place on every run. The yak merge driver only
sees pairs git matched by rename: its `merge=yaks` rule must match both old and new paths
during the transition (`*.md merge=yaks` does), otherwise renamed-and-edited yak files
merge as plain text.

**History:** `git log --follow` crosses the move for single files. That is already the
case for the schema-6 move: `yaks/yaks/yaks-0012.md` shows 5 commits with `--follow`
against 2 without, and every rename is R100. The branch has 2011 commits and 1246 files;
the move commit would be about 1240 renames, all R100, under git's default rename limit
of 1000 for `diff`, but **over it** for some operations:
- `diff.renameLimit` defaults to 1000 (merge uses 7000). With more than 1000 files,
  `git log -M` / `git show` of the move commit may fall back to add/delete pairs. `--follow`
  is per path, so it is unaffected.
- Recommend checking `git show --stat -M` on the rehearsal clone.

**Things that break silently** (no error, wrong behaviour):
- Without the `farm_git` fix, per-change commits stop: every edit stays uncommitted until
  `W`/`yaks commit`.
- Without the `yak_id_of` fix, commit messages and status lose per-yak verbs.
- LFS stops applying unless `.gitattributes` is rewritten.
- The land guard stops working.
- Views and drafts are lost (cache slug).
- `@sha` farm pins to pre-move commits break unless `read_at` gets the `.yaks/` fallback.
- `yaks session touched` misses pre-move commits.
- `check` staleness resets.

## 6. How today's stored link forms would behave on GitHub once (A) lands

This section is reasoned from GitHub's documented relative-link rules, not tested on
GitHub (that is yaks-7d13's job). A yak file is at `<herd>/yaks/<id>.md`.

| Form (writer) | From a yak | From a dok / session |
|---|---|---|
| `/doks/x.md#h`, `/sessions/…`, `/sources/<id>/source.md` (mv, promote, session Saved, skill) | works (root-relative on the branch) | works |
| `<id>/<file>` (attach) | works (sibling dir) | breaks (resolves under the dok's dir) |
| `x/shot.png` (attach to a dok) | n/a | works |
| bare id `yaks-1a2b` / `yaks-1a2b#<ts>` (skill, session Saved, `update --note` output) | 404 (`<herd>/yaks/yaks-1a2b`, no `.md`), and `#<ts>` names no heading GitHub generates | 404 |
| `code:src/x.rs#L1` | dead (unknown scheme) | dead |
| legacy `../src/x.rs`, `../blob/<sha>/…` | wrong (`<herd>/src/x.rs`) | refused today |
| `/doks/sessions/…`, `/sources/<id>` (dir) | 404 / tree view | same |

So (A) alone fixes the farm-root form, which is the most common one. (B) is then about
the yak-id, `code:` and cross-document attachment forms. A document-relative rewrite
(`../../yaks/yaks-1a2b.md`) collides with the grammar's no-`../` rule (`links.rs:126-138`,
`doctor`'s Upward), so going that way reverses design decision yaks-ccb1 ("Rejected:
keeping `../`"). The alternative keeps the stored forms and adds a render or export
step. Either way, the resolver in §3 is where the mapping lives.

## 7. Size per area, and how to split the work across workers

| # | Area (owned files) | Size | Notes |
|---|---|---|---|
| W1 | Discovery and identity: `store.rs` (discover, `farm_git` + `FarmGit.prefix`, `init`, `migration_refusal`), `init.rs` | M | **Must land first.** Everything else reads `farm_git().top` / `prefix`. |
| W2 | git-top users: `status.rs`, `sync.rs`, `sheds.rs`, `tui/farmsync.rs`, `farm.rs::code_repo`, `links::code_base` | S | Mechanical, after W1. |
| W3 | Repo-relative classifiers: `changes.rs`, `preflight.rs::yak_id_of`, `commit.rs::generate_message`, `land.rs` guard, `sessions.rs::changed_paths` | M | The land guard needs a design decision. |
| W4 | `.gitattributes` writers + migration: `mergedriver.rs` (ATTR_LINE), `weight.rs` (LFS rules, `farm_top`, `kind_of`), `migrate.rs` (schema 9 layout step + stub + attr rewrite + stray sweep), `store::SCHEMA`, `links::read_at` pre-move fallback | L | Rehearse on a clone of this farm (2011 commits) and `joel`. |
| W5 | Resolver seam: new `locate.rs` / `hosting.rs`, `links.rs` wrappers, alias rule (yaks-bede) | M | Independent of W1-W4 if it reads `FarmGit.prefix` through an `Option`. |
| W6 | TUI on top of W5: `tui/detail.rs` `Target`, `mdrender.rs::link_target`, the three dispatch matches (`detail_nav.rs`, `handlers.rs`), `follow.rs`, `source.rs` view-first (yaks-c3d0), new keymap verbs, history | L | `handlers.rs` and `keymap.rs` are contended: one worker. |
| W7 | Tests and fixtures: `tui/test_support.rs`, `tests/common`, `.yaks` expectations across about 220 unit and about 45 integration lines | M | Split per file alongside W1-W4. Each worker fixes its own module's tests. |
| W8 | Docs, skills and clap help (§4) | M | After behaviour settles. One worker owns `main.rs` doc comments. |
| W9 | Cache-slug migration for views and drafts (`tui/cache.rs`, `views_store.rs`, `drafts.rs`) | S | Independent. |

What can run in parallel without colliding:
- After W1 lands: {W2, W3, W9} and W5 together.
- W4 can start on `migrate.rs` in parallel with W1, but owns `mergedriver.rs` and `weight.rs`.
- W6 only after W5.
- W8 last.

Contention points to assign explicitly:
- `store.rs` (W1 only)
- `weight.rs` (W4 only)
- `farm.rs` (W2 touches only `code_repo`; W5/W6 may touch `promote`/`mv` writers if (B) changes the emitted forms)
- `main.rs` (W8)
- `tui/handlers.rs` and `tui/keymap.rs` (W6)
