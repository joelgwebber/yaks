//! yaks — a filesystem-native task tracker.
//!
//! This binary is a thin CLI over `farm::Farm` (the print-free core ops
//! facade): every command is parse args -> call one Farm op -> render (text or
//! --json). Tasks live as markdown files under `.yaks/`; status is the folder.

mod actor;
mod autocommit;
mod branchfarm;
mod branchlog;
mod brief;
mod changes;
mod check;
mod clipboard;
mod commit;
mod conflicts;
mod doks;
mod farm;
mod fields;
mod filter;
mod find;
mod gitrepo;
mod init;
mod json;
mod land;
mod links;
mod markdown_lint;
mod mergedriver;
mod migrate;
mod model;
mod preflight;
mod privatefarm;
mod recur;
mod refs;
mod rollup;
mod schemas;
mod sessions;
mod sheds;
mod skills;
mod sources;
mod status;
mod store;
mod sync;
mod tick;
mod tui;
mod wait;
mod weight;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::env;

use farm::{
    AttachOutcome, Commits, CreateOutcome, DepOutcome, Farm, Issue, IssueKind, LogEntry,
    MergeOutcome, MoveOutcome, NewTask, OpenError, Pickup, RefKind, RenameAttachmentOutcome,
    RenameOutcome, RenamePlan, Reparent, Show, SlaughterOutcome, SnoozeOutcome, Stats,
    TaskBackRefs, TaskEdit, TaskRefs, UpdateOutcome,
};
use filter::FilterSpec;
use model::{NEEDS_AGENT, Status, Task, normalize_tags};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "yaks",
    version,
    about = "Filesystem-native task tracker (Rust)"
)]
struct Cli {
    /// Run as if started in <PATH> (like `git -C`).
    ///
    /// Farm discovery,
    /// `commit`, `init`, `skills` and every other command see
    /// <PATH> as the current directory. A relative <PATH> is resolved against
    /// the real cwd; repeated `-C` compose, each relative to the previous.
    /// `$YAKS_DIR` still wins over discovery (a relative one is read from <PATH>).
    #[arg(short = 'C', value_name = "PATH", global = true)]
    chdir: Vec<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

/// Shared narrowing flags (AND across fields; OR within a repeated field).
#[derive(Args, Default)]
struct FilterFlags {
    #[arg(long)]
    status: Vec<String>,
    #[arg(long = "type")]
    kind: Vec<String>,
    #[arg(long)]
    priority: Vec<u8>,
    /// Keep only yaks with any of these tags. Repeatable; ORs. Comma- or
    /// space-separated lists are split (`--tag ui,docs`). `--label` is the
    /// old spelling.
    #[arg(long, alias = "label")]
    tag: Vec<String>,
    /// Keep only yaks in these herds (id prefixes). Repeatable; ORs.
    #[arg(long)]
    herd: Vec<String>,
    #[arg(long)]
    search: Option<String>,
    /// Keep only yaks whose `source:` is exactly this URL (dead ones too with
    /// `--all`): find the yak for an external issue or pull request.
    #[arg(long, value_name = "URL")]
    source: Option<String>,
    #[arg(long)]
    ready: bool,
    #[arg(long)]
    tangled: bool,
    /// Keep only yaks with a `needs` field set: awaiting a human (`needs:
    /// human`) or awaiting an agent (`needs: agent`: answered or requested).
    #[arg(long)]
    needs: bool,
    /// Keep only yaks whose description or any note has run-on lines:
    /// adjacent lines that markdown joins into one paragraph (a soft line
    /// break). Candidates for a tidy.
    #[arg(long = "run-on")]
    run_on: bool,
    #[arg(long = "parent-of")]
    parent_of: Option<String>,
}

impl FilterFlags {
    /// True when at least one selector is set. Bulk mutation refuses an
    /// unfiltered run, so it uses this to guarantee the whole farm is never the
    /// implicit target.
    fn any_set(&self) -> bool {
        !self.status.is_empty()
            || !self.kind.is_empty()
            || !self.priority.is_empty()
            || !self.tag.is_empty()
            || !self.herd.is_empty()
            || self.search.is_some()
            || self.source.is_some()
            || self.ready
            || self.tangled
            || self.needs
            || self.run_on
            || self.parent_of.is_some()
    }
}

#[derive(Args, Default)]
struct RollupArgs {
    #[command(flatten)]
    filter: FilterFlags,
    /// Print just the external keys (one per line) for pasting into a PR body.
    #[arg(long)]
    keys: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Subcommand)]
