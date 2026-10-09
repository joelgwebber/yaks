//! yaks — a filesystem-native task tracker.
//!
//! This binary is a thin CLI over `farm::Farm` (the print-free core ops
//! facade): every command is parse args -> call one Farm op -> render (text or
//! --json). Tasks live as markdown files under `.yaks/`; status is the folder.

mod actor;
mod brief;
mod changes;
mod clipboard;
mod commit;
mod discover;
mod farm;
mod filter;
mod init;
mod json;
mod model;
mod preflight;
mod refs;
mod rollup;
mod sheds;
mod skills;
mod status;
mod store;
mod tui;

use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::env;

use farm::{
    AttachOutcome, Commits, CreateOutcome, DepOutcome, Farm, Issue, IssueKind, LogEntry,
    MergeOutcome, MoveOutcome, NewTask, OpenError, Pickup, RefKind, RenameAttachmentOutcome,
    RenameOutcome, RenamePlan, Reparent, Show, SlaughterOutcome, Stats, TaskEdit, TaskRefs,
    UpdateOutcome,
};
use filter::FilterSpec;
use model::{NEEDS_AGENT, Status, Task, normalize_labels};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "yaks",
    version,
    about = "Filesystem-native task tracker (Rust)"
)]
struct Cli {
    /// Run as if started in <PATH> (like `git -C`): farm discovery, `sheds`,
    /// `discover`, `commit`, `init`, `skills` and every other command see
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
    /// Keep only yaks with any of these labels. Repeatable; ORs. Comma- or
    /// space-separated lists are split (`--label ui,docs`).
    #[arg(long)]
    label: Vec<String>,
    /// Keep only yaks in these herds (id prefixes). Repeatable; ORs.
    #[arg(long)]
    herd: Vec<String>,
    #[arg(long)]
    search: Option<String>,
    #[arg(long)]
    ready: bool,
    #[arg(long)]
    tangled: bool,
    /// Keep only yaks with a `needs` field set: awaiting a human (`needs:
    /// human`) or answered and awaiting an agent (`needs: agent`).
    #[arg(long)]
    needs: bool,
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
            || !self.label.is_empty()
            || !self.herd.is_empty()
            || self.search.is_some()
            || self.ready
            || self.tangled
            || self.needs
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
        #[arg(long)]
        json: bool,
    },
    /// Show one task by id.
    Show {
        id: String,
        #[arg(long)]
        json: bool,
    },
    /// Print the current on-disk file path of yaks, one per line: by id
    /// (`yaks path yaks-1a2b ...`) or by the shared filter flags
    /// (`yaks path --status shorn --label cli`). A transition moves a yak's file
    /// between status directories, so use this instead of hand-building
    /// `.yaks/<status>/<id>.md` for `git add`. Paths are absolute. An unknown
    /// id is reported on stderr and the exit status is non-zero (known ids are
    /// still printed). Give ids or filter flags, not both.
    Path {
        /// Yak ids to resolve.
        ids: Vec<String>,
        #[command(flatten)]
        filter: FilterFlags,
        /// With filter flags, also include dead yaks (as `list --all`).
        #[arg(long)]
        all: bool,
    },
    /// List the yaks a task points at (parent, deps, and id mentions in its
    /// text), flagging any formal reference that dangles.
    Refs { id: String },
    /// Show the git commits linked to a yak: those naming its id and those that
    /// touched its file across status moves.
    Commits { id: String },
    /// Run a yak's recorded `verify:` command and record the PASS/FAIL as an
    /// attributed evidence note. Explicit — yaks never auto-runs it. Exits
    /// non-zero if any verify fails (or a yak has no `verify:` command).
    Verify {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the result note to this actor (stamped as `[actor]`).
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Attach a local file to a yak as evidence: copy it under
    /// `.yaks/artifacts/<id>/` and link it in the body. Use for artifact
    /// evidence a human or coordinator can look at (screenshots, TUI frames).
    Attach {
        id: String,
        /// Path to the local file to attach.
        path: PathBuf,
        /// Store the attachment under this filename instead of the source's
        /// (sanitized; the source's extension is kept if NAME omits one).
        #[arg(long)]
        name: Option<String>,
        /// An optional attributed note to record alongside the attachment.
        #[arg(long)]
        note: Option<String>,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Rename a yak's attachment in `.yaks/artifacts/<id>/` and rewrite every
    /// link to it across the farm. NEW is sanitized and keeps OLD's extension
    /// if it omits one; refuses to overwrite an existing attachment.
    RenameAttachment {
        id: String,
        /// Current filename under `.yaks/artifacts/<id>/`.
        old: String,
        /// New filename (e.g. `login-page`, becomes `login-page.png`).
        new: String,
    },
    /// Scan text for tokens that are real yak-ids in this farm — a leak check
    /// for private-mode farms (pre-commit / PR hook). Reads a FILE and/or
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
    /// Timestamped notes and status moves across a filtered set, oldest first (an
    /// activity log).
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
    /// List the other checkouts of this repo (git worktrees and Delta clones)
    /// and what each one's farm CHANGED since the shed forked (the oldest
    /// entry of its HEAD reflog, so a worker is not credited with what its
    /// coordinator did first; or, for a shed synced with this checkout since
    /// it forked, the merge-base with this checkout when that is nearer, so
    /// what it merged in is not counted): yaks only there, yaks in another
    /// status, new notes, `needs:` set. With no usable reflog the baseline is
    /// the merge-base with this checkout and the output says `(vs merge-base)`
    /// (JSON `farm.vs`: `fork`, `sync`, `merge-base` or `checkout`). Also how far
    /// ahead/behind its HEAD is of this checkout's (`+? -?`, JSON null, when
    /// there is no merge-base here, e.g. shallow history; the table then says
    /// once to `git fetch --unshallow`). A shed that is merely behind shows
    /// no changes. Each shed with entries of its own is
    /// labeled under its row: `who:` (the distinct actors on those new notes)
    /// and `shaving:` (yaks in shaving there that it wrote to); JSON `who`
    /// and `in_progress`. Read-only everywhere.
    Sheds {
        /// Emit the sheds as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Diagnostic: show the chain `yaks sheds` uses to find the other
    /// checkouts of this repo, step by step, from inside any checkout of it:
    /// `git worktree list` here; the HOST repo this checkout's
    /// `objects/info/alternates` names (the human's checkout, or Delta's
    /// managed bare repo on a machine the thread was shared to); every
    /// `refs/delta/<dir>/<name>/*` pin group there, resolved to a live
    /// checkout through the clone's git dir `core.worktree` (or reported
    /// gone); and, when the host is a checkout, its own `git worktree list`.
    /// Git data only: nothing is searched for. Read-only; needs no farm.
    Discover {
        /// Where to look from (default: the current directory): any directory
        /// inside a primary checkout, a git worktree, or a Delta clone.
        path: Option<PathBuf>,
        /// Emit the report as JSON.
        #[arg(long)]
        json: bool,
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
        #[arg(long)]
        parent: Option<String>,
        /// Which herd (id prefix) the new yak joins; defaults to this repo's
        /// `.yaks` pointer herd, else the farm's configured `herd:`. Errors if
        /// neither is set (no silent `yak-` fallback). Lets one farm hold
        /// several herds.
        #[arg(long = "herd")]
        prefix: Option<String>,
        /// Labels for the new yak. Commas and spaces separate labels (a label
        /// may contain neither): `--labels ui,docs`, `--labels 'ui docs'`, and
        /// `--labels ui docs` are equivalent. Duplicates are dropped.
        #[arg(long, num_args = 1..)]
        labels: Vec<String>,
        #[arg(long = "depends-on", num_args = 1..)]
        depends_on: Vec<String>,
        #[arg(long)]
        source: Option<String>,
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
        /// Print the created task's id, path, and basic fields as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Update fields, labels, or append a timestamped note. Accepts multiple
    /// ids; the same edit is applied to each.
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
        /// Labels to add. Commas and spaces separate labels (`ui,docs`).
        #[arg(long = "add-label", num_args = 1..)]
        add_label: Vec<String>,
        /// Labels to remove. Commas and spaces separate labels (`ui,docs`).
        #[arg(long = "remove-label", num_args = 1..)]
        remove_label: Vec<String>,
        /// Set (or clear, with an empty string) this yak's `source:` URL.
        #[arg(long)]
        source: Option<String>,
        /// Set (or clear, with an empty string) this yak's `verify:` command.
        #[arg(long)]
        verify: Option<String>,
        /// Append a timestamped note (markdown). `-` reads it from stdin, so a
        /// multi-line heredoc or pipe arrives intact.
        #[arg(long, conflicts_with = "note_file")]
        note: Option<String>,
        /// Read the note from this file (`-` is stdin).
        #[arg(long = "note-file", value_name = "PATH")]
        note_file: Option<String>,
        /// Attribute the note to this actor (stamped as `[actor]`). Defaults to
        /// $YAKS_ACTOR, then the harness identity (`delta:<thread title>`, else
        /// `delta:<thread id>`, from Delta's environment), then the git user;
        /// never implies ownership.
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Block a yak on a human decision and record the question. Sets the `needs`
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
    /// Answer a question: record the reply and set `needs: agent`, so the yak
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
    /// Pick up an answered yak: clear `needs: agent` and record who took the
    /// work on (a `picked up` note). The only command that clears it; errors
    /// on a yak that is not `needs: agent`.
    Pickup {
        id: String,
        /// Optional note (what you will do with the answer). `-` reads stdin.
        #[arg(long, conflicts_with = "note_file")]
        note: Option<String>,
        /// Read the note from this file (`-` is stdin).
        #[arg(long = "note-file", value_name = "PATH")]
        note_file: Option<String>,
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// List yaks with a `needs` value, across all statuses, in two sections:
    /// awaiting a human (`needs: human`; `replied` marks one whose latest note
    /// after the ask is by someone other than the asker) and answered, awaiting
    /// an agent (`needs: agent`). `--for human|agent` shows one section.
    Inbox {
        #[command(flatten)]
        filter: FilterFlags,
        /// Show only one section.
        #[arg(long = "for", value_enum)]
        audience: Option<Audience>,
        /// After the inbox, list the open asks in the OTHER sheds (git
        /// worktrees and Delta clones of this repository; see `sheds`): per
        /// shed, its name and path, then each yak with `needs` set in that
        /// shed's own farm and the question text. Asks identical in this
        /// checkout's farm are not repeated; sheds with no farm of their own
        /// show nothing. `--for` and the filter flags apply. Read-only.
        #[arg(long)]
        sheds: bool,
        /// JSON array of yaks, each with `needs` and `replied`. With
        /// `--sheds`, an object instead: `inbox` (that array) and `sheds`
        /// (one object per ask in another shed, with `shed`, `path`, `id`,
        /// `title`, `status`, `needs`, `replied` and `question`).
        #[arg(long)]
        json: bool,
    },
    /// Start shaving one or more yaks (move to shaving).
    #[command(visible_alias = "work")]
    Shave {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then the harness identity, then the git user.
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Mark one or more yaks shorn (move to shorn).
    #[command(visible_alias = "close")]
    Shorn {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then the harness identity, then the git user.
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Regrow one or more shorn yaks (move back to hairy).
    #[command(visible_alias = "reopen")]
    Regrow {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then the harness identity, then the git user.
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Slaughter one or more yaks (move to dead).
    ///
    /// Refuses a yak that still has live (non-dead) descendants, since that
    /// would orphan them; pass --family to slaughter the whole family instead.
    Slaughter {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Also slaughter every live descendant (children, grandchildren, ...)
        /// of each id, instead of refusing.
        #[arg(long)]
        family: bool,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then the harness identity, then the git user.
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Revive one or more dead yaks (move back to hairy).
    Revive {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        /// Attribute the move to this actor (stamped as `[actor]` on its
        /// `moved: <from> -> <to>` log entry). Resolved like `update --as`:
        /// $YAKS_ACTOR, then the harness identity, then the git user.
        #[arg(long = "as")]
        as_actor: Option<String>,
    },
    /// Add or remove a dependency.
    Dep {
        #[command(subcommand)]
        action: DepAction,
    },
    /// Move one or more tasks under a new parent (--parent) or to top-level
    /// (--unparent). Every id is reparented to the same destination.
    Reparent {
        #[arg(required = true, num_args = 1..)]
        ids: Vec<String>,
        #[arg(long)]
        parent: Option<String>,
        #[arg(long)]
        unparent: bool,
    },
    /// Apply the same field edit (and/or reparent) to every yak matching a
    /// filter. DRY-RUN BY DEFAULT: without --commit it only prints the matched
    /// set and the mutation, changing nothing. Requires at least one filter flag
    /// (never operates on the whole farm) and at least one mutation flag. Field
    /// edits + reparent only — no state transitions (see yaks-7cc8).
    Bulk {
        #[command(flatten)]
        filter: FilterFlags,
        /// Labels to add to every matched yak. Commas and spaces separate labels.
        #[arg(long = "add-label", num_args = 1..)]
        add_label: Vec<String>,
        /// Labels to remove from every matched yak. Commas and spaces separate
        /// labels.
        #[arg(long = "remove-label", num_args = 1..)]
        remove_label: Vec<String>,
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
        /// Actually apply the mutation. Without it, this is a dry run.
        #[arg(long)]
        commit: bool,
    },
    /// Rename a yak, updating its file + id and every reference to it
    /// (parent, depends_on, and body/title mentions) across the farm.
    Rename {
        old: String,
        new: String,
        /// Preview the change without writing anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Rename a whole herd: migrate every yak from one id prefix to another
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
    /// Merge another farm's yaks into this one (consolidation). Copies every yak
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
    /// Create a .yaks/ farm in the current directory (hairy/ shaving/ shorn/
    /// dead/ + config.yaml + schema). Works without an existing farm.
    ///
    /// With no --mode/--skills/--path this only creates a committed (team) farm
    /// and tells you how to install skills. Give --mode and/or --skills for the
    /// full setup: the farm per mode, the git excludes it implies, and the
    /// project-local skills (default set unless --skills none). Every step is
    /// idempotent, so the same command can be re-run, and a re-run with more
    /// flags adds only what is missing. A different --mode on an existing farm
    /// is an error, never a silent conversion.
    Init {
        /// Default herd (id prefix) for new yaks (default: yak). With
        /// --mode pointer it is also this repo's `herd:` in the pointer file.
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
        /// Farm mode (a new farm is team when only --skills is given). team: the
        /// farm in .yaks/, committed with the code. private: the farm in
        /// .yaks/, listed in .git/info/exclude. pointer: a `.yaks` pointer file
        /// to a farm at --path (created if absent), listed in
        /// .git/info/exclude. private and pointer need a git repo.
        #[arg(long, value_enum)]
        mode: Option<init::Mode>,
        /// For --mode pointer: the directory holding the farm (its `.yaks/`,
        /// or the farm itself). Written to the pointer file as given.
        #[arg(long, value_name = "DIR")]
        path: Option<String>,
        /// Skills to install project-local in the git top-level, through the
        /// same install as `yaks skills install` (never overwrites an edited
        /// skill). In private/pointer modes their directories go in
        /// .git/info/exclude too. With --mode alone the default is `default`.
        #[arg(long, value_enum)]
        skills: Option<init::SkillSet>,
    },
    /// Install the bundled agent skills (yaks, yaks-tracker) into a skills
    /// directory. Works anywhere — no farm required.
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },
    /// Group yaks by the external issue they roll up to.
    Rollup(RollupArgs),
    /// Read-only farm-integrity check: report duplicate-status ids and dangling
    /// parent/depends_on references. Exits non-zero when any issue is found.
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
    /// Read-only landing-readiness check for a team farm: nothing under
    /// `.yaks/` untracked or with unstaged changes in git, the verify command of
    /// each shorn yak in the change last PASSed, no yak in two status dirs. Prints
    /// one line per failure and exits non-zero, else `preflight: ok`. A private
    /// farm skips the git check (said so) and checks every shorn yak. With
    /// `--push-main` also checks that the `local` remote is the human's checkout
    /// (not bare, has `main`, not dirty on `main`) before `git push local
    /// <branch>:main`; read-only, never pushes.
    Preflight {
        /// Shorn yaks to check (default: the shorn yaks in the change in git,
        /// i.e. staged, modified or new under `.yaks/`). Scopes the verify check
        /// only; the git and duplicate-status checks are farm-wide.
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
    /// Commit the farm's own changes (everything under `.yaks/`: yak files,
    /// artifacts, config) and nothing else, with a generated message such as
    /// `yaks: shorn yaks-abc1; updated yaks-def2`. Code staged or modified
    /// elsewhere is left alone (staged files stay staged). Runs a normal
    /// `git commit`, so the repo's hooks apply; never pushes. Fails in a private
    /// farm (nothing under `.yaks/` tracked by git) and while a merge or
    /// cherry-pick is in progress (git forbids a partial commit then; finish it
    /// with a plain `git commit`); a farm with no changes is a no-op that exits 0.
    Commit {
        /// Commit message (default: generated from the changed files).
        #[arg(short, long)]
        message: Option<String>,
        /// Print the files and message without staging or committing.
        #[arg(long)]
        dry_run: bool,
    },
    /// Show what the farm has changed that git does not have yet, one line per
    /// yak in yak terms (`created`, `moved hairy -> shaving`, `notes +2`,
    /// `edited`, `removed`, `artifacts`; `config` for config.yaml), `*` marking
    /// what is staged, ordered by id, then what `yaks commit` would do (and its
    /// message). Uses the same classification as `yaks commit`. Read-only. Code
    /// changed elsewhere in the repo is only counted, never listed. A clean farm
    /// prints `farm clean: nothing to commit`; a private farm (nothing under
    /// `.yaks/` tracked by git) says so; both exit 0. While a merge or
    /// cherry-pick is in progress it says `yaks commit` will refuse.
    Status {
        /// Exit 1 when the farm has uncommitted changes (for scripts and
        /// hooks); the output is the same. Without it the exit code is 0.
        #[arg(long)]
        check: bool,
        /// Emit JSON: `clean`, `private`, `merge_in_progress`, `yaks` (array of
        /// `{id, changes, staged}`), `other` (farm files that are not yaks, as
        /// `{path, staged}`), `outside_farm` (count) and `message`.
        #[arg(long)]
        json: bool,
    },
    /// Print the worker brief for one yak: what a cold worker needs to start,
    /// derived from the yak, the config and the farm mode. Names the yak and
    /// that it is already `shaving`, the `YAKS_ACTOR=<name>` prefix for every
    /// command (plus `YAKS_DIR` for a private or pointer farm, or with
    /// `--yaks-dir`), this binary's path, the yak's `verify:` gate (else the
    /// config default), the forbidden moves, how to finish (team: one commit
    /// with the yak move; private: no yak ids in commits), how to ask, the
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
        /// does not hold the farm. Default: the farm in use for a private or
        /// pointer farm, nothing for a team farm.
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
enum SkillsAction {
    /// Write the bundled agent skills (yaks, yaks-tracker; --with adds more)
    /// into a skills directory. By default that is ./.agents/skills in the git
    /// top-level of the current directory, so the skills travel with the
    /// project; outside a git repo it is ~/.agents/skills. Prints the
    /// destination it chose. Works without a farm.
    Install {
        /// Target skills directory (e.g. ~/.claude/skills). Wins over --user
        /// and the git-repo default.
        #[arg(long, conflicts_with = "user")]
        dir: Option<String>,
        /// Install into the user directory, ~/.agents/skills, even inside a git repo.
        #[arg(long)]
        user: bool,
        /// Also install an opt-in skill group. `coordination` adds
        /// yaks-coordinating (+ -team, -private, -worktrees, -delta) and
        /// yaks-working, for running several agents over one farm.
        #[arg(long, value_name = "GROUP", value_delimiter = ',',
              value_parser = clap::builder::PossibleValuesParser::new(skills::GROUPS))]
        with: Vec<String>,
        /// Overwrite skills that were edited after they were installed. (Never
        /// permits writing into yaks' own .agents/skills/ source.)
        #[arg(long)]
        force: bool,
    },
    /// Report whether each installed skill is current, stale, or locally
    /// edited, by comparing its provenance stamp against this binary. Looks in
    /// the same default directory as `install`.
    Status {
        /// Skills directory to inspect. Defaults like `install`: the git
        /// top-level's .agents/skills, else ~/.agents/skills.
        #[arg(long, conflicts_with = "user")]
        dir: Option<String>,
        /// Inspect the user directory, ~/.agents/skills.
        #[arg(long)]
        user: bool,
        /// Also report on an opt-in skill group even if not installed yet
        /// (installed ones are always reported).
        #[arg(long, value_name = "GROUP", value_delimiter = ',',
              value_parser = clap::builder::PossibleValuesParser::new(skills::GROUPS))]
        with: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // `-C`: change directory once, before anything reads the cwd, so every
    // `env::current_dir()` below (discovery, init, skills, sheds) agrees.
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
        mode,
        path,
        skills,
    } = &cli.command
    {
        return init::run(init::Args {
            herd: prefix.clone(),
            kind: kind.clone(),
            priority: *priority,
            emacs: *emacs,
            mode: *mode,
            path: path.clone(),
            skills: *skills,
        });
    }
    if let Command::Skills { action } = &cli.command {
        return run_skills(action);
    }
    if let Command::Discover { path, json } = &cli.command {
        let anchor = match path {
            Some(p) => p.clone(),
            None => env::current_dir()?,
        };
        let yaks_dir = env::var("YAKS_DIR").ok();
        let report = match discover::run(&anchor, yaks_dir.as_deref()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("error: {e:#}");
                std::process::exit(1);
            }
        };
        if *json {
            json::print(&discover::to_json(&report))?;
        } else {
            print!("{}", discover::render(&report));
        }
        return Ok(());
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

    let farm = match Farm::open(&env::current_dir()?) {
        Ok(h) => h,
        Err(OpenError::SchemaTooNew { found, supported }) => {
            eprintln!(
                "error: this farm uses schema v{found}, newer than this yaks supports (v{supported}). Upgrade yaks."
            );
            std::process::exit(1);
        }
        Err(OpenError::NoFarm(m)) => {
            eprintln!("error: {m}");
            std::process::exit(1);
        }
    };
    if let Some(w) = &farm.schema_warning {
        eprintln!("warning: {w}");
    }

    match cli.command {
        Command::List { filter, all, json } => {
            let rows = farm.list(build_spec(filter), all)?;
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
        Command::Log {
            filter,
            since,
            by,
            json,
        } => {
            let entries = farm.log(build_spec(filter), since.as_deref(), by.as_deref())?;
            render_log(&entries, json)?;
        }
        Command::Show { id, json } => match farm.show(&id)? {
            None => {
                eprintln!("no such task: {id}");
                std::process::exit(1);
            }
            Some(s) => {
                if json {
                    json::print(&json::show_value(&s.task, &s.children))?;
                } else {
                    render_show(&s);
                }
            }
        },
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
                        println!("  {id} [{}]", st.dir());
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
        Command::Init { .. } => unreachable!("init is handled before opening a farm"),
        Command::Skills { .. } => unreachable!("skills is handled before opening a farm"),
        Command::Discover { .. } => unreachable!("discover is handled before opening a farm"),
        Command::Refs { id } => match farm.refs(&id)? {
            None => {
                eprintln!("no such task: {id}");
                std::process::exit(1);
            }
            Some(r) => render_refs(&r),
        },
        Command::Commits { id } => match farm.commits(&id)? {
            None => {
                eprintln!("no such task: {id}");
                std::process::exit(1);
            }
            Some(c) => render_commits(&c),
        },
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
                    None => match cfg
                        .resolve_verify(&show.task.labels, show.task.id.split('-').next())
                    {
                        Some(c) => (c, "config".to_string()),
                        None => {
                            eprintln!(
                                "error: {id} has no verify: command and no config default \
                                 for its labels (set one with `yaks update {id} --verify \
                                 '<cmd>'`, or add a config `verify:` entry)"
                            );
                            std::process::exit(1);
                        }
                    },
                };
                println!("verify {id} ({source}): {cmd}");
                let (ok, verdict) = run_verify_command(&cmd)?;
                all_ok &= ok;
                farm.update(
                    id,
                    TaskEdit {
                        note: Some(format!("verify: `{cmd}` -> {verdict}")),
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
            path,
            name,
            note,
            as_actor,
        } => {
            let data = match std::fs::read(&path) {
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
            let name = match name {
                None => src_name.to_string(),
                Some(raw) => match farm::attachment_name(&raw, src_name) {
                    Some(n) => n,
                    None => {
                        eprintln!("error: invalid attachment name: {raw:?}");
                        std::process::exit(1);
                    }
                },
            };
            match farm.attach(&id, &name, &data)? {
                AttachOutcome::NotFound => {
                    eprintln!("no such task: {id}");
                    std::process::exit(1);
                }
                AttachOutcome::Attached(n) => {
                    println!("attached {n} to {id} (.yaks/artifacts/{id}/{n})");
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
                    "renamed {old} -> {name} on {id} (.yaks/artifacts/{id}/{name}); \
                     links rewritten in {rewritten} yak(s)"
                ),
                RenameAttachmentOutcome::Unchanged => println!("{old}: name unchanged"),
                refused => {
                    let why = match refused {
                        RenameAttachmentOutcome::TaskNotFound => format!("no such task: {id}"),
                        RenameAttachmentOutcome::NoSuchAttachment(n) => {
                            format!("no attachment {n} on {id} (.yaks/artifacts/{id}/)")
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
        Command::Sheds { json } => {
            let sheds = farm.sheds(&env::current_dir()?)?;
            if json {
                json::print(&sheds::to_json(&sheds))?;
            } else {
                print!("{}", sheds::render(&sheds));
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
            labels,
            depends_on,
            source,
            description,
            description_file,
            verify,
            json,
        } => {
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
                labels,
                depends_on,
                source,
                description,
                verify,
            };
            match farm.create(new)? {
                CreateOutcome::ParentNotFound(p) => {
                    eprintln!("error: parent task {p} not found");
                    std::process::exit(1);
                }
                CreateOutcome::InvalidPrefix(p) => {
                    eprintln!("error: invalid prefix {p:?} (use lowercase letters and digits)");
                    std::process::exit(1);
                }
                CreateOutcome::Created(t) => {
                    if json {
                        let path = farm
                            .root()
                            .join(t.status.dir())
                            .join(format!("{}.md", t.id));
                        json::print(&json::create_value(&t, &path))?;
                    } else {
                        println!("Created {}: {}", t.id, t.title);
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
            add_label,
            remove_label,
            source,
            verify,
            note,
            note_file,
            as_actor,
        } => {
            let mut stdin = StdinText::real();
            let description =
                resolve_text("--description", description, description_file, &mut stdin)?;
            let note = resolve_text("--note", note, note_file, &mut stdin)?;
            let actor = note
                .as_ref()
                .and_then(|_| actor::resolve(as_actor.as_deref()));
            let edit = TaskEdit {
                title,
                kind,
                priority,
                description,
                add_labels: add_label,
                remove_labels: remove_label,
                source,
                verify,
                note,
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
            as_actor,
        } => {
            let note = resolve_text("--note", note, note_file, &mut StdinText::real())?;
            let actor = actor::resolve(as_actor.as_deref());
            match farm.pickup(&id, actor.as_deref(), note.as_deref())? {
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
        Command::Inbox {
            filter,
            audience,
            sheds,
            json,
        } => {
            let spec = build_spec(filter);
            let rows = farm.inbox(spec.clone())?;
            let other: Option<Vec<farm::ShedAsk>> = if sheds {
                let all = farm.inbox_sheds(&env::current_dir()?, spec)?;
                Some(
                    all.into_iter()
                        .filter(|a| Audience::keeps(audience, &a.task))
                        .collect(),
                )
            } else {
                None
            };
            render_inbox(&rows, other.as_deref(), audience, json)?;
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
            add_label,
            remove_label,
            set_priority,
            set_type,
            reparent,
            unparent,
            commit,
        } => {
            // Safety rule 1: never operate on the whole farm implicitly.
            if !filter.any_set() {
                eprintln!(
                    "error: bulk requires at least one filter flag (e.g. --label, --status, --priority); refusing to select the whole farm"
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
            let add_label = normalize_labels(&add_label);
            let remove_label = normalize_labels(&remove_label);
            let does_field_edit = !add_label.is_empty()
                || !remove_label.is_empty()
                || set_priority.is_some()
                || set_type.is_some();
            let does_reparent = unparent || reparent.is_some();
            // Safety rule 2: require at least one mutation flag.
            if !does_field_edit && !does_reparent {
                eprintln!(
                    "error: bulk requires at least one mutation flag (--add-label/--remove-label/--set-priority/--set-type/--reparent/--unparent)"
                );
                std::process::exit(1);
            }

            let rows = farm.list(build_spec(filter), false)?;
            if rows.is_empty() {
                println!("No yaks matched the filter; nothing to do.");
                return Ok(());
            }
            let ids: Vec<String> = rows.iter().map(|t| t.id.clone()).collect();
            let mutation = describe_bulk_mutation(
                &add_label,
                &remove_label,
                set_priority,
                set_type.as_deref(),
                reparent.as_deref(),
                unparent,
            );

            // Safety rule 3: dry-run is the default — print and change nothing.
            if !commit {
                println!("would update {} yaks:", rows.len());
                for t in &rows {
                    println!("  {}  {}", t.id, t.title);
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
                    add_labels: add_label,
                    remove_labels: remove_label,
                    source: None,
                    verify: None,
                    note: None,
                    actor: None,
                };
                update_many(&farm, &ids, edit)?;
            }
            if does_reparent {
                let new_parent = if unparent { None } else { reparent };
                reparent_many(&farm, &ids, new_parent)?;
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
        Command::Preflight {
            ids,
            all,
            push_main,
            json,
        } => {
            let report = preflight::run(&farm, &ids, all, push_main)?;
            if json {
                json::print(&serde_json::json!({
                    "ok": report.ok(),
                    "failures": report.failures.iter().map(|f| serde_json::json!({
                        "check": f.check.code(),
                        "message": f.message,
                        "subjects": f.subjects,
                    })).collect::<Vec<_>>(),
                    "skipped": report.skipped,
                }))?;
            } else {
                for line in &report.skipped {
                    println!("preflight: {line}");
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
    if ids.is_empty() == !filter.any_set() {
        writeln!(
            err,
            "error: path takes yak ids or filter flags (not both, not neither)"
        )?;
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

fn build_spec(f: FilterFlags) -> FilterSpec {
    FilterSpec {
        statuses: f.status.iter().filter_map(|s| parse_status(s)).collect(),
        types: f.kind,
        priorities: f.priority,
        labels: normalize_labels(&f.label),
        herds: f.herd,
        search: f.search,
        ready_only: f.ready,
        tangled_only: f.tangled,
        needs_only: f.needs,
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
/// success (moved or already there) and `false` on failure (not found), so a
/// batch caller can process every id and set the exit code once at the end.
fn transition(
    farm: &Farm,
    id: &str,
    dest: Status,
    already: &str,
    done: &str,
    actor: Option<&str>,
) -> Result<bool> {
    match farm.transition(id, dest, actor)? {
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
            Ok(true)
        }
    }
}

/// Transition every id in `ids`, one at a time. All ids are processed even if
/// one fails (no abort-on-first-error); if any id was not found, the process
/// exits non-zero after the whole batch is handled.
fn transition_many(
    farm: &Farm,
    ids: &[String],
    dest: Status,
    already: &str,
    done: &str,
    actor: Option<&str>,
) -> Result<()> {
    let mut any_failed = false;
    for id in ids {
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
/// refused for having live descendants exits non-zero afterwards (yaks-05da).
fn slaughter_many(farm: &Farm, ids: &[String], family: bool, actor: Option<&str>) -> Result<()> {
    let mut any_failed = false;
    for id in ids {
        match farm.slaughter(id, family, actor)? {
            SlaughterOutcome::NotFound => {
                eprintln!("error: task {id} not found");
                any_failed = true;
            }
            SlaughterOutcome::AlreadyDead => println!("{id} is already dead"),
            SlaughterOutcome::HasLiveDescendants(kids) => {
                let n = kids.len();
                let noun = if n == 1 { "descendant" } else { "descendants" };
                eprintln!(
                    "error: {id} has {n} live {noun} ({}); slaughter them first, \
                     or use --family to slaughter the whole family",
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
/// when the id was not found (so the caller can exit non-zero).
fn update_one(farm: &Farm, id: &str, edit: TaskEdit) -> Result<bool> {
    match farm.update(id, edit)? {
        UpdateOutcome::NotFound => {
            eprintln!("error: task {id} not found");
            Ok(false)
        }
        UpdateOutcome::Updated => {
            println!("Updated {id}");
            Ok(true)
        }
        UpdateOutcome::NoChanges => {
            println!("No changes specified.");
            Ok(true)
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

/// Human-readable one-line summary of the field edit and/or reparent a bulk run
/// would apply. Used in both the dry-run preview and the commit log line.
fn describe_bulk_mutation(
    add_label: &[String],
    remove_label: &[String],
    set_priority: Option<u8>,
    set_type: Option<&str>,
    reparent: Option<&str>,
    unparent: bool,
) -> String {
    let mut parts: Vec<String> = Vec::new();
    if !add_label.is_empty() {
        parts.push(format!("add labels [{}]", add_label.join(", ")));
    }
    if !remove_label.is_empty() {
        parts.push(format!("remove labels [{}]", remove_label.join(", ")));
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
    parts.join("; ")
}

/// Which side of the `needs` hand-off `inbox --for` keeps.
#[derive(Clone, Copy, clap::ValueEnum)]
enum Audience {
    /// Open questions (`needs: human`).
    Human,
    /// Answered, awaiting pickup (`needs: agent`).
    Agent,
}

impl Audience {
    /// Whether `audience` (`None` = both) keeps a yak with this `needs`.
    fn keeps(audience: Option<Audience>, t: &Task) -> bool {
        match audience {
            None => true,
            Some(Audience::Human) => t.awaiting_human(),
            Some(Audience::Agent) => t.awaiting_agent(),
        }
    }
}

/// `inbox` output: a flat JSON array (each yak with `needs` and `replied`), or
/// two text sections, "awaiting a human" and "answered, awaiting an agent".
/// With `sheds` (`inbox --sheds`, already narrowed by `audience`), the JSON is
/// `{inbox, sheds}` instead and the text gains a section per shed.
fn render_inbox(
    rows: &[Task],
    sheds: Option<&[farm::ShedAsk]>,
    audience: Option<Audience>,
    json: bool,
) -> Result<()> {
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
        match sheds {
            None => json::print(&json::inbox_array(&kept))?,
            Some(s) => json::print(&json::inbox_sheds_value(
                &kept,
                &s.iter().collect::<Vec<_>>(),
            ))?,
        }
        return Ok(());
    }
    if human.is_empty() && agent.is_empty() {
        println!(
            "{}",
            match audience {
                Some(Audience::Human) => "Inbox empty: nothing awaiting a human.",
                Some(Audience::Agent) => "Inbox empty: nothing answered and awaiting an agent.",
                None => "Inbox empty: nothing awaiting a human or an agent.",
            }
        );
    }
    for (title, section) in [
        ("Awaiting a human:", &human),
        ("Answered, awaiting an agent (`yaks pickup <id>`):", &agent),
    ] {
        if section.is_empty() {
            continue;
        }
        println!("{title}");
        for t in section {
            let replied = if store::is_replied(t) {
                " \u{21a9} replied"
            } else {
                ""
            };
            println!("{}{replied}", fmt_row(t));
        }
    }
    if let Some(sheds) = sheds {
        render_shed_asks(sheds);
    }
    Ok(())
}

/// The `inbox --sheds` text: one block per shed (`asks` arrive grouped by
/// shed), each ask a normal inbox row followed by its question text indented.
fn render_shed_asks(asks: &[farm::ShedAsk]) {
    if asks.is_empty() {
        println!("No open asks in other sheds.");
        return;
    }
    let mut last: Option<(&str, &std::path::Path)> = None;
    for a in asks {
        if last != Some((a.shed.as_str(), a.path.as_path())) {
            println!("Shed {} ({}):", a.shed, a.path.display());
            last = Some((a.shed.as_str(), a.path.as_path()));
        }
        let replied = if store::is_replied(&a.task) {
            " \u{21a9} replied"
        } else {
            ""
        };
        println!("{}{replied}", fmt_row(&a.task));
        match &a.question {
            Some(q) => {
                for line in q.lines() {
                    println!("      {line}");
                }
            }
            None => println!("      (no question text)"),
        }
    }
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
    let stuck: Vec<_> = skills::status(&base, &skills::select_for_status(&base, &[]))
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

fn run_skills(action: &SkillsAction) -> Result<()> {
    match action {
        SkillsAction::Install {
            dir,
            user,
            with,
            force,
        } => {
            let target = skills::resolve_target(dir.as_deref(), *user, &env::current_dir()?);
            let base = target.dir.clone();
            println!("Installing skills into {}", target.describe());
            let installed = skills::install(&base, *force, &skills::select(with))?;
            for i in &installed {
                println!(
                    "{}",
                    skills::render_installed(i, " (use --force to overwrite)")
                );
            }
            println!(
                "\nThe skill activates when a .yaks/ directory is present. For another agent, re-run with --dir pointing at its skills directory (e.g. --dir ~/.claude/skills)."
            );
            if matches!(target.kind, skills::TargetKind::Project { .. }) {
                println!(
                    "These are files in your working tree: commit them to share them, and re-run `yaks skills install` after upgrading yaks (only ~/.agents/skills is refreshed automatically; yaks tells you once per release when these fall behind)."
                );
            }
            Ok(())
        }
        SkillsAction::Status { dir, user, with } => {
            let target = skills::resolve_target(dir.as_deref(), *user, &env::current_dir()?);
            let base = target.dir.clone();
            println!("{}", target.describe());
            let mut blocked = 0;
            let statuses = skills::status(&base, &skills::select_for_status(&base, with));
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
                    "\n{} stale; run `{}` to update them.",
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

fn render_commits(c: &Commits) {
    println!("Commits mentioning {}:", c.id);
    if c.by_message.is_empty() {
        println!("  (none)");
    } else {
        for l in &c.by_message {
            println!("  {l}");
        }
    }
    println!("\nCommits touching {}:", c.path.display());
    if c.by_file.is_empty() {
        println!("  (none)");
    } else {
        for l in &c.by_file {
            println!("  {l}");
        }
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
    if !t.labels.is_empty() {
        println!("labels:   {}", t.labels.join(", "));
    }
    if !t.depends_on.is_empty() {
        println!("depends:  {}", t.depends_on.join(", "));
    }
    if let Some(src) = &t.source {
        println!("source:   {src}");
    }
    if let Some(n) = &t.needs {
        println!("needs:    {n}");
    }
    // Preserved but unmodeled frontmatter (Task.extra), surfaced read-only so a
    // hand-added/newer key is visible in `show` without this binary owning it.
    if !t.extra.is_empty() {
        println!("\nOther fields:");
        for line in &t.extra {
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

/// `  [X] id  pN type     title [labels] (deps: ...) ⚠ needs:<who>`
fn fmt_row(t: &Task) -> String {
    let labels = if t.labels.is_empty() {
        String::new()
    } else {
        format!(" [{}]", t.labels.join(","))
    };
    let deps = if t.depends_on.is_empty() {
        String::new()
    } else {
        format!(" (deps: {})", t.depends_on.join(","))
    };
    // Make a needs-blocked yak visually distinct from a ready one in list/next;
    // an answered one (`needs: agent`) is not a warning, it is waiting on us.
    let needs = match &t.needs {
        Some(_) if t.awaiting_agent() => " \u{2713} answered (needs:agent)".to_string(),
        Some(who) => format!(" \u{26a0} needs:{who}"),
        None => String::new(),
    };
    format!(
        "  [{}] {}  p{} {:8} {}{}{}{}",
        t.status.glyph(),
        t.id,
        t.priority,
        t.kind,
        t.title,
        labels,
        deps,
        needs
    )
}

/// `  id  pN type     title` (no status glyph) — matches Python cmd_next;
/// marks an answered yak (`needs: agent`) that is ready but awaits pickup.
fn fmt_plain_row(t: &Task) -> String {
    let answered = if t.awaiting_agent() {
        " \u{2713} answered, awaiting pickup"
    } else {
        ""
    };
    format!(
        "  {}  p{} {:8} {}{answered}",
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
        for st in [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead] {
            std::fs::create_dir_all(root.join(st.dir())).unwrap();
        }
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
                t.labels = vec!["cli".into()];
            }
            store::write::save(&root, &t).unwrap();
        }
        let farm = match Farm::open(&parent) {
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
        for (id, dir) in [
            ("yak-0001", "hairy"),
            ("yak-0002", "shaving"),
            ("yak-0003", "shorn"),
            ("yak-0004", "dead"),
        ] {
            let (code, out, _) = path_cmd(&farm, &[id], FilterFlags::default(), false);
            let want = root.join(dir).join(format!("{id}.md"));
            assert_eq!(code, 0);
            assert_eq!(out, vec![want.display().to_string()]);
            assert!(want.is_file());
        }
    }

    #[test]
    fn path_follows_a_transition() {
        let (root, farm) = path_farm("move");
        let before = farm.path_of("yak-0001").unwrap();
        std::fs::rename(&before, root.join("shorn/yak-0001.md")).unwrap();
        assert_eq!(
            farm.path_of("yak-0001").unwrap(),
            root.join("shorn/yak-0001.md")
        );
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
                root.join("shorn/yak-0003.md").display().to_string(),
                root.join("hairy/yak-0001.md").display().to_string(),
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
            vec![root.join("hairy/yak-0001.md").display().to_string()]
        );
        assert!(err.contains("no such task: yak-nope"), "{err}");
    }

    #[test]
    fn path_filter_form_selects_like_list() {
        let (root, farm) = path_farm("filter");
        let by = |f: FilterFlags, all| path_cmd(&farm, &[], f, all);
        let label = FilterFlags {
            label: vec!["cli".into()],
            ..Default::default()
        };
        let (code, out, _) = by(label, false);
        assert_eq!(code, 0);
        assert_eq!(
            out,
            vec![root.join("shorn/yak-0003.md").display().to_string()]
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
            vec![root.join("dead/yak-0004.md").display().to_string()]
        );
    }

    #[test]
    fn path_needs_ids_xor_filter() {
        let (_root, farm) = path_farm("xor");
        let (code, out, err) = path_cmd(&farm, &[], FilterFlags::default(), false);
        assert_eq!((code, out.len()), (1, 0));
        assert!(err.contains("ids or filter flags"), "{err}");
        let label = FilterFlags {
            label: vec!["cli".into()],
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
            labels: vec![],
            depends_on: vec![],
            source: None,
            needs: None,
            verify: None,
            extra: vec![],
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
