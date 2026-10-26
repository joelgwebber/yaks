//! The core operations facade. `Farm` owns every yaks operation as a typed,
//! print-free method so the CLI, the TUI, and (later) a long-lived process
//! serving editor/IDE plugins all sit thinly on top of the same logic. Each
//! method performs a WHOLE operation (validation + fs mutation) and returns a
//! typed result; nothing here touches argv or stdout.
//!
//! Kept free of `clap` and rendering so it can extract to a `yaks-core` lib
//! crate unchanged.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::Utc;

use crate::filter::{self, FilterSpec};
use crate::model::{NEEDS_AGENT, Status, Task, is_canonical_label, normalize_labels};
use crate::refs;
use crate::rollup;
use crate::store;

pub use crate::store::{DepOutcome, MoveOutcome, Reparent};

const NON_DEAD: [Status; 3] = [Status::Hairy, Status::Shaving, Status::Shorn];
const EVERY: [Status; 4] = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];

/// One attachment file of a yak: see [`list_attachments`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundAttachment {
    /// The attachment's file name (`<herd>/yaks/<id>/<name>`).
    pub name: String,
    /// Absolute path of the file.
    pub path: PathBuf,
}

/// Where yak `id`'s attachment `name` lives in the farm at `root`, when it
/// is a file there. The one resolution rule behind the TUI's open and `yaks
/// attachment`. A name that could leave the yak's directory is refused.
pub fn locate_attachment(root: &Path, id: &str, name: &str) -> Option<FoundAttachment> {
    if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
        return None;
    }
    let path = store::artifact_dir(root, id).join(name);
    path.is_file().then(|| FoundAttachment {
        name: name.to_string(),
        path,
    })
}

/// Every attachment of yak `id` in the farm at `root`, sorted by name.
pub fn list_attachments(root: &Path, id: &str) -> Vec<FoundAttachment> {
    let Ok(rd) = std::fs::read_dir(store::artifact_dir(root, id)) else {
        return Vec::new();
    };
    let mut out: Vec<FoundAttachment> = rd
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            (!name.starts_with('.') && e.path().is_file()).then(|| FoundAttachment {
                name,
                path: e.path(),
            })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// The yaks in `tasks` carrying a `needs` value, whatever their status (a set
/// value is never invisible), narrowed by `spec`'s other filters.
fn needs_rows<'a>(tasks: &'a [Task], spec: &FilterSpec) -> Vec<&'a Task> {
    let mut spec = spec.clone();
    spec.statuses = EVERY.to_vec();
    filter::apply(tasks, &spec, true)
        .into_iter()
        .filter(|t| t.needs.is_some())
        .collect()
}

/// What `ask` / `answer` did to a yak's `needs`: its status (so callers can
/// warn about asking on finished work) and the value before and after.
pub struct NeedsChange {
    pub status: Status,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Result of `pickup`.
pub enum Pickup {
    NotFound,
    /// The yak was not `needs: agent`; carries what it was (`None` = no block).
    NotAwaitingAgent(Option<String>),
    Picked,
}

/// Result of `request`.
pub enum Request {
    NotFound,
    /// The yak is awaiting a human (carries its `needs` value): answer it
    /// instead; nothing was written.
    AwaitingHuman(String),
    /// Recorded: the yak's status and its `needs` before (`None`, or `agent`
    /// for a second request) and after (always `agent`).
    Requested(NeedsChange),
}

/// Why opening a farm failed.
pub enum OpenError {
    NoFarm(String),
    SchemaTooNew {
        found: u32,
        supported: u32,
    },
    /// The farm is on the old layout (schema 3); the message says which farm
    /// and the one remedy.
    OldLayout(String),
    /// The farm is flat but on schema 4: `yaks migrate` first (message says
    /// which farm and how). Only `yaks migrate` opens such a farm.
    NeedsMigration(String),
}

/// A handle to one `.yaks/` farm. Cheap to construct; holds no cache yet
/// (leaves room for a future stat-validated index without changing callers).
pub struct Farm {
    root: PathBuf,
    /// This repo's default herd (id prefix), when discovery followed a `.yaks`
    /// pointer file carrying `prefix:`. Used by `create` below an explicit
    /// `--prefix` and above the farm's config default.
    pointer_prefix: Option<String>,
}

/// Fields for a new task (defaults resolved from config inside `create`).
#[derive(Default)]
pub struct NewTask {
    pub title: String,
    /// Which herd (id prefix) the new yak joins. `None` uses the repo's pointer
    /// herd, else the farm's configured default herd; with neither, `create`
    /// errors rather than guess. One farm can hold several herds.
    pub prefix: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<u8>,
    pub parent: Option<String>,
    pub labels: Vec<String>,
    pub depends_on: Vec<String>,
    pub source: Option<String>,
    pub description: Option<String>,
    pub verify: Option<String>,
}

/// A set of edits to apply to a task in one operation.
#[derive(Default, Clone)]
pub struct TaskEdit {
    pub title: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<u8>,
    pub description: Option<String>,
    /// The body `description` was edited against (optimistic concurrency,
    /// yaks-9f5c). `None` overwrites the body outright (the CLI's explicit
    /// `--description`). `Some(base)`: if the file still holds `base` the
    /// description is written; if it holds `base` plus entries appended since
    /// (a note, a move) they are kept after the new description; anything else
    /// writes nothing and returns [`UpdateOutcome::Conflict`].
    pub base: Option<String>,
    pub add_labels: Vec<String>,
    pub remove_labels: Vec<String>,
    /// Set the external `source:` URL; an empty string clears it.
    pub source: Option<String>,
    /// A rerunnable verification command to set on the yak (see `Task::verify`).
    pub verify: Option<String>,
    pub note: Option<String>,
    /// Actor to attribute an appended note to (stamped as `[actor]`). Only
    /// meaningful alongside `note`; ownership is never implied.
    pub actor: Option<String>,
}

pub enum CreateOutcome {
    Created(Box<Task>),
    ParentNotFound(String),
    /// An explicit `--prefix` that would mint an id outside the reference
    /// grammar (prefix must be `[a-z0-9]+`).
    InvalidPrefix(String),
}

/// Outcome of merging another farm into this one ([`Farm::merge`]).
pub enum MergeOutcome {
    /// Merge is valid, and (unless `dry_run`) was applied.
    Done(MergePlan),
    /// No `.yaks/` farm was found at the given path (or it resolves to this
    /// same farm).
    NoSource(String),
    /// Ids present in BOTH farms; nothing was written. Reconcile the source's
    /// prefix with `rename-prefix`, then retry.
    Collision(Vec<String>),
}

/// The per-yak plan produced by [`Farm::merge`].
pub struct MergePlan {
    /// Whether the plan was applied (`false` for a dry run).
    pub applied: bool,
    /// The resolved source farm root the yaks came from.
    pub source: PathBuf,
    /// Each merged yak as `(id, status)`, sorted by id.
    pub yaks: Vec<(String, Status)>,
    /// Ids whose attachment directory was also copied.
    pub artifacts: Vec<String>,
    /// Incoming herds (id prefixes) newly declared in this farm's `herds:`
    /// config, sorted — so merged yaks' herds join the known-herd set the UI
    /// pickers offer (yaks-095a). For a dry run, the herds that *would* be
    /// declared.
    pub herds: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateOutcome {
    Updated,
    NoChanges,
    NotFound,
    /// A description edit with a `base` the file no longer matches, other than
    /// by appended entries: nothing was written (see [`TaskEdit::base`]).
    Conflict,
}

/// The body to write for a description `new` edited against `base`, given the
/// file's `current` body: `new` when nothing moved, `new` plus the entries
/// appended since `base` was read, or `None` when the body changed some other
/// way (yaks-9f5c). An entry is what [`store::append_note`] adds: a
/// `---` / `▸ ` header after the trimmed body.
fn rebase_description(base: &str, current: &str, new: String) -> Option<String> {
    let base = base.trim_end();
    if current.trim_end() == base {
        return Some(new);
    }
    let rest = current.strip_prefix(base)?;
    let entry = if base.is_empty() {
        rest.trim_start_matches('\n')
    } else {
        rest.strip_prefix("\n\n")?
    };
    if !entry.starts_with("---\n\u{25b8} ") {
        return None;
    }
    let new = new.trim_end();
    Some(if new.is_empty() {
        entry.to_string()
    } else {
        format!("{new}\n\n{entry}")
    })
}

/// Result of attaching an artifact to a task.
pub enum AttachOutcome {
    Attached(String),
    NotFound,
}

/// Result of [`Farm::slaughter`] (yaks-05da).
#[derive(Debug, PartialEq, Eq)]
pub enum SlaughterOutcome {
    NotFound,
    /// Already dead, and no live descendants left to take with it.
    AlreadyDead,
    /// Refused: slaughtering would orphan these live (non-dead) descendants.
    /// Pass `family = true` to slaughter them along with the yak.
    HasLiveDescendants(Vec<String>),
    /// Moved to dead: the live descendants (deepest first), then the yak itself
    /// (omitted if it was already dead).
    Slaughtered(Vec<String>),
}

/// Every live (non-dead) descendant of `id` in `tasks`, found transitively via
/// the `parent:` field and ordered deepest-first (children before their
/// parents), so slaughtering in order never leaves a live yak under a dead one
/// mid-sweep. Cycle-safe. Shared by the CLI and the TUI (yaks-05da).
pub fn live_descendants(tasks: &[Task], id: &str) -> Vec<String> {
    let mut seen: Vec<String> = vec![id.to_string()];
    let mut out: Vec<String> = Vec::new();
    let mut frontier: Vec<String> = vec![id.to_string()];
    while !frontier.is_empty() {
        let mut next = Vec::new();
        for p in &frontier {
            let mut kids: Vec<&Task> = tasks
                .iter()
                .filter(|t| t.parent.as_deref() == Some(p.as_str()))
                .filter(|t| !seen.contains(&t.id))
                .collect();
            kids.sort_by(|a, b| a.id.cmp(&b.id));
            for k in kids {
                seen.push(k.id.clone());
                next.push(k.id.clone());
                if k.status != Status::Dead {
                    out.push(k.id.clone());
                }
            }
        }
        frontier = next;
    }
    out.reverse();
    out
}

/// Result of renaming an attachment in `.yaks/<id>/`.
#[derive(Debug, PartialEq)]
pub enum RenameAttachmentOutcome {
    /// Renamed to `name`; `rewritten` yak files had links to it updated.
    Renamed { name: String, rewritten: usize },
    /// No yak with that id.
    TaskNotFound,
    /// No such file under the yak's attachment directory.
    NoSuchAttachment(String),
    /// The requested name sanitizes to nothing usable.
    Invalid(String),
    /// A different attachment already has the target name.
    Collision(String),
    /// The sanitized name equals the current one.
    Unchanged,
}

/// Sanitize a user-supplied attachment filename: keep only the final path
/// component, turn anything but letters, digits, `.`, `_`, `-`, `+` into `-`
/// (so the name is safe on disk and inside a markdown `![](...)` link),
/// collapse `-` runs, and strip leading/trailing `.`/`-`. When the result has
/// no extension, `keep_ext_of`'s extension (if any) is appended, so renaming
/// `paste-….png` to `login` yields `login.png`. `None` when nothing usable is
/// left.
pub fn attachment_name(raw: &str, keep_ext_of: &str) -> Option<String> {
    let base = raw.trim().rsplit(['/', '\\']).next().unwrap_or("");
    let mut out = String::new();
    for c in base.chars() {
        let c = if c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '+') {
            c
        } else {
            '-'
        };
        if c == '-' && out.ends_with('-') {
            continue;
        }
        out.push(c);
    }
    let out = out.trim_matches(|c| c == '.' || c == '-').to_string();
    if out.is_empty() {
        return None;
    }
    if Path::new(&out).extension().is_none() {
        if let Some(ext) = Path::new(keep_ext_of).extension().and_then(|e| e.to_str()) {
            return Some(format!("{out}.{ext}"));
        }
    }
    Some(out)
}

/// Replace every `old` (`[stem](path)`), with or without a leading `!`, in
/// `text` by `new` (a whole link, `!` included when it is an image).
fn replace_link(text: &str, old: &str, new: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(old) {
        let head = &rest[..i];
        out.push_str(head.strip_suffix('!').unwrap_or(head));
        out.push_str(new);
        rest = &rest[i + old.len()..];
    }
    out.push_str(rest);
    out
}

/// Replace every whole-path occurrence of `old` in `text` with `new`. A match
/// must not be glued to a longer path/name on either side (so renaming
/// `a.png` never touches `a.png.bak` or `xartifacts/...`).
fn replace_path_token(text: &str, old: &str, new: &str) -> String {
    let name_char = |c: char| c.is_alphanumeric() || matches!(c, '.' | '_' | '-' | '+');
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find(old) {
        let before = rest[..i].chars().last().or_else(|| out.chars().last());
        let after = rest[i + old.len()..].chars().next();
        out.push_str(&rest[..i]);
        let glued = before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-')
            || after.is_some_and(|c| name_char(c) && c != '.')
            || (after == Some('.')
                && rest[i + old.len() + 1..]
                    .chars()
                    .next()
                    .is_some_and(name_char));
        out.push_str(if glued { old } else { new });
        rest = &rest[i + old.len()..];
    }
    out.push_str(rest);
    out
}

pub struct Stats {
    pub total: usize,
    pub hairy: usize,
    pub shaving: usize,
    pub shorn: usize,
    pub by_type: Vec<(String, usize)>,
    pub by_priority: Vec<(u8, usize)>,
}

/// A task plus its immediate children, for `show`.
pub struct Show {
    pub task: Task,
    pub children: Vec<Task>,
}

/// What kind of pointer an outgoing reference is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefKind {
    Parent,
    Depends,
    Mention,
}

/// One outgoing reference from a task to another yak.
pub struct RefEntry {
    pub kind: RefKind,
    pub id: String,
    /// False only for a formal parent/dependency pointing at an id that no
    /// longer exists (a dangling reference). Informal mentions are
    /// validation-gated by the resolver, so they are always resolved.
    pub resolved: bool,
    /// 1-based body line for a mention; `None` for formal refs and title mentions.
    pub line: Option<usize>,
}

/// Every outgoing reference a task carries, formal and informal — the
/// integrity-inspection view over the shared resolver.
pub struct TaskRefs {
    pub id: String,
    pub title: String,
    pub entries: Vec<RefEntry>,
}

/// How another yak refers to the subject of [`Farm::backrefs`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackRefVia {
    /// Its `parent:` is the subject (it is a child).
    Child,
    /// Its `depends_on:` lists the subject (the subject blocks it).
    Depends,
    /// Its title mentions the subject's id.
    Title,
    /// Its description mentions the id on this 1-based body line.
    Body { line: usize },
    /// One of its notes mentions the id.
    Note { ts: String, actor: Option<String> },
}

impl BackRefVia {
    /// The formal/informal kind, the same words `yaks refs` uses outgoing:
    /// `child`, `depends`, or `mention`.
    pub fn kind(&self) -> &'static str {
        match self {
            BackRefVia::Child => "child",
            BackRefVia::Depends => "depends",
            _ => "mention",
        }
    }

    /// Where a mention sits (`title`, `body:L3`, `note:<ts> [actor]`); empty
    /// for the formal kinds.
    pub fn location(&self) -> String {
        match self {
            BackRefVia::Child | BackRefVia::Depends => String::new(),
            BackRefVia::Title => "title".into(),
            BackRefVia::Body { line } => format!("body:L{line}"),
            BackRefVia::Note { ts, actor } => match actor {
                Some(a) => format!("note:{ts} [{a}]"),
                None => format!("note:{ts}"),
            },
        }
    }

    /// One short human label: `child`, `depends`, `title`, `body:L3`,
    /// `note:<ts> [actor]` (the TUI picker's row tag).
    pub fn label(&self) -> String {
        match self {
            BackRefVia::Child | BackRefVia::Depends => self.kind().into(),
            _ => self.location(),
        }
    }
}

/// One incoming reference: yak `id` refers to the subject `via` something.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackRef {
    pub id: String,
    pub title: String,
    pub status: Status,
    pub via: BackRefVia,
}

/// Every yak that points at a task — the inverse of [`TaskRefs`], computed on
/// demand by scanning the farm (there is no stored back-reference index).
pub struct TaskBackRefs {
    pub id: String,
    pub title: String,
    pub entries: Vec<BackRef>,
}

