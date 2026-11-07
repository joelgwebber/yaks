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

use crate::fields::{Fields, Value};
use crate::filter::{self, FilterSpec};
use crate::model::{NEEDS_AGENT, Status, Task, is_canonical_tag, normalize_tags};
use crate::refs;
use crate::rollup;
use crate::store;

pub use crate::store::{DepOutcome, MoveOutcome, Reparent};

const NON_DEAD: [Status; 3] = [Status::Hairy, Status::Shaving, Status::Shorn];
pub(crate) const EVERY: [Status; 4] = [Status::Hairy, Status::Shaving, Status::Shorn, Status::Dead];

/// An exact-text edit of a dok body: `old` must occur once (yaks-252c).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DokEdit {
    pub old: String,
    pub new: String,
}

/// The result of [`Farm::dok_set`]: whether the file changed, and its hash
/// now (what the next `--if-hash` takes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DokSet {
    pub changed: bool,
    pub hash: String,
}

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
    /// `drop_request` on a yak awaiting an agent because of an answer, not a
    /// request: nothing was written (an answer is never deleted).
    NotARequest,
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
    /// This repo's default herd (id prefix): `git config yaks.herd`, if set.
    /// Used by `create` below an explicit `--herd` and above the farm's
    /// config default.
    repo_herd: Option<String>,
}