enum Command {
    /// List tasks (non-dead by default; --all also includes dead).
    List {
        #[command(flatten)]
        filter: FilterFlags,
        #[arg(long)]
        all: bool,
        /// Only yaks with a `due:`, sorted by it (soonest first).
        #[arg(long)]
        due: bool,
        /// With `--due`: only yaks due before this date or datetime.
        #[arg(long, value_name = "DATE", requires = "due")]
        before: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show one task by id.
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Print the on-disk file path of yaks, by id or by filter flags.
    ///
    /// One path per line: by id
    /// (`yaks path yaks-1a2b ...`) or by the shared filter flags
    /// (`yaks path --status shorn --tag cli`). Each yak's file is
    /// `.yaks/<herd>/yaks/<id>.md`, the same path for its whole life (a status move edits
    /// it in place), handy for `git add`. Paths are absolute. An unknown
    /// id is reported on stderr and the exit status is non-zero (known ids are
    /// still printed). Give ids or filter flags, not both. With neither, print
    /// the farm root this command resolved (the farm branch's worktree when
    /// the farm is a branch): which farm am I on?
    Path {
        /// Yak ids to resolve.
        ids: Vec<String>,
        #[command(flatten)]
        filter: FilterFlags,
        /// With filter flags, also include dead yaks (as `list --all`).
        #[arg(long)]
        all: bool,
    },
    /// List the yaks a task points at, or (--incoming) the yaks pointing at it.
    ///
    /// Outgoing references are the parent, deps, and id mentions in its text;
    /// any formal reference that dangles is flagged. With --incoming, list the
    /// yaks that point at it instead. Given a dok path (`doks/x.md`) instead
    /// of an id, `--incoming` lists the yaks and doks that link to it, and
    /// plain `refs` the links the dok holds.
    Refs {
        id: String,
        /// Incoming references: every yak whose `parent:` or `depends_on:` names
        /// this one, or whose title, description or a note mentions its id (whole
        /// real ids only), each labelled with how it refers. Computed by scanning
        /// the farm; no index.
        #[arg(long)]
        incoming: bool,
        /// Machine output (with --incoming).
        #[arg(long, requires = "incoming")]
        json: bool,
    },
    /// Show the git commits linked to a yak.
    ///
    /// Sections. "Recorded" lists the notes that carry a `commit:` key
    /// (what `yaks land` writes): kind, short sha, branch, the subject from the
    /// code repository (or "(not in this repository)"), and the note's time and
    /// actor. "Commits naming" greps commit messages for the id in the code
    /// repository: the git top-level of the directory yaks was started from
    /// (respecting `-C`), across every branch but the farm's, newest first.
    /// Started outside any code checkout (not in git, or in the farm's own
    /// worktree) it says so and lists nothing. "Farm history" is the yak file's
    /// own edit history on the farm, followed across status moves.
    ///
    /// "Branch <remote>/<branch>" (yaks-4728) lists, for each branch the yak's notes record
    /// (`--meta branch=<b>`, as a checkpoint note does; optional `remote=`,
    /// default `origin`, and `base=`, default the remote's default branch, else
    /// `main`), the commits `git log <base>..<remote>/<branch>` shows, with the
    /// recorded checkpoint commit marked, or a line saying it is not on the
    /// branch (rewritten, or never pushed). It reads the remote-tracking ref
    /// already here and says it is as of the last fetch; `--fetch` first runs
    /// `git fetch <remote> +refs/heads/<branch>:refs/remotes/<remote>/<branch>` for each (the only thing here that uses the
    /// network). A fetch that fails prints one line for that branch and goes on;
    /// the exit status stays 0 when the yak exists.
    Commits {
        id: String,
        /// Fetch each recorded branch from its remote first.
        #[arg(long)]
        fetch: bool,
        /// Machine output: `recorded`, `branches`, `named` and `farm_history`.
        #[arg(long)]
        json: bool,
    },
    /// Run a yak's `verify:` command and record PASS/FAIL as a note.
    ///
    /// Runs the yak's recorded `verify:` command and records the PASS/FAIL as an
    /// attributed evidence note. Explicit — yaks never auto-runs it. Exits
    /// non-zero if any verify fails (or a yak has no `verify:` command).
    Verify {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the result note to this actor (stamped as `[actor]`).
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Attach a local file to a yak as evidence.
    ///
    /// Copies the file under
    /// `.yaks/<herd>/yaks/<id>/` (beside the yak's file) and link it in the body as
    /// `![name](<id>/<file>)` for an image (png, jpg, jpeg, gif, webp, svg)
    /// or `[name](<id>/<file>)` for any other file. Use for artifact
    /// evidence a human or coordinator can look at (screenshots, TUI frames).
    ///
    /// `--caption` writes the image's markdown title
    /// (`![name](<id>/<file> "CAPTION")`), which `yaks tui` shows under the
    /// picture; prefer it to a lead-in line such as `Context: ![..](..)`.
    ///
    /// Several files attach in one write. For a markdown document and the
    /// images it embeds, attach them together with `--no-link` and link only
    /// the document from the yak: the document's own `![x](shot.png)` (the
    /// bare file name) resolves to its sibling attachments, in `yaks tui` and
    /// in any markdown viewer.
    Attach {
        id: String,
        /// Paths of the local files to attach.
        #[arg(required = true)]
        paths: Vec<PathBuf>,
        /// Store the attachment under this filename instead of the source's
        /// (sanitized; the source's extension is kept if NAME omits one).
        /// Only with a single file.
        #[arg(long)]
        name: Option<String>,
        /// Store the files without adding a link to the yak's body.
        #[arg(long)]
        no_link: bool,
        /// A caption for the image, written as the link's markdown title
        /// (`![name](<id>/<file> "CAPTION")`): `yaks tui` shows it under the
        /// picture. One file, and not with `--no-link`.
        #[arg(long)]
        caption: Option<String>,
        /// An optional attributed note to record alongside the attachment.
        #[arg(long)]
        note: Option<String>,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Rename a yak's attachment and rewrite every link to it.
    ///
    /// Renames the attachment in `.yaks/<herd>/yaks/<id>/` and rewrites every
    /// link to it across the farm (the link text follows the new stem, and
    /// the `!` of an image link follows the new extension). NEW is sanitized
    /// and keeps OLD's extension if it omits one; refuses to overwrite an
    /// existing attachment.
    RenameAttachment {
        id: String,
        /// Current filename under `.yaks/<herd>/yaks/<id>/`.
        old: String,
        /// New filename (e.g. `login-page`, becomes `login-page.png`).
        new: String,
    },
    /// Find a yak's attachments (their absolute paths).
    ///
    /// With no FILE, lists the absolute path of each attachment of the yak.
    /// With FILE, prints that one's absolute path, or exits 1 when the yak
    /// has no such attachment. Read-only.
    Attachment {
        id: String,
        /// An attachment filename under `.yaks/<herd>/yaks/<id>/` (as in its link).
        file: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Scan text for real yak ids (a leak check for private farms).
    ///
    /// Finds tokens that are real yak-ids in this farm — a leak check
    /// for private farms (pre-commit / PR hook). Reads a FILE and/or
    /// stdin, prints each found id as `line:col  id`, and EXITS NON-ZERO if
    /// any are found (clean text exits zero).
    ScanIds {
        /// File to scan. Omit (or combine with) piped stdin; both are scanned.
        #[arg(value_name = "FILE")]
        file: Option<String>,
        /// Emit machine-readable JSON instead of `line:col  id` lines.
        #[arg(long)]
        json: bool,
    },
    /// List hairy tasks whose dependencies are all resolved.
    #[command(visible_alias = "ready")]
    Next {
        #[command(flatten)]
        filter: FilterFlags,
        #[arg(long)]
        json: bool,
    },
    /// List hairy tasks with at least one unresolved dependency.
    #[command(visible_alias = "blocked")]
    Tangled {
        #[command(flatten)]
        filter: FilterFlags,
        #[arg(long)]
        json: bool,
    },
    /// Activity log: timestamped notes and status moves, oldest first.
    ///
    /// Covers a filtered set of yaks.
    Log {
        #[command(flatten)]
        filter: FilterFlags,
        /// Only entries at or after this point: a duration (2h, 3d, 1w), a date
        /// (YYYY-MM-DD), or an RFC3339 timestamp. Omit for the full log.
        #[arg(long)]
        since: Option<String>,
        /// Only entries attributed to this actor (matches the `[actor]` stamp).
        #[arg(long)]
        by: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Substring search over id/title/description.
    Search {
        query: String,
        #[command(flatten)]
        filter: FilterFlags,
        #[arg(long)]
        json: bool,
    },
    /// Full-text search over the farm: yaks (notes included), doks, sources'
    /// text and sessions.
    ///
    /// Reads the files directly (no index) and prints one line per match,
    /// `path:line: text`, grouped by file under its title. The match ignores
    /// case and is a literal substring unless `--regex`. Searches yak files
    /// (title, description, notes), doks (frontmatter and body), a source's
    /// `source.md` and `extracted.md` (never the original file), and
    /// `sessions/**.md`. `--scope` keeps paths under a prefix (`doks/topics/`,
    /// `sources`, a herd such as `web`; repeatable), `--kind` keeps one of
    /// yaks, doks, sources or sessions (repeatable). `--json` prints
    /// `[{path, kind, title, line, text}]`. Exits 1 when nothing matches, like
    /// `grep`. `yaks search` is the quick yak-only lookup with the shared
    /// filters.
    Find {
        /// The text to find (a regular expression with `--regex`).
        pattern: String,
        /// Treat the pattern as a regular expression: `. [a-z] \d \w \s \b ^ $
        /// ( ) | * + ? {n,m}`.
        #[arg(long)]
        regex: bool,
        /// Only paths under this prefix of the farm (repeatable): `doks/`,
        /// `sources`, `doks/topics/vec`, or a herd name for its yaks and doks.
        #[arg(long = "scope", value_name = "PREFIX")]
        scopes: Vec<String>,
        /// Only this kind of file (repeatable).
        #[arg(long, value_parser = ["yaks", "doks", "sources", "sessions"])]
        kind: Vec<String>,
        /// Match case exactly (the default ignores case).
        #[arg(long)]
        case_sensitive: bool,
        /// Ignore case. This is already the default; accepted like grep's `-i`.
        #[arg(short = 'i', long, conflicts_with = "case_sensitive")]
        ignore_case: bool,
        #[arg(long)]
        json: bool,
    },
    /// Who is working on what: one row per actor, read from the farm alone.
    ///
    /// A shed is a worker in the farm: an actor, the `[actor]` stamp on its
    /// notes and moves. Nothing here looks at git checkouts, so the same
    /// (synced) farm reads the same on every machine. Per actor: `shaving`
    /// the yaks it is working on (`shaving` now, and it is the most recent
    /// write since the yak moved into shaving, or the `for:` of that move;
    /// `asking you` when one waits on a human), `asking` its other yaks with
    /// `needs: human`, and `recent` the other yaks it wrote, newest first
    /// with their status now (five, then `+N more`). The header is `active`
    /// (wrote within 2 hours), `stale` (holds a shaving yak but has not
    /// written for longer) or `idle`, and when it last wrote. Actors with an
    /// open ask come first, then active, stale, idle, then the most recent
    /// writer. Idle actors with no claim and no ask are only counted unless
    /// `--all`. An actor's latest `checkpoint` note naming a branch (on any
    /// yak) shows as `branch <b> @ <short sha> (<ago>)` under its header: where
    /// its code is, read from the note, never fetched (`yaks commits <id>
    /// --fetch` lists the branch). A first line says how far the farm branch is ahead of or
    /// behind its upstream as of the last fetch, when the upstream last changed
    /// (any machine's last sync) and when this farm last fetched (never
    /// fetches; omitted with no upstream). JSON: `farm`, and `sheds` (every actor, idle too) with
    /// `name`, `liveness`, `last_write`, `checkpoint` (`branch`, `commit`,
    /// `at`, or null), `working`, `asking`, `touched`.
    /// Read-only.
    Sheds {
        /// Emit the sheds as JSON (every actor, idle ones too).
        #[arg(long)]
        json: bool,
        /// List idle actors with no claim and no ask too, not just count them.
        #[arg(long)]
        all: bool,
    },
    /// Show (or set) the name this checkout's notes are stamped with.
    ///
    /// The actor is `--as`, else `$YAKS_ACTOR`, else this checkout's
    /// `git config yaks.actor` (what `--set` writes), else a name derived from
    /// the harness (a Delta thread title's leading `<name>:`, as `delta:<name>`,
    /// else the whole title, else the thread id), else the git `user.name`.
    /// Prints the name and where it came from.
    ///
    /// `--set <name>` stores the name in this checkout only (a linked `git
    /// worktree` gets its own worktree config), so every later command, in
    /// any fresh shell, uses it: the way for an agent to name itself once in a
    /// harness whose terminal calls don't share an environment. Pick a
    /// specific name nobody else in the farm uses (`door-2`, `herd-interact`,
    /// not `herd`); it warns when the farm already has notes under that name.
    Whoami {
        /// Store this name as the checkout's actor (`git config yaks.actor`).
        #[arg(long, value_name = "NAME")]
        set: Option<String>,
    },
    /// Internal: git's merge driver for yak files.
    ///
    /// Git's merge driver for yak files (`git config merge.yaks.driver
    /// "yaks merge-driver %O %A %B %P"`, registered for the farm branch with
    /// `.yaks/**/*.md merge=yaks`). Merges the three versions of one yak into
    /// `<ours>` as a valid yak, never conflict markers: notes unioned by
    /// timestamp, tags/deps merged, the later `updated`, and a field both
    /// sides changed differently keeps this side and records the other in a
    /// `merge-driver` note. Exits 1, leaving `<ours>` alone, when an input is
    /// not a yak file. Needs no farm.
    #[command(hide = true)]
    MergeDriver {
        /// The common ancestor (`%O`).
        base: PathBuf,
        /// This side's version (`%A`); the merge result is written here.
        ours: PathBuf,
        /// The other side's version (`%B`).
        theirs: PathBuf,
        /// The path being merged in the repository (`%P`), for messages.
        path: String,
    },
    /// Show task statistics.
    Stats {
        #[arg(long)]
        json: bool,
    },
    /// Create a new task (in hairy).
    Create {
        /// Task title (positional). Wins over --title if both are given.
        title: Option<String>,
        /// Task title (flag form). Prefer the positional; kept for back-compat.
        #[arg(long = "title")]
        title_flag: Option<String>,
        #[arg(long = "type")]
        kind: Option<String>,
        #[arg(long)]
        priority: Option<u8>,
        /// Create it as a child of this yak, which must be hairy or shaving: a
        /// shorn or dead yak holds no open children (reopen it first, or file
        /// the new yak under an open ancestor or at the top level).
        #[arg(long)]
        parent: Option<String>,
        /// Find or create by external key: if any yak (any state, dead
        /// included) already has this `source:` URL, print it and create
        /// nothing (exit 0; a dead or shorn yak is left as it is); otherwise
        /// create the yak with this as its `source:`. `--json` adds `created:
        /// true|false`. Not with `--source`.
        #[arg(long, value_name = "URL", conflicts_with = "source")]
        key: Option<String>,
        /// Which herd (id prefix) the new yak joins; defaults to this repo's
        /// `git config yaks.herd`, else the farm's configured `herd:`. Errors if
        /// neither is set (no silent `yak-` fallback). Lets one farm hold
        /// several herds.
        #[arg(long = "herd")]
        prefix: Option<String>,
        /// Tags for the new yak. Commas and spaces separate tags (a tag may
        /// contain neither): `--tags ui,docs`, `--tags 'ui docs'`, and
        /// `--tags ui docs` are equivalent. Duplicates are dropped. `--labels`
        /// is the old spelling.
        #[arg(long, alias = "labels", num_args = 1..)]
        tags: Vec<String>,
        #[arg(long = "depends-on", num_args = 1..)]
        depends_on: Vec<String>,
        #[arg(long)]
        source: Option<String>,
        /// A pull-request URL for the yak's `prs:` list. Repeat the flag for
        /// several: `--pr URL1 --pr URL2`.
        #[arg(long = "pr", value_name = "URL")]
        prs: Vec<String>,
        /// The yak's body (markdown). `-` reads it from stdin, so a multi-line
        /// heredoc or pipe arrives intact: `yaks create "T" --description - <<'EOF'`.
        #[arg(long, conflicts_with = "description_file")]
        description: Option<String>,
        /// Read the body from this file (see `--description`; `-` is stdin).
        #[arg(long = "description-file", value_name = "PATH")]
        description_file: Option<String>,
        /// A rerunnable verification command for this yak (see `yaks verify`).
        #[arg(long)]
        verify: Option<String>,
        /// When the yak should be done: a date (`2026-11-15`), an RFC 3339
        /// datetime, or `+2d` / `+1w` from today (stored as that date). Sugar
        /// for `--set due=...`; a hairy or shaving yak past it is listed as
        /// overdue.
        #[arg(long, value_name = "WHEN")]
        due: Option<String>,
        /// Make the yak recur: an ISO 8601 duration (`P1D`, `P2W`, `P1M`,
        /// `P1Y`; a run of them in Y M W D order). Sugar for `--set every=...`;
        /// `yaks regrow-due` reopens it once shorn and the next occurrence
        /// has arrived.
        #[arg(long, value_name = "DURATION")]
        every: Option<String>,
        /// Who a recurring yak goes to when it regrows: `agent` (`needs:
        /// agent`, in the agent inbox) or `human` (`needs: human`). Without
        /// it the yak only becomes hairy. Sugar for `--set wake=...`.
        #[arg(long, value_name = "WHO")]
        wake: Option<String>,
        /// Print the created task's id, path, and basic fields as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Update fields, tags, or append a timestamped note.
    ///
    /// Accepts multiple ids; the same edit is applied to each.
    Update {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        #[arg(long)]
        title: Option<String>,
        #[arg(long = "type")]
        kind: Option<String>,
        #[arg(long)]
        priority: Option<u8>,
        /// Replace the body (markdown). `-` reads it from stdin.
        #[arg(long, conflicts_with = "description_file")]
        description: Option<String>,
        /// Replace the body with this file's text (`-` is stdin).
        #[arg(long = "description-file", value_name = "PATH")]
        description_file: Option<String>,
        /// Tags to add. Commas and spaces separate tags (`ui,docs`).
        /// `--add-label` is the old spelling.
        #[arg(long = "add-tag", alias = "add-label", num_args = 1..)]
        add_tag: Vec<String>,
        /// Tags to remove. Commas and spaces separate tags (`ui,docs`).
        /// `--remove-label` is the old spelling.
        #[arg(long = "remove-tag", alias = "remove-label", num_args = 1..)]
        remove_tag: Vec<String>,
        /// Set (or clear, with an empty string) this yak's `source:` URL.
        #[arg(long)]
        source: Option<String>,
        /// Add a pull-request URL to this yak's `prs:` list (repeatable; a URL
        /// already listed is a no-op).
        #[arg(long = "pr", value_name = "URL")]
        add_pr: Vec<String>,
        /// Remove a pull-request URL from this yak's `prs:` list (repeatable).
        #[arg(long = "remove-pr", value_name = "URL")]
        remove_pr: Vec<String>,
        /// Set (or clear, with an empty string) this yak's `verify:` command.
        #[arg(long)]
        verify: Option<String>,
        /// When the yak should be done: a date (`2026-11-15`), an RFC 3339
        /// datetime, or `+2d` / `+1w` from today; an empty string clears it.
        /// Sugar for `--set due=...`.
        #[arg(long, value_name = "WHEN")]
        due: Option<String>,
        /// Make the yak recur: an ISO 8601 duration (`P1D`, `P2W`, `P1M`,
        /// `P1Y`); an empty string clears it. Sugar for `--set every=...`.
        #[arg(long, value_name = "DURATION")]
        every: Option<String>,
        /// Who a recurring yak goes to when it regrows: `agent` or `human`;
        /// an empty string clears it. Sugar for `--set wake=...`.
        #[arg(long, value_name = "WHO")]
        wake: Option<String>,
        /// Set any other frontmatter field, as `key=value` (repeatable):
        /// `--set due=2026-11-05`, `--set commit=[3f2a9c1,e99754d]` (a list),
        /// `--set 'owner={team: web}'` (a map). An empty value removes the
        /// field. Built-in fields (title, tags, ...) have their own flags.
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,
        /// Remove a frontmatter field set with `--set` (repeatable).
        #[arg(long = "unset", value_name = "KEY")]
        unset: Vec<String>,
        /// Append a timestamped note (markdown). `-` reads it from stdin, so a
        /// multi-line heredoc or pipe arrives intact. Prints the note's link
        /// (`<id>#<timestamp>`), for citing it from elsewhere.
        #[arg(long, conflicts_with = "note_file")]
        note: Option<String>,
        /// Read the note from this file (`-` is stdin).
        #[arg(long = "note-file", value_name = "PATH")]
        note_file: Option<String>,
        /// Metadata for the note, as `key=value` (repeatable): `key: value`
        /// lines under its marker, like `kind=checkpoint`. Needs a note.
        #[arg(long, value_name = "KEY=VALUE")]
        meta: Vec<String>,
        /// Attribute the note to this actor (stamped as `[actor]`). Defaults to
        /// $YAKS_ACTOR, then this checkout's `git config yaks.actor` (`yaks
        /// whoami --set`), then the harness identity (from Delta's environment:
        /// `delta:<name>` for a thread title starting `<name>:`, else
        /// `delta:<thread title>`, else `delta:<thread id>`), then the git user;
        /// never implies ownership. `yaks whoami` shows which one applies.
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Block a yak on a human decision and record the question.
    ///
    /// Sets the `needs`
    /// field so the yak drops out of `next`; `answer` hands it to an agent
    /// (`needs: agent`) and `pickup` clears that. Asking a yak that is already
    /// answered (`needs: agent`) flips it back to `human`.
    Ask {
        id: String,
        /// The question for the human (recorded as an attributed note). `-`
        /// reads it from stdin.
        #[arg(long, conflicts_with = "note_file")]
        note: Option<String>,
        /// Read the question from this file (`-` is stdin).
        #[arg(long = "note-file", value_name = "PATH")]
        note_file: Option<String>,
        /// What the yak is waiting on. Defaults to `human`.
        #[arg(long, default_value = "human")]
        needs: String,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Answer a question: record the reply and hand the yak to an agent.
    ///
    /// Records the reply and sets `needs: agent`, so the yak
    /// shows in `inbox` (and, if hairy, `next`) as answered and awaiting an
    /// agent. The human-reserved counterpart to `ask`. `--done` clears `needs`
    /// instead, for an answer that needs no follow-up. A second answer keeps
    /// `needs: agent` and appends the note; on a yak with no `needs` it only
    /// records the note.
    Answer {
        id: String,
        /// The reply/decision (recorded as an attributed note). `-` reads it
        /// from stdin.
        #[arg(long, conflicts_with = "note_file")]
        note: Option<String>,
        /// Read the reply from this file (`-` is stdin).
        #[arg(long = "note-file", value_name = "PATH")]
        note_file: Option<String>,
        /// Clear `needs` entirely instead of handing the yak to an agent.
        #[arg(long)]
        done: bool,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Pick up a yak awaiting an agent: clear `needs: agent` and note who took it.
    ///
    /// Clears `needs: agent` (answered or requested) and records who took the
    /// work on (a `picked up` note). The only command that clears it; errors
    /// on a yak that is not `needs: agent`.
    Pickup {
        id: String,
        /// Optional note (what you will do with the answer or request). `-`
        /// reads stdin.
        #[arg(long, conflicts_with = "note_file")]
        note: Option<String>,
        /// Read the note from this file (`-` is stdin).
        #[arg(long = "note-file", value_name = "PATH")]
        note_file: Option<String>,
        /// Finish a done request cleanly: remove its `requested` note instead
        /// of adding a `picked up` one, so the yak shows only the result (git
        /// keeps the record). Refuses a yak awaiting an agent because of an
        /// answer.
        #[arg(long = "drop-request", conflicts_with_all = ["note", "note_file"])]
        drop_request: bool,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Hand a yak to an agent: set `needs: agent` and record the request.
    ///
    /// For work a human wants done with no question before it (tidy this,
    /// summarize that). Records a `requested: needs agent` note with the
    /// request, so the yak shows in `inbox --for agent` marked `requested`
    /// until an agent runs `pickup`. On a yak already `needs: agent` it keeps
    /// the state and appends the request. Refuses a yak awaiting a human:
    /// `answer` that instead.
    ///
    /// `--kind tidy|summarize` records a canned chore: the note is the kind's
    /// full instruction (first line `Tidy request: ...` or `Summarize request:
    /// ...`), then `--note` (if any) after a blank line. Give `--kind`,
    /// `--note`, or both.
    #[command(group(clap::ArgGroup::new("what").required(true).multiple(true).args(["kind", "note", "note_file"])))]
    Request {
        id: String,
        /// A canned request whose instruction text is written into the note:
        /// `tidy` (reshape the body so it renders as CommonMark and scans
        /// easily, adding missing code formatting, keeping what it says) or `summarize` (append one summary
        /// note).
        #[arg(long, value_enum)]
        kind: Option<store::RequestKind>,
        /// What the agent should do (recorded as an attributed note; after
        /// the kind's instruction with `--kind`). `-` reads it from stdin.
        #[arg(long, conflicts_with = "note_file")]
        note: Option<String>,
        /// Read the request from this file (`-` is stdin).
        #[arg(long = "note-file", value_name = "PATH")]
        note_file: Option<String>,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// List yaks awaiting a human or awaiting an agent.
    ///
    /// Lists yaks with a `needs` value, across all statuses, in two sections:
    /// awaiting a human (`needs: human`; `replied` marks one whose latest note
    /// after the ask is by someone other than the asker) and awaiting an agent
    /// (`needs: agent`; each marked `answered` or `requested`, from the latest
    /// ask/request note). `--for human|agent` shows one section.
    Inbox {
        #[command(flatten)]
        filter: FilterFlags,
        /// Show only one section.
        #[arg(long = "for", value_enum)]
        audience: Option<Audience>,
        /// JSON array of yaks, each with `needs`, `replied` and `why`
        /// (`answered`|`requested` when awaiting an agent, else null).
        #[arg(long)]
        json: bool,
    },
    /// Block until the inbox has something, then print it like `inbox`.
    ///
    /// Polls the farm files (no lock, no daemon) every `--interval` until the
    /// inbox for `--for human|agent` (both sections when omitted), narrowed by
    /// the filter flags, is non-empty; then prints exactly what `yaks inbox`
    /// prints for the same flags and exits 0. On `--timeout` it prints nothing
    /// on stdout, a one-line message on stderr, and exits 2. Run it as a
    /// background task to be woken when a question is answered or a request
    /// arrives (`yaks wait --for agent --timeout 10m`).
    Wait {
        #[command(flatten)]
        filter: FilterFlags,
        /// Wait on one section only.
        #[arg(long = "for", value_enum)]
        audience: Option<Audience>,
        /// Give up after this long (`30s`, `10m`, `2h`; `0s` checks once).
        /// Omitted: wait forever.
        #[arg(long, value_name = "DUR")]
        timeout: Option<String>,
        /// Ignore yaks already in the inbox when the wait starts; a yak whose
        /// `needs` value changes (human to agent) counts as new.
        #[arg(long)]
        new: bool,
        /// How often to re-read the farm (a tuning knob; whole units, `1s` up).
        #[arg(long, value_name = "DUR", default_value = "2s")]
        interval: String,
        /// Print the hit as `inbox --json` does.
        #[arg(long)]
        json: bool,
    },
    /// Start shaving one or more yaks (move to shaving).
    ///
    /// Also shaves each hairy ancestor (its parent if hairy, that one's parent
    /// if hairy, ...): starting part of the work starts the whole. Each prints
    /// as `Shaving <parent> (parent of <id>)`, and its move entry carries
    /// `child: <id>`.
    ///
    /// Refuses a yak whose parent is shorn or dead: reopen the parent first
    /// (`yaks regrow`/`revive`; one call with both takes parents first), or move
    /// the yak out of the family with `yaks reparent`.
    #[command(visible_alias = "work")]
    Shave {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then `git config yaks.actor`, then the harness
        /// identity, then the git user (`yaks whoami` shows which).
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Mark one or more yaks shorn (move to shorn).
    ///
    /// Refuses a yak with hairy or shaving children: a shorn yak's children
    /// are all shorn or dead. Shear them in the same call (children are taken
    /// before their parents, in any order given), or graduate them out of the
    /// family first with `yaks reparent <child> --parent <other>` or
    /// `--unparent`. A refused id exits non-zero after the rest of the batch.
    #[command(visible_alias = "close")]
    Shorn {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then `git config yaks.actor`, then the harness
        /// identity, then the git user (`yaks whoami` shows which).
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Regrow one or more shorn yaks (move back to hairy).
    ///
    /// Refuses a yak whose parent is shorn or dead: regrow the parent too (a
    /// batch takes parents before children, in any order given), or move the
    /// yak out of the family with `yaks reparent`.
    #[command(visible_alias = "reopen")]
    Regrow {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then `git config yaks.actor`, then the harness
        /// identity, then the git user (`yaks whoami` shows which).
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Kick off a scheduling round: reopen what is due, then say what needs you.
    ///
    /// Runs `regrow-due` (a recurring yak whose next occurrence has arrived
    /// goes back to hairy; one with `wake: agent|human` also lands in the
    /// inbox), then prints one report: what regrew or was held, what is
    /// overdue or due soon, what awaits a human or an agent, stale claims,
    /// and how many findings `yaks check` has. Idempotent: a second tick
    /// regrows nothing. It never syncs, runs no commands and posts no
    /// notifications; `--dry-run` writes nothing and `--json` is the stable
    /// shape for a harness. Exit 0 unless a write failed.
    Tick {
        /// Report what would regrow without changing anything.
        #[arg(long)]
        dry_run: bool,
        /// How far ahead "due soon" looks: a number and `m`/`h`/`d`/`w`.
        #[arg(long, value_name = "SPAN", default_value = tick::DEFAULT_WITHIN)]
        within: String,
        #[arg(long)]
        json: bool,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Put off one or more open yaks: move `due:` and leave a note saying so.
    ///
    /// `--to` takes a date (`2026-11-15`), an RFC 3339 datetime, or `+2d` /
    /// `+1w` counted from today; without it, a day from today. A shorn or dead
    /// yak has nothing to put off and is refused. Every id is tried; the exit
    /// is non-zero if any was refused or missing.
    Snooze {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// The new deadline: a date, an RFC 3339 datetime, or `+Nd` / `+Nw`
        /// from today (default `+1d`).
        #[arg(long, value_name = "WHEN", default_value = "+1d")]
        to: String,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Reopen the recurring yaks whose next occurrence has arrived.
    ///
    /// A shorn yak with `every:` (an ISO 8601 duration) regrows once the
    /// latest of `start + k * every` (k >= 1) is not in the future, counting
    /// from its `regrown:` / `due:` / shear time: it goes back to hairy,
    /// `due:` becomes that occurrence plus one period, `regrown:` the time
    /// now, and a `kind: regrow` note says which occurrence and how many were
    /// skipped. Idempotent, so `yaks tick`, a harness or the TUI can call it as
    /// often as they like. Dead yaks never regrow; a yak under a shorn parent
    /// reopens the parent too. A yak with `wake: agent|human` also gets
    /// `needs:` set (and a request or ask note), so it lands in the inbox.
    /// One commit per yak.
    #[command(name = "regrow-due")]
    RegrowDue {
        /// List what would regrow without changing anything.
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        json: bool,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Slaughter one or more yaks (move to dead).
    ///
    /// Refuses a yak that still has hairy or shaving descendants, since a dead
    /// yak never holds open work; pass --family to slaughter those along with
    /// it, or graduate them first (`yaks reparent`). Shorn descendants are
    /// finished work: they never block it and stay shorn.
    Slaughter {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Also slaughter every hairy or shaving descendant (children,
        /// grandchildren, ...) of each id, instead of refusing. Shorn ones stay
        /// shorn.
        #[arg(long)]
        family: bool,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then `git config yaks.actor`, then the harness
        /// identity, then the git user (`yaks whoami` shows which).
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Revive one or more dead yaks (move back to hairy).
    ///
    /// Refuses a yak whose parent is shorn or dead, like `regrow`: reopen the
    /// parent too (parents go first), or move the yak with `yaks reparent`.
    Revive {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then `git config yaks.actor`, then the harness
        /// identity, then the git user (`yaks whoami` shows which).
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Add or remove a dependency.
    Dep {
        #[command(subcommand)]
        action: DepAction,
    },
    /// Print only what a link names: a line range, a markdown section, or a
    /// document's frontmatter and first paragraph.
    ///
    /// The link is written as in a yak or dok: `/doks/x.md#setup` (a dok's
    /// section), `code:src/foo.rs#L12-L40` (lines of a code file, from the
    /// code checkout yaks was started in), `code:src/foo.rs@3f2a9c1#L12` (as
    /// at a commit), `yaks-1a2b/log.txt#L5` (an attachment),
    /// `yaks-1a2b#2026-11-04T22:13:03Z` (one note of a yak, by its timestamp;
    /// `-1`, `-2` for later notes of the same second). Cheaper context than
    /// reading the whole file.
    Peek {
        link: String,
        /// Widen a line range by this many lines either side.
        #[arg(long, default_value_t = 0)]
        context: u32,
    },
    /// What the farm costs to clone: its commits, history size, the bytes a
    /// clone of this repository fetches for it, the current tree by kind
    /// (yaks, attachments, doks, sources), and the largest blobs ever
    /// committed.
    ///
    /// Every branch comes with a plain `git clone`, so a code clone pays for
    /// the farm branch's whole history. Read-only.
    Weight {
        /// Machine output.
        #[arg(long)]
        json: bool,
    },
    /// Store the farm's binary originals (images, PDFs, office files, audio,
    /// video, archives under `.yaks/`) in Git LFS from now on.
    ///
    /// Writes the rules into the farm branch's `.gitattributes`, commits only
    /// that file, and installs LFS's hooks for this repository (`git lfs
    /// install --local`), so a clone of the repository fetches small pointers
    /// and the bytes come down only where the farm is checked out. Binaries
    /// already in the tree are converted in the same commit; older history
    /// keeps their bytes. Needs `git-lfs` installed, and a remote that
    /// accepts LFS (GitHub does). Idempotent.
    Lfs,
    /// List farm documents holding a merge conflict to settle.
    ///
    /// The merge driver never fails a merge on prose: where both sides changed
    /// the same lines of a yak's description or a dok body, both versions stay
    /// in the text between `<<<<<<< here` and `>>>>>>> other side`. A dok
    /// field both sides set differently keeps this side's value and records
    /// the other under `merge_conflicts:` (a yak gets a `merge-driver` note
    /// instead). Edit the file to settle it; `doctor` reports the same list.
    /// Exits 1 when there is anything to settle.
    Conflicts {
        /// Machine output: `[{path, marker_lines, fields}]`.
        #[arg(long)]
        json: bool,
    },
    /// Revert farm commits: the given ones, or every commit `--by` an actor
    /// `--since` a time.
    ///
    /// Every write commits on the farm branch with a `Yaks-Actor` trailer,
    /// so a session's changes can be found and undone together. The revert
    /// is one new commit (nothing is rewritten), merged through the yak merge
    /// driver. Refuses while the farm has uncommitted changes, or when a later
    /// change touched the same lines. With no commits named, lists what it
    /// would undo; `--yes` does it.
    Undo {
        /// Farm commits to revert (as `git log` shows them in the farm).
        commits: Vec<String>,
        /// Every commit whose `Yaks-Actor` is this name.
        #[arg(long)]
        by: Option<String>,
        /// Only commits newer than this (`2h`, `1d`, an ISO date).
        #[arg(long)]
        since: Option<String>,
        /// With --by/--since: revert them (without it, only list).
        #[arg(long)]
        yes: bool,
    },
    /// Move a dok and rewrite every link to it, in yaks and doks.
    ///
    /// Links are rewritten to the farm-root form (`/doks/new/path.md`),
    /// keeping their `@rev` and `#anchor`. `--dry-run` lists what would
    /// change.
    Mv {
        from: String,
        to: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Save outside material as a source, or list the farm's sources.
    ///
    /// A source is a snapshot under `sources/<date>-<sha8>/`: the
    /// original, `extracted.md` (its text, for text files), and `source.md`
    /// saying what it is and where it came from. Cite its `source.md` from a
    /// dok's `sources:` or link it from a yak. yaks fetches nothing and
    /// converts nothing: save a downloaded copy with `--url` naming the dok,
    /// and your conversion of it with `--text`, as one source. Material that
    /// changes under a stable URL is updated in place with `add --refresh`.
    Source {
        #[command(subcommand)]
        action: SourceAction,
    },
    /// Session notes: a short account of a working session, under
    /// `sessions/`, for a later review and reflection to learn from.
    ///
    /// `note` saves one at the end of a session (the `yaks-upkeep` skill says
    /// when and what goes in it); `touched` lists what you wrote in the farm
    /// during the session, as links to your yak notes and doks; `list
    /// --unreviewed` and `promote` are a review's two mechanical halves.
    Session {
        #[command(subcommand)]
        action: SessionAction,
    },
    /// Move a yak's attachment into the farm's knowledge: a source, or (a
    /// markdown attachment, with `--dok <path>`) a dok. Every link to the
    /// attachment is rewritten to the new place.
    Promote {
        /// The attachment, `<id>/<file>` (as the yak links it).
        attachment: String,
        /// Make it a dok at this path (`doks/topics/x.md`) instead of a source.
        #[arg(long)]
        dok: Option<String>,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Work with the farm's doks: markdown documents that are not yaks.
    ///
    /// A dok lives under `doks/` (farm-wide) or `<herd>/doks/` (one herd's),
    /// at any depth, and is named by its path from the farm root
    /// (`doks/topics/vector-search.md`; a leading `/` is accepted). Its
    /// frontmatter has `type:` (note, topic, howto, summary, or a typed kind),
    /// `title:`, and any other fields (`description`, `tags`, `sources`, ...).
    /// Doks are plain files: editing one by hand is fine, and `yaks doctor`
    /// checks them.
    Dok {
        #[command(subcommand)]
        action: DokAction,
    },
    /// Move tasks under a new parent (--parent) or to top-level (--unparent).
    ///
    /// Every id is reparented to the same destination. A hairy or shaving yak
    /// never goes under a shorn or dead one (a finished yak holds no open
    /// children); this is also how a parent that is done graduates its
    /// unfinished children before it is shorn.
    Reparent {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        unparent: bool,
    },
    /// Apply one field edit (and/or reparent) to every yak matching a filter.
    ///
    /// DRY-RUN BY DEFAULT: without --commit it only prints the matched
    /// set and the mutation, changing nothing. Requires at least one filter flag
    /// (never operates on the whole farm) and at least one mutation flag. Field
    /// edits, reparent, and `--request` only — no state transitions (see
    /// yaks-7cc8).
    ///
    /// `--request tidy|summarize` queues that canned request on every matched
    /// yak, exactly as `yaks request <id> --kind <kind>` would; it combines
    /// with the field edits or stands alone. A yak awaiting a human is skipped
    /// and reported (answer it instead); a skip is not a failure.
    Bulk {
        #[command(flatten)]
        filter: FilterFlags,
        /// Tags to add to every matched yak. Commas and spaces separate tags.
        /// `--add-label` is the old spelling.
        #[arg(long = "add-tag", alias = "add-label", num_args = 1..)]
        add_tag: Vec<String>,
        /// Tags to remove from every matched yak. Commas and spaces separate
        /// tags. `--remove-label` is the old spelling.
        #[arg(long = "remove-tag", alias = "remove-label", num_args = 1..)]
        remove_tag: Vec<String>,
        /// Set the priority of every matched yak.
        #[arg(long = "set-priority")]
        set_priority: Option<u8>,
        /// Set the type of every matched yak.
        #[arg(long = "set-type")]
        set_type: Option<String>,
        /// Reparent every matched yak under this id.
        #[arg(long)]
        reparent: Option<String>,
        /// Reparent every matched yak to top-level.
        #[arg(long)]
        unparent: bool,
        /// Queue this canned request on every matched yak (as `request
        /// --kind`): `tidy` or `summarize`.
        #[arg(long, value_enum, value_name = "KIND")]
        request: Option<store::RequestKind>,
        /// Extra text recorded after the request's instruction (needs
        /// `--request`). `-` reads it from stdin.
        #[arg(long, requires = "request")]
        note: Option<String>,
        /// Actually apply the mutation. Without it, this is a dry run.
        #[arg(long)]
        commit: bool,
    },
    /// Rename a yak, updating its file, id and every reference to it.
    ///
    /// References (parent, depends_on, and body/title mentions) are rewritten
    /// across the farm.
    Rename {
        old: String,
        new: String,
        /// Preview the change without writing anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Rename a whole herd: move every yak to a new id prefix.
    ///
    /// Migrates every yak from one id prefix to another
    /// (e.g. yaksrs -> yaks), rewriting all references and updating the farm's
    /// configured herd. (`rename-prefix` still works as an alias.)
    #[command(name = "rename-herd", alias = "rename-prefix")]
    RenamePrefix {
        old: String,
        new: String,
        /// Preview the change without writing anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Bring an older-schema farm up to this yaks' schema, or move it to a branch.
    ///
    /// Brings a farm on an older schema up to this yaks' (schema 4, 5, 6 or 7 -> 8):
    /// moves every yak file and attachment dir from the farm root into its
    /// herd's directory (`<herd>/yaks/<id>.md`, `<herd>/yaks/<id>/`), renames
    /// `labels:` to `tags:`, puts the blank line after every comment's
    /// `▸ <ts> [actor]` header, fixes attachment links, and (schema 8) renames
    /// the wiki to doks: `wiki/` and `<herd>/wiki/` become `doks/` and
    /// `<herd>/doks/`, `/wiki/...` links become `/doks/...`, and a schema's
    /// `type: Page Type` becomes `Dok Type`. Works under the
    /// farm lock, then sets `.yaks/schema` to 8. Idempotent: a current farm
    /// reports "nothing to migrate". Every other command refuses an older farm
    /// and names this one. Never commits: it prints the commit to make (a
    /// migration touches nearly every yak file, so land it on its own).
    ///
    /// Once synced, every machine with an older yaks is refused the farm: a
    /// human decides when to migrate (agents ask). Upgrade every machine
    /// first (`min_version:` in config.yaml warns the stragglers), then
    /// migrate once, sync, and restart every running `yaks tui`.
    Migrate {
        /// Print what would change; write nothing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Mark this repository's farm private: it never leaves the machine.
    ///
    /// It never leaves the machine through
    /// yaks or a stray `git push`. Sets `git config yaks.private true` and
    /// installs a `pre-push` hook (in the hooks dir `git` uses, so
    /// `core.hooksPath` is honored and every worktree shares it) that refuses a
    /// push of the farm branch (`yaks`, or `git config yaks.branch`), including
    /// `git push --all`; `yaks sync` commits and pulls but never
    /// pushes it. Pushing
    /// any other branch is unaffected. An existing `pre-push` hook that is not
    /// yaks's is left untouched (the command says how to chain the check).
    ///
    /// `--off` unsets the flag and removes yaks's own hook only. Touches no farm
    /// and no remote.
    Private {
        /// Undo: unset `yaks.private` and remove yaks's `pre-push` hook.
        #[arg(long)]
        off: bool,
    },
    /// Merge another farm's yaks into this one (consolidation).
    ///
    /// Copies every yak
    /// (all statuses) and its artifacts, preserving ids and status, and declares
    /// any incoming herd in this farm's config; refuses on id collisions.
    /// Non-destructive — the source is left intact.
    Merge {
        /// Path to the source farm: a `.yaks/` directory or a dir containing one.
        source: String,
        /// Preview the plan without copying anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Start a farm on its own branch, or join another repo's, and install the skills.
    ///
    /// Every farm is an orphan git branch (`yaks`, or --branch) holding only
    /// `.yaks/`, checked out once per machine at
    /// `~/.local/share/yaks/farms/<repo>-<hash>` (named by the `origin` URL, so
    /// every copy of the repository shares it). A branch that already exists
    /// (locally or on origin) only gets its worktree. Nothing is committed or
    /// pushed; `yaks sync` publishes the farm, `yaks private` keeps it local.
    ///
    ///   yaks init                           # this repo's own farm + skills
    ///   yaks init --farm ../api --herd web  # use ../api's farm, herd `web`
    ///
    /// Idempotent: re-run it to add what is missing. A setting that differs
    /// from what is there is an error, never a silent change.
    #[command(verbatim_doc_comment)]
    Init {
        /// Default herd (id prefix) for new yaks (default: yak). With --farm,
        /// this repo's herd in the shared farm (`git config yaks.herd`).
        #[arg(long = "herd")]
        prefix: Option<String>,
        /// Default task type for new yaks (default: task).
        #[arg(long = "type")]
        kind: Option<String>,
        /// Default priority for new yaks (default: 3).
        #[arg(long)]
        priority: Option<u8>,
        /// Use emacs keybindings in embedded editors instead of vim.
        #[arg(long)]
        emacs: bool,
        /// The farm branch's name (default: `git config yaks.branch`, else
        /// `yaks`). A non-default name is recorded in `git config yaks.branch`.
        #[arg(long, value_name = "NAME")]
        branch: Option<String>,
        /// Use the farm of another repository (any checkout of it) instead of
        /// starting one: sets `git config yaks.farm`. That repo needs a farm.
        #[arg(long, value_name = "CHECKOUT")]
        farm: Option<String>,
        /// Do not install the skills. By default they go project-local in the
        /// git top-level, through the same install as `yaks skills install`.
        #[arg(long)]
        no_skills: bool,
    },
    /// Install or check the bundled agent skills.
    ///
    /// Installs the bundled agent skills (yaks, yaks-tracker, yaks-coordinating,
    /// yaks-working) into a skills directory. Works anywhere — no farm required.
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },
    /// Group yaks by the external issue they roll up to.
    Rollup(RollupArgs),
    /// Read-only farm-integrity check; exits non-zero on any issue.
    ///
    /// Reports an invalid `status:` value, one
    /// id in two yak files, dangling parent/depends_on references, and a
    /// shorn or dead yak over hairy or shaving children, among others (see
    /// docs/cli.md). Exits non-zero when any issue is found.
    Doctor {
        /// Emit the issues as JSON.
        #[arg(long)]
        json: bool,
        /// Also flag shorn yaks with no recorded note, and shorn yaks whose
        /// `verify:` command did not last PASS (evidence-before-shear). Dead
        /// (abandoned) yaks are exempt.
        #[arg(long)]
        strict: bool,
    },
    /// Read-only upkeep check on the farm's doks and sources.
    ///
    /// Reports what a person or agent should look at, and never edits: doks
    /// past their `stale_after` (the dok's own, else its type's schema; an
    /// ISO 8601 duration such as `P6M` counted from `verified:`, else the
    /// dok's last commit on the farm branch, or a date), sources that no dok
    /// or yak cites (a `sources/<id>/` link or mention; keep one on
    /// purpose with `yaks source mark`), dok kinds that
    /// differ only by plural or hyphens, and `doctor`'s `broken-link`,
    /// `schema-mismatch`, `merge-conflict` and `dok-problem`. Exits 1 when
    /// there is any finding.
    Check {
        /// Emit the findings as JSON: `[{kind, message, ids}]`.
        #[arg(long)]
        json: bool,
    },
    /// Read-only landing-readiness check.
    ///
    /// Checks that the verify command of each shorn yak in the change last
    /// PASSed, no yak file has an invalid `status:` or shares an id, and the
    /// farm has no unresolved merge conflict. Prints one line per failure and
    /// exits non-zero, else `preflight: ok`. Uncommitted farm changes are one
    /// warning (`run yaks commit`), not a failure. With `--push-main` also
    /// checks that the `local` remote is the human's checkout (not bare, has
    /// `main`, not dirty on `main`) before `git push local <branch>:main`;
    /// never pushes.
    Preflight {
        /// Shorn yaks to check (default: the shorn yaks with uncommitted farm
        /// changes, i.e. modified or new). Scopes the verify check
        /// only; the git and integrity checks are farm-wide.
        ids: Vec<String>,
        /// Check the verify command of EVERY shorn yak, not only those in the
        /// change (old shorn yaks that never ran `verify` will fail).
        #[arg(long)]
        all: bool,
        /// Also check that the `local` remote is the human's checkout, for the
        /// coordinator about to `git push local <branch>:main`: a path on this
        /// machine, a non-bare repo with `refs/heads/main`, and not checked out
        /// on `main` with uncommitted changes (git would refuse the push). Fails
        /// with the `pr/<name>` fallback to use instead. Off by default: it
        /// depends on another checkout's state, not on the change being landed.
        #[arg(long)]
        push_main: bool,
        /// Emit the result as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Sync the farm branch with its remote: commit, rebase, push.
    ///
    /// Commits what is dirty (`yaks
    /// commit`), fetch and rebase onto `<remote>/<branch>` (concurrent edits to
    /// one yak merge into a valid yak through yaks's merge driver, which this
    /// registers), then push, creating the remote branch with the same name on
    /// the first publish. The remote is `branch.<branch>.remote`, else `origin`.
    /// Only for a farm on its own branch; runs in the farm worktree whatever
    /// the cwd. A private farm (`yaks private`) is never pushed. A rebase that
    /// stops on a conflict the driver cannot settle (a non-yak file such as
    /// `config.yaml`) is aborted, leaving the branch as it was, and names the
    /// files. The CLI never syncs by itself; the TUI does, every `sync_interval`
    /// in `config.yaml` (default 5m, `off` disables).
    Sync {
        /// Do not commit first (a dirty farm then skips the pull too).
        #[arg(long)]
        no_commit: bool,
        /// Do not fetch or rebase (commit and push only).
        #[arg(long)]
        no_pull: bool,
        /// Do not push (commit and pull only).
        #[arg(long)]
        no_push: bool,
        /// Message for the commit (else generated, as `yaks commit`).
        #[arg(short, long)]
        message: Option<String>,
    },
    /// Commit the farm's own changes (only `.yaks/`) with a generated message.
    ///
    /// Commits everything under `.yaks/` (yak files, attachments, config) and
    /// nothing else, with a generated message such
    /// as `yaks: shorn yaks-abc1; updated yaks-def2`. Code
    /// staged or modified elsewhere is left alone (staged files stay staged). Runs a normal
    /// `git commit`, so the repo's hooks apply; never pushes. Fails in a private
    /// farm (nothing under `.yaks/` tracked by git) and while a merge or
    /// cherry-pick is in progress (git forbids a partial commit then; finish it
    /// with a plain `git commit`); a farm with no changes is a no-op that exits 0.
    ///
    /// When the farm is a branch (yaks-73f4) this commits in the farm's own
    /// worktree, on that branch, whatever the cwd: no code checkout is touched.
    Commit {
        /// Commit message (default: generated from the changed files).
        #[arg(short, long)]
        message: Option<String>,
        /// Print the files and message without staging or committing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Land a Delta worker's commit by SHA on the current branch.
    ///
    /// Lands it on the current branch of the code
    /// checkout you are in: one single-parent squash commit with the worker's
    /// message and author plus a `Squashed-from: <full sha>` trailer (never a
    /// merge commit), or a `git cherry-pick` when the worker's base is no longer
    /// an ancestor of HEAD or the merge would conflict. Delta's applied state
    /// (a pending merge, the worker's files in the working tree) is not trusted:
    /// it is reported, then cleared, and the result is checked against
    /// `git merge-tree --write-tree`. Refuses (exit 3) when the working tree holds
    /// changes the worker's commit does not explain, unless `--force-discard`;
    /// refuses (exit 6) a SHA that is already an ancestor of HEAD but whose
    /// change is missing from HEAD's tree (a merge that dropped it); refuses (exit
    /// 7) a commit that touches `.yaks/` when HEAD tracks none (a branch farm).
    /// After landing, a note `kind: landed` (`commit:` the landed sha, `source:`
    /// the worker's sha, `branch:`) is appended to every yak the landed
    /// commit's message names, and to each `--yak`; `yaks commits <id>` reads
    /// them. An unknown id is warned about, not fatal; `--dry-run` says what it
    /// would record and writes nothing. Never pushes. Needs git 2.38+ (2.40+ to
    /// check an already-landed SHA); needs no farm, and records nothing without
    /// one.
    Land {
        /// The worker's commit (a SHA or any rev that names a commit).
        sha: String,
        /// Report the pending state, the merged tree, what the working tree holds
        /// that the commit does not explain, and what would happen; change nothing.
        #[arg(long)]
        dry_run: bool,
        /// Land even when the working tree holds changes the worker's commit does
        /// not explain (they are discarded).
        #[arg(long)]
        force_discard: bool,
        /// Also record the landing on this yak (repeatable): a private farm
        /// keeps ids out of commit messages, so name the yak here.
        #[arg(long = "yak", value_name = "ID")]
        yaks: Vec<String>,
    },
    /// Show what the farm has changed that git does not have yet.
    ///
    /// One line per
    /// yak in yak terms (`created`, `moved hairy -> shaving`, `notes +2`,
    /// `edited`, `removed`, `artifacts`; `config` for config.yaml), `*` marking
    /// what is staged, ordered by id, then what `yaks commit` would do (and its
    /// message). Uses the same classification as `yaks commit`. Read-only. Code
    /// changed elsewhere in the repo is only counted, never listed. A clean farm
    /// prints `farm clean: nothing to commit` and exits 0. While a merge or
    /// cherry-pick is in progress it says `yaks commit` will refuse. When the
    /// farm is a branch, a last line says whether that branch is published,
    /// ahead or behind its upstream, as of the last fetch (never fetches).
    Status {
        /// Exit 1 when the farm has uncommitted changes (for scripts and
        /// hooks); the output is the same. Without it the exit code is 0.
        #[arg(long)]
        check: bool,
        /// Emit JSON: `clean`, `untracked`, `merge_in_progress`, `yaks` (array of
        /// `{id, changes, staged}`), `other` (farm files that are not yaks, as
        /// `{path, staged}`), `outside_farm` (count), `message` and `sync`
        /// (`{branch, upstream, ahead, behind}` on a farm branch, else null).
        #[arg(long)]
        json: bool,
    },
    /// Print the worker brief for one yak: what a cold worker needs to start.
    ///
    /// Derived from the yak, the config and the farm. Names the yak and that
    /// it is already `shaving`, the `YAKS_ACTOR=<name>` prefix for every
    /// command (plus `YAKS_DIR` with `--yaks-dir`, or for a farm that is not a
    /// farm branch), this binary's path, the yak's `verify:` gate (else the
    /// config default), the forbidden moves, how to finish (never stage
    /// `.yaks/`; after `yaks private`, no yak ids in commits), how to ask, the
    /// `Decision:` note rule and the final-message format. Read-only. Claim the
    /// yak first (`yaks shave`): an unclaimed yak still prints, with a warning
    /// on stderr. An unknown id fails with nothing on stdout. The coordinator
    /// adds what a yak cannot know (spawn title, landing, model).
    Brief {
        id: String,
        /// The worker's name, used as `YAKS_ACTOR` (e.g. `sheds-1`).
        #[arg(long = "as", value_name = "NAME")]
        as_actor: String,
        /// Print `YAKS_DIR=<PATH>` on every command, for a worker whose checkout
        /// cannot discover the farm. Default: nothing for a farm branch (every
        /// checkout finds it), the farm's path otherwise.
        #[arg(long, value_name = "PATH")]
        yaks_dir: Option<String>,
    },
    /// Open the interactive terminal UI.
    Tui {
        /// Drive the TUI headlessly: read actions on stdin, emit text snapshots
        /// on stdout (for agents / scripted tests). No real terminal is used.
        #[arg(long)]
        headless: bool,
        /// Terminal size for headless mode, e.g. "100x30" (default 80x24).
        #[arg(long)]
        size: Option<String>,
        /// Headless: after the first frame, emit only changed body lines.
        #[arg(long)]
        diff: bool,
    },
}

#[derive(Subcommand)]
enum DepAction {
    /// Add DEP_ID as a dependency of ID.
    Add { id: String, dep_id: String },
    /// Remove DEP_ID as a dependency of ID.
    Remove { id: String, dep_id: String },
}

#[derive(Subcommand)]
enum SessionAction {
    /// Save a session note: `sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md`.
    ///
    /// The body is markdown with `## Summary` (a short narrative tying the
    /// work together), `## Turns` (one `- <verdict>: <reason>` per request,
    /// verdict good, partial, failed or unclear), `## Lessons` (`- none` when
    /// none) and optionally `## Discussion` (exposition no yak holds) and
    /// `## Saved`. Without `## Saved`, yaks writes it from what you wrote in
    /// the farm since `--since` (see `touched`). The frontmatter (actor,
    /// person, host, harness, thread, day, time, verdict counts) is yaks'.
    Note {
        /// A few words naming the session.
        #[arg(long)]
        title: String,
        /// The note's markdown. `-` reads it from stdin.
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the body from this file (`-` is stdin).
        #[arg(long = "body-file")]
        body_file: Option<String>,
        /// Start of the session, for a generated `## Saved` (`2h`, a date or
        /// an RFC 3339 time). Default: your latest session note, at most 12
        /// hours back.
        #[arg(long)]
        since: Option<String>,
        /// The agent app (`delta`, `claude-code`, `pi`, ...). Default: the one
        /// the environment names, if any.
        #[arg(long)]
        harness: Option<String>,
    },
    /// What you wrote in the farm since `--since`: each yak with links to
    /// your notes on it, then the doks, sources and schemas you changed.
    /// Markdown for a note's `## Saved`; `--json` for a harness.
    Touched {
        /// Default: your latest session note, at most 12 hours back.
        #[arg(long)]
        since: Option<String>,
        /// Whose writes (default: you).
        #[arg(long)]
        by: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// List session notes, oldest first: path, day, actor, verdicts, title.
    List {
        /// Only agents' own notes nobody has reviewed yet (no `promoted:`).
        #[arg(long)]
        unreviewed: bool,
        #[arg(long)]
        json: bool,
    },
    /// Mark session notes reviewed: stamp `promoted: <today>` once their
    /// facts worth keeping are in doks. One commit for all of them.
    Promote {
        #[arg(required = true, num_args = 1..)]
        paths: Vec<String>,
    },
}

#[derive(Subcommand)]
enum SourceAction {
    /// Save a file or a small directory (at most 200 files, 5 MiB) as a
    /// source. Saving the same content again reports the existing source;
    /// `--refresh` updates one in place, `--text-only` keeps a markdown file as
    /// its text alone.
    Add {
        path: PathBuf,
        /// A title naming the thing, not the file (default: the file name).
        #[arg(long)]
        title: Option<String>,
        /// The web page this file is a saved copy of: where it came from, in
        /// place of the path of your download.
        #[arg(long)]
        url: Option<String>,
        /// Your conversion of the file to text (a dok's HTML to markdown, a
        /// PDF to text): it becomes `extracted.md`, so one source holds the
        /// original and the text you read. Not for a directory.
        #[arg(long)]
        text: Option<PathBuf>,
        /// `sources/<id>/source.md` of the source this is a part of.
        #[arg(long = "part-of")]
        part_of: Option<String>,
        /// Update in place: when a source already names `--url` as its
        /// resource, replace that source's content (same directory and id,
        /// old hash kept as `previous_sha256`) instead of saving a second
        /// one. Saves a new source when none has the URL. Needs `--url`.
        #[arg(long)]
        refresh: bool,
        /// Keep only `extracted.md`, with no separate copy of the original:
        /// for a markdown or plain-text file that already is the text to
        /// read (an exported document). The original's file name is not kept.
        /// Not for a directory, a non-text file, or together with `--text`.
        #[arg(long = "text-only")]
        text_only: bool,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// List sources: id, media type, title and where each came from.
    List {
        #[arg(long)]
        json: bool,
    },
    /// Mark a source on purpose for `yaks check`: kept without a citation
    /// (`--uncited WHY`), replaced by a newer source (`--superseded-by`), or
    /// `--clear` the mark. Sets the field `uncited:` / `superseded_by:` in
    /// its `source.md` (one mark at a time) as one commit.
    Mark {
        /// The source: its id, `sources/<id>`, or `sources/<id>/source.md`.
        source: String,
        /// Keep it without a citation, and say why.
        #[arg(long, value_name = "WHY", group = "how")]
        uncited: Option<String>,
        /// Replaced by this source (its id or `sources/<id>/source.md`);
        /// it must exist.
        #[arg(long = "superseded-by", value_name = "SOURCE", group = "how")]
        superseded_by: Option<String>,
        /// Remove the mark, so `yaks check` lists the source again when
        /// nothing cites it.
        #[arg(long, group = "how")]
        clear: bool,
    },
}

#[derive(Subcommand)]
enum DokAction {
    /// List doks: path, type and title, sorted by path.
    List {
        /// Only doks under this path (`doks/topics`, `yaks/doks`).
        prefix: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Print one dok: its file text, or `--json` with fields parsed.
    Show {
        path: String,
        #[arg(long)]
        json: bool,
    },
    /// Create a dok. Refuses a path that exists or is not
    /// `doks/<kebab-parts>.md` / `<herd>/doks/<kebab-parts>.md`.
    New {
        path: String,
        /// The dok's kind (default `note`).
        #[arg(long = "type")]
        kind: Option<String>,
        /// Its title (default: from the file name).
        #[arg(long)]
        title: Option<String>,
        /// Any other field, `key=value` (repeatable): `--set 'tags=[a, b]'`.
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,
        /// The body (markdown). `-` reads it from stdin.
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Read the body from this file (`-` is stdin).
        #[arg(long = "body-file", value_name = "PATH")]
        body_file: Option<String>,
    },
    /// Set or remove frontmatter fields of a dok, or replace its body.
    Set {
        path: String,
        /// `key=value` (repeatable); an empty value removes the field.
        #[arg(long = "set", value_name = "KEY=VALUE")]
        set: Vec<String>,
        /// Remove a field (repeatable).
        #[arg(long = "unset", value_name = "KEY")]
        unset: Vec<String>,
        /// Replace the body (markdown). `-` reads it from stdin.
        #[arg(long, conflicts_with = "body_file")]
        body: Option<String>,
        /// Replace the body with this file's text (`-` is stdin).
        #[arg(long = "body-file", value_name = "PATH")]
        body_file: Option<String>,
    },
    /// The kinds of dok: the untyped ones and every typed kind with a
    /// schema (`schemas/<type>.md`), with its fields.
    Types,
    /// Write or extend a typed kind's schema, `schemas/<kind>.md`, in the
    /// shape `dok types` reads: `--field name=type` (repeatable; `name=type!`
    /// is required; types text, number, bool, date, datetime, money, link,
    /// list). An existing schema keeps every field, key and its prose: new
    /// fields are added, and a field whose type or required flag differs is
    /// refused unless `--replace`. One commit.
    Define {
        /// The kind's name: lowercase letters, digits and `-`.
        kind: String,
        /// A field, `name=type` or `name=type!` for a required one
        /// (repeatable).
        #[arg(long = "field", value_name = "NAME=TYPE[!]")]
        field: Vec<String>,
        /// The kind's title (a new schema defaults to the kind's name).
        #[arg(long)]
        title: Option<String>,
        /// What the kind is for.
        #[arg(long)]
        description: Option<String>,
        /// How long a dok of this kind stays current: `P6M`, `P1Y`, `P2W` or
        /// a date. `yaks check` reports doks past it.
        #[arg(long = "stale-after", value_name = "WHEN")]
        stale_after: Option<String>,
        /// Allow changing the type or required flag of a field the schema
        /// already has (its default and description stay).
        #[arg(long)]
        replace: bool,
    },
    /// List doks whose fields match: `--type T`, `--where key=value`
    /// (repeatable, ANDed; a list field matches when it holds the value).
    /// `--rows` lists the rows of matching table doks instead (the first
    /// markdown table in each, one row per line), filtered by the same
    /// `--where` tests on the row's columns.
    Query {
        /// List table rows instead of doks.
        #[arg(long)]
        rows: bool,
        #[arg(long = "type")]
        kind: Option<String>,
        #[arg(long = "where", value_name = "KEY=VALUE")]
        where_: Vec<String>,
        /// Only doks under this path.
        #[arg(long)]
        prefix: Option<String>,
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum SkillsAction {
    /// Write the bundled agent skills into a skills directory.
    ///
    /// Always the whole set: yaks, yaks-tracker, yaks-coordinating and
    /// yaks-working. By default they go to ./.agents/skills in the git
    /// top-level of the current directory, so the skills travel with the
    /// project; outside a git repo, to ~/.agents/skills. Prints the destination
    /// it chose. Works without a farm.
    Install {
        /// Target skills directory (e.g. ~/.claude/skills). Wins over --user
        /// and the git-repo default.
        #[arg(long, conflicts_with = "user")]
        dir: Option<String>,
        /// Install into the user directory, ~/.agents/skills, even inside a git repo.
        #[arg(long)]
        user: bool,
        /// Overwrite skills that were edited after they were installed. (Never
        /// permits writing into yaks' own .agents/skills/ source.)
        #[arg(long)]
        force: bool,
    },
    /// Report whether each installed skill is current, stale, or edited.
    ///
    /// Compares each skill's provenance stamp against this binary. Looks in the
    /// same default directory as `install`.
    Status {
        /// Skills directory to inspect. Defaults like `install`: the git
        /// top-level's .agents/skills, else ~/.agents/skills.
        #[arg(long, conflicts_with = "user")]
        dir: Option<String>,
        /// Inspect the user directory, ~/.agents/skills.
        #[arg(long)]
        user: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Git's merge driver runs inside a merge, on temp files: no `-C`, no farm,
    // no skill top-up. Handled before anything else touches the checkout.
    if let Command::MergeDriver {
        base,
        ours,
        theirs,
        path,
    } = &cli.command
    {
        match mergedriver::run(base, ours, theirs, path) {
            Ok(0) => return Ok(()),
            Ok(code) => std::process::exit(code),
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(2);
            }
        }
    }

    // `-C`: change directory once, before anything reads the cwd, so every
    // `env::current_dir()` below (discovery, init, skills) agrees.
    // Each step is relative to the previous one, as in git.
    for dir in &cli.chdir {
        env::set_current_dir(dir)
            .with_context(|| format!("cannot change to directory '{}'", dir.display()))?;
    }

    // `init` creates a farm where none exists, and `skills` installs the skill
    // that teaches an agent how to use yaks (likely before any farm exists) —
    // both must run without opening a farm, so handle them first.
    if let Command::Init {
        prefix,
        kind,
        priority,
        emacs,
        branch,
        farm,
        no_skills,
    } = &cli.command
    {
        return init::run(init::Args {
            herd: prefix.clone(),
            kind: kind.clone(),
            priority: *priority,
            emacs: *emacs,
            branch: branch.clone(),
            farm: farm.clone(),
            no_skills: *no_skills,
        });
    }
    if let Command::Skills { action } = &cli.command {
        return run_skills(action);
    }
    // `whoami` needs no farm (it only uses one, if found, to warn on reuse).
    if let Command::Whoami { set } = &cli.command {
        return run_whoami(set.as_deref());
    }
    // The CLI (never the TUI) says once when a write's actor was derived.
    if !matches!(cli.command, Command::Tui { .. }) {
        actor::warn_on_derived();
    }
    // `private` marks the repository, not a farm: it needs none.
    if let Command::Private { off } = &cli.command {
        match privatefarm::run(&env::current_dir()?, *off) {
            Ok(report) => print!("{report}"),
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    // `land` runs in the code checkout and needs no farm.
    if let Command::Land {
        sha,
        dry_run,
        force_discard,
        yaks,
    } = &cli.command
    {
        std::process::exit(land::cli(&land::Opts {
            sha: sha.clone(),
            dry_run: *dry_run,
            force_discard: *force_discard,
            yaks: yaks.clone(),
        }));
    }
    // Keep the user-level skills current. The overwhelmingly common failure is
    // an agent running against a stale skill, and nobody remembers to re-run
    // the installer — so ordinary commands top it up. Strictly narrow: only an
    // absent or cleanly-outdated skill is written (never a local edit, never a
    // downgrade), only ~/.agents/skills, and never when YAKS_SKILLS_AUTOSYNC is
    // off. `skills` and `init` are handled above, so they never trigger it.
    for name in skills::auto_sync() {
        eprintln!(
            "note: updated the {name} skill for yaks {}",
            skills::version()
        );
    }
    // Project-local skills are never written behind your back, so say so once
    // per yaks version per checkout when they are stale. Not for `--json`
    // (machine consumers); `skills` and `init` returned above.
    if !env::args_os().any(|a| a == "--json") {
        if let Some(msg) = skills::project_stale_notice(&env::current_dir()?) {
            eprintln!("note: {msg}");
        }
    }

    // `migrate` is the one command that opens a farm behind the current schema.
    let open = if matches!(cli.command, Command::Migrate { .. }) {
        Farm::open_for_migration
    } else {
        Farm::open
    };
    let farm = match open(&env::current_dir()?) {
        Ok(h) => h,
        Err(OpenError::SchemaTooNew { found, supported }) => {
            eprintln!(
                "error: this farm uses schema v{found}, newer than this yaks ({}) supports (v{supported}). Upgrade with: {}",
                skills::version(),
                store::UPGRADE_HINT
            );
            std::process::exit(1);
        }
        Err(OpenError::OldLayout(m) | OpenError::NeedsMigration(m)) => {
            eprintln!("error: {m}");
            std::process::exit(1);
        }
        Err(OpenError::NoFarm(m)) => {
            eprintln!("error: {m}");
            std::process::exit(1);
        }
    };
    // The farm's `min_version:` (set by whoever plans a schema bump): warn an
    // older binary on every command, so it upgrades before the bump refuses it.
    if !env::args_os().any(|a| a == "--json") {
        if let Some(msg) = store::min_version_notice(farm.root()) {
            eprintln!("warning: {msg}");
        }
    }

    match cli.command {
        Command::List {
            filter,
            all,
            due,
            before,
            json,
        } => {
            let mut rows = farm.list(build_spec(filter), all)?;
            if due {
                let before = before
                    .as_deref()
                    .map(|b| {
                        recur::When::parse(b).map(|w| w.at).ok_or_else(|| {
                            anyhow::anyhow!(
                                "--before {b:?}: expected a date (2026-11-15) or an RFC 3339 datetime"
                            )
                        })
                    })
                    .transpose()?;
                rows = recur::due_rows(rows, before);
            }
            render_rows(&rows, json, "No tasks found.")?;
        }
        Command::Search {
            query,
            filter,
            json,
        } => {
            let mut spec = build_spec(filter);
            spec.search = Some(query);
            let rows = farm.list(spec, false)?;
            render_rows(&rows, json, "No tasks found.")?;
        }
        Command::Find {
            pattern,
            regex,
            scopes,
            kind,
            case_sensitive,
            ignore_case: _,
            json,
        } => {
            let fold = !case_sensitive;
            let pattern = if regex {
                find::Pattern::regex(&pattern, fold)
            } else {
                find::Pattern::literal(&pattern, fold)
            }
            .map_err(|e| anyhow::anyhow!("invalid pattern: {e}"))?;
            let kinds = kind
                .iter()
                .map(|k| match k.as_str() {
                    "yaks" => find::Kind::Yaks,
                    "doks" => find::Kind::Doks,
                    "sources" => find::Kind::Sources,
                    _ => find::Kind::Sessions,
                })
                .collect();
            let hits = find::find(
                farm.root(),
                &find::Options {
                    pattern,
                    scopes,
                    kinds,
                },
            );
            if json {
                json::print(&find::to_json(&hits))?;
            } else {
                print!("{}", find::render_text(&hits));
            }
            if hits.is_empty() {
                std::process::exit(1);
            }
        }
        Command::Log {
            filter,
            since,
            by,
            json,
        } => {
            let entries = farm.log(build_spec(filter), since.as_deref(), by.as_deref())?;
            render_log(&entries, json)?;
        }
        Command::Show { id, json } => {
            let Some(shown) = farm.show(&id)? else {
                eprintln!("no such task: {id}");
                std::process::exit(1);
            };
            if json {
                json::print(&json::show_value(&shown.task, &shown.children))?;
            } else {
                render_show(&shown);
            }
        }
        Command::Path { ids, filter, all } => {
            let code = run_path(
                &farm,
                &ids,
                filter,
                all,
                &mut std::io::stdout().lock(),
                &mut std::io::stderr().lock(),
            )?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Command::Rename { old, new, dry_run } => report_rename(farm.rename(&old, &new, dry_run)?),
        Command::RenamePrefix { old, new, dry_run } => {
            report_rename(farm.rename_prefix(&old, &new, dry_run)?)
        }
        Command::Merge { source, dry_run } => {
            match farm.merge(std::path::Path::new(&source), dry_run)? {
                MergeOutcome::NoSource(p) => {
                    eprintln!("error: no .yaks/ farm found at {p}");
                    std::process::exit(1);
                }
                MergeOutcome::Collision(ids) => {
                    eprintln!(
                        "error: {} id(s) exist in both farms; reconcile the source's herd with `yaks rename-herd` first:",
                        ids.len()
                    );
                    for id in ids {
                        eprintln!("  {id}");
                    }
                    std::process::exit(1);
                }
                MergeOutcome::Done(plan) => {
                    let verb = if plan.applied {
                        "merged"
                    } else {
                        "would merge"
                    };
                    println!(
                        "{verb} {} yak(s) from {}",
                        plan.yaks.len(),
                        plan.source.display()
                    );
                    for (id, st) in &plan.yaks {
                        println!("  {id} [{}]", st.as_str());
                    }
                    if !plan.artifacts.is_empty() {
                        let a = if plan.applied { "copied" } else { "to copy" };
                        println!("{} artifact set(s) {a}", plan.artifacts.len());
                    }
                    if !plan.herds.is_empty() {
                        let h = if plan.applied {
                            "declared herd(s)"
                        } else {
                            "would declare herd(s)"
                        };
                        println!("{h}: {}", plan.herds.join(", "));
                    }
                    if plan.applied {
                        println!("source left intact at {}", plan.source.display());
                    } else {
                        println!("(dry run — nothing written; re-run without --dry-run to apply)");
                    }
                }
            }
        }
        Command::Migrate { dry_run, .. } => report_migrate(&farm.migrate(dry_run)?),
        Command::Init { .. } => unreachable!("init is handled before opening a farm"),
        Command::Skills { .. } => unreachable!("skills is handled before opening a farm"),
        Command::Whoami { .. } => unreachable!("whoami is handled before opening a farm"),
        Command::Private { .. } => unreachable!("private is handled before opening a farm"),
        Command::Land { .. } => unreachable!("land is handled before opening a farm"),
        Command::MergeDriver { .. } => {
            unreachable!("merge-driver is handled before opening a farm")
        }
        Command::Refs { id, incoming, json } if doks::is_dok_path(&doks::normalize(&id)) => {
            run_dok_refs(&farm, &id, incoming, json)?
        }
        Command::Peek { link, context } => {
            let cwd = env::current_dir()?;
            let code = links::code_base(farm.root(), Some(&cwd));
            let ids = store::all_ids(farm.root());
            let parsed = links::parse(&link, links::From::Cli, |s| ids.contains(s));
            let bases = links::Bases {
                farm: farm.root(),
                code: code.as_deref(),
            };
            match links::peek(&parsed, &bases, context) {
                Ok(p) => {
                    println!("── {}", p.label);
                    print!("{}", p.text);
                }
                Err(e) => {
                    eprintln!("error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Command::Weight { json } => {
            let r = weight::report(farm.root())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&r.to_json())?);
            } else {
                print!("{}", r.render());
            }
        }
        Command::Lfs => match weight::enable_lfs(farm.root())? {
            weight::LfsOutcome::Enabled { commit, converted } => println!(
                "lfs: binary originals under .yaks/ now go to Git LFS (commit {commit}: \
                 .gitattributes, and {converted} file(s) already in the tree converted to pointers; \
                 older history keeps their bytes)"
            ),
            weight::LfsOutcome::Already => println!("lfs: already on"),
        },
        Command::Conflicts { json } => {
            let found = conflicts::scan(farm.root());
            if json {
                let out: Vec<serde_json::Value> = found
                    .iter()
                    .map(|c| {
                        serde_json::json!({
                            "path": c.path,
                            "marker_lines": c.marker_lines,
                            "fields": c.fields,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else if found.is_empty() {
                println!("no merge conflicts");
            } else {
                for c in &found {
                    println!("{}: {}", c.path, farm::conflict_summary(c));
                }
            }
            if !found.is_empty() {
                std::process::exit(1);
            }
        }
        Command::Undo {
            commits,
            by,
            since,
            yes,
        } => {
            let shas = if commits.is_empty() {
                if by.is_none() && since.is_none() {
                    anyhow::bail!(
                        "name the commits to undo, or select them with --by and/or --since"
                    );
                }
                let since = since
                    .as_deref()
                    .map(|s| store::parse_since(s, chrono::Utc::now()).map(|t| t.timestamp()))
                    .transpose()?;
                let list = autocommit::farm_commits(farm.root(), by.as_deref(), since)?;
                if list.is_empty() {
                    println!("No farm commits match.");
                    return Ok(());
                }
                for c in &list {
                    println!(
                        "  {}  {}  [{}]",
                        &c.sha[..7],
                        c.subject,
                        c.actor.as_deref().unwrap_or("-")
                    );
                }
                if !yes {
                    println!("{} commit(s); add --yes to undo them.", list.len());
                    return Ok(());
                }
                list.into_iter().map(|c| c.sha).collect()
            } else {
                commits
            };
            let _lock = store::lock_uncommitted(farm.root())?;
            let new = autocommit::revert(farm.root(), &shas)?;
            println!("Undid {} commit(s) as {new}", shas.len());
        }
        Command::Mv { from, to, dry_run } => {
            let changed = farm.dok_move(&from, &to, dry_run)?;
            let verb = if dry_run { "Would move" } else { "Moved" };
            println!(
                "{verb} {} -> {}",
                doks::normalize(&from),
                doks::normalize(&to)
            );
            for (doc, n) in &changed {
                let s = if *n == 1 { "" } else { "s" };
                println!("  {doc}: {n} link{s}");
            }
        }
        Command::Refs {
            id,
            incoming: true,
            json,
        } => match farm.backrefs(&id)? {
            None => {
                eprintln!("no such task: {id}");
                std::process::exit(1);
            }
            Some(r) if json => json::print(&backrefs_json(&r))?,
            Some(r) => render_backrefs(&r),
        },
        Command::Refs { id, .. } => match farm.refs(&id)? {
            None => {
                eprintln!("no such task: {id}");
                std::process::exit(1);
            }
            Some(r) => render_refs(&r),
        },
        Command::Commits { id, fetch, json } => {
            match farm.commits(&id, &env::current_dir()?, fetch)? {
                None => {
                    eprintln!("no such task: {id}");
                    std::process::exit(1);
                }
                Some(c) if json => json::print(&commits_json(&c))?,
                Some(c) => render_commits(&c),
            }
        }
        Command::Verify { ids, as_actor } => {
            let actor = actor::resolve(as_actor.as_deref());
            let cfg = farm.config();
            let mut all_ok = true;
            for id in &ids {
                let Some(show) = farm.show(id)? else {
                    eprintln!("no such task: {id}");
                    std::process::exit(1);
                };
                // Explicit per-yak verify: wins; otherwise fall back to the
                // config `verify:` default resolved by the yak's labels.
                let (cmd, source) = match show.task.verify.clone() {
                    Some(c) => (c, "yak".to_string()),
                    None => {
                        match cfg.resolve_verify(&show.task.tags, show.task.id.split('-').next()) {
                            Some(c) => (c, "config".to_string()),
                            None => {
                                eprintln!(
                                    "error: {id} has no verify: command and no config default \
                                 for its labels (set one with `yaks update {id} --verify \
                                 '<cmd>'`, or add a config `verify:` entry)"
                                );
                                std::process::exit(1);
                            }
                        }
                    }
                };
                println!("verify {id} ({source}): {cmd}");
                let (ok, verdict) = run_verify_command(&cmd)?;
                all_ok &= ok;
                farm.update(
                    id,
                    TaskEdit {
                        note: Some(format!("verify: `{cmd}` -> {verdict}")),
                        meta: vec![
                            ("kind".into(), "verify".into()),
                            ("result".into(), if ok { "pass" } else { "fail" }.into()),
                        ],
                        actor: actor.clone(),
                        ..Default::default()
                    },
                )?;
                println!("{id}: {verdict}");
            }
            if !all_ok {
                std::process::exit(1);
            }
        }
        Command::Attach {
            id,
            paths,
            name,
            no_link,
            caption,
            note,
            as_actor,
        } => {
            if name.is_some() && paths.len() > 1 {
                eprintln!("error: --name names one file; attach several without it");
                std::process::exit(1);
            }
            if caption.is_some() && (paths.len() > 1 || no_link) {
                eprintln!(
                    "error: --caption captions the one image it links; use it with a single file, without --no-link"
                );
                std::process::exit(1);
            }
            let mut files = Vec::new();
            for path in &paths {
                let data = match std::fs::read(path) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("error reading {}: {e}", path.display());
                        std::process::exit(1);
                    }
                };
                let Some(src_name) = path.file_name().and_then(|s| s.to_str()) else {
                    eprintln!("error: attachment path has no filename: {}", path.display());
                    std::process::exit(1);
                };
                let stored = match &name {
                    None => src_name.to_string(),
                    Some(raw) => match farm::attachment_name(raw, src_name) {
                        Some(n) => n,
                        None => {
                            eprintln!("error: invalid attachment name: {raw:?}");
                            std::process::exit(1);
                        }
                    },
                };
                files.push((stored, data));
            }
            match farm.attach_many(&id, &files, !no_link, caption.as_deref())? {
                AttachOutcome::NotFound => {
                    eprintln!("no such task: {id}");
                    std::process::exit(1);
                }
                AttachOutcome::Attached(n) => {
                    let dir = store::artifact_dir(std::path::Path::new(".yaks"), &id);
                    let linked = if no_link { ", not linked" } else { "" };
                    println!("attached {n} to {id} ({}{linked})", dir.display());
                    if let Some(text) = note {
                        let actor = actor::resolve(as_actor.as_deref());
                        farm.update(
                            &id,
                            TaskEdit {
                                note: Some(text),
                                actor,
                                ..Default::default()
                            },
                        )?;
                    }
                }
            }
        }
        Command::RenameAttachment { id, old, new } => {
            match farm.rename_attachment(&id, &old, &new)? {
                RenameAttachmentOutcome::Renamed { name, rewritten } => println!(
                    "renamed {old} -> {name} on {id} ({}); \
                     links rewritten in {rewritten} yak(s)",
                    store::artifact_dir(std::path::Path::new(".yaks"), &id)
                        .join(&name)
                        .display()
                ),
                RenameAttachmentOutcome::Unchanged => println!("{old}: name unchanged"),
                refused => {
                    let why = match refused {
                        RenameAttachmentOutcome::TaskNotFound => format!("no such task: {id}"),
                        RenameAttachmentOutcome::NoSuchAttachment(n) => {
                            format!(
                                "no attachment {n} on {id} ({}/)",
                                store::artifact_dir(std::path::Path::new(".yaks"), &id).display()
                            )
                        }
                        RenameAttachmentOutcome::Invalid(n) => {
                            format!("invalid attachment name: {n:?}")
                        }
                        RenameAttachmentOutcome::Collision(n) => {
                            format!("{id} already has an attachment named {n}")
                        }
                        _ => unreachable!("handled above"),
                    };
                    eprintln!("error: {why}");
                    std::process::exit(1);
                }
            }
        }
        Command::Attachment { id, file, json } => {
            let found = match &file {
                Some(f) => farm::locate_attachment(farm.root(), &id, f)
                    .into_iter()
                    .collect(),
                None => farm::list_attachments(farm.root(), &id),
            };
            if json {
                let items: Vec<serde_json::Value> = found
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "name": a.name,
                            "path": a.path.display().to_string(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&items)?);
            } else {
                for a in &found {
                    println!("{}", a.path.display());
                }
            }
            if found.is_empty() {
                match &file {
                    Some(f) => eprintln!("no attachment {f} on {id}"),
                    None => eprintln!("no attachments on {id}"),
                }
                std::process::exit(1);
            }
        }
        Command::ScanIds { file, json } => {
            let text = read_scan_input(file.as_deref())?;
            // Validate against the farm's real ids, the same membership test the
            // renderer highlights links with (refs is prefix-agnostic).
            let known = store::all_ids(farm.root());
            let found = refs::scan_text(&text, |t| known.contains(t));
            render_scan_ids(&found, json)?;
            if !found.is_empty() {
                // Non-zero so the command is usable as a pre-commit / CI gate.
                std::process::exit(1);
            }
        }
        Command::Next { filter, json } => {
            let rows = farm.next(build_spec(filter))?;
            if json {
                json::print(&json::tasks_array(&rows))?;
            } else if rows.is_empty() {
                println!("No yaks ready to shave.");
            } else {
                println!("Ready to shave (all dependencies met):");
                for t in &rows {
                    println!("{}", fmt_plain_row(t));
                }
            }
        }
        Command::Tangled { filter, json } => {
            let rows = farm.tangled(build_spec(filter))?;
            if json {
                json::print(&json::tangled_array(&rows))?;
            } else if rows.is_empty() {
                println!("No tangled yaks.");
            } else {
                println!("Tangled yaks:");
                for (t, waiting) in &rows {
                    println!(
                        "  {}  {}  (waiting on: {})",
                        t.id,
                        t.title,
                        waiting.join(", ")
                    );
                }
            }
        }
        Command::Sheds { json, all } => {
            let report = sheds::collect(farm.root())?;
            if json {
                json::print(&sheds::to_json(&report))?;
            } else {
                print!("{}", sheds::render(&report, all));
            }
        }
        Command::Stats { json } => {
            let s = farm.stats()?;
            if json {
                json::print(&json::stats_value(&s))?;
            } else {
                render_stats(&s);
            }
        }
        Command::Create {
            title,
            title_flag,
            kind,
            priority,
            parent,
            prefix,
            tags,
            depends_on,
            source,
            key,
            prs,
            description,
            description_file,
            verify,
            due,
            every,
            wake,
            json,
        } => {
            if key.as_deref().is_some_and(|k| k.trim().is_empty()) {
                anyhow::bail!("--key: the key is empty (it is the external issue or PR URL)");
            }
            let is_keyed = key.is_some();
            let fields = due_every_fields(due, every, wake)?;
            let description = resolve_text(
                "--description",
                description,
                description_file,
                &mut StdinText::real(),
            )?;
            let title = match title.or(title_flag) {
                Some(t) => t,
                None => {
                    eprintln!("error: a title is required (positional or --title)");
                    std::process::exit(1);
                }
            };
            let new = NewTask {
                title,
                prefix,
                kind,
                priority,
                parent,
                tags,
                depends_on,
                source,
                key: key.map(|k| k.trim().to_string()),
                prs,
                description,
                verify,
                fields,
            };
            match farm.create(new)? {
                CreateOutcome::ParentNotFound(p) => {
                    eprintln!("error: parent task {p} not found");
                    std::process::exit(1);
                }
                CreateOutcome::ParentFinished(msg) => {
                    eprintln!("error: {msg}");
                    std::process::exit(1);
                }
                CreateOutcome::InvalidPrefix(p) => {
                    eprintln!("error: invalid prefix {p:?} (use lowercase letters and digits)");
                    std::process::exit(1);
                }
                outcome @ (CreateOutcome::Created(_) | CreateOutcome::Exists(_)) => {
                    let (t, created) = match outcome {
                        CreateOutcome::Created(t) => (t, true),
                        CreateOutcome::Exists(t) => (t, false),
                        _ => unreachable!(),
                    };
                    if json {
                        let path = store::task_path(farm.root(), &t.id);
                        let mut v = json::create_value(&t, &path);
                        // `created` only where the caller could have asked for
                        // either outcome, so plain `create --json` is unchanged.
                        if !created || is_keyed {
                            v["created"] = serde_json::json!(created);
                        }
                        json::print(&v)?;
                    } else if created {
                        println!("Created {}: {}", t.id, t.title);
                    } else {
                        println!("Exists {}: {}", t.id, t.title);
                    }
                }
            }
        }
        Command::Update {
            ids,
            title,
            kind,
            priority,
            description,
            description_file,
            add_tag,
            remove_tag,
            source,
            add_pr,
            remove_pr,
            verify,
            due,
            every,
            wake,
            set,
            unset,
            note,
            note_file,
            meta,
            as_actor,
        } => {
            // The sugar flags go first, so a later `--set due=...` wins.
            let mut fields = due_every_fields(due, every, wake)?;
            fields.extend(parse_field_args(&set, &unset)?);
            let mut stdin = StdinText::real();
            let description =
                resolve_text("--description", description, description_file, &mut stdin)?;
            let note = resolve_text("--note", note, note_file, &mut stdin)?;
            let meta = parse_meta_args(&meta, note.is_some())?;
            warn_branch_under_farm(&meta, farm.root());
            let actor = note
                .as_ref()
                .and_then(|_| actor::resolve(as_actor.as_deref()));
            let edit = TaskEdit {
                title,
                kind,
                priority,
                description,
                // An explicit overwrite (yaks-9f5c): the CLI never rebases.
                base: None,
                add_tags: add_tag,
                remove_tags: remove_tag,
                source,
                add_prs: add_pr,
                remove_prs: remove_pr,
                verify,
                fields,
                note,
                meta,
                actor,
            };
            update_many(&farm, &ids, edit)?;
        }
        Command::Ask {
            id,
            note,
            note_file,
            needs,
            as_actor,
        } => {
            let note = resolve_text("--note", note, note_file, &mut StdinText::real())?;
            let actor = actor::resolve(as_actor.as_deref());
            let Some(c) = farm.ask(&id, &needs, actor.as_deref(), note.as_deref())? else {
                eprintln!("error: task {id} not found");
                std::process::exit(1);
            };
            if matches!(c.status, Status::Shorn | Status::Dead) {
                eprintln!(
                    "warning: {id} is {:?} \u{2014} blocking finished work; did you mean a hairy yak? (it will still show in `inbox`)",
                    c.status
                );
            }
            let mut msg = format!("Asked {id}: needs {needs}");
            if c.before.as_deref() == Some(NEEDS_AGENT) {
                msg.push_str(" (was answered, awaiting an agent; now awaiting a human again)");
            } else if !matches!(c.status, Status::Shorn | Status::Dead) {
                msg.push_str(" (dropped from next until answered)");
            }
            println!("{msg}");
        }
        Command::Answer {
            id,
            note,
            note_file,
            done,
            as_actor,
        } => {
            let note = resolve_text("--note", note, note_file, &mut StdinText::real())?;
            let actor = actor::resolve(as_actor.as_deref());
            let Some(c) = farm.answer(&id, actor.as_deref(), note.as_deref(), done)? else {
                eprintln!("error: task {id} not found");
                std::process::exit(1);
            };
            match (&c.before, &c.after) {
                (None, _) => println!("Answered {id}: no needs block was set (note recorded)"),
                (_, None) => println!("Answered {id}: needs cleared (done, no follow-up)"),
                (Some(b), Some(_)) if b == NEEDS_AGENT => {
                    println!("Answered {id}: still needs agent (note appended)")
                }
                _ => println!("Answered {id}: needs agent (awaiting pickup; see `yaks inbox`)"),
            }
        }
        Command::Pickup {
            id,
            note,
            note_file,
            drop_request,
            as_actor,
        } => {
            let note = resolve_text("--note", note, note_file, &mut StdinText::real())?;
            let actor = actor::resolve(as_actor.as_deref());
            match farm.pickup(&id, actor.as_deref(), note.as_deref(), drop_request)? {
                Pickup::NotARequest => {
                    eprintln!(
                        "error: {id} is awaiting an agent because of an answer, not a request; `--drop-request` never removes an answer (pick it up without it)"
                    );
                    std::process::exit(1);
                }
                Pickup::Picked if drop_request => {
                    println!("Picked up {id}: needs cleared, request note removed")
                }
                Pickup::NotFound => {
                    eprintln!("error: task {id} not found");
                    std::process::exit(1);
                }
                Pickup::NotAwaitingAgent(state) => {
                    let state = match state {
                        Some(n) => format!("needs: {n}"),
                        None => "no needs block set".to_string(),
                    };
                    eprintln!("error: {id} is not awaiting an agent ({state}); nothing to pick up");
                    std::process::exit(1);
                }
                Pickup::Picked => println!("Picked up {id}: needs cleared"),
            }
        }
        Command::Request {
            id,
            kind,
            note,
            note_file,
            as_actor,
        } => {
            let note = resolve_text("--note", note, note_file, &mut StdinText::real())?;
            let note = store::compose_request(&id, kind, note.as_deref());
            let actor = actor::resolve(as_actor.as_deref());
            match farm.request(&id, actor.as_deref(), note.as_deref())? {
                farm::Request::NotFound => {
                    eprintln!("error: task {id} not found");
                    std::process::exit(1);
                }
                farm::Request::AwaitingHuman(n) => {
                    eprintln!(
                        "error: {id} is awaiting a human (needs: {n}); answer it instead (`yaks answer {id}`)"
                    );
                    std::process::exit(1);
                }
                farm::Request::Requested(c) => {
                    if c.before.as_deref() == Some(NEEDS_AGENT) {
                        println!("Requested {id}: still needs agent (request appended)");
                    } else {
                        println!("Requested {id}: needs agent (awaiting pickup; see `yaks inbox`)");
                    }
                }
            }
        }
        Command::Inbox {
            filter,
            audience,
            json,
        } => {
            let rows = farm.inbox(build_spec(filter))?;
            render_inbox(&rows, audience, json)?;
        }
        Command::Wait {
            filter,
            audience,
            timeout,
            new,
            interval,
            json,
        } => {
            let opts = wait::WaitOpts {
                timeout: timeout
                    .as_deref()
                    .map(|s| parse_wait_duration("--timeout", s))
                    .transpose()?,
                interval: parse_wait_duration("--interval", &interval)?,
                new_only: new,
            };
            if opts.interval.is_zero() {
                anyhow::bail!("--interval must be at least 1s");
            }
            let spec = build_spec(filter);
            let keep = |t: &Task| match audience {
                Some(Audience::Human) => t.awaiting_human(),
                Some(Audience::Agent) => t.awaiting_agent(),
                None => t.needs.is_some(),
            };
            match wait::wait_for(|| farm.inbox(spec.clone()), keep, &opts, &wait::RealClock)? {
                Some(rows) => render_inbox(&rows, audience, json)?,
                None => {
                    let who = match audience {
                        Some(Audience::Human) => "awaiting a human",
                        Some(Audience::Agent) => "awaiting an agent",
                        None => "awaiting a human or an agent",
                    };
                    let new = if new { "new " } else { "" };
                    eprintln!(
                        "yaks wait: timed out after {}: nothing {new}{who}",
                        timeout.as_deref().unwrap_or_default()
                    );
                    std::process::exit(2);
                }
            }
        }
        Command::Shave { ids, as_actor } => {
            let actor = actor::resolve(as_actor.as_deref());
            transition_many(
                &farm,
                &ids,
                Status::Shaving,
                "already being shaved",
                "Shaving",
                actor.as_deref(),
            )?
        }
        Command::Shorn { ids, as_actor } => {
            let actor = actor::resolve(as_actor.as_deref());
            transition_many(
                &farm,
                &ids,
                Status::Shorn,
                "already shorn",
                "Shorn!",
                actor.as_deref(),
            )?
        }
        Command::Regrow { ids, as_actor } => {
            let actor = actor::resolve(as_actor.as_deref());
            transition_many(
                &farm,
                &ids,
                Status::Hairy,
                "already hairy",
                "Regrown:",
                actor.as_deref(),
            )?
        }
        Command::Tick {
            dry_run,
            within,
            json,
            as_actor,
        } => {
            let within = store::parse_duration(&within)
                .filter(|d| *d >= chrono::Duration::zero())
                .with_context(|| {
                    format!("could not parse --within '{within}'; use a span like 3d, 36h or 1w")
                })?;
            let actor = actor::resolve(as_actor.as_deref());
            let now = chrono::Utc::now();
            let regrow = farm.regrow_due(now, dry_run, actor.as_deref())?;
            let tasks = tick::open_tasks(farm.root())?;
            let inbox = farm.inbox(FilterSpec::default())?;
            // `check` is the slowest part and only a count here; a farm it
            // cannot read is said as "unknown", not a failed tick.
            let findings = check::run(&farm).ok().map(|f| f.len());
            let digest = tick::build(&tasks, inbox, regrow, findings, now, within, dry_run);
            if json {
                json::print(&tick::to_json(&digest))?;
            } else {
                print!("{}", tick::render(&digest, fmt_row));
            }
        }
        Command::Snooze { ids, to, as_actor } => {
            let due = recur::resolve_due(&to, recur::today())
                .map_err(|why| anyhow::anyhow!("--to: {why}"))?;
            let actor = actor::resolve(as_actor.as_deref());
            let mut any_failed = false;
            for id in &ids {
                match farm.snooze(id, &due, actor.as_deref())? {
                    SnoozeOutcome::Snoozed => println!("Snoozed {id} until {due}"),
                    SnoozeOutcome::NotFound => {
                        eprintln!("error: task {id} not found");
                        any_failed = true;
                    }
                    SnoozeOutcome::Finished(st) => {
                        eprintln!(
                            "error: {id} is {} and has nothing to put off; `yaks regrow {id}` reopens it",
                            st.as_str()
                        );
                        any_failed = true;
                    }
                }
            }
            if any_failed {
                std::process::exit(1);
            }
        }
        Command::RegrowDue {
            dry_run,
            json,
            as_actor,
        } => {
            let actor = actor::resolve(as_actor.as_deref());
            let report = farm.regrow_due(chrono::Utc::now(), dry_run, actor.as_deref())?;
            if json {
                json::print(&json::regrow_due_value(&report, dry_run))?;
            } else {
                print!("{}", render_regrow_due(&report, dry_run));
            }
        }
        Command::Slaughter {
            ids,
            family,
            as_actor,
        } => {
            let actor = actor::resolve(as_actor.as_deref());
            slaughter_many(&farm, &ids, family, actor.as_deref())?
        }
        Command::Revive { ids, as_actor } => {
            let actor = actor::resolve(as_actor.as_deref());
            transition_many(
                &farm,
                &ids,
                Status::Hairy,
                "already hairy",
                "Revived:",
                actor.as_deref(),
            )?
        }
        Command::Dep { action } => match action {
            DepAction::Add { id, dep_id } => match farm.dep_add(&id, &dep_id)? {
                DepOutcome::TaskNotFound => {
                    eprintln!("error: task {id} not found");
                    std::process::exit(1);
                }
                DepOutcome::DepNotFound => {
                    eprintln!("error: dependency task {dep_id} not found");
                    std::process::exit(1);
                }
                DepOutcome::AlreadyDep => println!("{dep_id} is already a dependency of {id}"),
                DepOutcome::Added => println!("Added dependency: {id} -> {dep_id}"),
                _ => {}
            },
            DepAction::Remove { id, dep_id } => match farm.dep_remove(&id, &dep_id)? {
                DepOutcome::TaskNotFound => {
                    eprintln!("error: task {id} not found");
                    std::process::exit(1);
                }
                DepOutcome::NotDep => println!("{dep_id} is not a dependency of {id}"),
                DepOutcome::Removed => println!("Removed dependency: {id} -> {dep_id}"),
                _ => {}
            },
        },
        Command::Dok { action } => run_dok(&farm, action)?,
        Command::Session { action } => run_session(&farm, action)?,
        Command::Source { action } => match action {
            SourceAction::Add {
                path,
                title,
                url,
                text,
                part_of,
                refresh,
                text_only,
                as_actor,
            } => {
                let actor = actor::resolve(as_actor.as_deref()).unwrap_or_default();
                let saved = farm.source_add(&sources::Request {
                    path,
                    title,
                    url,
                    part_of,
                    text,
                    actor,
                    origin: None,
                    refresh,
                    text_only,
                })?;
                match &saved {
                    sources::Saved::New(d) => println!("Saved {d}/source.md"),
                    sources::Saved::Already(d) => println!("Already saved: {d}/source.md"),
                    sources::Saved::Refreshed(d) => println!("Refreshed {d}/source.md"),
                }
            }
            SourceAction::Mark {
                source,
                uncited,
                superseded_by,
                clear,
            } => {
                let how = match (uncited, superseded_by, clear) {
                    (Some(why), None, false) => sources::Mark::Uncited(why),
                    (None, Some(by), false) => sources::Mark::SupersededBy(by),
                    (None, None, true) => sources::Mark::Clear,
                    _ => anyhow::bail!(
                        "give one of --uncited <why>, --superseded-by <source> or --clear"
                    ),
                };
                let (dir, changed) = farm.source_mark(&source, &how)?;
                match (&how, changed) {
                    (_, false) => println!("No changes to {dir}/source.md"),
                    (sources::Mark::Clear, true) => println!("Cleared the mark on {dir}/source.md"),
                    (sources::Mark::Uncited(_), true) => {
                        println!("Marked {dir}/source.md uncited on purpose")
                    }
                    (sources::Mark::SupersededBy(_), true) => {
                        println!("Marked {dir}/source.md superseded")
                    }
                }
            }
            SourceAction::List { json } => {
                let all = sources::list(farm.root());
                if json {
                    json::print(&serde_json::Value::Array(
                        all.iter()
                            .map(|s| {
                                serde_json::json!({
                                    "dir": s.dir, "title": s.title, "media_type": s.media_type,
                                    "resource": s.resource, "fields": s.fields.to_json(),
                                })
                            })
                            .collect(),
                    ))?;
                } else if all.is_empty() {
                    println!("No sources.");
                } else {
                    for s in &all {
                        println!(
                            "  {}  {:<16}  {}  ({})",
                            s.dir, s.media_type, s.title, s.resource
                        );
                    }
                }
            }
        },
        Command::Promote {
            attachment,
            dok,
            as_actor,
        } => {
            let Some((id, name)) = attachment.split_once('/') else {
                anyhow::bail!("{attachment}: give the attachment as <id>/<file>");
            };
            let actor = actor::resolve(as_actor.as_deref()).unwrap_or_default();
            let to = farm.promote(id, name, dok.as_deref(), &actor)?;
            println!("Promoted {attachment} -> {to}; links to it now point there");
        }
        Command::Reparent {
            ids,
            parent,
            unparent,
        } => {
            let new_parent = if unparent {
                None
            } else if parent.is_some() {
                parent
            } else {
                eprintln!("error: specify --parent TASK_ID or --unparent");
                std::process::exit(1);
            };
            reparent_many(&farm, &ids, new_parent)?;
        }
        Command::Bulk {
            filter,
            add_tag,
            remove_tag,
            set_priority,
            set_type,
            reparent,
            unparent,
            request,
            note,
            commit,
        } => {
            // Safety rule 1: never operate on the whole farm implicitly.
            if !filter.any_set() {
                eprintln!(
                    "error: bulk requires at least one filter flag (e.g. --tag, --status, --priority); refusing to select the whole farm"
                );
                std::process::exit(1);
            }
            // Reparent target: --reparent and --unparent are mutually exclusive.
            if unparent && reparent.is_some() {
                eprintln!("error: specify either --reparent ID or --unparent, not both");
                std::process::exit(1);
            }
            // Normalize up front so the dry-run preview shows the labels that
            // will actually be written (yaks-7cb3).
            let add_tag = normalize_tags(&add_tag);
            let remove_tag = normalize_tags(&remove_tag);
            let does_field_edit = !add_tag.is_empty()
                || !remove_tag.is_empty()
                || set_priority.is_some()
                || set_type.is_some();
            let does_reparent = unparent || reparent.is_some();
            // Safety rule 2: require at least one mutation flag.
            if !does_field_edit && !does_reparent && request.is_none() {
                eprintln!(
                    "error: bulk requires at least one mutation flag (--add-tag/--remove-tag/--set-priority/--set-type/--reparent/--unparent/--request)"
                );
                std::process::exit(1);
            }
            let note = resolve_text("--note", note, None, &mut StdinText::real())?;

            let rows = farm.list(build_spec(filter), false)?;
            if rows.is_empty() {
                println!("No yaks matched the filter; nothing to do.");
                return Ok(());
            }
            let ids: Vec<String> = rows.iter().map(|t| t.id.clone()).collect();
            let mutation = describe_bulk_mutation(
                &add_tag,
                &remove_tag,
                set_priority,
                set_type.as_deref(),
                reparent.as_deref(),
                unparent,
                request,
                note.as_deref(),
            );

            // Safety rule 3: dry-run is the default — print and change nothing.
            if !commit {
                println!("would update {} yaks:", rows.len());
                for t in &rows {
                    let skip = if request.is_some() && t.awaiting_human() {
                        "  (awaiting a human: request skipped)"
                    } else {
                        ""
                    };
                    println!("  {}  {}{skip}", t.id, t.title);
                }
                println!("mutation: {mutation}");
                println!("(dry run — pass --commit to apply)");
                return Ok(());
            }

            // Safety rule 4: only with --commit do we apply, reusing the same
            // per-id update/reparent path as the id-list commands. update_many /
            // reparent_many exit non-zero if any id failed.
            println!("updating {} yaks: {mutation}", rows.len());
            if does_field_edit {
                let edit = TaskEdit {
                    title: None,
                    kind: set_type,
                    priority: set_priority,
                    description: None,
                    base: None,
                    add_tags: add_tag,
                    remove_tags: remove_tag,
                    source: None,
                    add_prs: Vec::new(),
                    remove_prs: Vec::new(),
                    verify: None,
                    fields: Vec::new(),
                    note: None,
                    meta: Vec::new(),
                    actor: None,
                };
                update_many(&farm, &ids, edit)?;
            }
            if does_reparent {
                let new_parent = if unparent { None } else { reparent };
                reparent_many(&farm, &ids, new_parent)?;
            }
            if let Some(kind) = request {
                request_many(&farm, &ids, kind, note.as_deref())?;
            }
        }
        Command::Rollup(args) => {
            let (groups, unsourced) = farm.rollup(&build_spec(args.filter))?;
            if args.keys {
                let mut seen = std::collections::HashSet::new();
                let keys: Vec<String> = groups
                    .iter()
                    .map(|g| g.head())
                    .filter(|k| seen.insert(k.clone()))
                    .collect();
                if args.json {
                    json::print(&serde_json::json!(keys))?;
                } else {
                    for k in keys {
                        println!("{k}");
                    }
                }
            } else if args.json {
                json::print(&json::rollup_value(&groups))?;
            } else if groups.is_empty() {
                println!("No yaks with an external source.");
            } else {
                for g in &groups {
                    println!("{}  ({})  {}", g.head(), g.tracker, g.source);
                    for y in &g.yaks {
                        let mut row = fmt_row(&y.task);
                        if let Some(from) = &y.inherited_from {
                            row.push_str(&format!("  (via {from})"));
                        }
                        println!("{row}");
                    }
                    println!();
                }
                if unsourced > 0 {
                    let noun = if unsourced == 1 { "yak" } else { "yaks" };
                    println!("{unsourced} {noun} in scope with no external source (omitted).");
                }
            }
        }
        Command::Doctor { json, strict } => {
            let issues = farm.doctor(strict)?;
            if json {
                json::print(&json::doctor_array(&issues))?;
            } else {
                render_doctor(&issues);
            }
            if !issues.is_empty() {
                std::process::exit(1);
            }
        }
        Command::Check { json } => {
            let found = check::run(&farm)?;
            if json {
                json::print(&check::to_json(&found))?;
            } else {
                print!("{}", check::render(&found));
            }
            if !found.is_empty() {
                std::process::exit(1);
            }
        }
        Command::Preflight {
            ids,
            all,
            push_main,
            json,
        } => {
            let report = preflight::run(&farm, &env::current_dir()?, &ids, all, push_main)?;
            if json {
                json::print(&serde_json::json!({
                    "ok": report.ok(),
                    "failures": report.failures.iter().map(|f| serde_json::json!({
                        "check": f.check.code(),
                        "message": f.message,
                        "subjects": f.subjects,
                    })).collect::<Vec<_>>(),
                    "skipped": report.skipped,
                    "warnings": report.warnings,
                }))?;
            } else {
                for line in &report.skipped {
                    println!("preflight: {line}");
                }
                for line in &report.warnings {
                    println!("preflight: warning: {line}");
                }
                for f in &report.failures {
                    println!("preflight: FAIL {}: {}", f.check.code(), f.message);
                }
                if report.ok() {
                    println!("preflight: ok");
                }
            }
            if !report.ok() {
                std::process::exit(1);
            }
        }
        Command::Sync {
            no_commit,
            no_pull,
            no_push,
            message,
        } => {
            let opts = sync::Opts {
                no_commit,
                no_pull,
                no_push,
                message,
            };
            print!("{}", sync::run(farm.root(), &opts)?.render());
        }
        Command::Commit { message, dry_run } => {
            match commit::run(farm.root(), message.as_deref(), dry_run)? {
                commit::Outcome::Nothing => println!("commit: nothing to commit under the farm"),
                commit::Outcome::Done(plan) => {
                    let verb = if dry_run { "would commit" } else { "committed" };
                    for (state, path) in &plan.files {
                        println!("  {state} {path}");
                    }
                    if !plan.left_staged.is_empty() {
                        println!(
                            "commit: left {} other staged file(s) out of the commit: {}",
                            plan.left_staged.len(),
                            plan.left_staged.join(", ")
                        );
                    }
                    match &plan.commit {
                        Some(sha) => println!("commit: {verb} {sha} {}", plan.message),
                        None => println!("commit: {verb}: {}", plan.message),
                    }
                }
            }
        }
        Command::Status { check, json } => {
            let report = status::collect(farm.root())?;
            if json {
                json::print(&status::to_json(&report))?;
            } else {
                print!("{}", status::render(&report));
            }
            if check && !report.clean() {
                std::process::exit(1);
            }
        }
        Command::Brief {
            id,
            as_actor,
            yaks_dir,
        } => {
            // A worker's name must be fresh: notes under a reused name merge two
            // workers into one sheds row (yaks-5d1c).
            if let Some(w) = sheds::reuse_warning(farm.root(), as_actor.trim(), None) {
                eprintln!("warning: {w}; brief a fresh worker name instead");
            }
            let code = brief::run(
                &farm,
                &brief::Request {
                    id: &id,
                    actor: &as_actor,
                    explicit_dir: yaks_dir.as_deref(),
                    cwd: &env::current_dir()?,
                    binary: &env::current_exe()?,
                },
                &mut std::io::stdout().lock(),
                &mut std::io::stderr().lock(),
            )?;
            if code != 0 {
                std::process::exit(code);
            }
        }
        Command::Tui {
            headless,
            size,
            diff,
        } => {
            // Each saved field is a write; fold a quick burst into one commit.
            autocommit::set_coalesce(true);
            let app = tui::App::with_farm(farm)?;
            if headless {
                let (w, h) = parse_size(size.as_deref());
                toque::run(
                    app,
                    toque::DriverOpts {
                        width: w,
                        height: h,
                        diff,
                    },
                )?;
            } else {
                tui::run(app)?;
            }
        }
    }
    if let Some(e) = autocommit::take_error() {
        eprintln!("yaks: {e}");
    }
    Ok(())
}

/// Parse a "WxH" size string; falls back to 80x24 on absence or bad input.
fn parse_size(s: Option<&str>) -> (u16, u16) {
    let default = (80, 24);
    let Some(s) = s else { return default };
    let Some((w, h)) = s.split_once(['x', 'X']) else {
        return default;
    };
    match (w.trim().parse(), h.trim().parse()) {
        (Ok(w), Ok(h)) if w > 0 && h > 0 => (w, h),
        _ => default,
    }
}

// -- CLI arg mapping ------------------------------------------------------

/// `yaks path`: print the file path of each yak named by `ids`, or selected by
/// `filter`. Returns the process exit code (non-zero on a usage error or an
/// unknown id); writers are injected so the exit behavior is testable.
fn run_path(
    farm: &Farm,
    ids: &[String],
    filter: FilterFlags,
    all: bool,
    out: &mut impl std::io::Write,
    err: &mut impl std::io::Write,
) -> Result<i32> {
    if ids.is_empty() && !filter.any_set() {
        // Which farm am I on? The root discovery resolved, redirects included.
        writeln!(out, "{}", farm.root().display())?;
        return Ok(0);
    }
    if !ids.is_empty() && filter.any_set() {
        writeln!(err, "error: path takes yak ids or filter flags, not both")?;
        return Ok(1);
    }
    if ids.is_empty() {
        for p in farm.paths(build_spec(filter), all)? {
            writeln!(out, "{}", p.display())?;
        }
        return Ok(0);
    }
    let mut code = 0;
    for id in ids {
        match farm.path_of(id) {
            Some(p) => writeln!(out, "{}", p.display())?,
            None => {
                writeln!(err, "no such task: {id}")?;
                code = 1;
            }
        }
    }
    Ok(code)
}

/// A `wait` duration flag (`30s`, `10m`, `2h`), in the grammar `log --since`
/// uses for its relative form.
fn parse_wait_duration(flag: &str, s: &str) -> Result<std::time::Duration> {
    store::parse_duration(s)
        .and_then(|d| d.to_std().ok())
        .with_context(|| {
            format!("could not parse {flag} '{s}'; use a duration like 30s, 10m or 2h")
        })
}

fn build_spec(f: FilterFlags) -> FilterSpec {
    FilterSpec {
        statuses: f.status.iter().filter_map(|s| parse_status(s)).collect(),
        types: f.kind,
        priorities: f.priority,
        tags: normalize_tags(&f.tag),
        herds: f.herd,
        search: f.search,
        source: f.source,
        due_only: false,
        ready_only: f.ready,
        tangled_only: f.tangled,
        needs_only: f.needs,
        run_on: f.run_on,
        parent: f.parent_of,
    }
}

fn parse_status(s: &str) -> Option<Status> {
    match s.to_lowercase().as_str() {
        "hairy" => Some(Status::Hairy),
        "shaving" => Some(Status::Shaving),
        "shorn" => Some(Status::Shorn),
        "dead" => Some(Status::Dead),
        _ => None,
    }
}

// -- rendering ------------------------------------------------------------

/// Transition a single yak, printing a per-id result line. Returns `true` on
/// success (moved or already there) and `false` on failure (not found, or a
/// move the family invariant refuses), so a batch caller can process
/// every id and set the exit code once at the end.
fn transition(
    farm: &Farm,
    id: &str,
    dest: Status,
    already: &str,
    done: &str,
    actor: Option<&str>,
) -> Result<bool> {
    let (outcome, lifted) = match farm.transition_lifting(id, dest, actor) {
        Ok(o) => o,
        Err(e)
            if e.downcast_ref::<farm::UnfinishedChildren>().is_some()
                || e.downcast_ref::<farm::FinishedParent>().is_some() =>
        {
            eprintln!("error: {e}");
            return Ok(false);
        }
        Err(e) => return Err(e),
    };
    match outcome {
        MoveOutcome::NotFound => {
            eprintln!("error: task {id} not found");
            Ok(false)
        }
        MoveOutcome::AlreadyThere => {
            println!("{id} is {already}");
            Ok(true)
        }
        MoveOutcome::Moved => {
            println!("{done} {id}");
            for a in lifted {
                println!("{done} {a} (parent of {id})");
            }
            Ok(true)
        }
    }
}

/// Transition every id in `ids`, one at a time. All ids are processed even if
/// one fails (no abort-on-first-error); if any id was not found or refused,
/// the process exits non-zero after the whole batch is handled. The ids go in
/// [`Farm::batch_order`]: a shear takes children before their parents, a
/// regrow or revive parents before their children, whatever order they were
/// given in (yaks-62f2, yaks-35a4).
fn transition_many(
    farm: &Farm,
    ids: &[String],
    dest: Status,
    already: &str,
    done: &str,
    actor: Option<&str>,
) -> Result<()> {
    let ids = farm.batch_order(ids, dest)?;
    let mut any_failed = false;
    for id in &ids {
        if !transition(farm, id, dest, already, done, actor)? {
            any_failed = true;
        }
    }
    if any_failed {
        std::process::exit(1);
    }
    Ok(())
}

/// Slaughter every id in `ids` (see [`Farm::slaughter`]). Like
/// [`transition_many`], the whole batch is processed; a missing id or a yak
/// refused for having open descendants exits non-zero afterwards (yaks-05da).
fn slaughter_many(farm: &Farm, ids: &[String], family: bool, actor: Option<&str>) -> Result<()> {
    let mut any_failed = false;
    for id in ids {
        match farm.slaughter(id, family, actor)? {
            SlaughterOutcome::NotFound => {
                eprintln!("error: task {id} not found");
                any_failed = true;
            }
            SlaughterOutcome::AlreadyDead => println!("{id} is already dead"),
            SlaughterOutcome::HasOpenDescendants(kids) => {
                let n = kids.len();
                let noun = if n == 1 { "descendant" } else { "descendants" };
                eprintln!(
                    "error: {id} has {n} unfinished {noun} ({}); finish or slaughter them \
                     first, use --family to slaughter them with it, or graduate them out of \
                     the family: `yaks reparent <child> --parent <other>` or `--unparent`",
                    kids.join(", ")
                );
                any_failed = true;
            }
            SlaughterOutcome::Slaughtered(moved) => {
                for m in moved {
                    println!("Slaughtered: {m}");
                }
            }
        }
    }
    if any_failed {
        std::process::exit(1);
    }
    Ok(())
}

/// Apply `edit` to a single id, printing the per-id result. Returns false only
/// when the id was not found or (never, without a `base`) conflicted, so the
/// caller can exit non-zero.
fn update_one(farm: &Farm, id: &str, edit: TaskEdit) -> Result<bool> {
    let noted = edit.note.is_some();
    match farm.update(id, edit)? {
        UpdateOutcome::NotFound => {
            eprintln!("error: task {id} not found");
            Ok(false)
        }
        UpdateOutcome::Updated => {
            // A new note's link (yaks-c5bd), so nobody builds one by hand.
            let link = noted
                .then(|| store::load_task_by_id(farm.root(), id).ok().flatten())
                .flatten()
                .and_then(|t| links::latest_note_link(id, &t.body));
            match link {
                Some(l) => println!("Updated {id}: note {l}"),
                None => println!("Updated {id}"),
            }
            Ok(true)
        }
        UpdateOutcome::NoChanges => {
            println!("No changes specified.");
            Ok(true)
        }
        // Only an edit with a `base` can conflict; the CLI sets none.
        UpdateOutcome::Conflict => {
            eprintln!("error: task {id} changed on disk; not updated");
            Ok(false)
        }
    }
}

/// Apply the same `edit` to every id, one at a time. All ids are processed even
/// if one is missing (no abort-on-first-error); if any id was not found, the
/// process exits non-zero after the whole batch is handled. Mirrors
/// `transition_many`.
/// Run a yak's `verify:` command via the shell, streaming its output live so the
/// human/agent sees the real artifact. Returns `(passed, verdict-string)`;
/// `passed` is true iff the command exited 0.
fn run_verify_command(cmd: &str) -> std::io::Result<(bool, String)> {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .status()?;
    let verdict = match status.code() {
        Some(0) => "PASS (exit 0)".to_string(),
        Some(n) => format!("FAIL (exit {n})"),
        None => "FAIL (terminated by signal)".to_string(),
    };
    Ok((status.success(), verdict))
}

/// The `--meta key=value` arguments of `update`, split and checked: each needs
/// a `=` and a key that is a valid metadata key, and any needs a note to
/// attach to. Values may contain `=`.
/// Warn when a recorded `branch:` sits under the farm branch's name (`yaks/...`
/// beside a farm branch `yaks`): git cannot hold both refs, so that branch can
/// never have been pushed, and nobody can fetch what the note names.
fn warn_branch_under_farm(meta: &[(String, String)], root: &std::path::Path) {
    let Some(farm) = store::farm_branch(root) else {
        return;
    };
    let prefix = format!("{farm}/");
    for (k, v) in meta {
        if k == "branch" && v.starts_with(&prefix) {
            eprintln!(
                "warning: branch `{v}` is under `{prefix}`, which the farm branch `{farm}` blocks (git cannot have both refs), so it cannot be pushed; use another prefix, e.g. `work/...`"
            );
        }
    }
}

fn parse_meta_args(args: &[String], has_note: bool) -> Result<Vec<(String, String)>> {
    if !args.is_empty() && !has_note {
        anyhow::bail!("--meta needs a note to attach to (give --note, --note-file or `--note -`)");
    }
    args.iter()
        .map(|a| {
            let Some((k, v)) = a.split_once('=') else {
                anyhow::bail!("--meta {a:?}: expected key=value");
            };
            if !store::is_meta_key(k) {
                anyhow::bail!("--meta {a:?}: {k:?} is not a valid key (letters, digits, `_` and `-`; not starting with a digit or `-`)");
            }
            Ok((k.to_string(), v.to_string()))
        })
        .collect()
}

/// `--set key=value` and `--unset key` as field edits, in that order. A key
/// must be a plain YAML key and not one [`model::BUILT_IN_KEYS`] names.
fn parse_field_args(set: &[String], unset: &[String]) -> Result<Vec<(String, fields::Value)>> {
    let check = |flag: &str, k: &str| -> Result<()> {
        if !store::is_meta_key(k) {
            anyhow::bail!(
                "{flag} {k:?}: not a valid field name (letters, digits, `_` and `-`; not starting with a digit or `-`)"
            );
        }
        if model::BUILT_IN_KEYS.contains(&k) {
            anyhow::bail!(
                "{flag} {k:?}: a built-in field; use its own flag (see `yaks update --help`)"
            );
        }
        Ok(())
    };
    let mut out = Vec::new();
    for a in set {
        let Some((k, v)) = a.split_once('=') else {
            anyhow::bail!("--set {a:?}: expected key=value");
        };
        check("--set", k)?;
        out.push((k.to_string(), fields::Value::from_cli(v)));
    }
    for k in unset {
        check("--unset", k)?;
        out.push((k.clone(), fields::Value::Null));
    }
    Ok(out)
}

/// `--due`, `--every` and `--wake` as field edits; an empty string clears the field.
/// An unreadable value is refused here, before anything is written.
fn due_every_fields(
    due: Option<String>,
    every: Option<String>,
    wake: Option<String>,
) -> Result<Vec<(String, fields::Value)>> {
    let mut out = Vec::new();
    for (flag, key, text) in [
        ("--due", recur::DUE, due),
        ("--every", recur::EVERY, every),
        ("--wake", recur::WAKE, wake),
    ] {
        let Some(text) = text else { continue };
        // A deadline may be typed relative (`+2d`); it is stored as a date.
        let text = match key {
            recur::DUE if !text.trim().is_empty() => recur::resolve_due(&text, recur::today())
                .map_err(|why| anyhow::anyhow!("{flag}: {why}"))?,
            _ => text,
        };
        let value = fields::Value::from_cli(&text);
        if let Some(why) = recur::field_problem(key, &value) {
            anyhow::bail!("{flag} {text:?}: {why}");
        }
        out.push((key.to_string(), value));
    }
    Ok(out)
}

/// `regrow-due`'s report, one line per yak, and what it held back.
fn render_regrow_due(r: &recur::Report, dry_run: bool) -> String {
    let mut out = String::new();
    if r.regrew.is_empty() && r.held.is_empty() {
        return "Nothing to regrow.\n".into();
    }
    let verb = if dry_run { "Would regrow" } else { "Regrown" };
    for g in &r.regrew {
        for a in &g.reopened {
            out.push_str(&format!("{verb}: {a} (parent of {})\n", g.id));
        }
        let skipped = match g.plan.skipped {
            0 => String::new(),
            n => format!(", {n} skipped"),
        };
        let waits = match &g.awaiting {
            Some(a) if a.already => format!("; already awaiting {}", a.who),
            Some(a) => format!("; awaiting {}", a.who),
            None => String::new(),
        };
        out.push_str(&format!(
            "{verb}: {}  {} (for {}{skipped}; due {}{waits})\n",
            g.id,
            g.title,
            g.plan.occurrence.render(),
            g.plan.due.render()
        ));
    }
    for (id, why) in &r.held {
        out.push_str(&format!("Held: {id}: {why}\n"));
    }
    if dry_run && !r.regrew.is_empty() {
        out.push_str("(dry run \u{2014} nothing written; re-run without --dry-run to apply)\n");
    }
    out
}

fn update_many(farm: &Farm, ids: &[String], edit: TaskEdit) -> Result<()> {
    let mut any_failed = false;
    for id in ids {
        if !update_one(farm, id, edit.clone())? {
            any_failed = true;
        }
    }
    if any_failed {
        std::process::exit(1);
    }
    Ok(())
}

/// Reparent a single id, printing the per-id result. Returns false on any
/// reparent error (not found, cycle, already-a-child, ...) so the caller can
/// exit non-zero, matching the single-id command's behavior.
fn reparent_one(farm: &Farm, id: &str, new_parent: Option<String>) -> Result<bool> {
    match farm.reparent(id, new_parent)? {
        Reparent::Error(msg) => {
            eprintln!("error: {msg}");
            Ok(false)
        }
        Reparent::Done { new_parent: None } => {
            println!("Promoted {id} to top-level");
            Ok(true)
        }
        Reparent::Done {
            new_parent: Some(p),
        } => {
            println!("Reparented {id} under {p}");
            Ok(true)
        }
    }
}

/// Reparent every id to the same destination, one at a time. All ids are
/// processed even if one fails; if any failed, the process exits non-zero after
/// the whole batch is handled. Mirrors `transition_many`.
fn reparent_many(farm: &Farm, ids: &[String], new_parent: Option<String>) -> Result<()> {
    let mut any_failed = false;
    for id in ids {
        if !reparent_one(farm, id, new_parent.clone())? {
            any_failed = true;
        }
    }
    if any_failed {
        std::process::exit(1);
    }
    Ok(())
}

/// Queue the canned request `kind` (plus `note`) on each id, as `request
/// --kind` does, printing one line per id. A yak awaiting a human is skipped
/// and reported, not failed; an unknown id fails the run (exit 1) after the
/// rest are done, like `update_many`.
fn request_many(
    farm: &Farm,
    ids: &[String],
    kind: store::RequestKind,
    note: Option<&str>,
) -> Result<()> {
    let actor = actor::resolve(None);
    let mut skipped = Vec::new();
    let mut any_failed = false;
    for id in ids {
        let text = store::compose_request(id, Some(kind), note);
        match farm.request(id, actor.as_deref(), text.as_deref())? {
            farm::Request::NotFound => {
                eprintln!("error: task {id} not found");
                any_failed = true;
            }
            farm::Request::AwaitingHuman(_) => skipped.push(id.as_str()),
            farm::Request::Requested(_) => {
                println!("Requested {id}: {} (needs agent)", kind.as_str())
            }
        }
    }
    if !skipped.is_empty() {
        println!(
            "skipped {} awaiting a human (answer instead): {}",
            skipped.len(),
            skipped.join(", ")
        );
    }
    if any_failed {
        std::process::exit(1);
    }
    Ok(())
}

/// Human-readable one-line summary of the field edit, reparent and/or request
/// a bulk run would apply. Used in both the dry-run preview and the commit log
/// line.
#[allow(clippy::too_many_arguments)]
fn describe_bulk_mutation(
    add_tag: &[String],
    remove_tag: &[String],
    set_priority: Option<u8>,
    set_type: Option<&str>,
    reparent: Option<&str>,
    unparent: bool,
    request: Option<store::RequestKind>,
    note: Option<&str>,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !add_tag.is_empty() {
        parts.push(format!("add tags [{}]", add_tag.join(", ")));
    }
    if !remove_tag.is_empty() {
        parts.push(format!("remove tags [{}]", remove_tag.join(", ")));
    }
    if let Some(p) = set_priority {
        parts.push(format!("set priority {p}"));
    }
    if let Some(t) = set_type {
        parts.push(format!("set type {t}"));
    }
    if unparent {
        parts.push("reparent to top-level".to_string());
    } else if let Some(id) = reparent {
        parts.push(format!("reparent under {id}"));
    }
    if let Some(k) = request {
        match note {
            Some(n) => parts.push(format!("request {} (note: {n:?})", k.as_str())),
            None => parts.push(format!("request {}", k.as_str())),
        }
    }
    parts.join("; ")
}

/// Which side of the `needs` hand-off `inbox --for` keeps.
#[derive(Clone, Copy, clap::ValueEnum)]
enum Audience {
    /// Open questions (`needs: human`).
    Human,
    /// Awaiting pickup (`needs: agent`): answered or requested.
    Agent,
}

/// `inbox` output: a flat JSON array (each yak with `needs`, `replied` and
/// `why`), or two text sections, "awaiting a human" and "awaiting an agent"
/// (each agent row marked `answered` or `requested`).
fn render_inbox(rows: &[Task], audience: Option<Audience>, json: bool) -> Result<()> {
    let human: Vec<&Task> = rows
        .iter()
        .filter(|t| t.awaiting_human() && !matches!(audience, Some(Audience::Agent)))
        .collect();
    let agent: Vec<&Task> = rows
        .iter()
        .filter(|t| t.awaiting_agent() && !matches!(audience, Some(Audience::Human)))
        .collect();
    if json {
        let kept: Vec<&Task> = human.iter().chain(agent.iter()).copied().collect();
        json::print(&json::inbox_array(&kept))?;
        return Ok(());
    }
    if human.is_empty() && agent.is_empty() {
        println!(
            "{}",
            match audience {
                Some(Audience::Human) => "Inbox empty: nothing awaiting a human.",
                Some(Audience::Agent) => "Inbox empty: nothing awaiting an agent.",
                None => "Inbox empty: nothing awaiting a human or an agent.",
            }
        );
    }
    for (title, section) in [
        ("Awaiting a human:", &human),
        ("Awaiting an agent (`yaks pickup <id>`):", &agent),
    ] {
        if section.is_empty() {
            continue;
        }
        println!("{title}");
        for t in section {
            // An agent row's `answered`/`requested` mark comes from `fmt_row`.
            let replied = if store::is_replied(t) {
                " \u{21a9} replied"
            } else {
                ""
            };
            println!("{}{replied}", fmt_row(t));
        }
    }
    Ok(())
}

fn render_rows(rows: &[Task], json: bool, empty_msg: &str) -> Result<()> {
    if json {
        json::print(&json::tasks_array(rows))?;
    } else if rows.is_empty() {
        println!("{empty_msg}");
    } else {
        for t in rows {
            println!("{}", fmt_row(t));
        }
    }
    Ok(())
}

fn render_log(entries: &[LogEntry], json: bool) -> Result<()> {
    if json {
        json::print(&json::log_array(entries))?;
    } else if entries.is_empty() {
        println!("No notes found.");
    } else {
        for e in entries {
            let who = e
                .actor
                .as_deref()
                .map(|a| format!("  [{a}]"))
                .unwrap_or_default();
            println!("\u{25b8} {}  {}  {}{}", e.ts, e.id, e.title, who);
            for line in e.note.lines() {
                println!("    {line}");
            }
        }
    }
    Ok(())
}

/// Print a `yaks migrate` report: one line per fixup (with the yaks it
/// touches), then the total and, after a real run on a git-tracked farm, the
/// commit to make.
fn report_migrate(r: &migrate::Report) {
    if r.is_noop() {
        println!(
            "nothing to migrate: the farm is on schema {}",
            store::SCHEMA
        );
        return;
    }
    let from = r
        .from
        .map_or("unmarked".to_string(), |v| format!("schema {v}"));
    println!(
        "{} ({from} -> schema {}):",
        if r.applied {
            "migrated"
        } else {
            "would migrate"
        },
        store::SCHEMA
    );
    if !r.layout.is_empty() {
        let (files, dirs) = r
            .layout
            .iter()
            .fold((0, 0), |(f, d), (_, hf, hd)| (f + hf, d + hd));
        println!(
            "  herd-directories: {files} yak file(s) and {dirs} attachment dir(s) {} <herd>/yaks/",
            if r.applied {
                "moved into"
            } else {
                "to move into"
            }
        );
        for (herd, f, d) in &r.layout {
            println!("    {herd}/yaks/: {f} yak(s), {d} attachment dir(s)");
        }
    }
    if !r.doks_roots.is_empty() {
        println!(
            "  doks-directories: {} wiki director{} {} doks",
            r.doks_roots.len(),
            if r.doks_roots.len() == 1 { "y" } else { "ies" },
            if r.applied { "moved to" } else { "to move to" }
        );
        for (from, to) in &r.doks_roots {
            println!("    {from}/ -> {to}/");
        }
    }
    if !r.docs.is_empty() {
        let total: usize = r.docs.iter().map(|(_, n)| n).sum();
        println!(
            "  wiki-links-in-doks: {total} edit(s) in {} dok/schema/session file(s) \
             (`/wiki/` links, `type: Page Type`)",
            r.docs.len()
        );
        if !r.applied {
            for (path, n) in &r.docs {
                println!("    {path}: {n}");
            }
        }
    }
    for f in &r.fixups {
        let total: usize = f.yaks.iter().map(|(_, n)| n).sum();
        println!(
            "  {}: {total} {} in {} yak(s)",
            f.name,
            f.unit,
            f.yaks.len()
        );
        if !r.applied {
            for (id, n) in &f.yaks {
                println!("    {id}: {n}");
            }
        }
    }
    let mut line = format!(
        "{} yak file(s) {}",
        r.files,
        if r.applied { "rewritten" } else { "to rewrite" }
    );
    if r.applied && r.git_tracked {
        line.push_str(&format!(
            "; commit with: yaks commit -m \"yaks: migrate the farm to schema {}\"",
            store::SCHEMA
        ));
    } else if !r.applied {
        line.push_str(" (dry run: nothing written; re-run without --dry-run)");
    }
    println!("{line}");
}

/// `--set key=value` for a dok: any key (doks have no built-in fields
/// besides what OKF suggests), checked like note metadata keys.
fn parse_dok_fields(set: &[String], unset: &[String]) -> Result<Vec<(String, fields::Value)>> {
    let mut out = Vec::new();
    for a in set {
        let Some((k, v)) = a.split_once('=') else {
            anyhow::bail!("--set {a:?}: expected key=value");
        };
        if !store::is_meta_key(k) {
            anyhow::bail!("--set {a:?}: {k:?} is not a valid field name");
        }
        out.push((k.to_string(), fields::Value::from_cli(v)));
    }
    for k in unset {
        if !store::is_meta_key(k) {
            anyhow::bail!("--unset {k:?}: not a valid field name");
        }
        out.push((k.clone(), fields::Value::Null));
    }
    Ok(out)
}

/// Schema warnings for the dok at `path` after a write (advice; the write
/// stands).
fn print_schema_warnings(farm: &Farm, path: &str) -> Result<()> {
    if let Some(p) = farm.dok(path)? {
        for w in schemas::check_dok(farm.root(), &p) {
            eprintln!("warning: {w}");
        }
    }
    Ok(())
}

fn print_dok_rows(doks: &[&doks::Dok]) {
    if doks.is_empty() {
        println!("No doks found.");
        return;
    }
    let w = doks.iter().map(|p| p.path.len()).max().unwrap_or(0);
    let kw = doks.iter().map(|p| p.kind().len()).max().unwrap_or(0);
    for p in doks {
        println!("  {:w$}  {:kw$}  {}", p.path, p.kind(), p.title());
    }
}

/// The start of a session: `--since`, else the actor's latest session note,
/// at most 12 hours back.
fn session_since(
    farm: &Farm,
    actor: &str,
    since: Option<&str>,
) -> Result<chrono::DateTime<chrono::Utc>> {
    let now = chrono::Utc::now();
    match since {
        Some(s) => store::parse_since(s, now),
        None => Ok(sessions::default_since(
            farm.root(),
            actor,
            now - chrono::Duration::hours(12),
        )),
    }
}

/// The agent app the environment names, if any.
fn harness_from_env() -> Option<String> {
    let set = |k: &str| env::var(k).is_ok_and(|v| !v.is_empty());
    if set("DELTA_CURRENT_THREAD_ID") || set("DELTA_THREAD_TITLE") {
        Some("delta".into())
    } else if set("CLAUDECODE") {
        Some("claude-code".into())
    } else {
        None
    }
}

fn run_session(farm: &Farm, action: SessionAction) -> Result<()> {
    let me = || {
        actor::resolve(None).ok_or_else(|| {
            anyhow::anyhow!("no actor name: name yourself once with `yaks whoami --set <name>`")
        })
    };
    match action {
        SessionAction::Note {
            title,
            body,
            body_file,
            since,
            harness,
        } => {
            let Some(body) = resolve_text("--body", body, body_file, &mut StdinText::real())?
            else {
                anyhow::bail!(
                    "give the note's markdown with --body or --body-file (`-` reads stdin)"
                );
            };
            let actor = me()?;
            let from = session_since(farm, &actor, since.as_deref())?;
            let saved = sessions::render_saved(&sessions::touched(farm.root(), &actor, from)?);
            let person = std::process::Command::new("git")
                .arg("-C")
                .arg(farm.root())
                .args(["config", "user.name"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .filter(|s| !s.is_empty());
            let origin = sessions::Origin {
                actor,
                person,
                host: Some(sources::host()),
                harness: harness.or_else(harness_from_env),
                thread: actor::harness_thread(),
            };
            let path = farm.session_note(&title, &body, Some(&saved), &origin)?;
            println!("Saved session note {path}");
            println!("{}", farm.root().join(&path).display());
        }
        SessionAction::Touched { since, by, json } => {
            let actor = match by {
                Some(b) => b,
                None => me()?,
            };
            let from = session_since(farm, &actor, since.as_deref())?;
            let t = sessions::touched(farm.root(), &actor, from)?;
            if json {
                json::print(&serde_json::json!({
                    "actor": actor,
                    "since": from.format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    "yaks": t.yaks.iter().map(|y| serde_json::json!({
                        "id": y.id,
                        "title": y.title,
                        "created": y.created,
                        "moves": y.moves.iter().map(|(a, b)| serde_json::json!({"from": a, "to": b})).collect::<Vec<_>>(),
                        "notes": y.notes.iter().map(|n| serde_json::json!({
                            "link": format!("{}#{}", y.id, n.anchor),
                            "kind": n.kind,
                            "gist": n.gist,
                        })).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                    "files": t.files.iter().map(|f| serde_json::json!({
                        "path": f.path, "verb": f.verb, "title": f.title,
                    })).collect::<Vec<_>>(),
                }))?;
            } else {
                print!("{}", sessions::render_saved(&t));
            }
        }
        SessionAction::List { unreviewed, json } => {
            let all = sessions::load_all(farm.root())?;
            let rows: Vec<&doks::Dok> = all
                .iter()
                .filter(|d| !unreviewed || sessions::is_unreviewed(d))
                .collect();
            let verdicts = |d: &doks::Dok| {
                sessions::VERDICTS
                    .iter()
                    .filter_map(|v| d.str_field(v).map(|n| format!("{v} {n}")))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            let who = |d: &doks::Dok| {
                d.str_field("actor")
                    .or_else(|| d.str_field("person"))
                    .unwrap_or_default()
            };
            if json {
                json::print(&serde_json::Value::Array(
                    rows.iter().map(|d| json::dok_value(d, false)).collect(),
                ))?;
            } else if rows.is_empty() {
                println!(
                    "{}",
                    if unreviewed {
                        "No unreviewed session notes."
                    } else {
                        "No session notes."
                    }
                );
            } else {
                for d in rows {
                    let promoted = d
                        .str_field(sessions::PROMOTED)
                        .map(|p| format!("  promoted {p}"))
                        .unwrap_or_default();
                    println!(
                        "{}  {}  {}  [{}]  {}{promoted}",
                        d.path,
                        d.str_field("day").unwrap_or_default(),
                        who(d),
                        verdicts(d),
                        d.title()
                    );
                }
            }
        }
        SessionAction::Promote { paths } => {
            let day = chrono::Local::now().format("%Y-%m-%d").to_string();
            let done = farm.session_promote(&paths, &day)?;
            for p in &paths {
                let p = doks::normalize(p);
                if done.contains(&p) {
                    println!("Promoted {p}");
                } else {
                    println!("{p} was already promoted");
                }
            }
        }
    }
    Ok(())
}

fn run_dok(farm: &Farm, action: DokAction) -> Result<()> {
    let under = |p: &doks::Dok, prefix: &Option<String>| {
        prefix.as_ref().is_none_or(|x| {
            let x = doks::normalize(x);
            let x = x.trim_end_matches('/');
            p.path == x || p.path.starts_with(&format!("{x}/"))
        })
    };
    match action {
        DokAction::List { prefix, json } => {
            let all = farm.doks()?;
            let rows: Vec<&doks::Dok> = all.iter().filter(|p| under(p, &prefix)).collect();
            if json {
                json::print(&serde_json::Value::Array(
                    rows.iter().map(|p| json::dok_value(p, false)).collect(),
                ))?;
            } else {
                print_dok_rows(&rows);
            }
        }
        DokAction::Show { path, json } => {
            let Some(p) = farm.dok(&path)? else {
                eprintln!(
                    "error: no dok {path} (doks live under doks/ or <herd>/doks/; `yaks dok list`)"
                );
                std::process::exit(1);
            };
            if json {
                json::print(&json::dok_value(&p, true))?;
            } else {
                print!("{}", p.render());
            }
        }
        DokAction::New {
            path,
            kind,
            title,
            set,
            body,
            body_file,
        } => {
            let fields = parse_dok_fields(&set, &[])?;
            let body = resolve_text("--body", body, body_file, &mut StdinText::real())?
                .unwrap_or_default();
            let file = farm.dok_new(&path, kind.as_deref(), title.as_deref(), &fields, &body)?;
            println!("Created {}", doks::normalize(&path));
            println!("{}", file.display());
            print_schema_warnings(&farm, &path)?;
        }
        DokAction::Set {
            path,
            set,
            unset,
            body,
            body_file,
        } => {
            let fields = parse_dok_fields(&set, &unset)?;
            let body = resolve_text("--body", body, body_file, &mut StdinText::real())?;
            if fields.is_empty() && body.is_none() {
                anyhow::bail!("nothing to set: give --set, --unset, --body or --body-file");
            }
            match farm.dok_set(&path, &fields, body.as_deref())? {
                None => {
                    eprintln!("error: no dok {path}");
                    std::process::exit(1);
                }
                Some(false) => println!("No changes to {}", doks::normalize(&path)),
                Some(true) => println!("Updated {}", doks::normalize(&path)),
            }
            print_schema_warnings(&farm, &path)?;
        }
        DokAction::Define {
            kind,
            field,
            title,
            description,
            stale_after,
            replace,
        } => {
            let fields = field
                .iter()
                .map(|f| schemas::FieldSpec::parse(f))
                .collect::<Result<Vec<_>>>()?;
            let done = farm.dok_define(&schemas::Define {
                kind,
                fields,
                title,
                description,
                stale_after,
                replace,
            })?;
            if done.is_noop() {
                println!("No changes to {}", done.path);
            } else if done.created {
                println!(
                    "Created {} ({})",
                    done.path,
                    match done.added.len() {
                        1 => "1 field".to_string(),
                        n => format!("{n} fields"),
                    }
                );
            } else {
                let mut what = Vec::new();
                if !done.added.is_empty() {
                    what.push(format!("added {}", done.added.join(", ")));
                }
                if !done.changed.is_empty() {
                    what.push(format!("changed {}", done.changed.join(", ")));
                }
                if !done.keys.is_empty() {
                    what.push(format!("set {}", done.keys.join(", ")));
                }
                println!("Updated {}: {}", done.path, what.join("; "));
            }
        }
        DokAction::Types => {
            println!(
                "Untyped kinds (no schema): {} ({} is the default)",
                doks::UNTYPED_KINDS.join(", "),
                doks::DEFAULT_KIND
            );
            let all = schemas::load_all(farm.root());
            if all.is_empty() {
                println!("No typed kinds yet (add schemas/<type>.md).");
            }
            for s in &all {
                println!("{}", schemas::describe(s));
                for w in &s.warnings {
                    println!("    warning: {w}");
                }
            }
        }
        DokAction::Query {
            rows: true,
            kind,
            where_,
            prefix,
            json,
        } => {
            let mut tests: Vec<(String, String)> = Vec::new();
            for w in &where_ {
                let Some((k, v)) = w.split_once('=') else {
                    anyhow::bail!("--where {w:?}: expected key=value");
                };
                tests.push((k.to_string(), v.to_string()));
            }
            let all = farm.doks()?;
            let mut out: Vec<serde_json::Value> = Vec::new();
            for p in all
                .iter()
                .filter(|p| under(p, &prefix) && kind.as_ref().is_none_or(|k| &p.kind() == k))
            {
                for row in schemas::table_rows(&p.body) {
                    if tests
                        .iter()
                        .all(|(k, v)| row.iter().any(|(c, x)| c == k && x == v))
                    {
                        let mut m = serde_json::Map::new();
                        m.insert("dok".into(), p.path.clone().into());
                        for (c, x) in row {
                            m.insert(c, x.into());
                        }
                        out.push(serde_json::Value::Object(m));
                    }
                }
            }
            if json {
                json::print(&serde_json::Value::Array(out))?;
            } else if out.is_empty() {
                println!("No rows found.");
            } else {
                for r in &out {
                    let o = r.as_object().expect("row");
                    let cells: Vec<String> = o
                        .iter()
                        .filter(|(k, _)| *k != "dok")
                        .map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or("")))
                        .collect();
                    println!(
                        "  {}  {}",
                        o["dok"].as_str().unwrap_or(""),
                        cells.join("  ")
                    );
                }
            }
        }
        DokAction::Query {
            rows: false,
            kind,
            where_,
            prefix,
            json,
        } => {
            let mut tests: Vec<(String, String)> = Vec::new();
            if let Some(k) = kind {
                tests.push(("type".into(), k));
            }
            for w in &where_ {
                let Some((k, v)) = w.split_once('=') else {
                    anyhow::bail!("--where {w:?}: expected key=value");
                };
                tests.push((k.to_string(), v.to_string()));
            }
            let all = farm.doks()?;
            let rows: Vec<&doks::Dok> = all
                .iter()
                .filter(|p| under(p, &prefix) && tests.iter().all(|(k, v)| doks::matches(p, k, v)))
                .collect();
            if json {
                json::print(&serde_json::Value::Array(
                    rows.iter().map(|p| json::dok_value(p, false)).collect(),
                ))?;
            } else {
                print_dok_rows(&rows);
            }
        }
    }
    Ok(())
}

fn render_doctor(issues: &[Issue]) {
    if issues.is_empty() {
        println!("All clear: no farm-integrity issues found.");
    } else {
        let noun = if issues.len() == 1 { "issue" } else { "issues" };
        println!("Found {} farm-integrity {noun}:", issues.len());
        let mut last: Option<IssueKind> = None;
        for i in issues {
            if last != Some(i.kind) {
                println!("\n{}:", i.kind.heading());
                last = Some(i.kind);
            }
            println!("  {}", i.message);
        }
    }
    render_skill_advisory();
}

/// Report installed skills that startup auto-sync deliberately won't touch.
///
/// This is an *environment* advisory, not farm integrity, so it sits outside
/// `Farm::doctor` and never affects its exit code. Auto-sync silently handles
/// absent and cleanly-stale user-level skills; what it can't resolve is a skill
/// someone edited by hand — which would otherwise sit stale forever with no
/// signal — and a stale project-local install, which is never written behind
/// your back (the ordinary-command notice says it once; this says it whenever
/// asked, in the same words).
fn render_skill_advisory() {
    let base = skills::default_dir();
    let stuck: Vec<_> = skills::status(&base)
        .into_iter()
        .filter(|s| s.state.has_local_edits())
        .collect();
    let stale = env::current_dir()
        .ok()
        .and_then(|cwd| skills::project_stale(&cwd));
    if stuck.is_empty() && stale.is_none() {
        return;
    }
    println!("\nSkills needing attention (not farm integrity):");
    for s in &stuck {
        println!("  {} [{}] {}", s.name, s.state.word(), s.path.display());
    }
    if !stuck.is_empty() {
        println!(
            "  These were edited after install, so yaks leaves them alone. \
             Run `yaks skills status` to compare, or `yaks skills install --user --force` to replace."
        );
    }
    if let Some(stale) = stale {
        println!("  {}", stale.message());
    }
}

fn render_stats(s: &Stats) {
    println!(
        "Total: {}  Hairy: {}  Shaving: {}  Shorn: {}",
        s.total, s.hairy, s.shaving, s.shorn
    );
    if !s.by_type.is_empty() {
        println!("By type:");
        for (k, v) in &s.by_type {
            println!("  {k}: {v}");
        }
    }
    if !s.by_priority.is_empty() {
        println!("By priority:");
        for (k, v) in &s.by_priority {
            println!("  p{k}: {v}");
        }
    }
}

fn report_rename(out: RenameOutcome) {
    match out {
        RenameOutcome::NotFound(id) => {
            eprintln!("error: no such task: {id}");
            std::process::exit(1);
        }
        RenameOutcome::Invalid(id) => {
            eprintln!("error: invalid target id: {id}");
            std::process::exit(1);
        }
        RenameOutcome::Collision(id) => {
            eprintln!("error: target id already exists: {id}");
            std::process::exit(1);
        }
        RenameOutcome::NothingToRename => println!("Nothing to rename."),
        RenameOutcome::Done(plan) => render_rename(&plan),
    }
}

/// `yaks whoami [--set <name>]`.
fn run_whoami(set: Option<&str>) -> Result<()> {
    let cwd = env::current_dir()?;
    if let Some(name) = set {
        let wrote = actor::set_config(&cwd, name)?;
        println!("actor: {} (set in {wrote})", name.trim());
        if let Some((now, src @ (actor::Source::Env | actor::Source::Explicit))) =
            actor::resolve_detail(None)
        {
            eprintln!(
                "note: {} ({now}) still overrides it in this shell",
                src.as_str()
            );
        }
    }
    match actor::resolve_detail(None) {
        Some((name, source)) => {
            if set.is_none() {
                println!("actor: {name} (from {})", source.as_str());
            }
            if let Some(t) = actor::harness_thread() {
                println!("thread: {t}");
            }
            if let Ok(found) = store::discover(&cwd) {
                if let Some(w) =
                    sheds::reuse_warning(&found.root, &name, actor::harness_thread().as_deref())
                {
                    eprintln!("warning: {w}");
                }
            }
            if source == actor::Source::Harness {
                eprintln!("hint: name yourself once with `yaks whoami --set <name>`");
            }
        }
        None => println!("actor: none (notes are written unattributed)"),
    }
    Ok(())
}

fn run_skills(action: &SkillsAction) -> Result<()> {
    match action {
        SkillsAction::Install { dir, user, force } => {
            let target = skills::resolve_target(dir.as_deref(), *user, &env::current_dir()?);
            let base = target.dir.clone();
            println!("Installing skills into {}", target.describe());
            let installed = skills::install(&base, *force)?;
            for i in &installed {
                println!(
                    "{}",
                    skills::render_installed(i, " (use --force to overwrite)")
                );
            }
            println!(
                "\nFor another agent, re-run with --dir pointing at its skills directory (e.g. --dir ~/.claude/skills)."
            );
            if matches!(target.kind, skills::TargetKind::Project { .. }) {
                println!(
                    "These are files in your working tree: commit them to share them, and re-run `yaks skills install` after upgrading yaks (only ~/.agents/skills is refreshed automatically; yaks tells you once per release when these fall behind)."
                );
            }
            Ok(())
        }
        SkillsAction::Status { dir, user } => {
            let target = skills::resolve_target(dir.as_deref(), *user, &env::current_dir()?);
            let base = target.dir.clone();
            println!("{}", target.describe());
            let mut blocked = 0;
            let statuses = skills::status(&base);
            for s in &statuses {
                let detail = match &s.state {
                    skills::SkillState::Upgradable { from } => {
                        format!("  (installed {from}, this yaks is {})", skills::version())
                    }
                    skills::SkillState::Held { installed } => {
                        format!("  (installed {installed} is not older than this yaks)")
                    }
                    skills::SkillState::Modified { installed } => {
                        blocked += 1;
                        format!("  (edited since install of {installed})")
                    }
                    skills::SkillState::Unmanaged => {
                        blocked += 1;
                        "  (no yaks stamp \u{2014} hand-written or another tool's)".to_string()
                    }
                    skills::SkillState::Retired { from } => {
                        format!("  (retired; installed by {from}; `yaks skills install` removes it)")
                    }
                    skills::SkillState::RetiredKept { stamped } => {
                        if *stamped {
                            "  (retired, and edited since install \u{2014} remove it by hand)"
                        } else {
                            "  (retired, and no yaks stamp \u{2014} remove it by hand)"
                        }
                        .to_string()
                    }
                    skills::SkillState::SourceLinked => {
                        "  (resolves onto yaks' own .agents/skills/ source \u{2014} nothing to install)"
                            .to_string()
                    }
                    _ => String::new(),
                };
                // A file other than SKILL.md being the issue: say which.
                let file = match s.path.file_name().and_then(|f| f.to_str()) {
                    Some(f) if f != "SKILL.md" => format!(" [{f}]"),
                    _ => String::new(),
                };
                println!("  {:<9} {}{}{}", s.state.word(), s.name, file, detail);
            }
            if let Some(stale) = skills::Stale::of(&statuses) {
                let flag = match &target.kind {
                    skills::TargetKind::User => " --user".to_string(),
                    skills::TargetKind::Dir => format!(" --dir {}", base.display()),
                    _ => String::new(),
                };
                println!(
                    "\n{} stale or retired; run `{}` to update them.",
                    stale.count,
                    stale.command(&flag)
                );
            }
            if blocked > 0 {
                println!(
                    "{blocked} left alone because overwriting would lose local edits; \
                     re-run with --force to replace them."
                );
            }
            Ok(())
        }
    }
}

fn render_rename(plan: &RenamePlan) {
    let head = if plan.applied {
        "Renamed"
    } else {
        "Dry run \u{2014} would rename"
    };
    println!("{head}:");
    for (old, new) in &plan.renames {
        println!("  {old} -> {new}");
    }
    println!("Reference edits: {} file(s)", plan.edits.len());
    for e in &plan.edits {
        let mut parts: Vec<String> = Vec::new();
        if e.new_id.is_some() {
            parts.push("id".to_string());
        }
        for f in &e.fields {
            if *f != "body" {
                parts.push((*f).to_string());
            }
        }
        if !e.body_lines.is_empty() {
            let lines: Vec<String> = e.body_lines.iter().map(|n| format!("L{n}")).collect();
            parts.push(format!("body:{}", lines.join(",")));
        }
        println!("  {:<14} {}", e.id, parts.join(", "));
    }
}

/// `refs` for a dok: its outgoing links, or with `incoming` the links to it.
fn run_dok_refs(farm: &Farm, path: &str, incoming: bool, json: bool) -> Result<()> {
    let path = doks::normalize(path);
    let Some(dok) = farm.dok(&path)? else {
        eprintln!("error: no dok {path}");
        std::process::exit(1);
    };
    let edges: Vec<links::Edge> = if incoming {
        farm.links_to(&path)?
    } else {
        let ids = store::all_ids(farm.root());
        links::extract(&dok.body)
            .into_iter()
            .map(|f| links::Edge {
                from: path.clone(),
                line: f.line,
                link: links::parse(&f.dest, links::From::Dok(&path), |s| ids.contains(s)),
                dest: f.dest,
            })
            .collect()
    };
    if json {
        let rows: Vec<serde_json::Value> = edges
            .iter()
            .map(|e| {
                serde_json::json!({
                    "from": e.from,
                    "line": e.line,
                    "dest": e.dest,
                    "target": links::label(&e.link),
                    "exists": links::exists(farm.root(), &e.link),
                })
            })
            .collect();
        let key = if incoming { "incoming" } else { "links" };
        json::print(&serde_json::json!({ "path": path, "title": dok.title(), key: rows }))?;
        return Ok(());
    }
    println!("{path}  {}", dok.title());
    if edges.is_empty() {
        println!("  (no {}links)", if incoming { "incoming " } else { "" });
    }
    for e in &edges {
        if incoming {
            println!("  {}  L{}  {}", e.from, e.line, e.dest);
        } else {
            let mark = match links::exists(farm.root(), &e.link) {
                Some(false) => "  BROKEN",
                _ => "",
            };
            println!("  L{:<4} {}{mark}", e.line, links::label(&e.link));
        }
    }
    Ok(())
}

fn render_refs(r: &TaskRefs) {
    println!("{}  {}", r.id, r.title);
    if r.entries.is_empty() {
        println!("  (no references)");
        return;
    }
    for e in &r.entries {
        let kind = match e.kind {
            RefKind::Parent => "parent",
            RefKind::Depends => "depends",
            RefKind::Mention => "mention",
        };
        let status = if e.resolved { "ok" } else { "DANGLING" };
        let loc = e.line.map(|n| format!("  body:L{n}")).unwrap_or_default();
        println!("  {kind:<8} {:<14} {status}{loc}", e.id);
    }
}

fn render_backrefs(r: &TaskBackRefs) {
    println!("{}  {}", r.id, r.title);
    if r.entries.is_empty() {
        println!("  (no incoming references)");
        return;
    }
    for e in &r.entries {
        let loc = e.via.location();
        let loc = if loc.is_empty() {
            String::new()
        } else {
            format!("  {loc}")
        };
        println!(
            "  {:<8} {:<14} {}  {}{loc}",
            e.via.kind(),
            e.id,
            e.status.emoji(),
            e.title
        );
    }
}

/// `refs --incoming --json`: `{id, title, incoming: [...]}`, one object per
/// reference: the referring yak's `id`, `title`, `status`, `kind`
/// (`child`/`depends`/`mention`), and for a mention `in` (`title`/`body`/
/// `note`) plus `line` (body) or `ts` and `actor` (note, `actor` may be null).
fn backrefs_json(r: &TaskBackRefs) -> serde_json::Value {
    use farm::BackRefVia;
    let incoming: Vec<serde_json::Value> = r
        .entries
        .iter()
        .map(|e| {
            let mut v = serde_json::json!({
                "id": e.id,
                "title": e.title,
                "status": e.status.as_str(),
                "kind": e.via.kind(),
            });
            let m = v.as_object_mut().expect("object");
            match &e.via {
                BackRefVia::Child | BackRefVia::Depends => {}
                BackRefVia::Title => {
                    m.insert("in".into(), "title".into());
                }
                BackRefVia::Body { line } => {
                    m.insert("in".into(), "body".into());
                    m.insert("line".into(), (*line).into());
                }
                BackRefVia::Note { ts, actor } => {
                    m.insert("in".into(), "note".into());
                    m.insert("ts".into(), ts.clone().into());
                    m.insert("actor".into(), actor.clone().into());
                }
            }
            v
        })
        .collect();
    serde_json::json!({ "id": r.id, "title": r.title, "incoming": incoming })
}

fn commit_json(c: &farm::CommitRef) -> serde_json::Value {
    serde_json::json!({ "sha": c.sha, "short": c.short, "subject": c.subject })
}

fn branch_json(b: &branchlog::BranchCommits) -> serde_json::Value {
    serde_json::json!({
        "branch": b.branch,
        "remote": b.remote,
        "tracking": b.tracking,
        "base": b.base,
        "fetched": b.fetched,
        "fetch_error": b.fetch_error,
        "error": b.error,
        "commits": b.commits.iter().map(|c| {
            let mut v = commit_json(c);
            v["checkpoint"] = serde_json::json!(b.is_checkpoint(c));
            v
        }).collect::<Vec<_>>(),
        "checkpoints": b.checkpoints.iter().map(|k| serde_json::json!({
            "commit": k.commit,
            "ts": k.ts,
            "actor": k.actor,
            "state": k.state.as_str(),
        })).collect::<Vec<_>>(),
    })
}

fn render_branches(c: &Commits) {
    for b in &c.branches {
        println!();
        let base = b
            .base
            .as_deref()
            .map(|x| format!(" (since {x})"))
            .unwrap_or_default();
        println!("Branch {}{base}:", b.tracking);
        if let Some(e) = &b.fetch_error {
            println!("  fetch failed: {e}");
        }
        match (&b.error, b.fetched) {
            (Some(e), _) => println!("  {e}"),
            (None, true) => println!("  fetched just now"),
            (None, false) => println!(
                "  as of the last fetch of {} (yaks commits --fetch refreshes it)",
                b.remote
            ),
        }
        if b.error.is_none() {
            if b.commits.is_empty() {
                println!("  (no commits beyond the base)");
            }
            for l in &b.commits {
                let mark = if b.is_checkpoint(l) {
                    "  \u{2190} checkpoint"
                } else {
                    ""
                };
                println!("  {} {}{mark}", l.short, l.subject);
            }
        }
        for k in &b.checkpoints {
            let short: String = k.commit.chars().take(7).collect();
            let who = k
                .actor
                .as_deref()
                .map(|a| format!(" [{a}]"))
                .unwrap_or_default();
            match k.state {
                branchlog::CheckpointState::Listed => {}
                branchlog::CheckpointState::InBase => println!(
                    "  checkpoint {short} ({}{who}) is already in the base: landed or merged",
                    k.ts
                ),
                branchlog::CheckpointState::NotOnBranch => println!(
                    "  checkpoint {short} ({}{who}) is NOT on {}: the branch was rewritten, or the commit was never pushed",
                    k.ts, b.tracking
                ),
            }
        }
    }
}

fn commits_json(c: &Commits) -> serde_json::Value {
    let recorded: Vec<_> = c
        .recorded
        .iter()
        .map(|r| {
            serde_json::json!({
                "kind": r.kind,
                "commit": r.commit,
                "source": r.source,
                "branch": r.branch,
                "ts": r.ts,
                "actor": r.actor,
                "found": r.found.as_ref().map(commit_json),
            })
        })
        .collect();
    serde_json::json!({
        "id": c.id,
        "code_repo": c.code_repo.as_ref().map(|p| p.display().to_string()),
        "recorded": recorded,
        "fetch": c.fetch,
        "branches": c.branches.iter().map(branch_json).collect::<Vec<_>>(),
        "named": c.by_message.iter().map(commit_json).collect::<Vec<_>>(),
        "farm_history": c.by_file.iter().map(commit_json).collect::<Vec<_>>(),
    })
}

fn render_commits(c: &Commits) {
    println!("Recorded in {}'s notes:", c.id);
    if c.recorded.is_empty() {
        println!("  (none)");
    }
    for r in &c.recorded {
        let short: String = r.commit.chars().take(7).collect();
        let branch = r
            .branch
            .as_deref()
            .map(|b| format!(" on {b}"))
            .unwrap_or_default();
        let what = match &r.found {
            Some(f) => f.subject.clone(),
            None => "(not in this repository)".to_string(),
        };
        let who = r
            .actor
            .as_deref()
            .map(|a| format!(" [{a}]"))
            .unwrap_or_default();
        println!("  {} {short}{branch}  {what}  ({}{who})", r.kind, r.ts);
    }
    render_branches(c);
    println!();
    match &c.code_repo {
        Some(top) => println!("Commits naming {} in {}:", c.id, top.display()),
        None => println!("Commits naming {}:", c.id),
    }
    if c.code_repo.is_none() {
        println!(
            "  (not run from a code checkout: only the farm is here, so nothing was searched; run yaks from the code repository or pass -C)"
        );
    } else if c.by_message.is_empty() {
        println!("  (none)");
    }
    for l in &c.by_message {
        println!("  {} {}", l.short, l.subject);
    }
    println!(
        "\nFarm history of {} (edits to the yak file, not code):",
        c.path.display()
    );
    if c.by_file.is_empty() {
        println!("  (none)");
    }
    for l in &c.by_file {
        println!("  {} {}", l.short, l.subject);
    }
}

/// The process's stdin as a one-shot text source for `--note -` and
/// `--description -`. There is only one stdin, so the second `-` in a call is
/// an error. The reader and tty flag are fields so the rules are unit-testable.
struct StdinText {
    reader: Box<dyn std::io::Read>,
    is_tty: bool,
    used: bool,
}

impl StdinText {
    fn real() -> Self {
        use std::io::IsTerminal;
        StdinText {
            reader: Box::new(std::io::stdin()),
            is_tty: std::io::stdin().is_terminal(),
            used: false,
        }
    }

    fn take(&mut self, flag: &str) -> Result<String> {
        if self.used {
            anyhow::bail!(
                "stdin can be read only once per command; `-` was given for more than one of --note/--description"
            );
        }
        self.used = true;
        if self.is_tty {
            anyhow::bail!(
                "{flag} - reads stdin, but stdin is a terminal; pipe or redirect the text in (a heredoc works: `{flag} - <<'EOF'`)"
            );
        }
        let mut text = String::new();
        self.reader
            .read_to_string(&mut text)
            .map_err(|e| anyhow::anyhow!("cannot read {flag} text from stdin: {e}"))?;
        Ok(text)
    }
}

/// Resolve a text flag that accepts inline text, `-` (stdin), or a `-file`
/// path (`-` there is stdin too). Text is stored as given except trailing line
/// terminators are trimmed (a heredoc/`echo` always adds one); interior and
/// leading whitespace survive. Text from stdin or a file that is empty (or only
/// whitespace) is an error rather than an empty note. Inline text is returned
/// untouched, so `--description ""` still clears a body. Inline-plus-file is
/// rejected by clap (`conflicts_with`).
fn resolve_text(
    flag: &str,
    inline: Option<String>,
    file: Option<String>,
    stdin: &mut StdinText,
) -> Result<Option<String>> {
    let (text, origin) = match (inline, file) {
        (Some(t), _) if t != "-" => return Ok(Some(t)),
        (None, None) => return Ok(None),
        (_, Some(path)) if path != "-" => (
            std::fs::read_to_string(&path)
                .map_err(|e| anyhow::anyhow!("cannot read {flag}-file {path}: {e}"))?,
            format!("{flag}-file {path}"),
        ),
        _ => (stdin.take(flag)?, format!("{flag} - (stdin)")),
    };
    if text.trim().is_empty() {
        anyhow::bail!("{origin} is empty; refusing to store an empty text");
    }
    Ok(Some(text.trim_end_matches(['\n', '\r']).to_string()))
}

/// Gather the text `scan-ids` should scan: the contents of `file` (when given)
/// and piped stdin (when stdin is not a terminal), so a file arg, a `... |`
/// pipe, or both together all work. Joined with a newline so line numbers stay
/// sane across the two sources.
fn read_scan_input(file: Option<&str>) -> Result<String> {
    use std::io::{IsTerminal, Read};
    let mut text = String::new();
    if let Some(path) = file {
        let body = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("cannot read {path}: {e}"))?;
        text.push_str(&body);
    }
    if !std::io::stdin().is_terminal() {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        if !s.is_empty() {
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
            text.push_str(&s);
        }
    }
    Ok(text)
}

/// Report the yak-ids `scan-ids` found. Text mode prints one `line:col  id`
/// per hit (nothing when clean); JSON mode emits an array of `{line,col,id}`.
/// The non-zero exit is the caller's job so this stays a pure renderer.
fn render_scan_ids(found: &[refs::FoundRef], json: bool) -> Result<()> {
    if json {
        let arr: Vec<serde_json::Value> = found
            .iter()
            .map(|f| serde_json::json!({ "line": f.line, "col": f.col, "id": f.id }))
            .collect();
        json::print(&serde_json::Value::Array(arr))?;
    } else {
        for f in found {
            println!("{}:{}  {}", f.line, f.col, f.id);
        }
    }
    Ok(())
}

fn render_show(s: &Show) {
    let t = &s.task;
    println!("id:       {}", t.id);
    println!("title:    {}", t.title);
    println!("status:   {:?}", t.status);
    println!("type:     {}", t.kind);
    println!("priority: {}", t.priority);
    if let Some(c) = &t.created {
        println!("created:  {c}");
    }
    if let Some(u) = &t.updated {
        println!("updated:  {u}");
    }
    if let Some(p) = &t.parent {
        println!("parent:   {p}");
    }
    if !t.tags.is_empty() {
        println!("tags:     {}", t.tags.join(", "));
    }
    if !t.depends_on.is_empty() {
        println!("depends:  {}", t.depends_on.join(", "));
    }
    if let Some(src) = &t.source {
        println!("source:   {src}");
    }
    if !t.prs.is_empty() {
        println!("prs:      {}", t.prs.join(", "));
    }
    if let Some(n) = &t.needs {
        println!("needs:    {n}");
    }
    for key in recur::KEYS {
        if let Some(v) = t.extra.get(key).and_then(|v| v.as_str().map(String::from)) {
            let late = if key == recur::DUE && recur::is_overdue(t, chrono::Utc::now()) {
                " (overdue)"
            } else {
                ""
            };
            println!("{key:<9} {v}{late}");
        }
    }
    // Preserved but unmodeled frontmatter (Task.extra), surfaced read-only so a
    // hand-added/newer key is visible in `show` without this binary owning it.
    let mut other = t.extra.clone();
    for key in recur::KEYS {
        if t.extra.get(key).is_some_and(|v| v.as_str().is_some()) {
            other.remove(key);
        }
    }
    if !other.is_empty() {
        println!("\nOther fields:");
        for line in other.lines() {
            println!("  {line}");
        }
    }
    if !t.body.is_empty() {
        println!("\n{}", t.body);
    }
    if !s.children.is_empty() {
        println!("\nChildren:");
        for c in &s.children {
            println!("  [{}] {}  {}", c.status.glyph(), c.id, c.title);
        }
    }
}

/// `  [X] id  pN type     title [tags] (deps: ...) ⚠ needs:<who>`
fn fmt_row(t: &Task) -> String {
    let tags = if t.tags.is_empty() {
        String::new()
    } else {
        format!(" [{}]", t.tags.join(","))
    };
    let deps = if t.depends_on.is_empty() {
        String::new()
    } else {
        format!(" (deps: {})", t.depends_on.join(","))
    };
    // Make a needs-blocked yak visually distinct from a ready one in list/next;
    // one awaiting an agent (`needs: agent`) is not a warning, it is waiting on
    // us, and says why (answered or requested).
    let needs = match (&t.needs, store::agent_why(t)) {
        (Some(_), Some(why)) => format!(" \u{2713} {} (needs:agent)", why.as_str()),
        (Some(who), None) => format!(" \u{26a0} needs:{who}"),
        (None, _) => String::new(),
    };
    format!(
        "  [{}] {}  p{} {:8} {}{}{}{}{}",
        t.status.glyph(),
        t.id,
        t.priority,
        t.kind,
        t.title,
        tags,
        deps,
        recur::due_mark(t, chrono::Utc::now()),
        needs
    )
}

/// `  id  pN type     title` (no status glyph) — matches Python cmd_next;
/// marks a yak awaiting an agent (`needs: agent`, answered or requested) that
/// is ready but awaits pickup.
fn fmt_plain_row(t: &Task) -> String {
    let why = match store::agent_why(t) {
        Some(why) => format!(" \u{2713} {}, awaiting pickup", why.as_str()),
        None => String::new(),
    };
    let due = recur::due_mark(t, chrono::Utc::now());
    format!(
        "  {}  p{} {:8} {}{due}{why}",
        t.id, t.priority, t.kind, t.title
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stdin(text: &'static str, is_tty: bool) -> StdinText {
        StdinText {
            reader: Box::new(text.as_bytes()),
            is_tty,
            used: false,
        }
    }

    #[test]
    fn resolve_text_refuses_a_terminal_stdin_instead_of_hanging() {
        let mut s = stdin("never read", true);
        let e = resolve_text("--note", Some("-".into()), None, &mut s).unwrap_err();
        assert!(e.to_string().contains("stdin is a terminal"), "{e}");
    }

    #[test]
    fn resolve_text_reads_stdin_once() {
        let mut s = stdin("a\n\nb\n", false);
        let got = resolve_text("--note", Some("-".into()), None, &mut s).unwrap();
        assert_eq!(got.as_deref(), Some("a\n\nb"));
        let e = resolve_text("--description", Some("-".into()), None, &mut s).unwrap_err();
        assert!(e.to_string().contains("only once"), "{e}");
    }

    #[test]
    fn resolve_text_passes_inline_and_absent_through() {
        let mut s = stdin("unused", true);
        assert_eq!(resolve_text("--note", None, None, &mut s).unwrap(), None);
        let got = resolve_text("--note", Some("x\n".into()), None, &mut s).unwrap();
        assert_eq!(got.as_deref(), Some("x\n"), "inline text is not trimmed");
        assert!(!s.used);
    }

    #[test]
    fn run_verify_command_reports_pass_and_fail() {
        let (ok, verdict) = run_verify_command("true").unwrap();
        assert!(ok);
        assert_eq!(verdict, "PASS (exit 0)");
        let (ok, verdict) = run_verify_command("exit 3").unwrap();
        assert!(!ok);
        assert_eq!(verdict, "FAIL (exit 3)");
    }

    /// A temp farm with one yak per status directory (`yak-0001` hairy,
    /// `-0002` shaving, `-0003` shorn, `-0004` dead; 0003 carries label `cli`).
    fn path_farm(tag: &str) -> (PathBuf, Farm) {
        let parent = env::temp_dir().join(format!("yaks-path-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&parent);
        let root = parent.join(".yaks");
        std::fs::create_dir_all(&root).unwrap();
        for (n, st) in [
            (1, Status::Hairy),
            (2, Status::Shaving),
            (3, Status::Shorn),
            (4, Status::Dead),
        ] {
            let mut t = task();
            t.id = format!("yak-000{n}");
            t.status = st;
            if n == 3 {
                t.tags = vec!["cli".into()];
            }
            store::write::save(&root, &t).unwrap();
        }
        let farm = match Farm::at(&root) {
            Ok(f) => f,
            Err(_) => panic!("failed to open temp farm"),
        };
        (root, farm)
    }

    /// Run `run_path`; returns (exit code, stdout lines, stderr).
    fn path_cmd(
        farm: &Farm,
        ids: &[&str],
        filter: FilterFlags,
        all: bool,
    ) -> (i32, Vec<String>, String) {
        let ids: Vec<String> = ids.iter().map(|s| s.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run_path(farm, &ids, filter, all, &mut out, &mut err).unwrap();
        let lines = String::from_utf8(out)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect();
        (code, lines, String::from_utf8(err).unwrap())
    }

    #[test]
    fn path_resolves_a_yak_in_each_status_dir() {
        let (root, farm) = path_farm("each");
        // One flat path per yak, whatever its status.
        for id in ["yak-0001", "yak-0002", "yak-0003", "yak-0004"] {
            let (code, out, _) = path_cmd(&farm, &[id], FilterFlags::default(), false);
            let want = store::task_path(&root, id);
            assert_eq!(code, 0);
            assert_eq!(out, vec![want.display().to_string()]);
            assert!(want.is_file());
        }
    }

    #[test]
    fn path_is_stable_across_a_transition() {
        let (root, farm) = path_farm("move");
        let before = farm.path_of("yak-0001").unwrap();
        store::move_task(&root, "yak-0001", Status::Shorn, None).unwrap();
        assert_eq!(farm.path_of("yak-0001").unwrap(), before);
        assert_eq!(before, store::task_path(&root, "yak-0001"));
    }

    #[test]
    fn path_takes_several_ids_in_order() {
        let (root, farm) = path_farm("many");
        let (code, out, _) = path_cmd(
            &farm,
            &["yak-0003", "yak-0001"],
            FilterFlags::default(),
            false,
        );
        assert_eq!(code, 0);
        assert_eq!(
            out,
            vec![
                store::task_path(&root, "yak-0003").display().to_string(),
                store::task_path(&root, "yak-0001").display().to_string(),
            ]
        );
    }

    #[test]
    fn path_unknown_id_fails_but_still_prints_known_ones() {
        let (root, farm) = path_farm("unknown");
        let (code, out, err) = path_cmd(
            &farm,
            &["yak-0001", "yak-nope"],
            FilterFlags::default(),
            false,
        );
        assert_eq!(code, 1);
        assert_eq!(
            out,
            vec![store::task_path(&root, "yak-0001").display().to_string()]
        );
        assert!(err.contains("no such task: yak-nope"), "{err}");
    }

    #[test]
    fn path_filter_form_selects_like_list() {
        let (root, farm) = path_farm("filter");
        let by = |f: FilterFlags, all| path_cmd(&farm, &[], f, all);
        let label = FilterFlags {
            tag: vec!["cli".into()],
            ..Default::default()
        };
        let (code, out, _) = by(label, false);
        assert_eq!(code, 0);
        assert_eq!(
            out,
            vec![store::task_path(&root, "yak-0003").display().to_string()]
        );

        // Dead yaks are excluded by default, included with --all, and always
        // reachable by an explicit --status dead.
        let herd = || FilterFlags {
            herd: vec!["yak".into()],
            ..Default::default()
        };
        assert_eq!(by(herd(), false).1.len(), 3);
        assert_eq!(by(herd(), true).1.len(), 4);
        let dead = FilterFlags {
            status: vec!["dead".into()],
            ..Default::default()
        };
        assert_eq!(
            by(dead, false).1,
            vec![store::task_path(&root, "yak-0004").display().to_string()]
        );
    }

    #[test]
    fn path_with_nothing_prints_the_farm_root_and_refuses_ids_plus_filter() {
        let (root, farm) = path_farm("xor");
        // Which farm am I on?
        let (code, out, err) = path_cmd(&farm, &[], FilterFlags::default(), false);
        assert_eq!((code, out), (0, vec![root.display().to_string()]));
        assert!(err.is_empty(), "{err}");
        let label = FilterFlags {
            tag: vec!["cli".into()],
            ..Default::default()
        };
        let (code, out, _) = path_cmd(&farm, &["yak-0001"], label, false);
        assert_eq!((code, out.len()), (1, 0));
    }

    fn task() -> Task {
        Task {
            id: "yak-0001".into(),
            title: "a title".into(),
            kind: "feature".into(),
            priority: 3,
            status: Status::Hairy,
            created: None,
            updated: None,
            parent: None,
            tags: vec![],
            depends_on: vec![],
            source: None,
            prs: Vec::new(),
            needs: None,
            verify: None,
            extra: Default::default(),
            body: String::new(),
        }
    }

    #[test]
    fn fmt_row_marks_needs_blocked() {
        let mut t = task();
        t.needs = Some("human".into());
        let row = fmt_row(&t);
        // A blocked yak carries a visible, greppable marker naming who it needs.
        assert!(row.contains("\u{26a0} needs:human"), "row was: {row:?}");
    }

    #[test]
    fn fmt_row_ready_has_no_needs_marker() {
        // A ready yak (needs == None) must stay distinguishable: no marker at all.
        let row = fmt_row(&task());
        assert!(!row.contains("needs:"), "row was: {row:?}");
        assert!(!row.contains('\u{26a0}'), "row was: {row:?}");
    }
}