/// The incoming references to `id` among `all`: children, dependents, and id
/// mentions in another yak's title, description (one entry per line) or notes
/// (one entry per note), matched by the same whole-real-id resolver as
/// [`Farm::refs`] (`refs::scan`). A yak mentioning itself is not a back-ref.
/// Ordered by referring yak (in `all`'s order), then child, depends, title,
/// body, notes. Pure over a task slice so the TUI can run it on what it
/// already holds.
pub fn backrefs_in(all: &[Task], id: &str) -> Vec<BackRef> {
    let only = |t: &str| t == id;
    let mentions = |text: &str| !refs::scan(&refs::strip_wikilinks(text), only).is_empty();
    let mut out = Vec::new();
    for t in all.iter().filter(|t| t.id != id) {
        let mut push = |via| {
            out.push(BackRef {
                id: t.id.clone(),
                title: t.title.clone(),
                status: t.status,
                via,
            })
        };
        if t.parent.as_deref() == Some(id) {
            push(BackRefVia::Child);
        }
        if t.depends_on.iter().any(|d| d == id) {
            push(BackRefVia::Depends);
        }
        if mentions(&t.title) {
            push(BackRefVia::Title);
        }
        let lines: Vec<&str> = t.body.lines().collect();
        for (n, raw) in lines.iter().enumerate() {
            if store::is_note_delim(&lines, n) {
                break; // the description ends at the first note
            }
            if mentions(raw) {
                push(BackRefVia::Body { line: n + 1 });
            }
        }
        for note in store::parse_notes(&t.body) {
            if mentions(&note.text) {
                push(BackRefVia::Note {
                    ts: note.ts,
                    actor: note.actor,
                });
            }
        }
    }
    out
}

/// The git commits linked to a yak, recovered from history rather than stored:
/// those that name the id in a commit message, and those that touched the yak's
/// own file as it moved across statuses.
pub struct Commits {
    pub id: String,
    pub path: PathBuf,
    pub by_message: Vec<String>,
    pub by_file: Vec<String>,
}

/// Outcome of a rename operation ([`Farm::rename_many`]).
pub enum RenameOutcome {
    /// The rename was applied, or (when `plan.applied` is false) planned.
    Done(RenamePlan),
    /// No requested old id matched anything (e.g. a prefix with no yaks).
    NothingToRename,
    /// A requested old id does not exist.
    NotFound(String),
    /// A target id already exists (and is not being vacated) or is requested twice.
    Collision(String),
    /// A target id is malformed or identical to its source.
    Invalid(String),
}

/// What a rename changed, or would change when `applied` is false.
pub struct RenamePlan {
    pub applied: bool,
    /// The subject renames, `(old, new)`, sorted.
    pub renames: Vec<(String, String)>,
    /// Every task file touched, sorted by current id.
    pub edits: Vec<RenameEdit>,
}

/// One task file touched by a rename.
pub struct RenameEdit {
    /// The file's current on-disk id.
    pub id: String,
    /// Set when this file is itself a rename subject (its id becomes this).
    pub new_id: Option<String>,
    /// Which reference surfaces changed: any of `parent`, `depends_on`, `title`, `body`.
    pub fields: Vec<&'static str>,
    /// 1-based body line numbers whose mentions were rewritten.
    pub body_lines: Vec<usize>,
}

/// One timestamped note tagged with the yak it belongs to (the `log` view).
pub struct LogEntry {
    pub id: String,
    pub title: String,
    pub ts: String,
    pub actor: Option<String>,
    pub note: String,
}

/// One integrity problem found by [`Farm::doctor`]. Read-only: doctor never
/// mutates the farm, it only reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub kind: IssueKind,
    pub message: String,
    /// The yak id(s) the issue concerns: the subject first, then any id it
    /// points at (e.g. the missing parent for a dangling reference).
    pub ids: Vec<String>,
}

/// The classes of problem `doctor` looks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueKind {
    /// A yak file whose `status:` line is not one of the four statuses (read
    /// as hairy meanwhile).
    InvalidStatus,
    /// The same `id:` in the frontmatter of two yak files (say a hand copy, or
    /// a merge that kept both).
    DuplicateId,
    /// A task whose `parent` names an id no task has.
    DanglingParent,
    /// A task whose `depends_on` names an id no task has.
    DanglingDependsOn,
    /// A shorn yak with no recorded note — a completion without evidence. Dead
    /// (abandoned) yaks are deliberately exempt: they need no completion
    /// evidence. Only reported in strict mode (the yaks-working
    /// evidence-before-shear rule).
    MissingEvidence,
    /// A shorn yak that carries a `verify:` command whose most recent recorded
    /// run was not a PASS (or was never run). Setting `verify:` is a commitment
    /// that strict mode then enforces at shear. Only reported in strict mode.
    UnverifiedShear,
    /// A label containing a comma or whitespace (e.g. a legacy `ui,docs` from
    /// before labels were normalized; yaks-7cb3). Any label edit on the yak
    /// re-splits it.
    MalformedLabel,
    /// A yak file that is not at its one path, `<herd>/yaks/<id>.md` (in
    /// another herd's directory, under the wrong name, or loose at the farm
    /// root). Not loaded from there; move it to where `yaks path <id>` says.
    MisplacedFile,
}

impl IssueKind {
    /// A short, stable machine label (used for `--json` and grouping).
    pub fn code(self) -> &'static str {
        match self {
            IssueKind::InvalidStatus => "invalid-status",
            IssueKind::DuplicateId => "duplicate-id",
            IssueKind::DanglingParent => "dangling-parent",
            IssueKind::DanglingDependsOn => "dangling-depends-on",
            IssueKind::MissingEvidence => "missing-evidence",
            IssueKind::UnverifiedShear => "unverified-shear",
            IssueKind::MalformedLabel => "malformed-label",
            IssueKind::MisplacedFile => "misplaced-file",
        }
    }

    /// A human heading for grouped CLI output.
    pub fn heading(self) -> &'static str {
        match self {
            IssueKind::InvalidStatus => {
                "Invalid status (status: is not hairy, shaving, shorn or dead)"
            }
            IssueKind::DuplicateId => "Duplicate id (same id: in two yak files)",
            IssueKind::DanglingParent => "Dangling parent",
            IssueKind::DanglingDependsOn => "Dangling depends_on",
            IssueKind::MissingEvidence => "Missing evidence (shorn yak with no recorded note)",
            IssueKind::UnverifiedShear => {
                "Unverified shear (shorn yak whose verify: command did not last PASS)"
            }
            IssueKind::MalformedLabel => "Malformed label (contains a comma or whitespace)",
            IssueKind::MisplacedFile => "Misplaced yak file (not at <herd>/yaks/<id>.md)",
        }
    }
}

impl Farm {
    /// Discover the nearest `.yaks/` above `cwd` and apply the schema gate.
    pub fn open(cwd: &Path) -> std::result::Result<Farm, OpenError> {
        Self::open_inner(cwd, false)
    }

    /// Like [`Farm::open`], but also opens a farm that is behind the current
    /// schema (schema 4), which is what `yaks migrate` needs. Still refuses a
    /// newer farm and the old layout.
    pub fn open_for_migration(cwd: &Path) -> std::result::Result<Farm, OpenError> {
        Self::open_inner(cwd, true)
    }

    fn open_inner(cwd: &Path, allow_unmigrated: bool) -> std::result::Result<Farm, OpenError> {
        let found = store::discover(cwd).map_err(|e| OpenError::NoFarm(e.to_string()))?;
        let root = found.root;
        if let store::SchemaStatus::Newer(found) = store::schema_status(&root) {
            return Err(OpenError::SchemaTooNew {
                found,
                supported: store::SCHEMA,
            });
        }
        if let Some(why) = store::old_layout_refusal(&root) {
            return Err(OpenError::OldLayout(why));
        }
        if !allow_unmigrated {
            if let Some(why) = store::migration_refusal(&root) {
                return Err(OpenError::NeedsMigration(why));
            }
        }
        Ok(Farm {
            root,
            pointer_prefix: found.prefix,
        })
    }

    /// The effective per-farm config (prefix, defaults, editor mode).
    pub fn config(&self) -> store::Config {
        store::read_config(&self.root)
    }

    /// The farm's `.yaks/` root, e.g. for locating the per-user UI-state cache.
    pub fn root(&self) -> &Path {
        &self.root
    }

    // -- queries ----------------------------------------------------------

    /// Tasks matching `spec`, grouped by status then id (the `list`/`search` view).
    pub fn list(&self, spec: FilterSpec, include_dead: bool) -> Result<Vec<Task>> {
        let tasks = store::load(&self.root, &EVERY)?;
        let mut rows: Vec<Task> = filter::apply(&tasks, &spec, include_dead)
            .into_iter()
            .cloned()
            .collect();
        rows.sort_by(|a, b| {
            rank(a.status)
                .cmp(&rank(b.status))
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(rows)
    }

    /// Hairy tasks with all dependencies resolved.
    pub fn next(&self, mut spec: FilterSpec) -> Result<Vec<Task>> {
        spec.statuses = vec![Status::Hairy];
        spec.ready_only = true;
        let tasks = store::load(&self.root, &EVERY)?;
        Ok(filter::apply(&tasks, &spec, false)
            .into_iter()
            .cloned()
            .collect())
    }

    /// Raise a question: set `needs` (default `human`) and record the question
    /// as an `asked: needs <who>` note in the same write. Allowed on a yak that
    /// is already `needs: agent` (a follow-up question flips it back; `before`
    /// tells the caller). `None` if the id is unknown.
    pub fn ask(
        &self,
        id: &str,
        needs: &str,
        actor: Option<&str>,
        question: Option<&str>,
    ) -> Result<Option<NeedsChange>> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(None);
        };
        let before = task.needs.replace(needs.to_string());
        let ts = store::now_iso();
        task.body = store::append_note(&task.body, &ts, actor, &store::ask_text(needs, question));
        task.updated = Some(ts);
        store::write::save(&self.root, &task)?;
        Ok(Some(NeedsChange {
            status: task.status,
            before,
            after: task.needs,
        }))
    }