/// Pull-request URLs as given: trimmed, empties dropped, duplicates removed
/// (first occurrence wins). The one normalizer for `create --pr` and
/// `update --pr` / `--remove-pr`.
fn dedup_prs(raw: Vec<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for p in raw {
        let p = p.trim().to_string();
        if !p.is_empty() && !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// Fields for a new task (defaults resolved from config inside `create`).
#[derive(Default)]
pub struct NewTask {
    pub title: String,
    /// Which herd (id prefix) the new yak joins. `None` uses the repo's
    /// `yaks.herd`, else the farm's configured default herd; with neither, `create`
    /// errors rather than guess. One farm can hold several herds.
    pub prefix: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<u8>,
    pub parent: Option<String>,
    pub tags: Vec<String>,
    pub depends_on: Vec<String>,
    pub source: Option<String>,
    /// Find or create by `source:` (`create --key`): when any yak in any state,
    /// dead included, already has this `source:`, nothing is created and
    /// [`CreateOutcome::Exists`] names it. Otherwise the new yak gets it as its
    /// `source:` (so `key` and `source` are not both set).
    pub key: Option<String>,
    /// Pull-request URLs for the new yak's `prs:` (duplicates dropped).
    pub prs: Vec<String>,
    pub description: Option<String>,
    pub verify: Option<String>,
    /// Extra frontmatter fields (`due`, `every`, ...), set as `update --set` does.
    pub fields: Vec<(String, Value)>,
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
    pub add_tags: Vec<String>,
    pub remove_tags: Vec<String>,
    /// Set the external `source:` URL; an empty string clears it.
    pub source: Option<String>,
    /// Pull-request URLs to add to `prs:`; one already listed is a no-op.
    pub add_prs: Vec<String>,
    /// Pull-request URLs to remove from `prs:`; one not listed is a no-op.
    pub remove_prs: Vec<String>,
    /// A rerunnable verification command to set on the yak (see `Task::verify`).
    pub verify: Option<String>,
    /// Other frontmatter fields to set (`commit`, `due`, anything but a
    /// built-in key), in order; a [`Value::Null`] removes the field.
    pub fields: Vec<(String, Value)>,
    pub note: Option<String>,
    /// Metadata for the appended `note` (`key: value` lines under its marker;
    /// keys per `store::is_meta_key`). Ignored without a `note`.
    pub meta: Vec<(String, String)>,
    /// Actor to attribute an appended note to (stamped as `[actor]`). Only
    /// meaningful alongside `note`; ownership is never implied.
    pub actor: Option<String>,
}

pub enum CreateOutcome {
    Created(Box<Task>),
    /// `NewTask::key` matched a yak that already exists; it is left as it is
    /// (a dead one stays dead).
    Exists(Box<Task>),
    ParentNotFound(String),
    /// The parent is shorn or dead (yaks-35a4): the message names it and the
    /// fixes ([`finished_parent_refusal`]).
    ParentFinished(String),
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

pub enum SnoozeOutcome {
    Snoozed,
    NotFound,
    Finished(Status),
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
#[derive(Debug, PartialEq, Eq)]
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
    /// Refused: these open (hairy or shaving) descendants would be left under
    /// a dead yak. Pass `family = true` to slaughter them along with it.
    HasOpenDescendants(Vec<String>),
    /// Moved to dead: the open descendants (deepest first), then the yak
    /// itself (omitted if it was already dead). Shorn descendants stay shorn.
    Slaughtered(Vec<String>),
}

/// Every open (hairy or shaving) descendant of `id` in `tasks`, found
/// transitively via the `parent:` field and ordered deepest-first (children
/// before their parents), so slaughtering in order never leaves an open yak
/// under a dead one mid-sweep. Shorn descendants are finished work and are
/// left alone (yaks-27c1). Cycle-safe. Shared by the CLI and the TUI
/// (yaks-05da).
pub fn open_descendants(tasks: &[Task], id: &str) -> Vec<String> {
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
                if k.status.is_open() {
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

/// One dok or session note that links the subject of [`Farm::backrefs`]: the
/// farm-relative `path` of the document, the 1-based `line` of the link and,
/// when it names one of the subject's notes, that note's timestamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocBackRef {
    pub path: String,
    pub line: usize,
    pub note: Option<String>,
}

/// Every yak, dok and session note that points at a task — the inverse of
/// [`TaskRefs`], computed on demand by scanning the farm (there is no stored
/// back-reference index).
pub struct TaskBackRefs {
    pub id: String,
    pub title: String,
    pub entries: Vec<BackRef>,
    /// Doks and session notes that link the yak, one per link.
    pub docs: Vec<DocBackRef>,
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
        let desc_end = store::note_starts(&lines)
            .first()
            .copied()
            .unwrap_or(lines.len());
        for (n, raw) in lines[..desc_end].iter().enumerate() {
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

/// One git commit: full and short sha and the subject line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitRef {
    pub sha: String,
    pub short: String,
    pub subject: String,
}

/// A commit recorded in a yak's own notes (a note with a `commit:` key,
/// yaks-735a), with what the code repository says about it when it can.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedCommit {
    /// The note's `kind:` (`landed`, ...); `commit` when it has none.
    pub kind: String,
    /// The recorded sha, as written.
    pub commit: String,
    /// The worker's sha a landing came from, when the note has one.
    pub source: Option<String>,
    pub branch: Option<String>,
    /// The note's timestamp and actor.
    pub ts: String,
    pub actor: Option<String>,
    /// The commit as the code repository has it; `None` when there is no code
    /// repository here or it does not have the commit.
    pub found: Option<CommitRef>,
}

/// The git commits linked to a yak (`yaks commits`), from three places: shas
/// recorded in its notes, commits in the code repository whose message names
/// the id, and the yak file's own edit history on the farm.
pub struct Commits {
    pub id: String,
    pub path: PathBuf,
    /// The code repository searched (a git top-level), or `None` when yaks was
    /// started outside any code checkout (not in git, or in the farm's own
    /// worktree): then `by_message` is empty and says nothing.
    pub code_repo: Option<PathBuf>,
    pub recorded: Vec<RecordedCommit>,
    /// `--fetch` was asked: the recorded branches were fetched first.
    pub fetch: bool,
    /// The branches the notes record (checkpoints), each with its commits
    /// since the base (yaks-4728); empty when no note names a branch.
    pub branches: Vec<crate::branchlog::BranchCommits>,
    /// Commits naming the id, across every branch but the farm's, newest first.
    pub by_message: Vec<CommitRef>,
    /// Farm history: the commits that edited the yak's file on the farm.
    pub by_file: Vec<CommitRef>,
}

/// What `yaks land` landed, for [`Farm::record_landed`].
pub struct Landed {
    /// The full sha of the commit now in the code history.
    pub commit: String,
    /// The worker's full sha the landing came from, when it is another commit.
    pub source: Option<String>,
    /// The branch it landed on, when HEAD is on one.
    pub branch: Option<String>,
    /// The note body (`landed: <short> <subject>`).
    pub text: String,
}

/// What [`Farm::record_landed`] did (or, in a dry run, would do) for one yak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LandedOutcome {
    Recorded,
    /// The yak already has a `landed` note for this commit.
    AlreadyRecorded,
    NotFound,
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
    /// The note's metadata, in file order (empty for a plain note).
    pub meta: Vec<(String, String)>,
}

/// One integrity problem found by [`Farm::doctor`]. Read-only: doctor never
/// mutates the farm, it only reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub kind: IssueKind,
    pub message: String,
    /// The yak id(s) the issue concerns: the subject first, then any id it
    /// points at (e.g. the missing parent for a dangling reference). For a
    /// dok issue, the dok's farm-relative path.
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
    MalformedTag,
    /// A yak file that is not at its one path, `<herd>/yaks/<id>.md` (in
    /// another herd's directory, under the wrong name, or loose at the farm
    /// root). Not loaded from there; move it to where `yaks path <id>` says.
    MisplacedFile,
    /// The yak's `needs:` disagrees with its latest kinded `ask`, `answer`,
    /// `request` or `pickup` note (yaks-b8cd). A yak with no such note is never
    /// flagged.
    NeedsDrift,
    /// A shorn or dead yak with a hairy or shaving child (yaks-62f2,
    /// yaks-27c1): the parent was called done, or abandoned, while part of it
    /// was still open. The moves refuse this now; a farm from before, or a
    /// hand edit, can still hold it.
    UnfinishedChildren,
    /// A dok (`doks/**.md`, `<herd>/doks/**.md`) with no frontmatter, no
    /// `type:` or `title:`, a name that is not lowercase-kebab, or a value
    /// yaks can't read (yaks-9af0).
    DokProblem,
    /// A markdown link in a yak or dok that leads nowhere in the farm: a farm
    /// path or attachment that does not exist, or a `../` link that climbs out
    /// of its directory (link from the farm root instead) (yaks-101d).
    BrokenLink,
    /// A dok or yak whose fields do not match its schema (`schemas/<type>.md`,
    /// `schemas/yak.md`): a usually-set field missing, a value of the wrong
    /// kind, a field the schema does not list (yaks-8e75). Advice only.
    SchemaMismatch,
    /// A farm document holding a merge conflict to settle: conflict markers
    /// the merge driver left in its prose, or a `merge_conflicts:` list
    /// (yaks-4c8c). `yaks conflicts` lists them.
    MergeConflict,
}

/// Finishing a yak whose children are not all finished (yaks-62f2,
/// yaks-27c1): a shorn or dead yak has no hairy or shaving children. Finish
/// them first, or graduate them out of the family (`yaks reparent <child>
/// --parent <other>` / `--unparent`).
#[derive(Debug)]
pub struct UnfinishedChildren {
    pub id: String,
    /// Where the refused move was going: shorn or dead.
    pub dest: Status,
    pub children: Vec<(String, Status)>,
}

impl std::fmt::Display for UnfinishedChildren {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let list: Vec<String> = self
            .children
            .iter()
            .map(|(c, s)| format!("{c} ({})", s.as_str()))
            .collect();
        let first = if self.dest == Status::Dead {
            "Slaughter them too (`yaks slaughter --family`)"
        } else {
            "Shear them first (or with it: one `yaks shorn` takes children before parents)"
        };
        write!(
            f,
            "{} has unfinished children: {}. {first}, or graduate them out of the family: \
             `yaks reparent <child> --parent <other>` or `--unparent`",
            self.id,
            list.join(", ")
        )
    }
}

impl std::error::Error for UnfinishedChildren {}

/// Why a yak can't be opened (made hairy or shaving, created, or moved) under
/// `parent`, which is `status` (shorn or dead), and the fixes: reopen the
/// parent, or `elsewhere` (put the yak under an open yak or at the top
/// level). One wording for create, reparent and the moves (yaks-35a4).
pub fn finished_parent_refusal(parent: &str, status: Status, elsewhere: &str) -> String {
    let reopen = if status == Status::Dead {
        "revive"
    } else {
        "regrow"
    };
    format!(
        "{parent} is {}, and a finished yak holds no hairy or shaving children: reopen it \
         first (`yaks {reopen} {parent}`), or {elsewhere}",
        status.as_str()
    )
}

/// Opening a yak (regrow, revive, shave) whose parent is shorn or dead
/// (yaks-35a4); its message comes from [`finished_parent_refusal`].
#[derive(Debug)]
pub struct FinishedParent(pub String);

impl std::fmt::Display for FinishedParent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FinishedParent {}

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
            IssueKind::MalformedTag => "malformed-tag",
            IssueKind::MisplacedFile => "misplaced-file",
            IssueKind::NeedsDrift => "needs-drift",
            IssueKind::UnfinishedChildren => "unfinished-children",
            IssueKind::DokProblem => "dok-problem",
            IssueKind::BrokenLink => "broken-link",
            IssueKind::SchemaMismatch => "schema-mismatch",
            IssueKind::MergeConflict => "merge-conflict",
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
            IssueKind::MalformedTag => "Malformed tag (contains a comma or whitespace)",
            IssueKind::MisplacedFile => "Misplaced yak file (not at <herd>/yaks/<id>.md)",
            IssueKind::NeedsDrift => {
                "Needs drift (needs: disagrees with the latest ask/answer/request/pickup note)"
            }
            IssueKind::UnfinishedChildren => {
                "Finished over unfinished children (a shorn or dead yak with hairy or shaving children)"
            }
            IssueKind::DokProblem => "Dok problem (dok frontmatter or name)",
            IssueKind::BrokenLink => {
                "Broken link (a farm path or attachment that does not exist, or a ../ link)"
            }
            IssueKind::SchemaMismatch => {
                "Schema mismatch (fields that do not fit schemas/<type>.md)"
            }
            IssueKind::MergeConflict => {
                "Merge conflict (conflict markers or merge_conflicts: to settle)"
            }
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
        Self::open_root(found.root, found.prefix, allow_unmigrated)
    }

    /// Open the farm root `root` (a `.yaks/` directory) directly, with the same
    /// schema gate as [`Farm::open`] but no discovery: what `YAKS_DIR` does.
    /// Tests use it for a scratch farm outside any repository.
    #[cfg(test)]
    pub(crate) fn at(root: &Path) -> std::result::Result<Farm, OpenError> {
        Self::open_root(root.to_path_buf(), None, false)
    }

    /// [`Farm::at`] for a farm behind the current schema (what `migrate` opens).
    #[cfg(test)]
    pub(crate) fn at_for_migration(root: &Path) -> std::result::Result<Farm, OpenError> {
        Self::open_root(root.to_path_buf(), None, true)
    }

    fn open_root(
        root: PathBuf,
        repo_herd: Option<String>,
        allow_unmigrated: bool,
    ) -> std::result::Result<Farm, OpenError> {
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
        Ok(Farm { root, repo_herd })
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
        task.body = store::append_note_meta(
            &task.body,
            &ts,
            actor,
            &[("kind", "ask"), ("needs", needs)],
            &store::ask_text(needs, question),
        );
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
            // `needs` records the state this answer leaves (`none` once
            // cleared), so a done answer or one on a yak with no `needs` is
            // not mistaken for a hand-off by `doctor`.
            let after = task.needs.as_deref().unwrap_or(store::NEEDS_NONE);
            task.body = store::append_note_meta(
                &task.body,
                &ts,
                actor,
                &[("kind", "answer"), ("needs", after)],
                n,
            );
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
        task.body = store::append_note_meta(
            &task.body,
            &ts,
            actor,
            &[("kind", "request")],
            &store::request_text(note),
        );
        task.updated = Some(ts);
        store::write::save(&self.root, &task)?;
        Ok(Request::Requested(NeedsChange {
            status: task.status,
            before,
            after: task.needs,
        }))
    }

    /// Clear `needs: agent` with an attributed `picked up` note. The only thing
    /// that clears it. Refuses any other state, naming it. With `drop_request`,
    /// the clean finish of a done request (yaks-092f): remove the latest request
    /// note instead of appending a `picked up` one, so a tidy leaves only the
    /// tidied body behind (git keeps the record). Refuses a yak awaiting an
    /// agent because of an answer ([`Pickup::NotARequest`]).
    pub fn pickup(
        &self,
        id: &str,
        actor: Option<&str>,
        note: Option<&str>,
        drop_request: bool,
    ) -> Result<Pickup> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(Pickup::NotFound);
        };
        if !task.awaiting_agent() {
            return Ok(Pickup::NotAwaitingAgent(task.needs));
        }
        let ts = store::now_iso();
        if drop_request {
            if store::agent_why(&task) != Some(store::AgentWhy::Requested) {
                return Ok(Pickup::NotARequest);
            }
            let Some(body) = store::drop_latest_request(&task.body) else {
                return Ok(Pickup::NotARequest);
            };
            task.needs = None;
            task.body = body;
            task.updated = Some(ts);
            store::write::save(&self.root, &task)?;
            return Ok(Pickup::Picked);
        }
        task.needs = None;
        task.body = store::append_note_meta(
            &task.body,
            &ts,
            actor,
            &[("kind", "pickup")],
            &store::pickup_text(note),
        );
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
                    meta: note.meta,
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
    /// [`backrefs_in`] — and every dok and session note that links it (or one
    /// of its notes). Computed on demand by scanning the farm; `None` when
    /// no such task exists.
    pub fn backrefs(&self, id: &str) -> Result<Option<TaskBackRefs>> {
        let all = self.list(FilterSpec::default(), true)?;
        let Some(task) = all.iter().find(|t| t.id == id) else {
            return Ok(None);
        };
        // Doks and session notes: the links `links::edges` already reads from
        // them (a yak's own links are `entries`' business, and an attachment's
        // `from` is `<id>/<file>`, which is neither).
        let docs = crate::links::edges(&self.root)?
            .into_iter()
            .filter(|e| {
                crate::doks::is_dok_path(&e.from)
                    || e.from.starts_with(&format!("{}/", crate::sessions::DIR))
            })
            .filter_map(|e| match e.link {
                crate::links::Link::Yak { id: to, note } if to == id => Some(DocBackRef {
                    path: e.from,
                    line: e.line,
                    note,
                }),
                _ => None,
            })
            .collect();
        Ok(Some(TaskBackRefs {
            id: task.id.clone(),
            title: task.title.clone(),
            entries: backrefs_in(&all, id),
            docs,
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
            for l in t.tags.iter().filter(|l| !is_canonical_tag(l)) {
                let fixed = normalize_tags([l]);
                issues.push(Issue {
                    kind: IssueKind::MalformedTag,
                    message: format!(
                        "{} has tag {:?}; tags may not contain commas or spaces \
                         (would be [{}] — re-edit its tags to fix)",
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

        // `needs:` against the latest kinded note (yaks-b8cd). Only notes that
        // carry `kind` metadata count, so a yak from before metadata is never
        // flagged.
        for t in &tasks {
            if let Some(message) = needs_drift(t) {
                issues.push(Issue {
                    kind: IssueKind::NeedsDrift,
                    message,
                    ids: vec![t.id.clone()],
                });
            }
        }

        // A finished (shorn or dead) yak over hairy or shaving children
        // (yaks-62f2, yaks-27c1).
        for t in tasks.iter().filter(|t| !t.status.is_open()) {
            let open: Vec<&Task> = tasks
                .iter()
                .filter(|c| c.parent.as_deref() == Some(t.id.as_str()) && c.status.is_open())
                .collect();
            if !open.is_empty() {
                let list: Vec<String> = open
                    .iter()
                    .map(|c| format!("{} ({})", c.id, c.status.as_str()))
                    .collect();
                let mut ids = vec![t.id.clone()];
                ids.extend(open.iter().map(|c| c.id.clone()));
                issues.push(Issue {
                    kind: IssueKind::UnfinishedChildren,
                    message: format!(
                        "{} is {} but has unfinished children: {}; finish them, {} {}, or graduate them (`yaks reparent <child> --parent <other>` / `--unparent`)",
                        t.id,
                        t.status.as_str(),
                        list.join(", "),
                        if t.status == Status::Dead { "revive" } else { "regrow" },
                        t.id
                    ),
                    ids,
                });
            }
        }

        for p in crate::doks::check(&self.root)? {
            issues.push(Issue {
                kind: IssueKind::DokProblem,
                message: format!("{}: {}", p.path, p.message),
                ids: vec![p.path],
            });
        }
        for p in crate::doks::load_all(&self.root)? {
            for w in crate::schemas::check_dok(&self.root, &p) {
                issues.push(Issue {
                    kind: IssueKind::SchemaMismatch,
                    message: format!("{}: {w}", p.path),
                    ids: vec![p.path.clone()],
                });
            }
        }
        // `schemas/yak.md` describes the yak's extra fields; its built-in
        // fields are the yak's own and never reported.
        if let Some(s) = crate::schemas::load(&self.root, crate::schemas::YAK_TYPE) {
            for t in &tasks {
                // `due`, `every` and `regrown` are yaks' own (yaks-2fc5): a
                // schema need not list them.
                let known: Vec<&str> = crate::model::BUILT_IN_KEYS
                    .iter()
                    .chain(crate::recur::KEYS.iter())
                    .copied()
                    .collect();
                for w in crate::schemas::check(&s, &t.extra, &known) {
                    issues.push(Issue {
                        kind: IssueKind::SchemaMismatch,
                        message: format!("{}: {w}", t.id),
                        ids: vec![t.id.clone()],
                    });
                }
            }
        }
        for e in crate::links::edges(&self.root)? {
            if crate::links::exists(&self.root, &e.link) == Some(false) {
                let why = match &e.link {
                    crate::links::Link::Upward(_) => {
                        "climbs out with ../; link from the farm root (/doks/...)"
                    }
                    _ => "no such file in the farm",
                };
                issues.push(Issue {
                    kind: IssueKind::BrokenLink,
                    message: format!("{}:L{}: ({}) {why}", e.from, e.line, e.dest),
                    ids: vec![e.from],
                });
            }
        }

        for c in crate::conflicts::scan(&self.root) {
            issues.push(Issue {
                kind: IssueKind::MergeConflict,
                message: format!("{}: {}", c.path, conflict_summary(&c)),
                ids: vec![c.path],
            });
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

    /// The git commits linked to `id` (`None` if the id is not a task):
    /// shas recorded in its notes, then commits naming the id in the code
    /// repository of `cwd` (yaks-7a11: the farm's own worktree holds only farm
    /// commits, so the grep runs in the git top-level yaks was started from,
    /// over every branch but the farm's), then the yak file's own history on
    /// the farm (followed across its status moves). The branches its notes
    /// record are read from the code repository too; only `fetch` touches the
    /// network (yaks-4728).
    pub fn commits(&self, id: &str, cwd: &Path, fetch: bool) -> Result<Option<Commits>> {
        let Some(task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(None);
        };
        let Some((_, path)) = store::find_task_file(&self.root, id) else {
            return Ok(None);
        };
        let code_repo = self.code_repo(cwd);
        let notes = store::parse_notes(&task.body);
        let branches = match &code_repo {
            Some(top) => crate::branchlog::gather(top, &notes, fetch),
            None => Vec::new(),
        };
        let recorded = notes
            .into_iter()
            .filter_map(|n| {
                let commit = n.get("commit")?.to_string();
                let found = code_repo
                    .as_deref()
                    .and_then(|top| git_commit_ref(top, &commit));
                Some(RecordedCommit {
                    kind: n.kind().unwrap_or("commit").to_string(),
                    source: n.get("source").map(str::to_string),
                    branch: n.get("branch").map(str::to_string),
                    commit,
                    ts: n.ts,
                    actor: n.actor,
                    found,
                })
            })
            .collect();
        let by_message = match &code_repo {
            Some(top) => {
                // The farm branch holds `yaks: ...` commits naming the id; they
                // are farm history, listed below, not code. `--all` would add
                // the farm worktree's HEAD, which `--exclude` cannot remove, so
                // name the refs: branches, remotes, tags and this checkout's
                // HEAD (a `--exclude` applies to the one option after it, and for
                // `--branches`/`--remotes` omits the `refs/heads/`/`refs/remotes/`
                // prefix).
                let branch = store::configured_farm_branch(top);
                git_commits(
                    top,
                    &[
                        &format!("--exclude={branch}"),
                        "--branches",
                        &format!("--exclude=*/{branch}"),
                        "--remotes",
                        "--tags",
                        "HEAD",
                        "--fixed-strings",
                        &format!("--grep={id}"),
                    ],
                )?
            }
            None => Vec::new(),
        };
        let by_file = git_commits(&self.root, &["--follow", "--", &path.to_string_lossy()])?;
        Ok(Some(Commits {
            id: id.to_string(),
            path,
            code_repo,
            recorded,
            fetch,
            branches,
            by_message,
            by_file,
        }))
    }

    /// The code repository for a `yaks commits` run from `cwd`: its git
    /// top-level, unless that is the farm's own worktree (a branch farm), which
    /// holds no code, or `cwd` is not in git at all.
    fn code_repo(&self, cwd: &Path) -> Option<PathBuf> {
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(cwd)
            .args(["rev-parse", "--show-toplevel"])
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .ok()
            .filter(|o| o.status.success())?;
        let top = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
        let is_farm_worktree = store::farm_branch(&self.root).is_some()
            && self.root.parent().and_then(|p| p.canonicalize().ok()) == top.canonicalize().ok();
        (!is_farm_worktree).then_some(top)
    }

    /// Record that `commit` landed on each of `ids` (yaks-735a): under the
    /// farm lock, a note with metadata `kind: landed`, `commit`, `source` (the
    /// worker's sha, when it differs) and `branch` (when given), and `text` as
    /// its body. A yak that already has a `landed` note for `commit` is left
    /// alone, so landing twice records once. With `dry_run` nothing is locked or
    /// written and the outcomes say what would happen. Unknown ids are
    /// reported, never fatal. Outcomes come back in `ids` order.
    pub fn record_landed(
        &self,
        ids: &[String],
        landed: &Landed,
        actor: Option<&str>,
        dry_run: bool,
    ) -> Result<Vec<(String, LandedOutcome)>> {
        let _lock = (!dry_run).then(|| store::lock(&self.root)).transpose()?;
        let mut out = Vec::new();
        for id in ids {
            let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
                out.push((id.clone(), LandedOutcome::NotFound));
                continue;
            };
            let already = store::parse_notes(&task.body)
                .iter()
                .any(|n| n.kind() == Some("landed") && n.get("commit") == Some(&landed.commit));
            if already {
                out.push((id.clone(), LandedOutcome::AlreadyRecorded));
                continue;
            }
            if !dry_run {
                let ts = store::now_iso();
                let mut meta = vec![("kind", "landed"), ("commit", landed.commit.as_str())];
                if let Some(s) = landed.source.as_deref() {
                    meta.push(("source", s));
                }
                if let Some(b) = landed.branch.as_deref() {
                    meta.push(("branch", b));
                }
                task.body = store::append_note_meta(&task.body, &ts, actor, &meta, &landed.text);
                task.updated = Some(ts);
                store::write::save(&self.root, &task)?;
            }
            out.push((id.clone(), LandedOutcome::Recorded));
        }
        Ok(out)
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
        if let Some(key) = &new.key {
            // Under the lock, so two callers cannot both create. Every state
            // counts: a yak the human sheared or buried is the answer too.
            if let Some(t) = store::load(&self.root, &EVERY)?
                .into_iter()
                .find(|t| t.source.as_deref() == Some(key.as_str()))
            {
                return Ok(CreateOutcome::Exists(Box::new(t)));
            }
        }
        let cfg = store::read_config(&self.root);
        if let Some(p) = &new.parent {
            let Some(parent) = store::load_task_by_id(&self.root, p)? else {
                return Ok(CreateOutcome::ParentNotFound(p.clone()));
            };
            if !parent.status.is_open() {
                return Ok(CreateOutcome::ParentFinished(finished_parent_refusal(
                    p,
                    parent.status,
                    "create the yak under an open ancestor or at the top level",
                )));
            }
        }
        // Route to a specific herd (id prefix): an explicit `--herd` wins,
        // else this repo's `git config yaks.herd`, else the farm's config default
        // (erroring if there is none). A chosen prefix is validated via the
        // reference grammar so we never mint an un-referenceable id.
        let prefix = match new
            .prefix
            .as_deref()
            .or(self.repo_herd.as_deref())
            .filter(|p| !p.is_empty())
        {
            Some(p)
                if refs::has_ref_shape(&format!("{p}-0000"))
                    && !store::RESERVED_HERD_NAMES.contains(&p) =>
            {
                p.to_string()
            }
            Some(p) => return Ok(CreateOutcome::InvalidPrefix(p.to_string())),
            // No `--herd`, no repo herd: only a herd the farm's config
            // actually declares is a default. `cfg.prefix` alone isn't enough:
            // it is seeded with the built-in "yak" when config is silent, which
            // would silently mint an unintended `yak-` herd (yaks-b47b).
            None if cfg.prefix_declared => cfg.prefix.clone(),
            None => bail!(
                "no herd to create the yak in: pass `--herd <prefix>` or set a default herd \
                 (`herd: <prefix>` in .yaks/config.yaml, or `git config yaks.herd <prefix>`)"
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
            tags: normalize_tags(&new.tags),
            depends_on: new.depends_on,
            source: new.source.or(new.key),
            prs: dedup_prs(new.prs),
            needs: None,
            verify: new.verify,
            extra: {
                let mut extra = Fields::default();
                for (k, v) in &new.fields {
                    if *v != Value::Null {
                        extra.set(k, v);
                    }
                }
                extra
            },
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
        if !edit.add_tags.is_empty() || !edit.remove_tags.is_empty() {
            let remove = normalize_tags(&edit.remove_tags);
            task.tags = normalize_tags(task.tags.iter().chain(&edit.add_tags));
            task.tags.retain(|l| !remove.contains(l));
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
        if !edit.add_prs.is_empty() || !edit.remove_prs.is_empty() {
            let mut prs = task.prs.clone();
            for p in dedup_prs(edit.add_prs) {
                if !prs.contains(&p) {
                    prs.push(p);
                }
            }
            let remove = dedup_prs(edit.remove_prs);
            prs.retain(|p| !remove.contains(p));
            if prs != task.prs {
                task.prs = prs;
                changed = true;
            }
        }
        if let Some(v) = edit.verify {
            task.verify = if v.is_empty() { None } else { Some(v) };
            changed = true;
        }
        for (k, v) in &edit.fields {
            let before = task.extra.raw(k).map(<[String]>::to_vec);
            if *v == Value::Null {
                task.extra.remove(k);
            } else {
                task.extra.set(k, v);
            }
            changed |= task.extra.raw(k).map(<[String]>::to_vec) != before;
        }
        if let Some(n) = edit.note {
            let ts = store::now_iso();
            let meta: Vec<(&str, &str)> = edit
                .meta
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            task.body = store::append_note_meta(&task.body, &ts, edit.actor.as_deref(), &meta, &n);
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

    /// Move an open yak's `due:` to `due` (already resolved, see
    /// [`crate::recur::resolve_due`]) and say so in a `kind: snooze` note, so
    /// the history shows who put it off and from when. A finished yak has
    /// nothing to put off: refused with [`SnoozeOutcome::Finished`].
    pub fn snooze(&self, id: &str, due: &str, actor: Option<&str>) -> Result<SnoozeOutcome> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(SnoozeOutcome::NotFound);
        };
        if !task.status.is_open() {
            return Ok(SnoozeOutcome::Finished(task.status));
        }
        let was = crate::recur::due(&task).map(crate::recur::When::render);
        task.extra
            .set(crate::recur::DUE, &Value::Plain(due.to_string()));
        let text = match &was {
            Some(w) => format!("snoozed until {due} (was due {w})"),
            None => format!("snoozed until {due} (had no due date)"),
        };
        let ts = store::now_iso();
        task.body = store::append_note_meta(&task.body, &ts, actor, &[("kind", "snooze")], &text);
        task.updated = Some(ts);
        store::write::save(&self.root, &task)?;
        Ok(SnoozeOutcome::Snoozed)
    }

    /// Every dok of the farm (yaks-9af0), sorted by path.
    pub fn doks(&self) -> Result<Vec<crate::doks::Dok>> {
        crate::doks::load_all(&self.root)
    }

    /// The dok at farm-relative `path` (a leading `/` is the farm root).
    pub fn dok(&self, path: &str) -> Result<Option<crate::doks::Dok>> {
        crate::doks::load(&self.root, path)
    }

    /// The dok at `path` with its content hash ([`crate::doks::text_hash`]).
    pub fn dok_hashed(&self, path: &str) -> Result<Option<(crate::doks::Dok, String)>> {
        crate::doks::load_hashed(&self.root, path)
    }

    /// Create the dok at `path` (see [`crate::doks::new_dok`]) under the
    /// farm lock, stamped `created:` now (UTC, as a yak's) unless `fields`
    /// gives one (yaks-c316). Nothing stamps an `updated:`: hand edits would
    /// leave it stale. Returns its file path.
    pub fn dok_new(
        &self,
        path: &str,
        kind: Option<&str>,
        title: Option<&str>,
        fields: &[(String, Value)],
        body: &str,
    ) -> Result<std::path::PathBuf> {
        let _lock = store::lock(&self.root)?;
        // A schema's defaults fill fields the caller did not give.
        let mut all: Vec<(String, Value)> =
            crate::schemas::defaults(&self.root, kind.unwrap_or(crate::doks::DEFAULT_KIND))
                .into_iter()
                .filter(|(k, _)| !fields.iter().any(|(f, _)| f == k))
                .collect();
        all.extend(fields.iter().cloned());
        if !all.iter().any(|(k, _)| k == "created") {
            let now = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            all.insert(0, ("created".into(), Value::Str(now)));
        }
        let dok = crate::doks::new_dok(&self.root, path, kind, title, &all, body)?;
        crate::doks::save(&self.root, &dok)
    }

    /// Save a session note (yaks-17f5): checks the body, fills `## Saved`
    /// from `saved` when the body has none, and writes it under `sessions/`,
    /// under the farm lock. Returns the note's farm-relative path.
    pub fn session_note(
        &self,
        title: &str,
        body: &str,
        saved: Option<&str>,
        origin: &crate::sessions::Origin,
    ) -> Result<String> {
        let info = crate::sessions::check_body(body)?;
        let body = match saved {
            Some(s) if !info.has_saved => crate::sessions::with_saved(body, s),
            _ => body.to_string(),
        };
        let _lock = store::lock(&self.root)?;
        let dok = crate::sessions::build(&self.root, title, &body, origin, chrono::Utc::now())?;
        crate::doks::save(&self.root, &dok)?;
        Ok(dok.path)
    }

    /// Stamp `promoted: <day>` on each session note in `paths` (one commit
    /// for the lot), under the farm lock. Returns the paths it stamped; one
    /// already stamped is skipped. Every path is checked before any is
    /// written.
    pub fn session_promote(&self, paths: &[String], day: &str) -> Result<Vec<String>> {
        let _lock = store::lock(&self.root)?;
        for p in paths {
            let rel = crate::doks::normalize(p);
            if !rel.starts_with("sessions/") || !self.root.join(&rel).is_file() {
                anyhow::bail!("{rel}: no such session note (`yaks session list` shows them)");
            }
        }
        let mut done = Vec::new();
        for p in paths {
            if crate::sessions::promote(&self.root, p, day)? {
                done.push(crate::doks::normalize(p));
            }
        }
        Ok(done)
    }

    /// Set (a [`Value::Null`] removes) frontmatter fields of the dok at
    /// `path`, and replace its body when `body` is given or edit it in place
    /// when `edit` is, under the farm lock. With `if_hash`, refuses (writing
    /// nothing) unless the file still has that [`crate::doks::text_hash`]
    /// (yaks-252c). `Ok(None)`: no such dok.
    pub fn dok_set(
        &self,
        path: &str,
        fields: &[(String, Value)],
        body: Option<&str>,
        edit: Option<&DokEdit>,
        if_hash: Option<&str>,
    ) -> Result<Option<DokSet>> {
        let _lock = store::lock(&self.root)?;
        let Some((mut dok, current)) = crate::doks::load_hashed(&self.root, path)? else {
            return Ok(None);
        };
        if let Some(want) = if_hash {
            let want = want.trim();
            if want != current {
                anyhow::bail!(
                    "{} changed since it was read: its hash is {current}, not {want}; \
                     nothing was written (`yaks dok show` for the current text and hash, \
                     then redo the edit)",
                    dok.path
                );
            }
        }
        let before = dok.render();
        for (k, v) in fields {
            if *v == Value::Null {
                dok.fields.remove(k);
            } else {
                dok.fields.set(k, v);
            }
        }
        if let Some(b) = body {
            dok.body = b.to_string();
        }
        if let Some(e) = edit {
            if e.old.is_empty() {
                anyhow::bail!("--old is empty; give the exact text to replace");
            }
            match dok.body.matches(&e.old).count() {
                1 => dok.body = dok.body.replacen(&e.old, &e.new, 1),
                0 => anyhow::bail!(
                    "{}: --old text matches 0 times in the body; nothing was written \
                     (it must match exactly once: `yaks dok show` for the current text)",
                    dok.path
                ),
                n => anyhow::bail!(
                    "{}: --old text matches {n} times in the body; nothing was written \
                     (add surrounding text so it matches exactly once)",
                    dok.path
                ),
            }
        }
        let after = dok.render();
        if after == before {
            return Ok(Some(DokSet {
                changed: false,
                hash: current,
            }));
        }
        crate::doks::save(&self.root, &dok)?;
        Ok(Some(DokSet {
            changed: true,
            hash: crate::doks::text_hash(&after),
        }))
    }

    /// Create or update the row `key` of the table in the dok at `path`
    /// (see [`crate::rows::set_row`]), under the farm lock. `Ok(None)`: no
    /// such dok.
    pub fn dok_row_set(
        &self,
        path: &str,
        key: &str,
        cells: &[(String, String)],
    ) -> Result<Option<crate::rows::RowChange>> {
        let _lock = store::lock(&self.root)?;
        let Some(mut dok) = crate::doks::load(&self.root, path)? else {
            return Ok(None);
        };
        let (body, change) = crate::rows::set_row(&dok.body, key, cells)
            .with_context(|| format!("{}", crate::doks::normalize(path)))?;
        if change != crate::rows::RowChange::Unchanged {
            dok.body = body;
            crate::doks::save(&self.root, &dok)?;
        }
        Ok(Some(change))
    }

    /// Remove the row `key` of the table in the dok at `path`, under the
    /// farm lock. `Ok(false)`: no such dok.
    pub fn dok_row_delete(&self, path: &str, key: &str) -> Result<bool> {
        let _lock = store::lock(&self.root)?;
        let Some(mut dok) = crate::doks::load(&self.root, path)? else {
            return Ok(false);
        };
        dok.body = crate::rows::delete_row(&dok.body, key)
            .with_context(|| format!("{}", crate::doks::normalize(path)))?;
        crate::doks::save(&self.root, &dok)?;
        Ok(true)
    }

    /// Save a file or directory as a source (see [`crate::sources::add`]),
    /// under the farm lock.
    pub fn source_add(&self, req: &crate::sources::Request) -> Result<crate::sources::Saved> {
        let _lock = store::lock(&self.root)?;
        crate::sources::add(&self.root, req)
    }

    /// Set or clear the mark `yaks check` honours in a source's `source.md`
    /// (see [`crate::sources::mark`]), under the farm lock. Returns the
    /// source's directory and whether the file changed.
    pub fn source_mark(&self, source: &str, how: &crate::sources::Mark) -> Result<(String, bool)> {
        let _lock = store::lock(&self.root)?;
        crate::sources::mark(&self.root, source, how)
    }

    /// Write or merge `schemas/<kind>.md` (see [`crate::schemas::define`]),
    /// under the farm lock.
    pub fn dok_define(&self, def: &crate::schemas::Define) -> Result<crate::schemas::Defined> {
        let _lock = store::lock(&self.root)?;
        crate::schemas::define(&self.root, def)
    }

    /// Move yak `id`'s attachment `name` out of the yak: into `sources/` as a
    /// source (`to_dok: None`), or to a dok at `to_dok` (a markdown
    /// attachment, given frontmatter when it has none). Every link to the
    /// attachment, in any yak or dok, is rewritten to the new place. Under
    /// the farm lock. Returns the new farm-relative path.
    pub fn promote(
        &self,
        id: &str,
        name: &str,
        to_dok: Option<&str>,
        actor: &str,
    ) -> Result<String> {
        let _lock = store::lock(&self.root)?;
        let Some(found) = locate_attachment(&self.root, id, name) else {
            anyhow::bail!("{id}/{name}: no such attachment (`yaks attachment {id}` lists them)");
        };
        let new_path = match to_dok {
            None => {
                // `add` refuses a path inside the farm (cite it instead), so
                // save a copy taken out of it; the attachment is removed after.
                let tmp = std::env::temp_dir().join(format!("yaks-promote-{}", std::process::id()));
                std::fs::create_dir_all(&tmp)?;
                let copy = tmp.join(name);
                std::fs::copy(&found.path, &copy)?;
                let saved = crate::sources::add(
                    &self.root,
                    &crate::sources::Request {
                        path: copy,
                        title: Some(name.to_string()),
                        url: None,
                        part_of: None,
                        text: None,
                        actor: actor.to_string(),
                        origin: Some(format!("{id}/{name}")),
                        refresh: false,
                        text_only: false,
                    },
                );
                let _ = std::fs::remove_dir_all(&tmp);
                let saved = saved?;
                let dir = saved.dir().to_string();
                // The saved file's own name (made safe on the way in).
                let file = std::fs::read_to_string(self.root.join(&dir).join("source.md"))
                    .ok()
                    .and_then(|t| crate::doks::Dok::parse("x", &t).str_field("file"))
                    .unwrap_or_else(|| name.to_string());
                format!("{dir}/{file}")
            }
            Some(dok) => {
                let dok = crate::doks::normalize(dok);
                if let Some(why) = crate::doks::path_problem(&dok) {
                    anyhow::bail!("{dok}: {why}");
                }
                if self.root.join(&dok).exists() {
                    anyhow::bail!("{dok} already exists");
                }
                if !name.ends_with(".md") {
                    anyhow::bail!(
                        "{name}: only a markdown attachment becomes a dok; promote it as a source"
                    );
                }
                let text = std::fs::read_to_string(&found.path)?;
                // The dok lives elsewhere: its bare sibling links (`shot.png`)
                // keep naming the yak's attachments, as `<id>/<file>`.
                let ids = store::all_ids(&self.root);
                let (text, _) = crate::links::rewrite(&text, |dest| {
                    match crate::links::parse(dest, crate::links::From::Attachment(id), |s| {
                        ids.contains(s)
                    }) {
                        crate::links::Link::Attachment { name, .. } if !dest.contains('/') => {
                            Some(format!("{id}/{}", crate::links::percent_encode_path(&name)))
                        }
                        _ => None,
                    }
                });
                let mut p = crate::doks::Dok::parse(&dok, &text);
                if !p.has_frontmatter {
                    p = crate::doks::new_dok(&self.root, &dok, None, None, &[], &text)?;
                }
                p.path = dok.clone();
                crate::doks::save(&self.root, &p)?;
                dok
            }
        };
        std::fs::remove_file(&found.path)?;
        // An attachment directory left empty goes too.
        let adir = store::artifact_dir(&self.root, id);
        if std::fs::read_dir(&adir).is_ok_and(|mut d| d.next().is_none()) {
            let _ = std::fs::remove_dir(&adir);
        }
        self.rewrite_links_to(
            &crate::links::Link::Attachment {
                id: id.to_string(),
                name: name.to_string(),
                anchor: None,
            },
            &new_path,
        )?;
        Ok(new_path)
    }

    /// Rewrite every link that names `target` (an attachment or farm path,
    /// any anchor) to `/<new_path>`, keeping the anchor. The caller holds the
    /// lock.
    fn rewrite_links_to(&self, target: &crate::links::Link, new_path: &str) -> Result<()> {
        use crate::links::Link;
        let same = |l: &Link| match (l, target) {
            (Link::Attachment { id, name, .. }, Link::Attachment { id: i, name: n, .. }) => {
                id == i && name == n
            }
            (Link::Farm { path, .. }, Link::Farm { path: p, .. }) => path == p,
            _ => false,
        };
        let tasks = store::load(&self.root, &EVERY)?;
        let ids: std::collections::HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
        let is_yak = |s: &str| ids.contains(s);
        // `yaks attach` writes the file name as is, so a name with a space
        // gives `](<id>/a b.png)`, which CommonMark does not read as a link:
        // replace that literal form too.
        let literal = match target {
            Link::Attachment { id, name, .. } if name.contains(' ') => {
                Some(format!("]({id}/{name}"))
            }
            _ => None,
        };
        let fix = |text: &str, doc: crate::links::From| {
            let (mut text, mut n) = crate::links::rewrite(text, |dest| {
                let l = crate::links::parse(dest, doc, is_yak);
                same(&l).then(|| match dest.split_once('#') {
                    Some((_, frag)) => format!("/{new_path}#{frag}"),
                    None => format!("/{new_path}"),
                })
            });
            if let Some(lit) = &literal {
                let k = text.matches(lit.as_str()).count();
                if k > 0 {
                    text = text.replace(lit.as_str(), &format!("](/{new_path}"));
                    n += k;
                }
            }
            (text, n)
        };
        for mut t in tasks {
            let (body, n) = fix(&t.body, crate::links::From::Yak);
            if n > 0 {
                t.body = body;
                t.updated = Some(store::now_iso());
                store::write::save(&self.root, &t)?;
            }
        }
        for mut p in crate::doks::load_all(&self.root)? {
            let path = p.path.clone();
            let (body, n) = fix(&p.body, crate::links::From::Dok(&path));
            if n > 0 {
                p.body = body;
                crate::doks::save(&self.root, &p)?;
            }
        }
        // The markdown attachments of every yak (an evidence document
        // linking the file by its bare name).
        for id in &ids {
            for a in list_attachments(&self.root, id) {
                if !crate::links::is_markdown_name(&a.name) {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&a.path) else {
                    continue;
                };
                let (text, n) = fix(&text, crate::links::From::Attachment(id));
                if n > 0 {
                    std::fs::write(&a.path, text)?;
                }
            }
        }
        Ok(())
    }

    /// Every markdown link in the farm that names the dok or file at
    /// farm-relative `path` (any rev or anchor), with where it sits.
    pub fn links_to(&self, path: &str) -> Result<Vec<crate::links::Edge>> {
        let path = crate::doks::normalize(path);
        Ok(crate::links::edges(&self.root)?
            .into_iter()
            .filter(|e| crate::links::farm_path(&e.link).as_deref() == Some(path.as_str()))
            .collect())
    }

    /// Move the dok at `from` to `to` and rewrite every link to it, in yaks
    /// and doks, under the farm lock. A link keeps its rev and anchor and is
    /// written in the farm-root form (`/doks/...`). `dry_run` writes nothing.
    /// Returns the documents changed (`(yak id or dok path, links)`).
    pub fn dok_move(&self, from: &str, to: &str, dry_run: bool) -> Result<Vec<(String, usize)>> {
        let _lock = (!dry_run).then(|| store::lock(&self.root)).transpose()?;
        let from = crate::doks::normalize(from);
        let to = crate::doks::normalize(to);
        if crate::doks::load(&self.root, &from)?.is_none() {
            anyhow::bail!("no dok {from}");
        }
        if let Some(why) = crate::doks::path_problem(&to) {
            anyhow::bail!("{to}: {why}");
        }
        if self.root.join(&to).exists() {
            anyhow::bail!("{to} already exists");
        }
        let tasks = store::load(&self.root, &EVERY)?;
        let ids: std::collections::HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();
        let is_yak = |s: &str| ids.contains(s);
        // Rewrite links in `text` (a document at `doc`) to the farm-root form
        // `/<path>[@rev][#fragment as written]`: a link to `from` gets `to`;
        // in the moved dok itself, a relative link keeps naming what it
        // named from the old directory.
        let fix = |text: &str, doc: crate::links::From, moved_self: bool| {
            crate::links::rewrite(text, |dest| {
                let crate::links::Link::Farm { path, rev, .. } =
                    crate::links::parse(dest, doc, is_yak)
                else {
                    return None;
                };
                let target = if path == from {
                    to.clone()
                } else if moved_self && !dest.starts_with('/') {
                    path
                } else {
                    return None;
                };
                let mut s = format!("/{target}");
                if let Some(r) = rev {
                    s.push_str(&format!("@{r}"));
                }
                if let Some((_, frag)) = dest.split_once('#') {
                    s.push_str(&format!("#{frag}"));
                }
                Some(s)
            })
        };
        let mut changed = Vec::new();
        for mut t in tasks {
            let (body, n) = fix(&t.body, crate::links::From::Yak, false);
            if n > 0 {
                changed.push((t.id.clone(), n));
                if !dry_run {
                    t.body = body;
                    store::write::save(&self.root, &t)?;
                }
            }
        }
        for mut p in crate::doks::load_all(&self.root)? {
            let me = p.path == from;
            let (body, n) = fix(&p.body, crate::links::From::Dok(&p.path), me);
            if n > 0 {
                changed.push((p.path.clone(), n));
                p.body = body;
            }
            if me {
                p.path = to.clone();
                if !dry_run {
                    crate::doks::save(&self.root, &p)?;
                    std::fs::remove_file(self.root.join(&from))?;
                }
            } else if n > 0 && !dry_run {
                crate::doks::save(&self.root, &p)?;
            }
        }
        Ok(changed)
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
    /// inside the farm and is committed with it (`yaks commit`), so an
    /// attachment is shareable evidence, not just local scratch.
    pub fn attach(&self, id: &str, name: &str, data: &[u8]) -> Result<AttachOutcome> {
        self.attach_many(id, &[(name.to_string(), data.to_vec())], true, None)
    }

    /// [`Farm::attach`] for several files in one write (one farm commit).
    /// With `link` false the files are only stored beside the yak and its
    /// body is untouched: for the parts of a document that links them itself
    /// (an evidence dok whose `![x](shot.png)` names its sibling
    /// attachments, see [`crate::links::From::Attachment`]). `caption` is
    /// written as the link's markdown title (see
    /// [`store::attachment_link_titled`]); it is part of the link, so it is
    /// ignored without one.
    pub fn attach_many(
        &self,
        id: &str,
        files: &[(String, Vec<u8>)],
        link: bool,
        caption: Option<&str>,
    ) -> Result<AttachOutcome> {
        let _lock = store::lock(&self.root)?;
        let Some(mut task) = store::load_task_by_id(&self.root, id)? else {
            return Ok(AttachOutcome::NotFound);
        };
        let dir = store::artifact_dir(&self.root, id);
        std::fs::create_dir_all(&dir)?;
        let mut body = task.body.trim_end().to_string();
        for (name, data) in files {
            std::fs::write(dir.join(name), data)?;
            if link {
                if !body.is_empty() {
                    body.push_str("\n\n");
                }
                body.push_str(&store::attachment_link_titled(id, name, caption));
            }
        }
        if link {
            task.body = body;
            task.updated = Some(store::now_iso());
            store::write::save(&self.root, &task)?;
        }
        Ok(AttachOutcome::Attached(
            files
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        ))
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

        // Links hold the name percent-encoded (`a%20b.png`); older ones as is.
        let enc = crate::links::percent_encode_path;
        let old_path = format!("{id}/{}", enc(old));
        let old_literal = format!("{id}/{old}");
        let new_path = format!("{id}/{}", enc(&name));
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
            if !t.body.contains(&old_path) && !t.body.contains(&old_literal) {
                continue;
            }
            let body = replace_path_token(&t.body, &old_path, &new_path);
            let body = if old_literal != old_path {
                body.replace(&format!("]({old_literal})"), &format!("]({new_path})"))
            } else {
                body
            };
            let body = replace_link(&body, &old_text, &new_link);
            if body != t.body {
                t.body = body;
                t.updated = Some(store::now_iso());
                store::write::save(&self.root, &t)?;
                rewritten += 1;
            }
        }
        // Markdown attachments (of any yak) naming it: a sibling's bare
        // `old` stays bare, `<id>/old` stays qualified.
        let ids = store::all_ids(&self.root);
        for owner in &ids {
            for a in list_attachments(&self.root, owner) {
                if !crate::links::is_markdown_name(&a.name) {
                    continue;
                }
                let Ok(text) = std::fs::read_to_string(&a.path) else {
                    continue;
                };
                let (text, n) = crate::links::rewrite(&text, |dest| {
                    let link =
                        crate::links::parse(dest, crate::links::From::Attachment(owner), |s| {
                            ids.contains(s)
                        });
                    let crate::links::Link::Attachment { id: i, name: n, .. } = link else {
                        return None;
                    };
                    if i != id || n != old {
                        return None;
                    }
                    let frag = dest
                        .split_once('#')
                        .map_or(String::new(), |(_, f)| format!("#{f}"));
                    let bare = crate::links::percent_encode_path(&name);
                    Some(if dest.split('#').next().is_some_and(|d| d.contains('/')) {
                        format!("{id}/{bare}{frag}")
                    } else {
                        format!("{bare}{frag}")
                    })
                });
                if n > 0 {
                    std::fs::write(&a.path, text)?;
                    rewritten += 1;
                }
            }
        }
        Ok(R::Renamed { name, rewritten })
    }

    /// Move `id` to `dest`, recording a `moved: <from> -> <to>` entry attributed
    /// to `actor` (see [`store::move_task`]). Refuses a move that would break
    /// the family invariant ([`Farm::check_transition`]). Shaving a yak also
    /// shaves its hairy ancestors ([`Farm::shave_lifts`]).
    pub fn transition(&self, id: &str, dest: Status, actor: Option<&str>) -> Result<MoveOutcome> {
        Ok(self.transition_lifting(id, dest, actor)?.0)
    }

    /// [`Farm::transition`], also returning the hairy ancestors a shave moved
    /// to shaving with it, outermost first (yaks-cfa6): a shaving yak's
    /// parent is never hairy, since starting part of the work starts the
    /// whole. Each ancestor's entry carries `child: <id>`, the yak it moved
    /// for. Empty for every other move.
    pub fn transition_lifting(
        &self,
        id: &str,
        dest: Status,
        actor: Option<&str>,
    ) -> Result<(MoveOutcome, Vec<String>)> {
        let _lock = store::lock(&self.root)?;
        self.check_transition(id, dest)?;
        let lifted = if dest == Status::Shaving {
            self.shave_lifts(id)?
        } else {
            Vec::new()
        };
        let outcome = store::move_task(&self.root, id, dest, actor)?;
        let mut moved = Vec::new();
        if outcome == MoveOutcome::Moved {
            for a in &lifted {
                let extra = [("child", id)];
                if store::move_task_with(&self.root, a, Status::Shaving, actor, &extra)?
                    == MoveOutcome::Moved
                {
                    moved.push(a.clone());
                }
            }
        }
        Ok((outcome, moved))
    }

    /// Regrow every shorn yak with `every:` whose next occurrence has arrived
    /// by `now` (see [`crate::recur::plan`]): back to hairy, `due:` set to the
    /// next deadline, `regrown:` to `now`, and a `kind: regrow` note saying
    /// which occurrence it is and how many were skipped. Idempotent: the new
    /// `due:` is in the future, so a second run finds nothing. Dead yaks never
    /// regrow.
    ///
    /// A yak under a shorn ancestor reopens that ancestor first (the family
    /// rule: a hairy yak never sits under a finished one), each with a
    /// `child:` on its move note like `shave`'s lifts; one under a dead
    /// ancestor is held and reported, since reviving is not ours to decide.
    ///
    /// The scan is read-only; each yak is then re-planned and written under
    /// its own farm lock, so it is one per-change commit per yak and a yak
    /// someone shaved meanwhile is left alone. `dry_run` writes nothing.
    pub fn regrow_due(
        &self,
        now: chrono::DateTime<chrono::Utc>,
        dry_run: bool,
        actor: Option<&str>,
    ) -> Result<crate::recur::Report> {
        use crate::recur::{self, Regrew, Verdict};
        let mut report = recur::Report::default();
        let mut candidates: Vec<String> = store::load(&self.root, &[Status::Shorn])?
            .into_iter()
            .filter(|t| t.extra.contains(recur::EVERY))
            .map(|t| t.id)
            .collect();
        candidates.sort();
        for id in candidates {
            let _lock = if dry_run {
                None
            } else {
                Some(store::lock(&self.root)?)
            };
            let all = store::load(&self.root, &EVERY)?;
            let Some(task) = all.iter().find(|t| t.id == id) else {
                continue;
            };
            let plan = match recur::plan(task, now) {
                Verdict::Regrow(p) => p,
                Verdict::Invalid(why) => {
                    report.held.push((id, why));
                    continue;
                }
                Verdict::NotRecurring | Verdict::Waiting => continue,
            };
            // Ancestors, nearest first, to the root (cycle-safe).
            let mut ancestors: Vec<&Task> = Vec::new();
            let mut cur = task.parent.as_deref();
            while let Some(p) = cur.and_then(|p| all.iter().find(|t| t.id == p)) {
                if p.id == id || ancestors.iter().any(|a| a.id == p.id) {
                    break;
                }
                ancestors.push(p);
                cur = p.parent.as_deref();
            }
            if let Some(dead) = ancestors.iter().find(|a| a.status == Status::Dead) {
                report.held.push((
                    id,
                    format!(
                        "its ancestor {} is dead: revive it (`yaks revive {}`) or reparent the yak",
                        dead.id, dead.id
                    ),
                ));
                continue;
            }
            let mut reopened: Vec<String> = ancestors
                .iter()
                .filter(|a| a.status == Status::Shorn)
                .map(|a| a.id.clone())
                .collect();
            reopened.reverse();
            // `wake:` sends the regrown yak to the inbox, unless an earlier
            // round's question or request is still standing: that one is kept.
            let awaiting = plan.wake.map(|who| recur::Awaiting {
                who: task.needs.clone().unwrap_or_else(|| who.to_string()),
                already: task.needs.is_some(),
            });
            if !dry_run {
                for a in &reopened {
                    store::move_task_with(&self.root, a, Status::Hairy, actor, &[("child", &id)])?;
                }
                store::move_task(&self.root, &id, Status::Hairy, actor)?;
                let Some(mut t) = store::load_task_by_id(&self.root, &id)? else {
                    continue;
                };
                let stamp = now.format("%Y-%m-%dT%H:%M:%SZ").to_string();
                t.extra.set(recur::DUE, &Value::Plain(plan.due.render()));
                t.extra.set(recur::REGROWN, &Value::Plain(stamp.clone()));
                // Stamped when written, so the note follows the move note.
                let written = store::now_iso();
                t.body = store::append_note_meta(
                    &t.body,
                    &written,
                    actor,
                    &[
                        ("kind", "regrow"),
                        ("occurrence", &plan.occurrence.render()),
                    ],
                    &recur::regrow_note(&plan),
                );
                if let Some(a) = awaiting.as_ref().filter(|a| !a.already) {
                    // Its own note after the regrow note: the marker `inbox`
                    // and `doctor` read, the same one `ask`/`request` write.
                    let (kind, text) = if a.who == NEEDS_AGENT {
                        (
                            "request",
                            store::request_text(Some(&recur::wake_note(&plan))),
                        )
                    } else {
                        (
                            "ask",
                            store::ask_text(&a.who, Some(&recur::wake_note(&plan))),
                        )
                    };
                    let mut meta = vec![("kind", kind)];
                    if kind == "ask" {
                        meta.push(("needs", a.who.as_str()));
                    }
                    t.needs = Some(a.who.clone());
                    t.body =
                        store::append_note_meta(&t.body, &store::now_iso(), actor, &meta, &text);
                }
                t.updated = Some(store::now_iso());
                store::write::save(&self.root, &t)?;
            }
            report.regrew.push(Regrew {
                id: id.clone(),
                title: task.title.clone(),
                plan,
                reopened,
                awaiting,
            });
        }
        Ok(report)
    }

    /// The hairy ancestors shaving `id` takes along, outermost first: its
    /// parent if hairy, that one's parent if hairy, and so on up to the first
    /// ancestor that isn't (yaks-cfa6). Cycle-safe.
    pub fn shave_lifts(&self, id: &str) -> Result<Vec<String>> {
        let all = store::load(&self.root, &EVERY)?;
        let by_id: std::collections::HashMap<&str, &Task> =
            all.iter().map(|t| (t.id.as_str(), t)).collect();
        let mut out = Vec::new();
        let mut cur = by_id.get(id).and_then(|t| t.parent.as_deref());
        while let Some(p) = cur.and_then(|p| by_id.get(p)) {
            if p.status != Status::Hairy || out.contains(&p.id) || p.id == id {
                break;
            }
            out.push(p.id.clone());
            cur = p.parent.as_deref();
        }
        out.reverse();
        Ok(out)
    }

    /// Whether moving `id` to `dest` keeps the family invariant (a shorn or
    /// dead yak has no hairy or shaving children, yaks-9b41), without moving
    /// it. `Err` is the refusal [`Farm::transition`] would return: a finishing
    /// move over open children ([`UnfinishedChildren`]), or an opening move
    /// under a finished parent ([`FinishedParent`]). A move that is a no-op
    /// (already there) is never refused. Read-only and lock-free, so a caller
    /// can check before it writes a note that should only land with the move.
    pub fn check_transition(&self, id: &str, dest: Status) -> Result<()> {
        let all = store::load(&self.root, &EVERY)?;
        let Some(me) = all.iter().find(|t| t.id == id) else {
            return Ok(());
        };
        if me.status == dest {
            return Ok(());
        }
        if dest.is_open() {
            let parent = me
                .parent
                .as_deref()
                .and_then(|p| all.iter().find(|t| t.id == p));
            if let Some(p) = parent.filter(|p| !p.status.is_open()) {
                return Err(FinishedParent(finished_parent_refusal(
                    &p.id,
                    p.status,
                    &format!(
                        "move {id} out of the family first (`yaks reparent {id} --parent \
                         <other>` or `--unparent`)"
                    ),
                ))
                .into());
            }
        } else {
            let children: Vec<(String, Status)> = all
                .iter()
                .filter(|t| t.parent.as_deref() == Some(id) && t.status.is_open())
                .map(|t| (t.id.clone(), t.status))
                .collect();
            if !children.is_empty() {
                return Err(UnfinishedChildren {
                    id: id.to_string(),
                    dest,
                    children,
                }
                .into());
            }
        }
        Ok(())
    }

    /// `ids` in the order a batch move to `dest` takes them, so
    /// [`Farm::transition`]'s family checks pass whatever order they were
    /// given in: a finishing move (shorn, dead) takes children before their
    /// parents (deepest first), an opening one (hairy, shaving) parents before
    /// their children; otherwise the given order. `yaks shorn <parent>
    /// <child>`, `yaks regrow <child> <parent>` and the TUI's bulk moves walk
    /// this order (yaks-62f2, yaks-35a4). Unknown ids count as top-level.
    pub fn batch_order(&self, ids: &[String], dest: Status) -> Result<Vec<String>> {
        let all = store::load(&self.root, &EVERY)?;
        let parent: std::collections::HashMap<&str, &str> = all
            .iter()
            .filter_map(|t| Some((t.id.as_str(), t.parent.as_deref()?)))
            .collect();
        let depth = |id: &str| {
            let (mut d, mut cur, mut seen) = (0usize, id, std::collections::HashSet::new());
            while let Some(&p) = parent.get(cur) {
                if !seen.insert(p) {
                    break; // a parent cycle: doctor's problem, not ours
                }
                d += 1;
                cur = p;
            }
            d
        };
        let mut out = ids.to_vec();
        if dest.is_open() {
            out.sort_by_key(|id| depth(id));
        } else {
            out.sort_by_key(|id| std::cmp::Reverse(depth(id)));
        }
        Ok(out)
    }

    /// Slaughter `id` (move to dead). Refuses with
    /// [`SlaughterOutcome::HasOpenDescendants`] when that would leave open
    /// descendants under a dead yak, unless `family` is set, in which case
    /// every open descendant (deepest first), then `id`, goes (yaks-05da);
    /// shorn descendants stay shorn (yaks-27c1). Each
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
        let doomed = open_descendants(&all, id);
        if !doomed.is_empty() && !family {
            return Ok(SlaughterOutcome::HasOpenDescendants(doomed));
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

    /// Move `id` under `new_parent` (or to the top level). Refuses to put an
    /// open (hairy or shaving) yak under a shorn or dead one (yaks-35a4); the
    /// other guards are [`store::reparent`]'s.
    pub fn reparent(&self, id: &str, new_parent: Option<String>) -> Result<Reparent> {
        let _lock = store::lock(&self.root)?;
        if let (Some(np), Some(me)) = (&new_parent, store::load_task_by_id(&self.root, id)?) {
            if let Some(p) = store::load_task_by_id(&self.root, np)? {
                if me.status.is_open() && !p.status.is_open() {
                    return Ok(Reparent::Error(finished_parent_refusal(
                        np,
                        p.status,
                        &format!("reparent {id} under an open yak (or `--unparent`)"),
                    )));
                }
            }
        }
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

/// `git log <args>` in `dir` as commits (full sha, short sha, subject).
pub(crate) fn git_commits(dir: &Path, args: &[&str]) -> Result<Vec<CommitRef>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["log", COMMIT_FORMAT])
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
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
        .filter_map(parse_commit_line)
        .collect())
}

const COMMIT_FORMAT: &str = "--format=%H%x09%h%x09%s";

fn parse_commit_line(line: &str) -> Option<CommitRef> {
    let mut f = line.splitn(3, '\t');
    Some(CommitRef {
        sha: f.next()?.to_string(),
        short: f.next()?.to_string(),
        subject: f.next().unwrap_or("").to_string(),
    })
}

/// `sha` as the repository at `top` has it, or `None` when it is not a hex
/// sha or names no commit there.
pub(crate) fn git_commit_ref(top: &Path, sha: &str) -> Option<CommitRef> {
    if sha.len() < 4 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(top)
        .args(["log", "-1", COMMIT_FORMAT, &format!("{sha}^{{commit}}")])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    parse_commit_line(String::from_utf8_lossy(&out.stdout).lines().next()?)
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

/// What `needs:` the latest kinded ask/answer/request/pickup note of `t`
/// leaves, when that disagrees with `t.needs`. `ask` and `answer` record the
/// state they set in a `needs` field (`ask --needs <who>`; `answer` leaves
/// `agent`, or `none` when done or when the yak had none), defaulting to
/// `human` and `agent`; `request` leaves `agent`, `pickup` none.
///
/// Two commands change `needs:` without writing a note, so the check allows
/// what they can leave behind instead of crying wolf: a note-less `answer`
/// (or `--done`) after an `ask` or `answer` note (`agent` or none), and
/// `pickup --drop-request`, which removes the request note and so leaves an
/// earlier `answer` note over a cleared `needs:`.
fn needs_drift(t: &Task) -> Option<String> {
    let notes = store::parse_notes(&t.body);
    let note = notes
        .iter()
        .rfind(|n| matches!(n.kind(), Some("ask" | "answer" | "request" | "pickup")))?;
    let kind = note.kind()?;
    let expected = match (kind, note.get("needs")) {
        ("ask", got) => Some(got.unwrap_or("human")),
        ("answer", got) => Some(got.unwrap_or(NEEDS_AGENT)),
        ("request", _) => Some(NEEDS_AGENT),
        _ => None,
    }
    .filter(|n| *n != store::NEEDS_NONE);
    let have = t.needs.as_deref();
    let note_less = match kind {
        "ask" => matches!(have, None | Some(NEEDS_AGENT)),
        "answer" => have.is_none(),
        _ => false,
    };
    if have == expected || note_less {
        return None;
    }
    let show = |n: Option<&str>| n.map_or("no needs".to_string(), |n| format!("needs: {n}"));
    Some(format!(
        "{} has {} but its latest {kind} note ({}) leaves {}",
        t.id,
        show(have),
        note.ts,
        show(expected)
    ))
}

/// What a [`crate::conflicts::Conflict`] holds, in words.
pub fn conflict_summary(c: &crate::conflicts::Conflict) -> String {
    let mut parts = Vec::new();
    if !c.marker_lines.is_empty() {
        let at: Vec<String> = c.marker_lines.iter().map(|l| format!("L{l}")).collect();
        parts.push(format!("conflict markers at {}", at.join(", ")));
    }
    if !c.fields.is_empty() {
        parts.push(format!(
            "merge_conflicts: {} (the other side's value is recorded there; remove the entry once settled)",
            c.fields.join(", ")
        ));
    }
    parts.join("; ")
}

fn issue_rank(k: IssueKind) -> u8 {
    match k {
        IssueKind::InvalidStatus => 0,
        IssueKind::DuplicateId => 1,
        IssueKind::DanglingParent => 2,
        IssueKind::DanglingDependsOn => 3,
        IssueKind::MissingEvidence => 4,
        IssueKind::UnverifiedShear => 5,
        IssueKind::MalformedTag => 6,
        IssueKind::MisplacedFile => 7,
        IssueKind::NeedsDrift => 8,
        IssueKind::UnfinishedChildren => 9,
        IssueKind::DokProblem => 10,
        IssueKind::BrokenLink => 11,
        IssueKind::SchemaMismatch => 12,
        IssueKind::MergeConflict => 13,
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
        assert!(Farm::at(&parent.join(".yaks")).is_ok());
        std::fs::create_dir_all(root.join("hairy")).unwrap();
        std::fs::write(
            root.join("hairy/yak-0001.md"),
            store::write::render(&task("yak-0001", Status::Hairy)),
        )
        .unwrap();
        match Farm::at(&parent.join(".yaks")) {
            Err(OpenError::OldLayout(why)) => {
                assert!(why.contains("npx -y @j15r/yaks@0.0.16 migrate"), "{why}")
            }
            _ => panic!("expected an old-layout refusal"),
        }
        std::fs::remove_dir_all(root.join("hairy")).unwrap();
        std::fs::write(root.join("schema"), "3\n").unwrap();
        assert!(matches!(
            Farm::at(&parent.join(".yaks")),
            Err(OpenError::OldLayout(_))
        ));
    }

    /// A well-formed farm (resolvable parent + dep, one status per id) is clean.
    fn temp_farm() -> (PathBuf, Farm) {
        let mut parent = std::env::temp_dir();
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        parent.push(format!("yaksrs-farm-refs-{}-{}", std::process::id(), n));
        let root = parent.join(".yaks");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("schema"), format!("{}\n", store::SCHEMA)).unwrap();
        let farm = match Farm::at(&parent.join(".yaks")) {
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
            farm.pickup("yak-0001", Some("agt"), Some("on it"), false)
                .unwrap(),
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

    fn notes_of(farm: &Farm, id: &str) -> Vec<store::NoteEntry> {
        let t = store::load_task_by_id(&farm.root, id).unwrap().unwrap();
        store::parse_notes(&t.body)
    }

    /// Every command that writes a conventional note stamps its `kind` (and
    /// the fields that go with it), and keeps the body text readers match on.
    #[test]
    fn commands_write_kind_metadata_and_keep_the_body_text() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        farm.ask("yak-0001", "human", Some("a"), Some("which?"))
            .unwrap();
        farm.answer("yak-0001", Some("j"), Some("B"), false)
            .unwrap();
        farm.pickup("yak-0001", Some("a"), None, false).unwrap();
        farm.request("yak-0001", Some("j"), Some("tidy")).unwrap();
        farm.transition("yak-0001", Status::Shaving, Some("a"))
            .unwrap();
        let notes = notes_of(&farm, "yak-0001");
        let got: Vec<(Option<&str>, &str)> =
            notes.iter().map(|n| (n.kind(), n.text.as_str())).collect();
        assert_eq!(
            got,
            vec![
                (Some("ask"), "asked: needs human\n\nwhich?"),
                (Some("answer"), "B"),
                (Some("pickup"), "picked up"),
                (Some("request"), "requested: needs agent\n\ntidy"),
                (Some("move"), "moved: hairy -> shaving"),
            ]
        );
        assert_eq!(notes[0].get("needs"), Some("human"));
        assert_eq!(notes[1].get("needs"), Some("agent"));
        let mv = &notes[4];
        assert_eq!(
            (mv.get("from"), mv.get("to")),
            (Some("hairy"), Some("shaving"))
        );
    }

    /// `update` appends the given metadata under the note; without a note it
    /// has nothing to attach to and writes none.
    #[test]
    fn update_writes_the_edit_meta_under_the_note() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let meta = vec![
            ("kind".to_string(), "checkpoint".to_string()),
            ("commit".to_string(), "47076b0".to_string()),
        ];
        farm.update(
            "yak-0001",
            TaskEdit {
                note: Some("saved".into()),
                meta: meta.clone(),
                ..Default::default()
            },
        )
        .unwrap();
        farm.update(
            "yak-0001",
            TaskEdit {
                title: Some("t".into()),
                meta,
                ..Default::default()
            },
        )
        .unwrap();
        let notes = notes_of(&farm, "yak-0001");
        assert_eq!(notes.len(), 1);
        assert_eq!(
            (notes[0].kind(), notes[0].get("commit")),
            (Some("checkpoint"), Some("47076b0"))
        );
        assert_eq!(notes[0].text, "saved");
    }

    fn drift(farm: &Farm, id: &str) -> Vec<Issue> {
        farm.doctor(false)
            .unwrap()
            .into_iter()
            .filter(|i| i.kind == IssueKind::NeedsDrift && i.ids[0] == id)
            .collect()
    }

    /// The commands keep `needs:` and the notes in step, so doctor is quiet
    /// through a whole ask -> answer -> pickup loop, and for a yak whose notes
    /// carry no `kind` at all (older notes, hand-set `needs:`).
    #[test]
    fn doctor_needs_drift_is_quiet_when_commands_wrote_both() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        assert!(drift(&farm, "yak-0001").is_empty());
        farm.ask("yak-0001", "human", Some("a"), Some("q")).unwrap();
        assert!(drift(&farm, "yak-0001").is_empty());
        farm.answer("yak-0001", Some("j"), Some("r"), false)
            .unwrap();
        assert!(drift(&farm, "yak-0001").is_empty());
        farm.pickup("yak-0001", Some("a"), None, false).unwrap();
        assert!(drift(&farm, "yak-0001").is_empty());
        farm.request("yak-0001", Some("j"), None).unwrap();
        assert!(drift(&farm, "yak-0001").is_empty());
        farm.answer("yak-0001", Some("j"), Some("fyi"), true)
            .unwrap();
        assert!(
            drift(&farm, "yak-0001").is_empty(),
            "{:?}",
            farm.doctor(false).unwrap()
        );

        // The two commands that change `needs:` with no note stay quiet: a
        // note-less answer, and `pickup --drop-request` (which removes the
        // request note, leaving the earlier answer over a cleared `needs:`).
        store::write::save(&root, &task("yak-0003", Status::Hairy)).unwrap();
        farm.ask("yak-0003", "human", Some("a"), Some("q")).unwrap();
        farm.answer("yak-0003", Some("j"), None, false).unwrap();
        assert!(drift(&farm, "yak-0003").is_empty());
        farm.answer("yak-0003", Some("j"), None, true).unwrap();
        assert!(drift(&farm, "yak-0003").is_empty());
        store::write::save(&root, &task("yak-0004", Status::Hairy)).unwrap();
        farm.ask("yak-0004", "human", Some("a"), Some("q")).unwrap();
        farm.answer("yak-0004", Some("j"), Some("r"), false)
            .unwrap();
        farm.request("yak-0004", Some("j"), Some("tidy")).unwrap();
        farm.pickup("yak-0004", Some("a"), None, true).unwrap();
        assert_eq!(needs_of(&farm, "yak-0004"), None);
        assert!(drift(&farm, "yak-0004").is_empty());

        // No kinded notes: whatever `needs:` says, never flagged.
        let mut old = task("yak-0002", Status::Hairy);
        old.needs = Some("human".into());
        old.body = store::append_note(
            "",
            "2026-01-01T00:00:00Z",
            Some("a"),
            "asked: needs agent\n\nq",
        );
        store::write::save(&root, &old).unwrap();
        assert!(drift(&farm, "yak-0002").is_empty());
    }

    /// A hand edit of `needs:` that no longer matches the latest kinded note
    /// is reported, naming both sides.
    #[test]
    fn doctor_flags_needs_that_disagrees_with_the_latest_kinded_note() {
        let (root, farm) = temp_farm();
        let note = |ts: &str, kind: &str, text: &str| {
            store::append_note_meta("", ts, Some("a"), &[("kind", kind)], text)
        };
        let cases = [
            ("ask", Some("other"), "asked: needs human"),
            ("request", None, "requested: needs agent"),
            ("request", Some("human"), "requested: needs agent"),
            ("answer", Some("human"), "r"),
            ("pickup", Some("agent"), "picked up"),
        ];
        for (i, (kind, needs, text)) in cases.iter().enumerate() {
            let mut t = task(&format!("yak-{i:04}"), Status::Hairy);
            t.needs = needs.map(str::to_string);
            t.body = note("2026-01-01T00:00:00Z", kind, text);
            store::write::save(&root, &t).unwrap();
            let got = drift(&farm, &t.id);
            assert_eq!(got.len(), 1, "{kind} with {needs:?}");
            assert!(got[0].message.contains(kind), "{}", got[0].message);
        }
        // The latest kinded note decides, not an earlier one or a plain note.
        let mut t = task("yak-9999", Status::Hairy);
        t.body = note("2026-01-01T00:00:00Z", "ask", "asked: needs human");
        t.body = store::append_note_meta(
            &t.body,
            "2026-01-02T00:00:00Z",
            None,
            &[("kind", "pickup")],
            "picked up",
        );
        t.body = store::append_note(&t.body, "2026-01-03T00:00:00Z", None, "plain");
        store::write::save(&root, &t).unwrap();
        assert!(drift(&farm, "yak-9999").is_empty());
    }

    /// `pickup` refuses anything that is not `needs: agent`, naming the state;
    /// `ask` on `needs: agent` is allowed and flips it back to human.
    #[test]
    fn pickup_drop_request_removes_the_request_note_and_never_an_answer() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        farm.ask("yak-0001", "human", Some("agt"), Some("which?"))
            .unwrap();
        farm.answer("yak-0001", Some("joel"), Some("that one"), false)
            .unwrap();
        // An answer is never dropped: refused, nothing written.
        assert!(matches!(
            farm.pickup("yak-0001", None, None, true).unwrap(),
            Pickup::NotARequest
        ));
        assert_eq!(needs_of(&farm, "yak-0001").as_deref(), Some("agent"));
        farm.pickup("yak-0001", Some("agt"), None, false).unwrap();
        let before = note_texts(&farm, "yak-0001");

        farm.request("yak-0001", Some("joel"), Some("tidy it"))
            .unwrap();
        farm.update(
            "yak-0001",
            TaskEdit {
                note: Some("a later note".into()),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(matches!(
            farm.pickup("yak-0001", Some("agt"), None, true).unwrap(),
            Pickup::Picked
        ));
        assert_eq!(needs_of(&farm, "yak-0001"), None);
        let mut want = before;
        want.push("a later note".into());
        assert_eq!(
            note_texts(&farm, "yak-0001"),
            want,
            "only the request note went; no picked-up note was added"
        );
    }

    #[test]
    fn pickup_refuses_other_states_and_ask_flips_agent_back() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        assert!(matches!(
            farm.pickup("yak-0001", None, None, false).unwrap(),
            Pickup::NotAwaitingAgent(None)
        ));
        farm.ask("yak-0001", "human", Some("coord"), None).unwrap();
        assert!(matches!(
            farm.pickup("yak-0001", None, None, false).unwrap(),
            Pickup::NotAwaitingAgent(Some(n)) if n == "human"
        ));
        assert_eq!(
            needs_of(&farm, "yak-0001").as_deref(),
            Some("human"),
            "refusal changed nothing"
        );
        assert!(matches!(
            farm.pickup("nope", None, None, false).unwrap(),
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
            farm.pickup("yak-0001", Some("agt"), None, false).unwrap(),
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
        farm.pickup("yak-0001", Some("a"), None, false).unwrap();
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
            tags: vec![],
            depends_on: vec![],
            source: None,
            key: None,
            prs: vec![],
            description: None,
            verify: None,
            fields: vec![],
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
        // Regression (yaks-b47b): with no `--herd`, no repo herd and no
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

    /// The repo's herd (`git config yaks.herd`, through discovery) routes a
    /// new yak below an explicit `--herd`.
    #[test]
    fn create_routes_via_repo_herd_and_explicit_wins() {
        let (farm_root, _f) = temp_farm();
        let farm = Farm::open_root(farm_root.clone(), Some("web".into()), false)
            .unwrap_or_else(|_| panic!("open"));
        let a = created_id(farm.create(new_task("via repo herd", None)).unwrap());
        assert!(a.starts_with("web-"), "expected web- id, got {a}");
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

    /// Slaughtering a parent with open descendants is refused (listing them)
    /// unless `family` is set, which takes the open ones, deepest first;
    /// already-dead descendants are left alone (yaks-05da), and a shorn one
    /// neither blocks it nor dies: finished work stays shorn (yaks-27c1).
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
            SlaughterOutcome::HasOpenDescendants(vec!["yak-0002".into()])
        );
        assert!(at(&root, "yak-0001", Status::Hairy), "guard must not move");

        assert_eq!(
            farm.slaughter("yak-0001", true, None).unwrap(),
            SlaughterOutcome::Slaughtered(vec!["yak-0002".into(), "yak-0001".into()])
        );
        for id in ["yak-0001", "yak-0002", "yak-0004"] {
            assert!(at(&root, id, Status::Dead), "{id} dead");
        }
        assert!(
            at(&root, "yak-0003", Status::Shorn),
            "shorn work stays shorn"
        );
        assert!(farm.doctor(false).unwrap().is_empty());
        // A shorn child alone never blocks a slaughter.
        store::write::save(&root, &task("yak-0006", Status::Hairy)).unwrap();
        store::write::save(&root, &kid("yak-0007", "yak-0006", Status::Shorn)).unwrap();
        assert_eq!(
            farm.slaughter("yak-0006", false, None).unwrap(),
            SlaughterOutcome::Slaughtered(vec!["yak-0006".into()])
        );
        // A plain move to dead (the TUI's single slaughter) refuses open
        // children like a shear, with the slaughter-specific fix.
        store::write::save(&root, &task("yak-0008", Status::Hairy)).unwrap();
        store::write::save(&root, &kid("yak-0009", "yak-0008", Status::Hairy)).unwrap();
        let err = farm.transition("yak-0008", Status::Dead, None).unwrap_err();
        assert!(err.to_string().contains("--family"), "{err}");
        // doctor flags a dead parent over an open child from a hand edit.
        store::write::save(&root, &task("yak-0008", Status::Dead)).unwrap();
        assert!(
            farm.doctor(false)
                .unwrap()
                .iter()
                .any(|i| i.kind == IssueKind::UnfinishedChildren && i.ids[0] == "yak-0008")
        );
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

    /// A shorn yak has no hairy or shaving children (yaks-62f2): shearing the
    /// parent first refuses and names the open child; dead and shorn children
    /// don't count; `batch_order` orders a batch so it shears whole; and
    /// doctor flags the state when a hand edit produces it anyway.
    #[test]
    fn shorn_refuses_a_parent_over_unfinished_children() {
        let (root, farm) = temp_farm();
        let child = |id: &str, s: Status| {
            let mut t = task(id, s);
            t.parent = Some("yak-0001".into());
            t
        };
        store::write::save(&root, &task("yak-0001", Status::Shaving)).unwrap();
        store::write::save(&root, &child("yak-0002", Status::Hairy)).unwrap();
        store::write::save(&root, &child("yak-0003", Status::Dead)).unwrap();
        let mut grand = task("yak-0004", Status::Shaving);
        grand.parent = Some("yak-0002".into());
        store::write::save(&root, &grand).unwrap();

        let err = farm
            .transition("yak-0001", Status::Shorn, None)
            .unwrap_err();
        let refused = err
            .downcast_ref::<UnfinishedChildren>()
            .expect("typed refusal");
        assert_eq!(refused.children, vec![("yak-0002".into(), Status::Hairy)]);
        assert!(err.to_string().contains("yaks reparent"), "{err}");
        assert!(
            at(&root, "yak-0001", Status::Shaving),
            "refusal must not move it"
        );
        // Other moves of the parent are unaffected.
        assert_eq!(
            farm.transition("yak-0001", Status::Hairy, None).unwrap(),
            MoveOutcome::Moved
        );

        // Parent given first, grandchild last: still shears whole.
        let ids: Vec<String> = ["yak-0001", "yak-0002", "yak-0004"]
            .map(String::from)
            .into();
        let order = farm.batch_order(&ids, Status::Shorn).unwrap();
        assert_eq!(order, vec!["yak-0004", "yak-0002", "yak-0001"]);
        for id in &order {
            assert_eq!(
                farm.transition(id, Status::Shorn, None).unwrap(),
                MoveOutcome::Moved
            );
        }
        assert!(farm.doctor(false).unwrap().is_empty());

        // A hand edit back to hairy under the shorn parent: doctor flags it.
        store::write::save(&root, &child("yak-0002", Status::Hairy)).unwrap();
        let issues = farm.doctor(false).unwrap();
        assert!(
            issues
                .iter()
                .any(|i| i.kind == IssueKind::UnfinishedChildren
                    && i.ids == vec!["yak-0001".to_string(), "yak-0002".to_string()]),
            "{issues:?}"
        );
    }

    /// Shaving a yak shaves its hairy ancestors too, outermost first, up to
    /// the first one that isn't hairy; each lifted entry names the child it
    /// moved for. Other moves lift nothing (yaks-cfa6).
    #[test]
    fn shave_lifts_hairy_ancestors() {
        let (root, farm) = temp_farm();
        let kid = |id: &str, parent: &str, st: Status| {
            let mut t = task(id, st);
            t.parent = Some(parent.into());
            t
        };
        store::write::save(&root, &task("yak-0001", Status::Shaving)).unwrap();
        store::write::save(&root, &kid("yak-0002", "yak-0001", Status::Hairy)).unwrap();
        store::write::save(&root, &kid("yak-0003", "yak-0002", Status::Hairy)).unwrap();
        store::write::save(&root, &kid("yak-0004", "yak-0003", Status::Hairy)).unwrap();

        let (out, lifted) = farm
            .transition_lifting("yak-0004", Status::Shaving, Some("w"))
            .unwrap();
        assert_eq!(out, MoveOutcome::Moved);
        assert_eq!(
            lifted,
            vec!["yak-0002", "yak-0003"],
            "outermost first, stops at 0001"
        );
        for id in ["yak-0002", "yak-0003", "yak-0004"] {
            assert!(at(&root, id, Status::Shaving), "{id}");
        }
        let body = store::load_task_by_id(&root, "yak-0002")
            .unwrap()
            .unwrap()
            .body;
        assert!(body.contains("child: yak-0004"), "{body}");
        assert!(body.contains("[w]"), "{body}");

        // Already shaving: a no-op lifts nothing; a regrow never lifts.
        assert_eq!(
            farm.transition_lifting("yak-0004", Status::Shaving, None)
                .unwrap(),
            (MoveOutcome::AlreadyThere, vec![])
        );
        farm.transition("yak-0004", Status::Hairy, None).unwrap();
        farm.transition("yak-0003", Status::Hairy, None).unwrap();
        assert!(at(&root, "yak-0002", Status::Shaving));
    }

    /// Nothing opens under a finished parent (yaks-35a4): create, reparent
    /// and the opening moves refuse, naming the parent and both fixes; a
    /// finished yak still reparents freely, a no-op move is never refused, and
    /// `batch_order` takes parents first for an opening move.
    #[test]
    fn nothing_opens_under_a_finished_parent() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Shorn)).unwrap();
        store::write::save(&root, &task("yak-0002", Status::Dead)).unwrap();
        let mut kid = task("yak-0003", Status::Shorn);
        kid.parent = Some("yak-0001".into());
        store::write::save(&root, &kid).unwrap();
        store::write::save(&root, &task("yak-0004", Status::Hairy)).unwrap();

        // create --parent <shorn|dead>
        let mut n = new_task("follow-up", Some("yak"));
        n.parent = Some("yak-0001".into());
        match farm.create(n).unwrap() {
            CreateOutcome::ParentFinished(m) => {
                assert!(
                    m.contains("yak-0001 is shorn") && m.contains("yaks regrow yak-0001"),
                    "{m}"
                )
            }
            _ => panic!("expected ParentFinished"),
        }
        let mut n = new_task("follow-up", Some("yak"));
        n.parent = Some("yak-0002".into());
        match farm.create(n).unwrap() {
            CreateOutcome::ParentFinished(m) => assert!(m.contains("yaks revive yak-0002"), "{m}"),
            _ => panic!("expected ParentFinished"),
        }

        // reparent an open yak under a finished one: refused; a finished yak: fine.
        match farm.reparent("yak-0004", Some("yak-0001".into())).unwrap() {
            Reparent::Error(m) => assert!(m.contains("--unparent"), "{m}"),
            other => panic!("expected a refusal, got {other:?}"),
        }
        assert!(matches!(
            farm.reparent("yak-0003", Some("yak-0002".into())).unwrap(),
            Reparent::Done { .. }
        ));
        farm.reparent("yak-0003", Some("yak-0001".into())).unwrap();

        // regrow / shave a child of a shorn yak: refused, nothing moves.
        for dest in [Status::Hairy, Status::Shaving] {
            let err = farm.transition("yak-0003", dest, None).unwrap_err();
            assert!(err.downcast_ref::<FinishedParent>().is_some(), "{err}");
            assert!(err.to_string().contains("yaks reparent yak-0003"), "{err}");
        }
        assert!(at(&root, "yak-0003", Status::Shorn));
        // A no-op move (already there) is never refused.
        assert_eq!(
            farm.transition("yak-0003", Status::Shorn, None).unwrap(),
            MoveOutcome::AlreadyThere
        );

        // Child given first: an opening batch still takes the parent first.
        let ids: Vec<String> = ["yak-0003", "yak-0001"].map(String::from).into();
        let order = farm.batch_order(&ids, Status::Hairy).unwrap();
        assert_eq!(order, vec!["yak-0001", "yak-0003"]);
        for id in &order {
            assert_eq!(
                farm.transition(id, Status::Hairy, None).unwrap(),
                MoveOutcome::Moved
            );
        }
        assert!(farm.doctor(false).unwrap().is_empty());
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
        // Hairy, not shorn: a shorn parent over a hairy child is itself an
        // issue (yaks-62f2).
        store::write::save(&root, &task("yak-0002", Status::Hairy)).unwrap();
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

    /// A file name with a space is linked percent-encoded, so markdown reads
    /// it as a link that resolves; renaming it follows either form (yaks-fd7a).
    #[test]
    fn attach_encodes_a_name_with_spaces_and_rename_follows_it() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let _ = farm.attach("yak-0001", "my shot.png", b"x").unwrap();
        let body = |root: &Path| {
            store::load_task_by_id(root, "yak-0001")
                .unwrap()
                .unwrap()
                .body
        };
        assert!(
            body(&root).ends_with("![my shot](yak-0001/my%20shot.png)"),
            "{}",
            body(&root)
        );
        let edges = crate::links::edges(&root).unwrap();
        assert!(
            edges
                .iter()
                .any(|e| crate::links::exists(&root, &e.link) == Some(true)),
            "the encoded link resolves"
        );
        // An old literal link, as attach wrote before, is renamed too.
        let mut t = store::load_task_by_id(&root, "yak-0001").unwrap().unwrap();
        t.body.push_str("\n\n[old](yak-0001/my shot.png)");
        store::write::save(&root, &t).unwrap();
        farm.rename_attachment("yak-0001", "my shot.png", "final shot")
            .unwrap();
        let b = body(&root);
        assert!(b.contains("![final-shot](yak-0001/final-shot.png)"), "{b}");
        assert!(b.contains("[old](yak-0001/final-shot.png)"), "{b}");
        assert!(!b.contains("my shot") && !b.contains("my%20shot"), "{b}");
    }

    /// An evidence document and the images it embeds are attached together
    /// without links; the document's bare `![x](shot.png)` names its sibling
    /// attachments, so `doctor` is clean, and renaming or promoting a part
    /// keeps the document right (yaks-be20).
    #[test]
    fn an_evidence_document_links_its_sibling_attachments_by_bare_name() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let doc =
            "# Evidence\n\n![crop](a.png) and [again](yak-0001/a.png#x) and ![gone](nope.png)\n";
        let files = vec![
            ("evidence.md".to_string(), doc.as_bytes().to_vec()),
            ("a.png".to_string(), b"x".to_vec()),
        ];
        farm.attach_many("yak-0001", &files, false, None).unwrap();
        let body = |root: &Path| {
            store::load_task_by_id(root, "yak-0001")
                .unwrap()
                .unwrap()
                .body
        };
        assert!(!body(&root).contains("a.png"), "no link added to the yak");
        let broken = |root: &Path| -> Vec<String> {
            crate::links::edges(root)
                .unwrap()
                .into_iter()
                .filter(|e| crate::links::exists(root, &e.link) == Some(false))
                .map(|e| e.dest)
                .collect()
        };
        assert_eq!(
            broken(&root),
            ["nope.png"],
            "only the missing part is broken"
        );

        // Renaming a part rewrites the document, keeping each link's form.
        farm.rename_attachment("yak-0001", "a.png", "b").unwrap();
        let text = |root: &Path| {
            std::fs::read_to_string(store::artifact_dir(root, "yak-0001").join("evidence.md"))
                .unwrap()
        };
        let t = text(&root);
        assert!(t.contains("![crop](b.png)"), "{t}");
        assert!(t.contains("[again](yak-0001/b.png#x)"), "{t}");

        // Promoting the document to a dok keeps its parts reachable.
        farm.promote("yak-0001", "evidence.md", Some("doks/ev.md"), "t")
            .unwrap();
        let dok = std::fs::read_to_string(root.join("doks/ev.md")).unwrap();
        assert!(dok.contains("![crop](yak-0001/b.png)"), "{dok}");
        assert_eq!(broken(&root), ["yak-0001/nope.png"], "{dok}");
    }

    /// With links (the default) `attach_many` writes the same body as one
    /// `attach` per file.
    #[test]
    fn attach_many_links_each_file_in_order() {
        let (root, farm) = temp_farm();
        store::write::save(&root, &task("yak-0001", Status::Hairy)).unwrap();
        let files = vec![
            ("a.png".to_string(), b"x".to_vec()),
            ("b.txt".to_string(), b"y".to_vec()),
        ];
        let out = farm.attach_many("yak-0001", &files, true, None).unwrap();
        assert_eq!(out, AttachOutcome::Attached("a.png, b.txt".into()));
        let b = store::load_task_by_id(&root, "yak-0001")
            .unwrap()
            .unwrap()
            .body;
        assert!(
            b.ends_with("![a](yak-0001/a.png)\n\n[b](yak-0001/b.txt)"),
            "{b}"
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
        std::fs::write(root.join("yak-0001/s.png"), "png").unwrap();
        // Not yaks: a loose note without frontmatter, and a stray dir.
        std::fs::write(root.join("README.md"), "farm notes\n").unwrap();
        std::fs::create_dir_all(root.join("misc")).unwrap();
        std::fs::remove_dir_all(root.join("yak")).ok();
        std::fs::remove_dir_all(root.join("web")).ok();
        std::fs::write(root.join("schema"), "4\n").unwrap();
        let before = std::fs::read(root.join("yak-0001.md")).unwrap();

        match Farm::at(&parent.join(".yaks")) {
            Err(OpenError::NeedsMigration(why)) => assert!(why.contains("yaks migrate"), "{why}"),
            _ => panic!("a schema-4 farm must refuse every command but migrate"),
        }
        let Ok(farm) = Farm::at_for_migration(&parent.join(".yaks")) else {
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
        assert_eq!(
            counts,
            [0, 1, 1, 0, 0],
            "no labels key, one old-shape comment, one non-image link, no spaced name, no wiki link"
        );
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
        assert!(Farm::at(&parent.join(".yaks")).is_ok());
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