    /// Answer a question: record the reply, then hand the yak to an agent
    /// (`needs: agent`) so the answer cannot vanish; `done` clears `needs`
    /// instead (an answer that needs no follow-up). On a yak already `agent` the
    /// state is kept and the note appended (a second answer); on one with no
    /// `needs` the note is recorded and nothing else changes. `None` if the id
    /// is unknown.
    pub fn answer(
        &self,
        id: &str,
        actor: Option<&str>,
        note: Option<&str>,
        done: bool,
    ) -> Result<Option<NeedsChange>> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(None);
        };
        let before = task.needs.clone();
        if before.is_some() {
            task.needs = (!done).then(|| NEEDS_AGENT.to_string());
        }
        let ts = store::now_iso();
        if let Some(n) = note {
            task.body = store::append_note(&task.body, &ts, actor, n);
        }
        task.updated = Some(ts);
        store::write::save(&self.root, &task)?;
        Ok(Some(NeedsChange {
            status: task.status,
            before,
            after: task.needs,
        }))
    }

    /// Hand a yak to an agent with no prior question: set `needs: agent` and
    /// record the request as a `requested: needs agent` note in the same write.
    /// On a yak already `needs: agent` the state is kept and the request
    /// appended. Refuses a yak awaiting a human (that needs an `answer`), and
    /// writes nothing then.
    pub fn request(&self, id: &str, actor: Option<&str>, note: Option<&str>) -> Result<Request> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(Request::NotFound);
        };
        if task.awaiting_human() {
            return Ok(Request::AwaitingHuman(task.needs.unwrap_or_default()));
        }
        let before = task.needs.replace(NEEDS_AGENT.to_string());
        let ts = store::now_iso();
        task.body = store::append_note(&task.body, &ts, actor, &store::request_text(note));
        task.updated = Some(ts);
        store::write::save(&self.root, &task)?;
        Ok(Request::Requested(NeedsChange {
            status: task.status,
            before,
            after: task.needs,
        }))
    }

    /// Clear `needs: agent` with an attributed `picked up` note. The only thing
    /// that clears it. Refuses any other state, naming it.
    pub fn pickup(&self, id: &str, actor: Option<&str>, note: Option<&str>) -> Result<Pickup> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(Pickup::NotFound);
        };
        if !task.awaiting_agent() {
            return Ok(Pickup::NotAwaitingAgent(task.needs));
        }
        task.needs = None;
        let ts = store::now_iso();
        task.body = store::append_note(&task.body, &ts, actor, &store::pickup_text(note));
        task.updated = Some(ts);
        store::write::save(&self.root, &task)?;
        Ok(Pickup::Picked)
    }

    /// Every yak carrying a `needs` value, regardless of status: open
    /// questions (awaiting a human) and answered or requested ones (awaiting an agent;
    /// see [`Task::awaiting_agent`]). The invariant is that a set value is
    /// never invisible: an `ask` on a shorn/dead yak, or an answer to one, must
    /// still surface here (that silent gap is exactly why this ignores status).
    /// Other filter flags (priority/label/search) still apply.
    pub fn inbox(&self, spec: FilterSpec) -> Result<Vec<Task>> {
        let tasks = store::load(&self.root, &EVERY)?;
        Ok(needs_rows(&tasks, &spec).into_iter().cloned().collect())
    }

    /// Hairy tasks with at least one unresolved dependency, each paired with
    /// the list of ids it is waiting on.
    pub fn tangled(&self, mut spec: FilterSpec) -> Result<Vec<(Task, Vec<String>)>> {
        spec.statuses = vec![Status::Hairy];
        spec.tangled_only = true;
        let tasks = store::load(&self.root, &EVERY)?;
        let resolved = filter::resolved_ids(&tasks);
        Ok(filter::apply(&tasks, &spec, false)
            .into_iter()
            .map(|t| {
                let waiting = filter::unresolved_deps(t, &resolved)
                    .into_iter()
                    .map(str::to_string)
                    .collect();
                (t.clone(), waiting)
            })
            .collect())
    }

    pub fn stats(&self) -> Result<Stats> {
        let tasks = store::load(&self.root, &NON_DEAD)?;
        let count = |s: Status| tasks.iter().filter(|t| t.status == s).count();
        let mut by_type = fold_counts(tasks.iter().map(|t| t.kind.clone()));
        by_type.sort();
        let mut by_priority = fold_counts(tasks.iter().map(|t| t.priority));
        by_priority.sort();
        Ok(Stats {
            total: tasks.len(),
            hairy: count(Status::Hairy),
            shaving: count(Status::Shaving),
            shorn: count(Status::Shorn),
            by_type,
            by_priority,
        })
    }

    pub fn show(&self, id: &str) -> Result<Option<Show>> {
        let all = store::load(&self.root, &EVERY)?;
        let Some(task) = all.iter().find(|t| t.id == id).cloned() else {
            return Ok(None);
        };
        let mut children: Vec<Task> = all
            .iter()
            .filter(|c| c.parent.as_deref() == Some(id))
            .cloned()
            .collect();
        children.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(Some(Show { task, children }))
    }

    /// The current on-disk file of the yak `id`, wherever its status put it, or
    /// `None` if there is no such yak. Backs `yaks path`: a state transition
    /// moves the file between status directories, so callers (e.g. `git add`)
    /// should ask rather than hand-build `.yaks/<status>/<id>.md`.
    pub fn path_of(&self, id: &str) -> Option<PathBuf> {
        store::find_task_file(&self.root, id).map(|(_, p)| p)
    }

    /// The on-disk files of every yak `spec` selects (the `list` view: dead yaks
    /// only with `include_dead` or an explicit `--status dead`), in `list` order.
    pub fn paths(&self, spec: FilterSpec, include_dead: bool) -> Result<Vec<PathBuf>> {
        Ok(self
            .list(spec, include_dead)?
            .iter()
            .map(|t| store::task_path(&self.root, &t.id))
            .collect())
    }

    pub fn rollup(&self, spec: &FilterSpec) -> Result<(Vec<rollup::Group>, usize)> {
        let tasks = store::load(&self.root, &NON_DEAD)?;
        Ok(rollup::build(&tasks, spec))
    }

    /// Timestamped notes across the filtered set, oldest first. `since`, when
    /// set, keeps only notes at or after that instant (see `store::parse_since`).
    /// Notes live in each task body, and so do state transitions (a note whose
    /// text is `moved: <from> -> <to>`, written by `store::move_task`), so this
    /// is the activity log. Yaks moved by a binary that predates transition
    /// entries have no such entry.
    pub fn log(
        &self,
        spec: FilterSpec,
        since: Option<&str>,
        by: Option<&str>,
    ) -> Result<Vec<LogEntry>> {
        let cutoff = match since {
            Some(s) => Some(store::parse_since(s, Utc::now())?),
            None => None,
        };
        let include_dead = spec.statuses.contains(&Status::Dead);
        let tasks = store::load(&self.root, &EVERY)?;
        let mut out = Vec::new();
        for t in filter::apply(&tasks, &spec, include_dead) {
            for note in store::parse_notes(&t.body) {
                if let Some(cut) = cutoff {
                    if let Some(ts) = store::parse_ts(&note.ts) {
                        if ts < cut {
                            continue;
                        }
                    }
                }
                if let Some(who) = by {
                    if note.actor.as_deref() != Some(who) {
                        continue;
                    }
                }
                out.push(LogEntry {
                    id: t.id.clone(),
                    title: t.title.clone(),
                    ts: note.ts,
                    actor: note.actor,
                    note: note.text,
                });
            }
        }
        out.sort_by(|a, b| a.ts.cmp(&b.ts).then_with(|| a.id.cmp(&b.id)));
        Ok(out)
    }

    /// List every yak this task points at — formal (parent, dependencies) and
    /// informal (id mentions in the title/body) — each flagged resolved or
    /// dangling. Both the TUI and this share the same resolver in `refs`, so
    /// what links here is exactly what lights up in the detail pane.
    pub fn refs(&self, id: &str) -> Result<Option<TaskRefs>> {
        let all = store::load(&self.root, &EVERY)?;
        let Some(task) = all.iter().find(|t| t.id == id).cloned() else {
            return Ok(None);
        };
        let ids: std::collections::HashSet<&str> = all.iter().map(|t| t.id.as_str()).collect();
        let known = refs::known_from(&ids);
        let mut entries = Vec::new();
        if let Some(p) = &task.parent {
            entries.push(RefEntry {
                kind: RefKind::Parent,
                id: p.clone(),
                resolved: known(p),
                line: None,
            });
        }
        for d in &task.depends_on {
            entries.push(RefEntry {
                kind: RefKind::Depends,
                id: d.clone(),
                resolved: known(d),
                line: None,
            });
        }
        for m in refs::scan(&refs::strip_wikilinks(&task.title), &known) {
            entries.push(RefEntry {
                kind: RefKind::Mention,
                id: m.id,
                resolved: true,
                line: None,
            });
        }
        for (n, raw) in task.body.lines().enumerate() {
            let line = refs::strip_wikilinks(raw);
            for m in refs::scan(&line, &known) {
                entries.push(RefEntry {
                    kind: RefKind::Mention,
                    id: m.id,
                    resolved: true,
                    line: Some(n + 1),
                });
            }
        }
        Ok(Some(TaskRefs {
            id: task.id,
            title: task.title,
            entries,
        }))
    }

    /// Every yak that points at this task (children, dependents, id mentions
    /// in titles, descriptions and notes), each saying how — see
    /// [`backrefs_in`]. Computed on demand by scanning the farm; `None` when
    /// no such task exists.
    pub fn backrefs(&self, id: &str) -> Result<Option<TaskBackRefs>> {
        let all = self.list(FilterSpec::default(), true)?;
        let Some(task) = all.iter().find(|t| t.id == id) else {
            return Ok(None);
        };
        Ok(Some(TaskBackRefs {
            id: task.id.clone(),
            title: task.title.clone(),
            entries: backrefs_in(&all, id),
        }))
    }

    /// A read-only farm-integrity pass: collect every problem worth a human's
    /// attention without touching a single file. Ordering is deterministic
    /// (by kind, then id) so text and `--json` output are stable and
    /// CI-diffable.
    ///
    /// Checks: an invalid `status:` value, the same `id:` in two yak files,
    /// and `parent` / `depends_on` references that point at no known yak.
    ///
    /// `strict` adds a skill-adherence check: any shorn yak lacking a recorded
    /// note is a completion without evidence. Dead (abandoned) yaks are exempt.
    pub fn doctor(&self, strict: bool) -> Result<Vec<Issue>> {
        let tasks = store::load(&self.root, &EVERY)?;
        let known = store::all_ids(&self.root);
        let mut issues = Vec::new();

        // One pass over the yak files themselves (the loader falls back
        // silently, so only each file's text can say): a `status:` line that is
        // not one of the four values, and the same `id:` in two files.
        let mut files_by_id: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for path in store::yak_files(&self.root) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or_default()
                .to_string();
            let rel = path
                .strip_prefix(&self.root)
                .unwrap_or(&path)
                .display()
                .to_string();
            if let Some(task) = store::parse_task(&text, Status::Hairy) {
                // Not at its path, and no file there: misplaced. With a file
                // at its path it is a second copy, reported as a duplicate.
                let want = store::task_path(&self.root, &task.id);
                if want != path && !want.is_file() {
                    issues.push(Issue {
                        kind: IssueKind::MisplacedFile,
                        message: format!(
                            "{rel} holds {id}, whose file belongs at {want} (move it there)",
                            id = task.id,
                            want = store::task_path(Path::new(""), &task.id).display()
                        ),
                        ids: vec![task.id.clone()],
                    });
                }
                files_by_id.entry(task.id).or_default().push(rel.clone());
            }
            if let Some(value) = invalid_status_value(&text) {
                let id = name.trim_end_matches(".md").to_string();
                issues.push(Issue {
                    kind: IssueKind::InvalidStatus,
                    message: format!(
                        "{id} has status: {value:?}, which is not hairy, shaving, shorn or \
                         dead (fix the status: line)"
                    ),
                    ids: vec![id],
                });
            }
        }
        // A yak file left at the farm root (a hand copy, or a pre-schema-6
        // leftover): not loaded from there.
        for path in root_md_files(&self.root) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Some(task) = store::parse_task(&text, Status::Hairy) {
                let name = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();
                if store::task_path(&self.root, &task.id).is_file() {
                    files_by_id
                        .entry(task.id)
                        .or_default()
                        .push(name.to_string());
                    continue;
                }
                issues.push(Issue {
                    kind: IssueKind::MisplacedFile,
                    message: format!(
                        "{name} at the farm root holds {id}, whose file belongs at {want} \
                         (move it there; it is not loaded from the root)",
                        id = task.id,
                        want = store::task_path(Path::new(""), &task.id).display()
                    ),
                    ids: vec![task.id.clone()],
                });
            }
        }
        for (id, names) in files_by_id.into_iter().filter(|(_, n)| n.len() > 1) {
            issues.push(Issue {
                kind: IssueKind::DuplicateId,
                message: format!(
                    "{id} is the id: of {} files: {}",
                    names.len(),
                    names.join(", ")
                ),
                ids: vec![id],
            });
        }

        // Dangling references: a `parent` or `depends_on` naming no known id.
        for t in &tasks {
            if let Some(p) = &t.parent {
                if !known.contains(p) {
                    issues.push(Issue {
                        kind: IssueKind::DanglingParent,
                        message: format!("{} has parent {} but no such yak exists", t.id, p),
                        ids: vec![t.id.clone(), p.clone()],
                    });
                }
            }
            for d in &t.depends_on {
                if !known.contains(d) {
                    issues.push(Issue {
                        kind: IssueKind::DanglingDependsOn,
                        message: format!("{} depends_on {} but no such yak exists", t.id, d),
                        ids: vec![t.id.clone(), d.clone()],
                    });
                }
            }
        }

        // Malformed labels: a comma or whitespace inside one label (legacy data
        // from before normalization). Fix by re-editing the yak's labels.
        for t in &tasks {
            for l in t.labels.iter().filter(|l| !is_canonical_label(l)) {
                let fixed = normalize_labels([l]);
                issues.push(Issue {
                    kind: IssueKind::MalformedLabel,
                    message: format!(
                        "{} has label {:?}; labels may not contain commas or spaces \
                         (would be [{}] — re-edit its labels to fix)",
                        t.id,
                        l,
                        fixed.join(", ")
                    ),
                    ids: vec![t.id.clone()],
                });
            }
        }

        // Strict: a shorn yak with no recorded note is a completion without
        // evidence (the yaks-working evidence-before-shear rule). Dead
        // (abandoned) yaks are exempt — abandonment needs no completion note.
        if strict {
            for t in &tasks {
                // A `moved:` transition entry is not evidence: it is written by the
                // move itself, so only a real note counts.
                if t.status == Status::Shorn
                    && store::parse_notes(&t.body)
                        .iter()
                        .all(|n| store::is_transition_text(&n.text))
                {
                    issues.push(Issue {
                        kind: IssueKind::MissingEvidence,
                        message: format!(
                            "{} is {} but has no recorded note (evidence before shear)",
                            t.id,
                            t.status.as_str()
                        ),
                        ids: vec![t.id.clone()],
                    });
                }
                // A `verify:` command is a commitment: a shorn yak that carries
                // one must show a recorded PASS as its most recent verify run.
                // (Catches shearing over a failing or never-run verify; it does
                // not catch verify-then-edit-then-shear — that needs a re-run,
                // which is deliberately explicit.)
                if t.status == Status::Shorn && t.verify.is_some() && !last_verify_passed(&t.body) {
                    issues.push(Issue {
                        kind: IssueKind::UnverifiedShear,
                        message: format!(
                            "{} is shorn with a verify: command but its last verify run was \
                             not a PASS (run `yaks verify {}`)",
                            t.id, t.id
                        ),
                        ids: vec![t.id.clone()],
                    });
                }
            }
        }

        // Deterministic order: group by kind, then by the ids involved. Dedup
        // drops any identical issue a duplicated-on-disk task could produce.
        issues.sort_by(|a, b| {
            issue_rank(a.kind)
                .cmp(&issue_rank(b.kind))
                .then_with(|| a.ids.first().cmp(&b.ids.first()))
                .then_with(|| a.ids.get(1).cmp(&b.ids.get(1)))
        });
        issues.dedup();
        Ok(issues)
    }

    /// Recover the git commits linked to `id` without any stored hash: commits
    /// whose message names the id, and commits that touched the yak's own file
    /// (followed across its status moves). `None` if the id is not a task.
    pub fn commits(&self, id: &str) -> Result<Option<Commits>> {
        let Some((_, path)) = store::find_task_file(&self.root, id) else {
            return Ok(None);
        };
        let by_message = git_log(&self.root, &["--oneline", &format!("--grep={id}")])?;
        let by_file = git_log(
            &self.root,
            &["--oneline", "--follow", "--", &path.to_string_lossy()],
        )?;
        Ok(Some(Commits {
            id: id.to_string(),
            path,
            by_message,
            by_file,
        }))
    }

    /// Rename one yak, a convenience over [`Farm::rename_many`].
    pub fn rename(&self, old: &str, new: &str, dry_run: bool) -> Result<RenameOutcome> {
        self.rename_many(&[(old.to_string(), new.to_string())], dry_run)
    }

    /// Migrate every yak whose id is `{old}-<tail>` to `{new}-<tail>`, rewriting
    /// all references (via [`Farm::rename_many`]) and, on a real successful run,
    /// flipping the farm's configured `prefix` to `new` so future ids match.
    /// The `{old}-` boundary means `rename_prefix("yaks", ..)` never touches a
    /// `yaksrs-` id.
    pub fn rename_prefix(&self, old: &str, new: &str, dry_run: bool) -> Result<RenameOutcome> {
        let _lock = (!dry_run).then(|| store::lock(&self.root)).transpose()?;
        let all = store::load(&self.root, &EVERY)?;
        let marker = format!("{old}-");
        let mut pairs = Vec::new();
        for t in &all {
            if let Some(tail) = t.id.strip_prefix(&marker) {
                pairs.push((t.id.clone(), format!("{new}-{tail}")));
            }
        }
        let outcome = self.rename_many_locked(&pairs, dry_run)?;
        if !dry_run {
            if let RenameOutcome::Done(_) = &outcome {
                store::set_config_prefix(&self.root, new)?;
            }
        }
        Ok(outcome)
    }

    /// Rename one or more yaks in a single pass, rewriting every reference to
    /// them across the whole farm: the file name + `id` of each subject, and the
    /// `parent`, `depends_on`, title, and body mentions of every task that
    /// points at one. References are matched as whole tokens validated against
    /// real ids (via `refs`), so lookalike prose is never touched. With
    /// `dry_run`, nothing is written and the returned plan describes what would
    /// change. This is the shared engine behind `yaks rename` (one pair) and the
    /// bulk prefix migration (many pairs).
    pub fn rename_many(&self, pairs: &[(String, String)], dry_run: bool) -> Result<RenameOutcome> {
        let _lock = (!dry_run).then(|| store::lock(&self.root)).transpose()?;
        self.rename_many_locked(pairs, dry_run)
    }

    /// [`Farm::rename_many`] for a caller that already holds the farm lock
    /// (the lock is not re-entrant) or is only planning (`dry_run`).
    fn rename_many_locked(
        &self,
        pairs: &[(String, String)],
        dry_run: bool,
    ) -> Result<RenameOutcome> {
        let all = store::load(&self.root, &EVERY)?;
        let existing: std::collections::HashSet<&str> = all.iter().map(|t| t.id.as_str()).collect();

        // Validate the requested pairs and build the old -> new map.
        let mut map: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        let mut targets: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (old, new) in pairs {
            if !existing.contains(old.as_str()) {
                return Ok(RenameOutcome::NotFound(old.clone()));
            }
            if new == old
                || !refs::has_ref_shape(new)
                || store::RESERVED_HERD_NAMES.contains(&store::herd_of(new))
            {
                return Ok(RenameOutcome::Invalid(new.clone()));
            }
            if !targets.insert(new.clone()) || map.insert(old.clone(), new.clone()).is_some() {
                return Ok(RenameOutcome::Collision(new.clone()));
            }
        }
        if map.is_empty() {
            return Ok(RenameOutcome::NothingToRename);
        }
        // A target that already exists and isn't itself being vacated would
        // clobber a live yak.
        for new in &targets {
            if existing.contains(new.as_str()) && !map.contains_key(new) {
                return Ok(RenameOutcome::Collision(new.clone()));
            }
            // A leftover attachment dir at the target would be merged into.
            if store::artifact_dir(&self.root, new).exists() && !map.contains_key(new) {
                return Ok(RenameOutcome::Collision(new.clone()));
            }
        }

        let lookup = |t: &str| map.get(t).cloned();
        let mut edits = Vec::new();
        let mut writes = Vec::new();
        for task in &all {
            let mut edit = RenameEdit {
                id: task.id.clone(),
                new_id: map.get(&task.id).cloned(),
                fields: Vec::new(),
                body_lines: Vec::new(),
            };
            let mut next = task.clone();
            if let Some(nid) = &edit.new_id {
                next.id = nid.clone();
            }
            if let Some(p) = &task.parent {
                if let Some(np) = map.get(p) {
                    next.parent = Some(np.clone());
                    edit.fields.push("parent");
                }
            }
            if task.depends_on.iter().any(|d| map.contains_key(d)) {
                next.depends_on = task
                    .depends_on
                    .iter()
                    .map(|d| map.get(d).cloned().unwrap_or_else(|| d.clone()))
                    .collect();
                edit.fields.push("depends_on");
            }
            let (title, tchanged) = refs::rewrite(&task.title, &lookup);
            if tchanged {
                next.title = title;
                edit.fields.push("title");
            }
            let mut body_changed = false;
            let mut lines: Vec<String> = Vec::new();
            for (n, line) in task.body.lines().enumerate() {
                let (nl, ch) = refs::rewrite(line, &lookup);
                if ch {
                    body_changed = true;
                    edit.body_lines.push(n + 1);
                }
                lines.push(nl);
            }
            if body_changed {
                next.body = lines.join("\n");
                edit.fields.push("body");
            }
            if edit.new_id.is_some() || !edit.fields.is_empty() {
                writes.push((task.status, task.id.clone(), next));
                edits.push(edit);
            }
        }

        if !dry_run {
            for (_st, _old, next) in &writes {
                store::write::save(&self.root, next)?;
            }
            // Remove each vacated subject file, unless its id is reused as a
            // rename target (a swap/chain, which stays put under its new writer).
            for (_st, old, _next) in &writes {
                if map.contains_key(old) && !targets.contains(old) {
                    let _ = std::fs::remove_file(store::task_path(&self.root, old));
                }
            }
            // Each renamed yak's attachment dir follows it (`.yaks/<old>/` ->
            // `.yaks/<new>/`); the links were rewritten with the ids above. Two
            // steps through a temp name, so a swap or chain never collides.
            let mut moving = Vec::new();
            for (old, new) in &map {
                let from = store::artifact_dir(&self.root, old);
                if from.is_dir() {
                    let tmp = self.root.join(format!(".{new}.rename-tmp"));
                    std::fs::rename(&from, &tmp)
                        .with_context(|| format!("moving {}", from.display()))?;
                    moving.push((tmp, store::artifact_dir(&self.root, new)));
                }
            }
            for (tmp, to) in moving {
                std::fs::rename(&tmp, &to)
                    .with_context(|| format!("moving into {}", to.display()))?;
            }
        }

        edits.sort_by(|a, b| a.id.cmp(&b.id));
        let mut renames: Vec<(String, String)> =
            map.iter().map(|(o, n)| (o.clone(), n.clone())).collect();
        renames.sort();
        Ok(RenameOutcome::Done(RenamePlan {
            applied: !dry_run,
            renames,
            edits,
        }))
    }

    // -- mutations (each a whole operation) -------------------------------

    pub fn create(&self, new: NewTask) -> Result<CreateOutcome> {
        let _lock = store::lock(&self.root)?;
        let cfg = store::read_config(&self.root);
        if let Some(p) = &new.parent {
            if !store::all_ids(&self.root).contains(p) {
                return Ok(CreateOutcome::ParentNotFound(p.clone()));
            }
        }
        // Route to a specific herd (id prefix): an explicit `--prefix` wins,
        // else this repo's pointer-file herd, else the farm's config default
        // (erroring if there is none). A chosen prefix is validated via the
        // reference grammar so we never mint an un-referenceable id.
        let prefix = match new
            .prefix
            .as_deref()
            .or(self.pointer_prefix.as_deref())
            .filter(|p| !p.is_empty())
        {
            Some(p)
                if refs::has_ref_shape(&format!("{p}-0000"))
                    && !store::RESERVED_HERD_NAMES.contains(&p) =>
            {
                p.to_string()
            }
            Some(p) => return Ok(CreateOutcome::InvalidPrefix(p.to_string())),
            // No `--herd`, no pointer herd: only a herd the farm's config
            // actually declares is a default. `cfg.prefix` alone isn't enough:
            // it is seeded with the built-in "yak" when config is silent, which
            // would silently mint an unintended `yak-` herd (yaks-b47b).
            None if cfg.prefix_declared => cfg.prefix.clone(),
            None => bail!(
                "no herd to create the yak in: pass `--herd <prefix>` or set a default herd \
                 (`herd: <prefix>` in .yaks/config.yaml, or `herd:` in the `.yaks` pointer file)"
            ),
        };
        let id = store::generate_id(&self.root, &prefix)?;
        let now = store::now_iso();
        let task = Task {
            id,
            title: new.title,
            kind: new.kind.unwrap_or_else(|| cfg.default_type_for(&prefix)),
            priority: new
                .priority
                .unwrap_or_else(|| cfg.default_priority_for(&prefix)),
            status: Status::Hairy,
            created: Some(now.clone()),
            updated: Some(now),
            parent: new.parent,
            labels: normalize_labels(&new.labels),
            depends_on: new.depends_on,
            source: new.source,
            needs: None,
            verify: new.verify,
            extra: Vec::new(),
            body: new.description.unwrap_or_default(),
        };
        store::write::save(&self.root, &task)?;
        Ok(CreateOutcome::Created(Box::new(task)))
    }

    pub fn update(&self, id: &str, edit: TaskEdit) -> Result<UpdateOutcome> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(UpdateOutcome::NotFound);
        };
        let mut changed = false;
        if let Some(t) = edit.title {
            task.title = t;
            changed = true;
        }
        if let Some(k) = edit.kind {
            task.kind = k;
            changed = true;
        }
        if let Some(p) = edit.priority {
            task.priority = p;
            changed = true;
        }
        if let Some(d) = edit.description {
            task.body = match edit.base {
                None => d,
                Some(base) => match rebase_description(&base, &task.body, d) {
                    Some(body) => body,
                    None => return Ok(UpdateOutcome::Conflict),
                },
            };
            changed = true;
        }
        // Labels are normalized on the way in (split on commas/whitespace,
        // deduped; yaks-7cb3). Any label edit also re-splits the yak's existing
        // labels, so a legacy `ui,docs` label is canonicalized on first touch.
        if !edit.add_labels.is_empty() || !edit.remove_labels.is_empty() {
            let remove = normalize_labels(&edit.remove_labels);
            task.labels = normalize_labels(task.labels.iter().chain(&edit.add_labels));
            task.labels.retain(|l| !remove.contains(l));
            changed = true;
        }
        if let Some(s) = edit.source {
            // An empty string clears the source (like `verify`).
            let new = if s.is_empty() { None } else { Some(s) };
            if new != task.source {
                task.source = new;
                changed = true;
            }
        }
        if let Some(v) = edit.verify {
            task.verify = if v.is_empty() { None } else { Some(v) };
            changed = true;
        }
        if let Some(n) = edit.note {
            let ts = store::now_iso();
            task.body = store::append_note(&task.body, &ts, edit.actor.as_deref(), &n);
            changed = true;
        }
        if changed {
            task.updated = Some(store::now_iso());
            store::write::save(&self.root, &task)?;
            Ok(UpdateOutcome::Updated)
        } else {
            Ok(UpdateOutcome::NoChanges)
        }
    }

    /// Bring a schema-4 farm up to the current schema: every fixup in
    /// [`crate::migrate::FIXUPS`] over every yak file, then the `schema`
    /// marker. Under the farm lock (a dry run only reads). Idempotent: on a
    /// current farm it changes nothing. Never touches git.
    pub fn migrate(&self, dry_run: bool) -> Result<crate::migrate::Report> {
        let _lock = (!dry_run).then(|| store::lock(&self.root)).transpose()?;
        crate::migrate::run(&self.root, dry_run)
    }

    /// Write `data` to `.yaks/{id}/{name}` ([`store::artifact_dir`], beside the
    /// yak's file) and append its markdown link to the task body
    /// ([`store::attachment_link`]: `![stem]({id}/{name})` for an image,
    /// `[stem]({id}/{name})` for any other file) (relative to the yak's own file, so it resolves in any
    /// markdown viewer). `name` should be a bare filename. The directory lives
    /// inside the farm and, in team mode, commits alongside `.yaks/`, so an
    /// attachment is shareable evidence, not just local scratch.
    pub fn attach(&self, id: &str, name: &str, data: &[u8]) -> Result<AttachOutcome> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(AttachOutcome::NotFound);
        };
        let dir = store::artifact_dir(&self.root, id);
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join(name), data)?;
        let link = store::attachment_link(id, name);
        let mut body = task.body.trim_end().to_string();
        if !body.is_empty() {
            body.push_str("\n\n");
        }
        body.push_str(&link);
        task.body = body;
        task.updated = Some(store::now_iso());
        store::write::save(&self.root, &task)?;
        Ok(AttachOutcome::Attached(name.to_string()))
    }

    /// Rename `.yaks/{id}/{old}` to a sanitized `new` (see
    /// [`attachment_name`]; the old extension is kept when `new` omits one)
    /// and rewrite every link to it across the farm — `{id}/{old}`
    /// path mentions in any yak's body, plus the `![alt]` / `[alt]` text of a
    /// link whose alt was the old file stem (what `attach` writes). Refuses to clobber an
    /// existing attachment.
    pub fn rename_attachment(
        &self,
        id: &str,
        old: &str,
        new: &str,
    ) -> Result<RenameAttachmentOutcome> {
        use RenameAttachmentOutcome as R;
        let _lock = store::lock(&self.root)?;
        if store::load_task_by_id(&self.root, id)?.is_none() {
            return Ok(R::TaskNotFound);
        }
        let dir = store::artifact_dir(&self.root, id);
        let src = dir.join(old);
        if old.contains(['/', '\\']) || old.is_empty() || !src.is_file() {
            return Ok(R::NoSuchAttachment(old.to_string()));
        }
        let Some(name) = attachment_name(new, old) else {
            return Ok(R::Invalid(new.to_string()));
        };
        if name == old {
            return Ok(R::Unchanged);
        }
        // A case-only rename on a case-insensitive filesystem "exists" already.
        if dir.join(&name).exists() && !name.eq_ignore_ascii_case(old) {
            return Ok(R::Collision(name));
        }
        std::fs::rename(&src, dir.join(&name))?;

        let old_path = format!("{id}/{old}");
        let new_path = format!("{id}/{name}");
        // The link text follows the stem, in either form (a rename can also
        // move the file across the image/non-image line, e.g. `.txt` -> `.png`).
        let stem = |n: &str| {
            Path::new(n)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(n)
                .to_string()
        };
        let old_text = format!("[{}]({new_path})", stem(old));
        let new_link = store::attachment_link(id, &name);
        let mut rewritten = 0;
        for mut t in store::load(&self.root, &EVERY)? {
            if !t.body.contains(&old_path) {
                continue;
            }
            let body = replace_path_token(&t.body, &old_path, &new_path);
            let body = replace_link(&body, &old_text, &new_link);
            if body != t.body {
                t.body = body;
                t.updated = Some(store::now_iso());
                store::write::save(&self.root, &t)?;
                rewritten += 1;
            }
        }
        Ok(R::Renamed { name, rewritten })
    }

    /// Move `id` to `dest`, recording a `moved: <from> -> <to>` entry attributed
    /// to `actor` (see [`store::move_task`]).
    pub fn transition(&self, id: &str, dest: Status, actor: Option<&str>) -> Result<MoveOutcome> {
        let _lock = store::lock(&self.root)?;
        store::move_task(&self.root, id, dest, actor)
    }

    /// Slaughter `id` (move to dead). Refuses with
    /// [`SlaughterOutcome::HasLiveDescendants`] when that would orphan live
    /// descendants, unless `family` is set, in which case the whole family
    /// (every live descendant, deepest first, then `id`) goes (yaks-05da). Each
    /// yak moved gets a transition entry attributed to `actor`.
    pub fn slaughter(
        &self,
        id: &str,
        family: bool,
        actor: Option<&str>,
    ) -> Result<SlaughterOutcome> {
        let _lock = store::lock(&self.root)?;
        let all = store::load(&self.root, &EVERY)?;
        let Some(me) = all.iter().find(|t| t.id == id) else {
            return Ok(SlaughterOutcome::NotFound);
        };
        let doomed = live_descendants(&all, id);
        if !doomed.is_empty() && !family {
            return Ok(SlaughterOutcome::HasLiveDescendants(doomed));
        }
        if me.status == Status::Dead && doomed.is_empty() {
            return Ok(SlaughterOutcome::AlreadyDead);
        }
        let mut moved = Vec::new();
        for d in doomed.iter().map(String::as_str).chain(std::iter::once(id)) {
            if store::move_task(&self.root, d, Status::Dead, actor)? == MoveOutcome::Moved {
                moved.push(d.to_string());
            }
        }
        Ok(SlaughterOutcome::Slaughtered(moved))
    }

    pub fn dep_add(&self, id: &str, dep: &str) -> Result<DepOutcome> {
        let _lock = store::lock(&self.root)?;
        store::add_dep(&self.root, id, dep)
    }

    pub fn dep_remove(&self, id: &str, dep: &str) -> Result<DepOutcome> {
        let _lock = store::lock(&self.root)?;
        store::remove_dep(&self.root, id, dep)
    }

    pub fn reparent(&self, id: &str, new_parent: Option<String>) -> Result<Reparent> {
        let _lock = store::lock(&self.root)?;
        store::reparent(&self.root, id, new_parent)
    }

    /// Merge every yak from the farm at `source` into this one, preserving
    /// each yak's status and file bytes verbatim, and copying its attachment
    /// dir (`<id>/`) alongside. A source on the old layout is refused (see
    /// [`store::old_layout_refusal`]). Non-destructive: the source is left
    /// intact (remove or symlink it yourself once satisfied). Refuses on any id
    /// collision — unique prefixes per herd make these rare; reconcile a clash
    /// with `rename-prefix` in the source first. `dry_run` reports the plan
    /// without writing.
    pub fn merge(&self, source: &Path, dry_run: bool) -> Result<MergeOutcome> {
        let Some(src_root) = resolve_farm_root(source) else {
            return Ok(MergeOutcome::NoSource(source.display().to_string()));
        };
        if let Some(why) = store::old_layout_refusal(&src_root) {
            bail!("{why}");
        }
        if let Some(why) = store::migration_refusal(&src_root) {
            bail!("{why}");
        }
        if let (Ok(a), Ok(b)) = (
            std::fs::canonicalize(&src_root),
            std::fs::canonicalize(&self.root),
        ) {
            if a == b {
                return Ok(MergeOutcome::NoSource(source.display().to_string()));
            }
        }
        // A dry run only reads; a real merge writes into this farm.
        let _lock = (!dry_run).then(|| store::lock(&self.root)).transpose()?;
        // Every source yak: `(id, status, its file)`. The file is copied as
        // raw bytes so frontmatter and body round-trip exactly.
        let mut found: Vec<(String, Status, PathBuf)> = Vec::new();
        for t in store::load(&src_root, &EVERY)? {
            if let Some((st, p)) = store::find_task_file(&src_root, &t.id) {
                found.push((t.id, st, p));
            }
        }
        // Refuse if any id already lives in the destination (would clobber).
        let dest_ids = store::all_ids(&self.root);
        let mut collisions: Vec<String> = found
            .iter()
            .filter(|(id, _, _)| dest_ids.contains(id))
            .map(|(id, _, _)| id.clone())
            .collect();
        collisions.sort();
        collisions.dedup();
        if !collisions.is_empty() {
            return Ok(MergeOutcome::Collision(collisions));
        }
        found.sort_by(|a, b| a.0.cmp(&b.0));
        let yaks: Vec<(String, Status)> =
            found.iter().map(|(id, st, _)| (id.clone(), *st)).collect();
        let mut artifacts: Vec<String> = found
            .iter()
            .filter(|(id, _, _)| store::artifact_dir(&src_root, id).is_dir())
            .map(|(id, _, _)| id.clone())
            .collect();
        artifacts.sort();
        // The incoming yaks' herds (id prefixes) that this farm doesn't already
        // declare. Merging yaks without registering their herd leaves them
        // invisible to the create/TUI herd pickers (yaks-095a).
        let cfg = store::read_config(&self.root);
        let mut incoming: Vec<String> = Vec::new();
        for (id, _, _) in &found {
            if let Some((prefix, _)) = id.split_once('-') {
                if !cfg.herds.contains_key(prefix) && !incoming.iter().any(|h| h == prefix) {
                    incoming.push(prefix.to_string());
                }
            }
        }
        incoming.sort();
        let mut herds = incoming;
        if !dry_run {
            for (id, _, path) in &found {
                let dest = store::task_path(&self.root, id);
                if let Some(dir) = dest.parent() {
                    std::fs::create_dir_all(dir)
                        .with_context(|| format!("creating {}", dir.display()))?;
                }
                std::fs::copy(path, dest).with_context(|| format!("copying {}", path.display()))?;
            }
            for id in &artifacts {
                copy_dir_all(
                    &store::artifact_dir(&src_root, id),
                    &store::artifact_dir(&self.root, id),
                )?;
            }
            // Declare the incoming herds, so the merged yaks' herds are
            // offered by the pickers. Report what was actually added.
            herds = store::declare_herds(&self.root, &herds)?;
        }
        Ok(MergeOutcome::Done(MergePlan {
            applied: !dry_run,
            source: src_root,
            yaks,
            artifacts,
            herds,
        }))
    }
}

// -- doctor helpers ------------------------------------------------------

/// Every `.md` file loose at the farm root (no yak file belongs there since
/// schema 6), by name.
fn root_md_files(root: &Path) -> Vec<PathBuf> {
    let Ok(rd) = std::fs::read_dir(root) else {
        return Vec::new();
    };
    let mut out: Vec<PathBuf> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("md"))
        .collect();
    out.sort();
    out
}

/// The value of a `status:` frontmatter line that is not one of the four
/// statuses, read from the file's text (the loader falls back silently). `None`
/// when there is no `status:` line or its value is valid.
fn invalid_status_value(text: &str) -> Option<String> {
    let rest = text
        .strip_prefix("---\n")
        .or_else(|| text.strip_prefix("---\r\n"))?;
    for line in rest.lines() {
        if line.trim_end() == "---" {
            break;
        }
        let Some(value) = line.strip_prefix("status:") else {
            continue;
        };
        let value = value.trim();
        let bare = ["\"", "'"]
            .iter()
            .find_map(|q| value.strip_prefix(q)?.strip_suffix(q))
            .unwrap_or(value);
        return Status::parse(bare).is_none().then(|| bare.to_string());
    }
    None
}

fn git_log(root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("log")
        .args(args)
        .output()
        .context("running git log")?;
    if !out.status.success() {
        anyhow::bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

fn rank(s: Status) -> u8 {
    match s {
        Status::Hairy => 0,
        Status::Shaving => 1,
        Status::Shorn => 2,
        Status::Dead => 3,
    }
}

/// True iff the most recent `yaks verify` run recorded in `body` was a PASS.
/// Verify-run notes are the ones `yaks verify` writes: `verify: <cmd> -> PASS…`
/// / `-> FAIL…`. A yak with no such note (verify: set but never run) is false.
pub(crate) fn last_verify_passed(body: &str) -> bool {
    store::parse_notes(body)
        .iter()
        .rev()
        .find(|n| n.text.starts_with("verify: ") && n.text.contains(" -> "))
        .is_some_and(|n| n.text.contains(" -> PASS"))
}

fn issue_rank(k: IssueKind) -> u8 {
    match k {
        IssueKind::InvalidStatus => 0,
        IssueKind::DuplicateId => 1,
        IssueKind::DanglingParent => 2,
        IssueKind::DanglingDependsOn => 3,
        IssueKind::MissingEvidence => 4,
        IssueKind::UnverifiedShear => 5,
        IssueKind::MalformedLabel => 6,
        IssueKind::MisplacedFile => 7,
    }
}

fn fold_counts<K: std::hash::Hash + Eq, I: Iterator<Item = K>>(it: I) -> Vec<(K, usize)> {
    let mut m: std::collections::HashMap<K, usize> = std::collections::HashMap::new();
    for k in it {
        *m.entry(k).or_insert(0) += 1;
    }
    m.into_iter().collect()
}

/// Resolve a user-supplied path to a farm root: accept either the `.yaks/`
/// directory itself or a directory that contains one.
fn resolve_farm_root(p: &Path) -> Option<PathBuf> {
    let is_farm = |d: &Path| {
        d.join("config.yaml").is_file()
            || d.join("schema").is_file()
            || store::old_layout(d)
            || store::load(d, &EVERY).is_ok_and(|t| !t.is_empty())
    };
    if is_farm(p) {
        return Some(p.to_path_buf());
    }
    let nested = p.join(".yaks");
    is_farm(&nested).then_some(nested)
}

/// Recursively copy `src` into `dst` (carries a merged yak's attachment dir).
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

    /// Yak `id` exists in `root` with status `st` (the file's own `status:`).
    fn at(root: &Path, id: &str, st: Status) -> bool {
        store::load_task_by_id(root, id)
            .ok()
            .flatten()
            .is_some_and(|t| t.status == st)
    }

    /// Every `](path)` in `body`.
    fn link_paths(body: &str) -> Vec<String> {
        body.split("](")
            .skip(1)
            .filter_map(|r| r.split(')').next().map(String::from))
            .collect()
    }

    /// `rename` moves the attachment dir with its yak, and the id rewrite
    /// already covers `](<old>/file)` links (in this yak and in others).
    #[test]
    fn rename_moves_the_attachment_dir_and_fixes_links() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let _ = farm.attach("yak-0001", "a.png", b"png").unwrap();
        let mut other = task("yak-0002", Status::Hairy);
        other.body = "![a](yak-0001/a.png)".into();
        store::write::save(&root, &other).unwrap();

        assert!(matches!(
            farm.rename("yak-0001", "yaks-0001", false).unwrap(),
            RenameOutcome::Done(_)
        ));
        assert!(!store::artifact_dir(&root, "yak-0001").exists());
        assert_eq!(
            std::fs::read(store::artifact_dir(&root, "yaks-0001").join("a.png")).unwrap(),
            b"png"
        );
        let renamed = store::load_task_by_id(&root, "yaks-0001").unwrap().unwrap();
        assert!(
            renamed.body.contains("![a](yaks-0001/a.png)"),
            "{}",
            renamed.body
        );
        let other = store::load_task_by_id(&root, "yak-0002").unwrap().unwrap();
        assert_eq!(other.body, "![a](yaks-0001/a.png)");
        for t in [&renamed, &other] {
            // An attachment link `<id>/<file>` resolves by id (as the TUI
            // and `yaks attachment` do), wherever the linking yak lives.
            for p in link_paths(&t.body) {
                let (id, name) = p.split_once('/').unwrap();
                assert!(
                    store::artifact_dir(&root, id).join(name).is_file(),
                    "{} links {p}",
                    t.id
                );
            }
        }
    }

    /// A source farm on the old layout is refused, naming the fix, and
    /// nothing is copied; a flat source merges byte for byte with its
    /// attachments.
    #[test]
    fn merge_refuses_an_old_layout_source_and_copies_a_flat_one() {
        let (src, src_farm) = temp_farm();
        store::write::save(&src, &task("yak-0001", Status::Shaving)).unwrap();
        src_farm.attach("yak-0001", "a.png", b"AAA").unwrap();
        let (root, farm) = temp_farm();

        // The source's `schema` says 3: refused, whatever else it holds.
        std::fs::write(src.join("schema"), "3\n").unwrap();
        let Err(err) = farm.merge(&src, false) else {
            panic!("an old-layout source must be refused");
        };
        let err = err.to_string();
        assert!(
            err.contains("old yak layout") && err.contains("0.0.16"),
            "{err}"
        );
        assert!(!store::task_path(&root, "yak-0001").exists());

        // A schema-4 source is behind this farm's shapes: migrate it first.
        std::fs::write(src.join("schema"), "4\n").unwrap();
        let Err(err) = farm.merge(&src, false) else {
            panic!("a schema-4 source must be refused");
        };
        assert!(err.to_string().contains("yaks migrate"), "{err}");
        assert!(!store::task_path(&root, "yak-0001").exists());

        // A schema-5 source (yaks at its root) must be migrated first too.
        std::fs::write(src.join("schema"), "5\n").unwrap();
        let Err(err) = farm.merge(&src, false) else {
            panic!("a schema-5 source must be refused");
        };
        assert!(err.to_string().contains("yaks migrate"), "{err}");

        std::fs::write(src.join("schema"), format!("{}\n", store::SCHEMA)).unwrap();
        let MergeOutcome::Done(plan) = farm.merge(&src, false).unwrap() else {
            panic!("expected a merge");
        };
        assert_eq!(plan.artifacts, ["yak-0001"]);
        assert!(at(&root, "yak-0001", Status::Shaving));
        assert_eq!(
            std::fs::read(store::artifact_dir(&root, "yak-0001").join("a.png")).unwrap(),
            b"AAA"
        );
        assert_eq!(
            std::fs::read(store::task_path(&root, "yak-0001")).unwrap(),
            std::fs::read(store::task_path(&src, "yak-0001")).unwrap()
        );
    }

    /// Opening a farm on the old layout refuses (schema marker, or a yak file
    /// in a status directory), a flat one opens.
    #[test]
    fn open_refuses_an_old_layout_farm() {
        let (root, _farm) = temp_farm();
        let parent = root.parent().unwrap().to_path_buf();
        assert!(Farm::open(&parent).is_ok());
        std::fs::create_dir_all(root.join("hairy")).unwrap();
        std::fs::write(
            root.join("hairy/yak-0001.md"),
            store::write::render(&task("yak-0001", Status::Hairy)),
        )
        .unwrap();
        match Farm::open(&parent) {
            Err(OpenError::OldLayout(why)) => {
                assert!(why.contains("npx -y @j15r/yaks@0.0.16 migrate"), "{why}")
            }
            _ => panic!("expected an old-layout refusal"),
        }
        std::fs::remove_dir_all(root.join("hairy")).unwrap();
        std::fs::write(root.join("schema"), "3\n").unwrap();
        assert!(matches!(Farm::open(&parent), Err(OpenError::OldLayout(_))));
    }

    /// A well-formed farm (resolvable parent + dep, one status per id) is clean.
    fn temp_farm() -> (PathBuf, Farm) {
        let mut parent = std::env::temp_dir();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        parent.push(format!("yaksrs-farm-refs-{}-{}", std::process::id(), n));
        let root = parent.join(".yaks");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("schema"), format!("{}\n", store::SCHEMA)).unwrap();
        let farm = match Farm::open(&parent) {
            Ok(h) => h,
            Err(_) => panic!("failed to open temp farm at {parent:?}"),
        };
        (root, farm)
    }

    fn task(id: &str, status: Status) -> Task {
        Task {
            id: id.into(),
            title: format!("title {id}"),
            kind: "task".into(),
            priority: 3,
            status,
            created: Some("2026-01-01T00:00:00Z".into()),
            updated: Some("2026-01-01T00:00:00Z".into()),
            parent: None,
            labels: vec![],
            depends_on: vec![],
            source: None,
            needs: None,
            verify: None,
            extra: Vec::new(),
            body: String::new(),
        }
    }

    /// A `needs` block is never invisible: `inbox` surfaces a blocked yak
    /// whatever its status (the shorn-yak silent-block gap), while unblocked
    /// yaks stay out.
    #[test]
    fn inbox_shows_blocked_yaks_regardless_of_status() {
        let (root, farm) = temp_farm();
        let mut blocked_hairy = task("yak-0001", Status::Hairy);
        blocked_hairy.needs = Some("human".into());
        let mut blocked_shorn = task("yak-0002", Status::Shorn);
        blocked_shorn.needs = Some("human".into());
        store::write::save(&root, &blocked_hairy).unwrap();
        store::write::save(&root, &blocked_shorn).unwrap();
        store::write::save(&root, &task("yak-0003", Status::Hairy)).unwrap(); // unblocked

        let mut got: Vec<String> = farm
            .inbox(FilterSpec::default())
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        got.sort();
        assert_eq!(got, vec!["yak-0001", "yak-0002"]); // shorn block included, unblocked excluded
    }

    fn needs_of(farm: &Farm, id: &str) -> Option<String> {
        store::load_task_by_id(&farm.root, id)
            .unwrap()
            .unwrap()
            .needs
    }

    fn note_texts(farm: &Farm, id: &str) -> Vec<String> {
        let t = store::load_task_by_id(&farm.root, id).unwrap().unwrap();
        store::parse_notes(&t.body)
            .into_iter()
            .map(|n| n.text)
            .collect()
    }

    /// `answer` on an open question records the note and hands the yak to an
    /// agent; the answer stays findable in `inbox` whatever the status (a shorn
    /// yak is not in `next`), and `pickup` is the only thing that clears it.
    #[test]
    fn answer_hands_to_agent_and_only_pickup_clears_it() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Shorn)).unwrap();
        farm.ask("yak-0001", "human", Some("coord"), Some("which?"))
            .unwrap();
        assert_eq!(needs_of(&farm, "yak-0001").as_deref(), Some("human"));

        let c = farm
            .answer("yak-0001", Some("joel"), Some("B"), false)
            .unwrap()
            .unwrap();
        assert_eq!(
            (c.before.as_deref(), c.after.as_deref()),
            (Some("human"), Some("agent"))
        );
        assert_eq!(needs_of(&farm, "yak-0001").as_deref(), Some("agent"));
        let inbox: Vec<String> = farm
            .inbox(FilterSpec::default())
            .unwrap()
            .into_iter()
            .map(|t| t.id)
            .collect();
        assert_eq!(
            inbox,
            vec!["yak-0001"],
            "answered shorn yak stays in the inbox"
        );

        // No other command clears it: a plain note and a status move leave it.
        let _ = farm
            .answer("yak-0001", Some("joel"), Some("more"), false)
            .unwrap();
        assert_eq!(needs_of(&farm, "yak-0001").as_deref(), Some("agent"));

        assert!(matches!(
            farm.pickup("yak-0001", Some("agt"), Some("on it")).unwrap(),
            Pickup::Picked
        ));
        assert_eq!(needs_of(&farm, "yak-0001"), None);
        assert!(farm.inbox(FilterSpec::default()).unwrap().is_empty());
        let notes = note_texts(&farm, "yak-0001");
        assert_eq!(notes.last().unwrap(), "picked up\n\non it");
    }

    #[test]
    fn answer_done_clears_needs_without_handing_off() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        farm.ask("yak-0001", "human", Some("coord"), Some("which?"))
            .unwrap();
        let c = farm
            .answer("yak-0001", Some("joel"), Some("ok"), true)
            .unwrap()
            .unwrap();
        assert_eq!(c.after, None);
        assert_eq!(needs_of(&farm, "yak-0001"), None);
        assert_eq!(note_texts(&farm, "yak-0001").last().unwrap(), "ok");
    }

    /// Edge cases: an answer on a yak with no `needs` keeps today's behaviour
    /// (note recorded, nothing else changes); a second answer on
    /// `needs: agent` keeps the state and appends the note.
    #[test]
    fn answer_edge_cases_no_needs_and_second_answer() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let c = farm
            .answer("yak-0001", Some("joel"), Some("fyi"), false)
            .unwrap()
            .unwrap();
        assert_eq!((c.before, c.after), (None, None));
        assert_eq!(needs_of(&farm, "yak-0001"), None);
        assert_eq!(note_texts(&farm, "yak-0001"), vec!["fyi"]);

        let mut t = task("yak-0002", Status::Hairy);
        t.needs = Some("agent".into());
        store::write::save(&root, &t).unwrap();
        let c = farm
            .answer("yak-0002", Some("joel"), Some("also"), false)
            .unwrap()
            .unwrap();
        assert_eq!(c.after.as_deref(), Some("agent"));
        assert_eq!(note_texts(&farm, "yak-0002"), vec!["also"]);
        assert!(farm.answer("nope", None, None, false).unwrap().is_none());
    }

    /// `pickup` refuses anything that is not `needs: agent`, naming the state;
    /// `ask` on `needs: agent` is allowed and flips it back to human.
    #[test]
    fn pickup_refuses_other_states_and_ask_flips_agent_back() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        assert!(matches!(
            farm.pickup("yak-0001", None, None).unwrap(),
            Pickup::NotAwaitingAgent(None)
        ));
        farm.ask("yak-0001", "human", Some("coord"), None).unwrap();
        assert!(matches!(
            farm.pickup("yak-0001", None, None).unwrap(),
            Pickup::NotAwaitingAgent(Some(n)) if n == "human"
        ));
        assert_eq!(
            needs_of(&farm, "yak-0001").as_deref(),
            Some("human"),
            "refusal changed nothing"
        );
        assert!(matches!(
            farm.pickup("nope", None, None).unwrap(),
            Pickup::NotFound
        ));

        farm.answer("yak-0001", Some("joel"), None, false).unwrap();
        let c = farm
            .ask("yak-0001", "human", Some("agt"), Some("one more thing"))
            .unwrap()
            .unwrap();
        assert_eq!(
            (c.before.as_deref(), c.after.as_deref()),
            (Some("agent"), Some("human"))
        );
    }

    /// `request` on a yak with no `needs` hands it to an agent with a marker
    /// note (inbox shows it, `pickup` clears it); on `needs: human` it refuses
    /// and writes nothing; on `needs: agent` it keeps the state and appends.
    #[test]
    fn request_hands_to_agent_refuses_human_and_appends_on_agent() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let Request::Requested(c) = farm
            .request("yak-0001", Some("joel"), Some("tidy the body"))
            .unwrap()
        else {
            panic!("expected Requested");
        };
        assert_eq!(
            (c.before.as_deref(), c.after.as_deref()),
            (None, Some("agent"))
        );
        assert_eq!(needs_of(&farm, "yak-0001").as_deref(), Some("agent"));
        assert_eq!(
            note_texts(&farm, "yak-0001"),
            vec!["requested: needs agent\n\ntidy the body"]
        );
        let rows = farm.inbox(FilterSpec::default()).unwrap();
        assert_eq!(store::agent_why(&rows[0]), Some(store::AgentWhy::Requested));

        // A second request keeps `needs: agent` and appends.
        let Request::Requested(c) = farm.request("yak-0001", Some("joel"), None).unwrap() else {
            panic!("expected Requested");
        };
        assert_eq!(
            (c.before.as_deref(), c.after.as_deref()),
            (Some("agent"), Some("agent"))
        );
        assert_eq!(note_texts(&farm, "yak-0001").len(), 2);
        assert!(matches!(
            farm.pickup("yak-0001", Some("agt"), None).unwrap(),
            Pickup::Picked
        ));
        assert_eq!(needs_of(&farm, "yak-0001"), None);

        // Awaiting a human: refused, nothing written.
        farm.ask("yak-0001", "human", Some("agt"), Some("q"))
            .unwrap();
        let before = note_texts(&farm, "yak-0001");
        assert!(matches!(
            farm.request("yak-0001", Some("joel"), Some("x")).unwrap(),
            Request::AwaitingHuman(n) if n == "human"
        ));
        assert_eq!(needs_of(&farm, "yak-0001").as_deref(), Some("human"));
        assert_eq!(note_texts(&farm, "yak-0001"), before);
        assert!(matches!(
            farm.request("nope", None, None).unwrap(),
            Request::NotFound
        ));
    }

    /// A `needs` change is not a status move: no `moved:` entry, status kept.
    #[test]
    fn needs_changes_write_no_transition_entry() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Shaving)).unwrap();
        farm.ask("yak-0001", "human", Some("a"), Some("q")).unwrap();
        farm.answer("yak-0001", Some("b"), Some("r"), false)
            .unwrap();
        farm.pickup("yak-0001", Some("a"), None).unwrap();
        let t = store::load_task_by_id(&root, "yak-0001").unwrap().unwrap();
        assert_eq!(t.status, Status::Shaving);
        assert!(
            note_texts(&farm, "yak-0001")
                .iter()
                .all(|n| !store::is_transition_text(n))
        );
    }

    /// `refs` reports formal parent/deps (flagging danglers) and validated body
    /// mentions, and never invents a mention for an id-shaped token that is not
    /// a real yak.
    #[test]
    fn refs_flags_danglers_and_validated_mentions() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0002", Status::Shorn)).unwrap();
        store::write::save(&root, &task("yak-0003", Status::Hairy)).unwrap();
        let mut subject = task("yak-0001", Status::Hairy);
        subject.title = "subject without an id token".into();
        subject.parent = Some("yak-0007".into()); // missing -> dangling
        subject.depends_on = vec!["yak-0002".into(), "yak-9999".into()]; // one dangling
        subject.body = "see yak-0002 and [[yak-0003]] but not yak-9999".into();
        store::write::save(&root, &subject).unwrap();

        let r = farm.refs("yak-0001").unwrap().expect("subject exists");
        let got: Vec<(RefKind, &str, bool, Option<usize>)> = r
            .entries
            .iter()
            .map(|e| (e.kind, e.id.as_str(), e.resolved, e.line))
            .collect();
        assert_eq!(
            got,
            vec![
                (RefKind::Parent, "yak-0007", false, None),
                (RefKind::Depends, "yak-0002", true, None),
                (RefKind::Depends, "yak-9999", false, None),
                (RefKind::Mention, "yak-0002", true, Some(1)),
                (RefKind::Mention, "yak-0003", true, Some(1)),
            ],
            "yak-9999 in the body must not become a mention (validation-gated)"
        );
    }

    #[test]
    fn refs_none_for_missing_task() {
        let (_root, farm) = temp_farm();
        assert!(farm.refs("yak-dead").unwrap().is_none());
    }

    /// `backrefs` finds every yak pointing at the subject — child, dependent,
    /// and whole-id mentions in a title, a description line and a note —
    /// labelled with how, and nothing for lookalike prose or the subject's own
    /// self-mention.
    #[test]
    fn backrefs_find_children_deps_and_mentions_but_not_lookalikes() {
        let (root, farm) = temp_farm();
        let mut subject = task("yak-0001", Status::Hairy);
        subject.body = "I am yak-0001 myself".into();
        store::write::save(&root, &subject).unwrap();
        let mut child = task("yak-0002", Status::Hairy);
        child.parent = Some("yak-0001".into());
        store::write::save(&root, &child).unwrap();
        let mut blocked = task("yak-0003", Status::Shaving);
        blocked.depends_on = vec!["yak-0001".into()];
        store::write::save(&root, &blocked).unwrap();
        let mut titled = task("yak-0004", Status::Shorn);
        titled.title = "follow-up to yak-0001.".into();
        store::write::save(&root, &titled).unwrap();
        let mut prose = task("yak-0005", Status::Hairy);
        // Lookalikes: a longer id, a suffix, bare hex. None is the whole id.
        prose.body = "intro\nsee [[yak-0001]] here\nnot yak-00011, xyak-0001 or 0001".into();
        store::write::save(&root, &prose).unwrap();
        farm.update(
            "yak-0005",
            TaskEdit {
                note: Some("blocked on yak-0001 still".into()),
                actor: Some("alice".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let mut unrelated = task("yak-0006", Status::Hairy);
        unrelated.body = "yak-0001x and yak-0001-2 are not it".into();
        store::write::save(&root, &unrelated).unwrap();

        let r = farm.backrefs("yak-0001").unwrap().expect("subject exists");
        let got: Vec<(&str, String, String)> = r
            .entries
            .iter()
            .map(|e| {
                let via = match &e.via {
                    BackRefVia::Note { actor, .. } => format!("note by {actor:?}"),
                    v => v.label(),
                };
                (e.id.as_str(), e.via.kind().to_string(), via)
            })
            .collect();
        let s = |a: &str| a.to_string();
        assert_eq!(
            got,
            vec![
                ("yak-0002", s("child"), s("child")),
                ("yak-0005", s("mention"), s("body:L2")),
                ("yak-0005", s("mention"), s("note by Some(\"alice\")")),
                ("yak-0003", s("depends"), s("depends")),
                ("yak-0004", s("mention"), s("title")),
            ],
            "lookalikes (yak-00011, xyak-0001, 0001, yak-0001x) and the self-mention must not count"
        );
        assert!(farm.backrefs("yak-dead").unwrap().is_none());
    }

    fn new_task(title: &str, prefix: Option<&str>) -> NewTask {
        NewTask {
            title: title.into(),
            prefix: prefix.map(Into::into),
            kind: None,
            priority: None,
            parent: None,
            labels: vec![],
            depends_on: vec![],
            source: None,
            description: None,
            verify: None,
        }
    }

    #[test]
    fn create_routes_to_the_requested_prefix_over_the_config_default() {
        // The farm's config prefix is `yak` (temp_farm default), but an explicit
        // prefix routes the new yak into a different farm within the same root.
        let (_root, farm) = temp_farm();
        match farm
            .create(new_task("cross-farm yak", Some("proj")))
            .unwrap()
        {
            CreateOutcome::Created(t) => {
                assert!(t.id.starts_with("proj-"), "expected proj- id, got {}", t.id)
            }
            _ => panic!("expected Created"),
        }
    }

    #[test]
    fn create_with_no_explicit_or_default_herd_errors() {
        // Regression (yaks-b47b): with no `--herd`, no pointer herd and no
        // `herd:` in config, `create` used to silently mint a `yak-` id (and so
        // an accidental new herd). It must fail and write nothing.
        let (root, farm) = temp_farm();
        let err = farm
            .create(new_task("orphan", None))
            .err()
            .expect("create with no herd must error");
        let msg = err.to_string();
        assert!(msg.contains("--herd"), "error should name --herd: {msg}");
        assert!(
            msg.contains("default herd"),
            "error should name the default herd: {msg}"
        );
        assert!(
            store::all_ids(&root).is_empty(),
            "no yak may be written on error"
        );
        // An empty `herd:` value is not a default either.
        std::fs::write(root.join("config.yaml"), "herd:\n").unwrap();
        assert!(farm.create(new_task("orphan", None)).is_err());
        // But an explicit herd still works with no default configured.
        assert!(farm.create(new_task("ok", Some("proj"))).is_ok());
    }

    #[test]
    fn create_without_a_prefix_uses_the_config_default() {
        let (root, farm) = temp_farm();
        std::fs::write(root.join("config.yaml"), "herd: core\n").unwrap();
        let default = farm.config().prefix;
        assert_eq!(default, "core");
        match farm.create(new_task("local yak", None)).unwrap() {
            CreateOutcome::Created(t) => assert!(
                t.id.starts_with(&format!("{default}-")),
                "expected {default}- id, got {}",
                t.id
            ),
            _ => panic!("expected Created"),
        }
    }

    #[test]
    fn create_rejects_a_malformed_prefix() {
        let (_root, farm) = temp_farm();
        // Uppercase/space are outside the reference grammar, so the yak is not
        // minted rather than producing an un-referenceable id.
        assert!(matches!(
            farm.create(new_task("bad", Some("Bad Prefix"))).unwrap(),
            CreateOutcome::InvalidPrefix(_)
        ));
    }

    fn created_id(out: CreateOutcome) -> String {
        match out {
            CreateOutcome::Created(t) => t.id,
            _ => panic!("expected Created"),
        }
    }

    #[test]
    fn merge_copies_a_disjoint_farm_preserving_status() {
        let (dest_root, dest) = temp_farm();
        let (src_root, src) = temp_farm();
        // Source yaks live in a different herd (prefix) so ids can't collide.
        let a = created_id(src.create(new_task("source hairy", Some("proj"))).unwrap());
        let b = created_id(src.create(new_task("source done", Some("proj"))).unwrap());
        src.transition(&b, Status::Shorn, None).unwrap();

        let plan = match dest.merge(&src_root, false).unwrap() {
            MergeOutcome::Done(p) => p,
            _ => panic!("expected Done"),
        };
        assert!(plan.applied && plan.yaks.len() == 2);
        // Each landed under its original status dir, and both now resolve.
        assert!(at(&dest_root, &a, Status::Hairy));
        assert!(at(&dest_root, &b, Status::Shorn));
        let ids = store::all_ids(&dest_root);
        assert!(ids.contains(&a) && ids.contains(&b));
        // Non-destructive: the source keeps its files.
        assert!(at(&src_root, &a, Status::Hairy));
    }

    #[test]
    fn merge_refuses_on_id_collision_and_writes_nothing() {
        let (dest_root, dest) = temp_farm();
        let (src_root, _src) = temp_farm();
        store::write::save(&dest_root, &task("yak-0001", Status::Hairy)).unwrap();
        store::write::save(&src_root, &task("yak-0001", Status::Shaving)).unwrap();
        store::write::save(&src_root, &task("yak-0002", Status::Hairy)).unwrap();
        match dest.merge(&src_root, false).unwrap() {
            MergeOutcome::Collision(ids) => assert_eq!(ids, vec!["yak-0001".to_string()]),
            _ => panic!("expected Collision"),
        }
        // Refused wholesale: the non-colliding source yak was not copied either.
        assert!(!store::all_ids(&dest_root).contains("yak-0002"));
    }

    #[test]
    fn merge_dry_run_reports_but_writes_nothing() {
        let (dest_root, dest) = temp_farm();
        let (src_root, src) = temp_farm();
        let id = created_id(src.create(new_task("s", Some("proj"))).unwrap());
        match dest.merge(&src_root, true).unwrap() {
            MergeOutcome::Done(p) => assert!(!p.applied && p.yaks.len() == 1),
            _ => panic!("expected Done"),
        }
        assert!(!store::all_ids(&dest_root).contains(&id));
    }

    #[test]
    fn merge_declares_the_incoming_herd_in_config() {
        // Regression (yaks-095a): merging yaks from another herd must also
        // register that herd in the destination's `herds:` config, or the
        // merged yaks' herd is missing from the known-herd set the pickers use.
        let (dest_root, dest) = temp_farm();
        let (src_root, src) = temp_farm();
        let id = created_id(src.create(new_task("s", Some("web"))).unwrap());
        assert!(id.starts_with("web-"));
        let plan = match dest.merge(&src_root, false).unwrap() {
            MergeOutcome::Done(p) => p,
            _ => panic!("expected Done"),
        };
        assert_eq!(plan.herds, vec!["web".to_string()]);
        // The incoming herd is now declared — and so is the destination's own
        // herd, which materializing the block must not drop.
        let known = store::read_config(&dest_root).known_herds();
        assert!(known.contains(&"web".to_string()), "incoming herd declared");
        let own = store::read_config(&dest_root).prefix;
        assert!(
            known.contains(&own),
            "destination's own herd kept: {known:?}"
        );
    }

    #[test]
    fn merge_herd_declaration_is_idempotent_and_dry_run_writes_nothing() {
        let (dest_root, dest) = temp_farm();
        let (src_root, src) = temp_farm();
        created_id(src.create(new_task("s", Some("web"))).unwrap());
        // Dry run: reports the herd it *would* declare, writes no config.
        let before = std::fs::read_to_string(dest_root.join("config.yaml")).unwrap_or_default();
        match dest.merge(&src_root, true).unwrap() {
            MergeOutcome::Done(p) => assert_eq!(p.herds, vec!["web".to_string()]),
            _ => panic!("expected Done"),
        }
        let after = std::fs::read_to_string(dest_root.join("config.yaml")).unwrap_or_default();
        assert_eq!(before, after, "dry run left config.yaml untouched");
        // Apply, then declaring the same herd again adds nothing.
        dest.merge(&src_root, false).unwrap();
        let added = store::declare_herds(&dest_root, &["web".to_string()]).unwrap();
        assert!(added.is_empty(), "already-declared herd is skipped");
    }

    fn temp_dir_named(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        p.push(format!("yaks-{tag}-{}-{}", std::process::id(), n));
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    /// An attachment resolves by name in its yak's directory, never outside it.
    #[test]
    fn attachments_resolve_in_the_yaks_directory() {
        let root = temp_dir_named("att-here");
        let dir = store::artifact_dir(&root, "y-1");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("b.txt"), "b").unwrap();
        std::fs::write(dir.join("a.png"), "a").unwrap();
        let found = |n| locate_attachment(&root, "y-1", n).map(|a| a.path);
        assert_eq!(found("b.txt"), Some(dir.join("b.txt")));
        assert_eq!(found("missing.txt"), None);
        assert_eq!(found("../y-1/b.txt"), None);
        let names: Vec<String> = list_attachments(&root, "y-1")
            .into_iter()
            .map(|a| a.name)
            .collect();
        assert_eq!(names, ["a.png", "b.txt"]);
        assert!(list_attachments(&root, "y-404").is_empty());
    }

    #[test]
    fn discover_follows_a_pointer_file_with_prefix() {
        let (farm_root, _f) = temp_farm();
        let repo = temp_dir_named("repo");
        std::fs::write(
            repo.join(".yaks"),
            format!("path: {}\nprefix: web\n", farm_root.display()),
        )
        .unwrap();
        let d = store::discover(&repo).unwrap();
        assert_eq!(d.root, farm_root);
        assert_eq!(d.prefix.as_deref(), Some("web"));
    }

    #[test]
    fn create_routes_via_pointer_prefix_and_explicit_wins() {
        let (farm_root, _f) = temp_farm();
        let repo = temp_dir_named("repo");
        std::fs::write(
            repo.join(".yaks"),
            format!("path: {}\nprefix: web\n", farm_root.display()),
        )
        .unwrap();
        let farm = match Farm::open(&repo) {
            Ok(f) => f,
            Err(_) => panic!("open via pointer failed"),
        };
        // No --prefix: the pointer's herd (web) routes the new yak.
        let a = created_id(farm.create(new_task("via pointer", None)).unwrap());
        assert!(a.starts_with("web-"), "expected web- id, got {a}");
        // An explicit --prefix beats the pointer.
        let b = created_id(farm.create(new_task("explicit", Some("core"))).unwrap());
        assert!(b.starts_with("core-"), "expected core- id, got {b}");
        let ids = store::all_ids(&farm_root);
        assert!(ids.contains(&a) && ids.contains(&b));
    }

    #[test]
    fn create_applies_herd_default_type_from_config() {
        let (root, farm) = temp_farm();
        std::fs::write(
            root.join("config.yaml"),
            "prefix: core\ndefault_type: task\nherds:\n  web:\n    default_type: feature\n",
        )
        .unwrap();
        // Into the web herd: inherits web's default_type (feature).
        match farm.create(new_task("web thing", Some("web"))).unwrap() {
            CreateOutcome::Created(t) => {
                assert!(t.id.starts_with("web-"));
                assert_eq!(t.kind, "feature");
            }
            _ => panic!("expected Created"),
        }
        // Into core (no per-herd override): the farm-global default_type (task).
        match farm.create(new_task("core thing", Some("core"))).unwrap() {
            CreateOutcome::Created(t) => assert_eq!(t.kind, "task"),
            _ => panic!("expected Created"),
        }
    }

    fn done(out: RenameOutcome) -> RenamePlan {
        match out {
            RenameOutcome::Done(p) => p,
            _ => panic!("expected RenameOutcome::Done"),
        }
    }

    #[test]
    fn rename_rewrites_every_reference_surface() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yaksrs-0001", Status::Hairy)).unwrap();
        let mut child = task("yaksrs-0002", Status::Hairy);
        child.parent = Some("yaksrs-0001".into());
        child.depends_on = vec!["yaksrs-0001".into()];
        child.body = "blocked by yaksrs-0001, see [[yaksrs-0001]] and yaksrs-0009".into();
        store::write::save(&root, &child).unwrap();

        let plan = done(farm.rename("yaksrs-0001", "yaks-0001", false).unwrap());
        assert!(plan.applied);
        assert_eq!(
            plan.renames,
            vec![("yaksrs-0001".to_string(), "yaks-0001".to_string())]
        );

        // Subject file moved.
        assert!(at(&root, "yaks-0001", Status::Hairy));
        assert!(!store::task_path(&root, "yaksrs-0001").exists());

        // Every referring surface updated; the non-id lookalike is left alone.
        let c = store::load_task_by_id(&root, "yaksrs-0002")
            .unwrap()
            .unwrap();
        assert_eq!(c.parent.as_deref(), Some("yaks-0001"));
        assert_eq!(c.depends_on, vec!["yaks-0001".to_string()]);
        assert!(c.body.contains("blocked by yaks-0001"));
        assert!(c.body.contains("[[yaks-0001]]"));
        assert!(
            c.body.contains("yaksrs-0009"),
            "a token that is not a real id must not be rewritten"
        );
    }

    #[test]
    fn rename_rejects_collision_with_existing_id() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yaksrs-0001", Status::Hairy)).unwrap();
        store::write::save(&root, &task("yaks-0001", Status::Hairy)).unwrap();
        match farm.rename("yaksrs-0001", "yaks-0001", true).unwrap() {
            RenameOutcome::Collision(id) => assert_eq!(id, "yaks-0001"),
            _ => panic!("expected collision"),
        }
    }

    #[test]
    fn rename_dry_run_leaves_files_untouched() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yaksrs-0001", Status::Hairy)).unwrap();
        let plan = done(farm.rename("yaksrs-0001", "yaks-0001", true).unwrap());
        assert!(!plan.applied);
        assert!(at(&root, "yaksrs-0001", Status::Hairy));
        assert!(!store::task_path(&root, "yaks-0001").exists());
    }

    #[test]
    fn rename_many_migrates_a_prefix_batch_across_statuses() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yaksrs-0001", Status::Hairy)).unwrap();
        let mut b = task("yaksrs-0002", Status::Shorn);
        b.depends_on = vec!["yaksrs-0001".into()];
        store::write::save(&root, &b).unwrap();

        let pairs = vec![
            ("yaksrs-0001".to_string(), "yaks-0001".to_string()),
            ("yaksrs-0002".to_string(), "yaks-0002".to_string()),
        ];
        let plan = done(farm.rename_many(&pairs, false).unwrap());
        assert_eq!(plan.renames.len(), 2);
        assert!(at(&root, "yaks-0001", Status::Hairy));
        assert!(at(&root, "yaks-0002", Status::Shorn));
        assert!(!store::task_path(&root, "yaksrs-0001").exists());
        assert!(!store::task_path(&root, "yaksrs-0002").exists());
        let b2 = store::load_task_by_id(&root, "yaks-0002").unwrap().unwrap();
        assert_eq!(b2.depends_on, vec!["yaks-0001".to_string()]);
    }

    #[test]
    fn rename_prefix_migrates_matching_ids_and_flips_config() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yaksrs-0001", Status::Hairy)).unwrap();
        let mut b = task("yaksrs-0002", Status::Hairy);
        b.depends_on = vec!["yaksrs-0001".into()];
        store::write::save(&root, &b).unwrap();
        // A different-prefix id must be left alone (the boundary is `yaksrs-`).
        store::write::save(&root, &task("yaks-abcd", Status::Hairy)).unwrap();

        let plan = done(farm.rename_prefix("yaksrs", "yaks", false).unwrap());
        let news: Vec<&str> = plan.renames.iter().map(|(_, n)| n.as_str()).collect();
        assert_eq!(news, vec!["yaks-0001", "yaks-0002"]);
        assert!(at(&root, "yaks-0001", Status::Hairy));
        assert!(at(&root, "yaks-abcd", Status::Hairy)); // untouched
        assert!(!store::task_path(&root, "yaksrs-0001").exists());
        let b2 = store::load_task_by_id(&root, "yaks-0002").unwrap().unwrap();
        assert_eq!(b2.depends_on, vec!["yaks-0001".to_string()]);
        assert_eq!(store::read_config(&root).prefix, "yaks");
    }

    #[test]
    fn rename_prefix_dry_run_does_not_flip_config_or_move_files() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yaksrs-0001", Status::Hairy)).unwrap();
        let plan = done(farm.rename_prefix("yaksrs", "yaks", true).unwrap());
        assert!(!plan.applied);
        assert!(!root.join("config.yaml").exists());
        assert!(at(&root, "yaksrs-0001", Status::Hairy));
    }

    /// Slaughtering a parent with live descendants is refused (listing them)
    /// unless `family` is set, which takes the whole live family, deepest
    /// first; already-dead descendants are left alone (yaks-05da).
    #[test]
    fn slaughter_guards_and_family_sweeps() {
        let (root, farm) = temp_farm();
        let kid = |id: &str, parent: &str, st: Status| {
            let mut t = task(id, st);
            t.parent = Some(parent.into());
            t
        };
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        store::write::save(&root, &kid("yak-0002", "yak-0001", Status::Shaving)).unwrap();
        store::write::save(&root, &kid("yak-0003", "yak-0002", Status::Shorn)).unwrap();
        store::write::save(&root, &kid("yak-0004", "yak-0001", Status::Dead)).unwrap();
        store::write::save(&root, &task("yak-0005", Status::Hairy)).unwrap();

        assert_eq!(
            farm.slaughter("yak-0001", false, None).unwrap(),
            SlaughterOutcome::HasLiveDescendants(vec!["yak-0003".into(), "yak-0002".into()])
        );
        assert!(at(&root, "yak-0001", Status::Hairy), "guard must not move");

        assert_eq!(
            farm.slaughter("yak-0001", true, None).unwrap(),
            SlaughterOutcome::Slaughtered(vec![
                "yak-0003".into(),
                "yak-0002".into(),
                "yak-0001".into()
            ])
        );
        for id in ["yak-0001", "yak-0002", "yak-0003", "yak-0004"] {
            assert!(at(&root, id, Status::Dead), "{id} dead");
        }
        assert!(at(&root, "yak-0005", Status::Hairy), "non-family untouched");

        assert_eq!(
            farm.slaughter("yak-0001", true, None).unwrap(),
            SlaughterOutcome::AlreadyDead
        );
        assert_eq!(
            farm.slaughter("yak-0005", false, None).unwrap(),
            SlaughterOutcome::Slaughtered(vec!["yak-0005".into()])
        );
        assert_eq!(
            farm.slaughter("yak-nope", true, None).unwrap(),
            SlaughterOutcome::NotFound
        );
    }

    /// The multi-id transition path (`yaks shorn a b c`) drives the CLI batch by
    /// calling [`Farm::transition`] once per id. A clean batch moves every id;
    /// a partial-failure batch (one good id + one nonexistent) still moves the
    /// good id and flags a failure, which the CLI turns into a non-zero exit.
    #[test]
    fn transition_batch_moves_valid_ids_and_flags_missing() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        store::write::save(&root, &task("yak-0002", Status::Hairy)).unwrap();

        // All-good batch: both ids move to shorn, nothing flagged.
        let mut any_failed = false;
        for id in ["yak-0001", "yak-0002"] {
            if farm.transition(id, Status::Shorn, None).unwrap() != MoveOutcome::Moved {
                any_failed = true;
            }
        }
        assert!(!any_failed, "an all-valid batch must not flag failure");
        assert!(at(&root, "yak-0001", Status::Shorn));
        assert!(at(&root, "yak-0002", Status::Shorn));

        // Partial failure: good id moves, missing id reports NotFound. The batch
        // must not abort on the first error, so the good id still moves.
        store::write::save(&root, &task("yak-0003", Status::Hairy)).unwrap();
        let mut outcomes = Vec::new();
        let mut any_failed = false;
        for id in ["yak-0003", "yak-nope"] {
            let outcome = farm.transition(id, Status::Shorn, None).unwrap();
            if outcome != MoveOutcome::Moved {
                any_failed = true;
            }
            outcomes.push(outcome);
        }
        assert_eq!(outcomes, vec![MoveOutcome::Moved, MoveOutcome::NotFound]);
        assert!(
            any_failed,
            "a missing id must flag the batch (non-zero exit)"
        );
        assert!(
            at(&root, "yak-0003", Status::Shorn),
            "the valid id must move despite a sibling failure"
        );
    }

    /// `doctor` flags a task whose formal `parent` and `depends_on` name ids no
    /// yak has, and stays silent about references that do resolve.
    #[test]
    fn doctor_flags_dangling_parent_and_depends_on() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0002", Status::Shorn)).unwrap();
        let mut subject = task("yak-0001", Status::Hairy);
        subject.parent = Some("yak-0404".into()); // no such yak
        subject.depends_on = vec!["yak-0002".into(), "yak-0405".into()]; // one dangles
        store::write::save(&root, &subject).unwrap();

        let issues = farm.doctor(false).unwrap();
        let got: Vec<(IssueKind, Vec<&str>)> = issues
            .iter()
            .map(|i| (i.kind, i.ids.iter().map(String::as_str).collect()))
            .collect();
        assert_eq!(
            got,
            vec![
                (IssueKind::DanglingParent, vec!["yak-0001", "yak-0404"]),
                (IssueKind::DanglingDependsOn, vec!["yak-0001", "yak-0405"]),
            ],
            "resolvable refs (yak-0002) must not be flagged"
        );
    }

    /// The same `id:` in the frontmatter of two yak files (say a hand copy)
    /// is flagged once, naming both files.
    #[test]
    fn doctor_flags_one_id_in_two_files() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        std::fs::copy(
            store::task_path(&root, "yak-0001"),
            root.join("copy-of-yak-0001.md"),
        )
        .unwrap();

        let issues = farm.doctor(false).unwrap();
        assert_eq!(
            issues.len(),
            1,
            "one duplicated id -> one clash: {issues:?}"
        );
        assert_eq!(issues[0].kind, IssueKind::DuplicateId);
        assert_eq!(issues[0].ids, vec!["yak-0001"]);
        assert!(
            issues[0].message.contains("yak-0001.md")
                && issues[0].message.contains("copy-of-yak-0001.md"),
            "message names both files: {}",
            issues[0].message
        );
    }

    /// A `status:` value that is not one of the four.
    #[test]
    fn doctor_flags_an_invalid_status() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Shorn)).unwrap();
        let bad = store::write::render(&task("yak-0003", Status::Hairy))
            .replace("status: hairy", "status: \"finished\"");
        std::fs::write(store::task_path(&root, "yak-0003"), bad).unwrap();

        let got: Vec<(IssueKind, Vec<String>)> = farm
            .doctor(false)
            .unwrap()
            .into_iter()
            .map(|i| (i.kind, i.ids))
            .collect();
        assert_eq!(
            got,
            vec![(IssueKind::InvalidStatus, vec!["yak-0003".to_string()])]
        );
    }

    /// A well-formed farm (resolvable parent + dep, one status per id) is clean.
    #[test]
    fn doctor_clean_farm_has_no_issues() {
        let (root, farm) = temp_farm();
        let mut a = task("yak-0001", Status::Hairy);
        a.parent = Some("yak-0002".into());
        a.depends_on = vec!["yak-0002".into()];
        store::write::save(&root, &a).unwrap();
        store::write::save(&root, &task("yak-0002", Status::Shorn)).unwrap();
        assert!(farm.doctor(false).unwrap().is_empty());
    }

    /// Strict mode flags a shorn yak with no recorded note (a completion without
    /// evidence), while a shorn yak carrying a note passes. A note-less DEAD
    /// (abandoned) yak is exempt — abandonment needs no completion evidence.
    /// Plain `doctor` ignores all of them.
    #[test]
    fn doctor_strict_flags_shorn_yak_without_a_note() {
        let (root, farm) = temp_farm();
        // Shorn with an evidence note: clean.
        let mut with_note = task("yak-0001", Status::Shorn);
        with_note.body =
            store::append_note("", "2026-01-02T00:00:00Z", Some("tester"), "did the work");
        store::write::save(&root, &with_note).unwrap();
        // Shorn with no note at all: an evidence-before-shear violation.
        store::write::save(&root, &task("yak-0002", Status::Shorn)).unwrap();
        // Dead with no note at all: exempt — abandonment needs no evidence.
        store::write::save(&root, &task("yak-0003", Status::Dead)).unwrap();

        // Non-strict doctor ignores evidence entirely.
        assert!(
            farm.doctor(false).unwrap().is_empty(),
            "plain doctor must not flag missing evidence"
        );

        // Strict flags only the note-less shorn yak; the note-less dead yak is exempt.
        let issues = farm.doctor(true).unwrap();
        assert_eq!(
            issues.len(),
            1,
            "only the note-less shorn yak is flagged; dead is exempt"
        );
        assert_eq!(issues[0].kind, IssueKind::MissingEvidence);
        assert_eq!(issues[0].ids, vec!["yak-0002"]);
    }

    /// A shorn yak whose only notes are the `moved:` entries written by the moves
    /// themselves has no evidence: strict doctor flags it; a real note clears it.
    #[test]
    fn doctor_strict_does_not_count_transition_entries_as_evidence() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        store::write::save(&root, &task("yak-0002", Status::Hairy)).unwrap();
        for id in ["yak-0001", "yak-0002"] {
            farm.transition(id, Status::Shaving, Some("tester"))
                .unwrap();
        }
        let mut t = store::load_task_by_id(&root, "yak-0002").unwrap().unwrap();
        t.body = store::append_note(
            &t.body,
            "2026-01-02T00:00:00Z",
            Some("tester"),
            "did the work",
        );
        store::write::save(&root, &t).unwrap();
        for id in ["yak-0001", "yak-0002"] {
            farm.transition(id, Status::Shorn, Some("tester")).unwrap();
        }
        let issues = farm.doctor(true).unwrap();
        assert_eq!(
            issues.len(),
            1,
            "only the yak with no real note: {issues:?}"
        );
        assert_eq!(issues[0].kind, IssueKind::MissingEvidence);
        assert_eq!(issues[0].ids, vec!["yak-0001"]);
    }

    /// Strict mode: a shorn yak that carries a `verify:` command must show a
    /// PASS as its most recent recorded verify run. A failing, never-run, or
    /// stale-then-failing verify is flagged; a passing (or fail-then-pass) one
    /// is clean; a yak with no `verify:` command is not subject to the check.
    #[test]
    fn doctor_strict_flags_shorn_yak_whose_verify_did_not_pass() {
        let (root, farm) = temp_farm();
        let note = |ts: &str, text: &str| store::append_note("", ts, Some("t"), text);

        // verify: + last run PASS -> clean.
        let mut ok = task("yak-0001", Status::Shorn);
        ok.verify = Some("true".into());
        ok.body = note("2026-01-02T00:00:00Z", "verify: `true` -> PASS (exit 0)");
        store::write::save(&root, &ok).unwrap();

        // verify: + last run FAIL -> flagged.
        let mut bad = task("yak-0002", Status::Shorn);
        bad.verify = Some("false".into());
        bad.body = note("2026-01-02T00:00:00Z", "verify: `false` -> FAIL (exit 1)");
        store::write::save(&root, &bad).unwrap();

        // verify: set but never run (has a note, so MissingEvidence is quiet) -> flagged.
        let mut never = task("yak-0003", Status::Shorn);
        never.verify = Some("true".into());
        never.body = note("2026-01-02T00:00:00Z", "did the work");
        store::write::save(&root, &never).unwrap();

        // No verify: command -> not subject to this check.
        let mut plain = task("yak-0004", Status::Shorn);
        plain.body = note("2026-01-02T00:00:00Z", "did it");
        store::write::save(&root, &plain).unwrap();

        // Multiple runs: FAIL then PASS -> the most recent wins -> clean.
        let mut recovered = task("yak-0005", Status::Shorn);
        recovered.verify = Some("flaky".into());
        let b = note("2026-01-02T00:00:00Z", "verify: `flaky` -> FAIL (exit 1)");
        recovered.body = store::append_note(
            &b,
            "2026-01-03T00:00:00Z",
            Some("t"),
            "verify: `flaky` -> PASS (exit 0)",
        );
        store::write::save(&root, &recovered).unwrap();

        // Plain doctor ignores verification entirely.
        assert!(
            farm.doctor(false)
                .unwrap()
                .iter()
                .all(|i| i.kind != IssueKind::UnverifiedShear)
        );

        let mut flagged: Vec<String> = farm
            .doctor(true)
            .unwrap()
            .into_iter()
            .filter(|i| i.kind == IssueKind::UnverifiedShear)
            .flat_map(|i| i.ids)
            .collect();
        flagged.sort();
        assert_eq!(flagged, vec!["yak-0002", "yak-0003"]);
    }

    #[test]
    fn attachment_name_sanitizes_and_keeps_extension() {
        assert_eq!(
            attachment_name("login", "paste-1.png").as_deref(),
            Some("login.png")
        );
        assert_eq!(
            attachment_name("login.jpg", "paste-1.png").as_deref(),
            Some("login.jpg")
        );
        assert_eq!(
            attachment_name(" ../my shot (v2)", "a.png").as_deref(),
            Some("my-shot-v2.png")
        );
        assert_eq!(attachment_name("../", "a.png"), None);
        assert_eq!(attachment_name("notes", "README").as_deref(), Some("notes"));
    }

    /// Renaming moves the file and rewrites links in the owner's body (alt text
    /// included) and in any other yak that references the artifact path, without
    /// touching lookalike paths.
    #[test]
    fn rename_attachment_moves_file_and_rewrites_links() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let mut other = task("yak-0002", Status::Shorn);
        other.body = "see .yaks/yak-0001/paste-1.png and yak-0001/paste-1.png.bak".into();
        store::write::save(&root, &other).unwrap();
        let _ = farm.attach("yak-0001", "paste-1.png", b"png").unwrap();

        let out = farm
            .rename_attachment("yak-0001", "paste-1.png", "login screen")
            .unwrap();
        assert_eq!(
            out,
            RenameAttachmentOutcome::Renamed {
                name: "login-screen.png".into(),
                rewritten: 2
            }
        );
        let dir = store::artifact_dir(&root, "yak-0001");
        assert!(!dir.join("paste-1.png").exists());
        assert_eq!(std::fs::read(dir.join("login-screen.png")).unwrap(), b"png");
        let owner = store::load_task_by_id(&root, "yak-0001").unwrap().unwrap();
        assert!(
            owner
                .body
                .contains("![login-screen](yak-0001/login-screen.png)"),
            "{}",
            owner.body
        );
        let other = store::load_task_by_id(&root, "yak-0002").unwrap().unwrap();
        assert_eq!(
            other.body,
            "see .yaks/yak-0001/login-screen.png and yak-0001/paste-1.png.bak"
        );
    }

    /// `attach` links an image as `![stem](..)` and any other file as
    /// `[stem](..)`; `rename-attachment` rewrites either form, and follows a
    /// rename across the image line (`.txt` -> `.png`) by gaining the `!`.
    #[test]
    fn attach_links_images_with_a_bang_and_other_files_without() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        for name in ["shot.png", "a.JPG", "v.svg", "log.txt", "notes.md", "noext"] {
            let _ = farm.attach("yak-0001", name, b"x").unwrap();
        }
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        for want in [
            "![shot](yak-0001/shot.png)",
            "![a](yak-0001/a.JPG)",
            "![v](yak-0001/v.svg)",
            "[log](yak-0001/log.txt)",
            "[notes](yak-0001/notes.md)",
            "[noext](yak-0001/noext)",
        ] {
            assert!(body.contains(want), "{want} missing from:\n{body}");
        }
        assert!(
            !body.contains("![log]") && !body.contains("![notes]"),
            "{body}"
        );

        // A non-image keeps its plain link through a rename...
        farm.rename_attachment("yak-0001", "log.txt", "build log")
            .unwrap();
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        assert!(
            body.contains("[build-log](yak-0001/build-log.txt)"),
            "{body}"
        );
        assert!(!body.contains("![build-log]"), "{body}");
        // ...an image keeps its `!`...
        farm.rename_attachment("yak-0001", "shot.png", "login")
            .unwrap();
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        assert!(body.contains("![login](yak-0001/login.png)"), "{body}");
        // ...and the form follows the extension when a rename crosses the line.
        farm.rename_attachment("yak-0001", "notes.md", "notes.png")
            .unwrap();
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        assert!(body.contains("![notes](yak-0001/notes.png)"), "{body}");
        // A schema-4 `![txt](..)` link (not yet migrated) is rewritten too.
        let mut t = store::load_task_by_id(&root, "yak-0001").unwrap().unwrap();
        t.body = "![build-log](yak-0001/build-log.txt)".into();
        store::write::save(&root, &t).unwrap();
        farm.rename_attachment("yak-0001", "build-log.txt", "ci")
            .unwrap();
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        assert_eq!(body, "[ci](yak-0001/ci.txt)");
    }

    /// `yaks migrate`: a schema-4 farm is refused by `Farm::open` (naming
    /// `yaks migrate`), migrated by `open_for_migration` + `migrate`, a dry run
    /// writes nothing, and a second run changes nothing.
    #[test]
    fn migrate_upgrades_a_schema_4_farm_to_herd_dirs_once() {
        let (root, _farm) = temp_farm();
        let parent = root.parent().unwrap().to_path_buf();
        let mut t = task("yak-0001", Status::Shaving);
        t.body = format!(
            "see ![log](yak-0001/log.txt) and ![s](yak-0001/s.png){}",
            "\n\n---\n\u{25b8} 2026-01-01T00:00:00Z [a]\nold shape\n\n---\n\u{25b8} 2026-01-02T00:00:00Z\n\nnew shape"
        );
        // A schema-4 farm keeps its yaks at the root (`<id>.md`, `<id>/`);
        // write it that way, with an attachment and a second herd.
        store::write::save(&root, &t).unwrap();
        store::write::save(&root, &task("web-0002", Status::Hairy)).unwrap();
        for id in ["yak-0001", "web-0002"] {
            std::fs::rename(store::task_path(&root, id), root.join(format!("{id}.md"))).unwrap();
        }
        std::fs::create_dir_all(root.join("yak-0001")).unwrap();
        std::fs::write(root.join("yak-0001/log.txt"), "log").unwrap();
        // Not yaks: a loose note without frontmatter, and a stray dir.
        std::fs::write(root.join("README.md"), "farm notes\n").unwrap();
        std::fs::create_dir_all(root.join("misc")).unwrap();
        std::fs::remove_dir_all(root.join("yak")).ok();
        std::fs::remove_dir_all(root.join("web")).ok();
        std::fs::write(root.join("schema"), "4\n").unwrap();
        let before = std::fs::read(root.join("yak-0001.md")).unwrap();

        match Farm::open(&parent) {
            Err(OpenError::NeedsMigration(why)) => assert!(why.contains("yaks migrate"), "{why}"),
            _ => panic!("a schema-4 farm must refuse every command but migrate"),
        }
        let Ok(farm) = Farm::open_for_migration(&parent) else {
            panic!("migrate must open a schema-4 farm");
        };

        let dry = farm.migrate(true).unwrap();
        assert!(!dry.applied && dry.files == 1);
        assert_eq!(
            dry.layout,
            [("web".to_string(), 1, 0), ("yak".to_string(), 1, 1)]
        );
        assert_eq!(std::fs::read(root.join("yak-0001.md")).unwrap(), before);
        assert!(
            !store::task_path(&root, "yak-0001").exists(),
            "dry run moves nothing"
        );
        assert_eq!(std::fs::read_to_string(root.join("schema")).unwrap(), "4\n");

        let done = farm.migrate(false).unwrap();
        assert!(done.applied && done.files == 1);
        assert_eq!(done.layout, dry.layout);
        // Every yak and attachment dir is in its herd directory; nothing
        // that is not a yak moved.
        assert!(!root.join("yak-0001.md").exists() && !root.join("web-0002.md").exists());
        assert!(store::task_path(&root, "web-0002").is_file());
        assert_eq!(
            std::fs::read_to_string(store::artifact_dir(&root, "yak-0001").join("log.txt"))
                .unwrap(),
            "log"
        );
        assert!(!root.join("yak-0001").exists());
        assert!(root.join("README.md").is_file() && root.join("misc").is_dir());
        let counts: Vec<usize> = done
            .fixups
            .iter()
            .map(|f| f.yaks.iter().map(|(_, n)| n).sum())
            .collect();
        assert_eq!(counts, [1, 1], "one old-shape comment, one non-image link");
        assert_eq!(
            std::fs::read_to_string(root.join("schema")).unwrap(),
            format!("{}\n", store::SCHEMA)
        );
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        assert_eq!(
            body,
            "see [log](yak-0001/log.txt) and ![s](yak-0001/s.png)\n\n---\n\u{25b8} 2026-01-01T00:00:00Z [a]\n\nold shape\n\n---\n\u{25b8} 2026-01-02T00:00:00Z\n\nnew shape"
        );
        // Frontmatter (incl. `updated:`) is untouched: only the shape moves.
        let after = std::fs::read_to_string(store::task_path(&root, "yak-0001")).unwrap();
        let front = |s: &str| s.split("\n---\n").next().unwrap().to_string();
        assert_eq!(front(&after), front(&String::from_utf8(before).unwrap()));

        // Now an ordinary farm, and a second run is a no-op.
        assert!(Farm::open(&parent).is_ok());
        let bytes = std::fs::read(store::task_path(&root, "yak-0001")).unwrap();
        let again = farm.migrate(false).unwrap();
        assert!(again.is_noop() && again.files == 0 && again.layout.is_empty());
        assert!(farm.doctor(false).unwrap().is_empty());
        assert_eq!(
            std::fs::read(store::task_path(&root, "yak-0001")).unwrap(),
            bytes
        );
    }

    /// Schema 6: a yak file away from `<herd>/yaks/<id>.md` is reported (and
    /// not loaded); `docs` can never be a herd.
    #[test]
    fn doctor_flags_a_misplaced_yak_file_and_docs_is_not_a_herd() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        store::write::save(&root, &task("web-0002", Status::Hairy)).unwrap();
        // In the wrong herd's directory, and loose at the root.
        std::fs::rename(
            store::task_path(&root, "web-0002"),
            store::herd_dir(&root, "yak").join("web-0002.md"),
        )
        .unwrap();
        std::fs::rename(
            store::task_path(&root, "yak-0001"),
            root.join("yak-0001.md"),
        )
        .unwrap();
        let issues = farm.doctor(false).unwrap();
        let kinds: Vec<_> = issues.iter().map(|i| (i.kind, i.ids.clone())).collect();
        assert!(
            kinds.contains(&(IssueKind::MisplacedFile, vec!["web-0002".to_string()])),
            "{issues:?}"
        );
        assert!(
            kinds.contains(&(IssueKind::MisplacedFile, vec!["yak-0001".to_string()])),
            "{issues:?}"
        );
        assert!(
            issues
                .iter()
                .any(|i| i.message.contains("web/yaks/web-0002.md"))
        );
        // A reserved name is refused as a herd.
        let out = farm
            .create(NewTask {
                title: "x".into(),
                prefix: Some("docs".into()),
                ..Default::default()
            })
            .unwrap();
        assert!(matches!(out, CreateOutcome::InvalidPrefix(p) if p == "docs"));
        // And a `docs/` dir with a `yaks/` inside is never read as a herd.
        let fake = root.join("docs").join("yaks");
        std::fs::create_dir_all(&fake).unwrap();
        std::fs::write(
            fake.join("docs-0001.md"),
            "---\nid: docs-0001\ntitle: t\n---\n",
        )
        .unwrap();
        assert!(!store::all_ids(&root).contains("docs-0001"));
    }

    #[test]
    fn rename_attachment_refuses_collision_and_missing() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let _ = farm.attach("yak-0001", "a.png", b"a").unwrap();
        let _ = farm.attach("yak-0001", "b.png", b"b").unwrap();
        use RenameAttachmentOutcome as R;
        assert_eq!(
            farm.rename_attachment("yak-0001", "a.png", "b").unwrap(),
            R::Collision("b.png".into())
        );
        assert_eq!(
            std::fs::read(store::artifact_dir(&root, "yak-0001").join("b.png")).unwrap(),
            b"b"
        );
        assert_eq!(
            farm.rename_attachment("yak-0001", "a.png", "a").unwrap(),
            R::Unchanged
        );
        assert_eq!(
            farm.rename_attachment("yak-0001", "nope.png", "c").unwrap(),
            R::NoSuchAttachment("nope.png".into())
        );
        assert_eq!(
            farm.rename_attachment("yak-0001", "a.png", "//").unwrap(),
            R::Invalid("//".into())
        );
        assert_eq!(
            farm.rename_attachment("yak-0009", "a.png", "c").unwrap(),
            R::TaskNotFound
        );
    }

    /// yaks-9f5c: a description saved against a stale base keeps a note
    /// another writer appended meanwhile.
    #[test]
    fn description_with_base_keeps_a_concurrent_note() {
        let (root, farm) = temp_farm();
        let mut t = task("yak-0001", Status::Hairy);
        t.body = "Old description.".into();
        store::write::save(&root, &t).unwrap();
        let base = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        let note = TaskEdit {
            note: Some("a concurrent note".into()),
            ..Default::default()
        };
        assert_eq!(
            farm.update("yak-0001", note).unwrap(),
            UpdateOutcome::Updated
        );
        let edit = TaskEdit {
            description: Some("New description.".into()),
            base: Some(base),
            ..Default::default()
        };
        assert_eq!(
            farm.update("yak-0001", edit).unwrap(),
            UpdateOutcome::Updated
        );
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        assert!(
            body.starts_with("New description.\n\n---\n\u{25b8} "),
            "{body}"
        );
        assert!(body.ends_with("\n\na concurrent note"), "{body}");
        assert!(!body.contains("Old description."), "{body}");
    }

    /// yaks-9f5c: a concurrent rewrite of the same text is a conflict, and the
    /// file is left exactly as the other writer left it.
    #[test]
    fn description_with_base_conflicts_on_a_concurrent_rewrite() {
        let (root, farm) = temp_farm();
        let mut t = task("yak-0001", Status::Hairy);
        t.body = "Old description.".into();
        store::write::save(&root, &t).unwrap();
        let base = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        let other = TaskEdit {
            description: Some("Their description.".into()),
            ..Default::default()
        };
        assert_eq!(
            farm.update("yak-0001", other).unwrap(),
            UpdateOutcome::Updated
        );
        let path = store::task_path(&root, "yak-0001");
        let on_disk = std::fs::read_to_string(&path).unwrap();
        let edit = TaskEdit {
            description: Some("My description.".into()),
            base: Some(base),
            title: Some("retitled".into()),
            ..Default::default()
        };
        assert_eq!(
            farm.update("yak-0001", edit).unwrap(),
            UpdateOutcome::Conflict
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), on_disk);
    }

    #[test]
    fn rebase_description_cases() {
        let note = store::append_note("base", "T", Some("a"), "n");
        // Unchanged (trailing whitespace aside): the new text as is.
        assert_eq!(
            rebase_description("base\n", "base", "new".into()).as_deref(),
            Some("new")
        );
        // Appended entry kept.
        assert_eq!(
            rebase_description("base", &note, "new".into()).unwrap(),
            note.replacen("base", "new", 1)
        );
        // From an empty base, and to an empty description.
        let first = store::append_note("", "T", None, "n");
        assert_eq!(
            rebase_description("", &first, "new".into()).unwrap(),
            format!("new\n\n{first}")
        );
        assert_eq!(
            rebase_description("base", &note, String::new()).unwrap(),
            &note[6..]
        );
        // Text appended that is not an entry, or an edited prefix: conflict.
        assert_eq!(
            rebase_description("base", "base and more", "new".into()),
            None
        );
        assert_eq!(
            rebase_description("base", "based\n\n---\n\u{25b8} x", "new".into()),
            None
        );
        assert_eq!(
            rebase_description("base", &note.replace("base", "BASE"), "new".into()),
            None
        );
    }

    #[test]
    fn concurrent_updates_to_one_yak_keep_every_note() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        // Each thread opens its own lock file, so this contends exactly as
        // separate processes do (flock conflicts across open file descriptions).
        std::thread::scope(|s| {
            for w in 0..8 {
                let farm = &farm;
                s.spawn(move || {
                    for n in 0..25 {
                        let edit = TaskEdit {
                            note: Some(format!("w{w}-n{n}")),
                            ..Default::default()
                        };
                        assert!(matches!(
                            farm.update("yak-0001", edit).unwrap(),
                            UpdateOutcome::Updated
                        ));
                    }
                });
            }
        });
        let body = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        for w in 0..8 {
            for n in 0..25 {
                let want = format!("w{w}-n{n}");
                assert_eq!(body.lines().filter(|l| *l == want).count(), 1, "{want}");
            }
        }
        assert_eq!(body.matches("\u{25b8} ").count(), 200);
        assert!(farm.doctor(false).unwrap().is_empty());
    }

    #[test]
    fn lock_ignores_itself_without_clobbering_an_existing_gitignore() {
        let (root, _farm) = temp_farm();
        std::fs::write(root.join(".gitignore"), "scratch/").unwrap();
        drop(store::lock(&root).unwrap());
        drop(store::lock(&root).unwrap());
        assert_eq!(
            std::fs::read_to_string(root.join(".gitignore")).unwrap(),
            "scratch/\n.lock\n"
        );
        assert!(root.join(".lock").is_file());
    }

    #[test]
    fn rename_prefix_does_not_deadlock_on_the_farm_lock() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("old-0001", Status::Hairy)).unwrap();
        assert!(matches!(
            farm.rename_prefix("old", "new", false).unwrap(),
            RenameOutcome::Done(_)
        ));
        assert!(at(&root, "new-0001", Status::Hairy));
    }
}
