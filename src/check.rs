//! `yaks check` (yaks-c0ac): Hearth's nightly check minus what needs a model.
//! It reads the farm and reports what a person or an agent should look at,
//! and never edits.
//!
//! Eleven kinds of finding. Four are `doctor`'s own issues, reused as they are
//! (`dok-problem`, `broken-link`, `schema-mismatch`, `merge-conflict`); three
//! are ported from Hearth's `internal/check/scan.go`, and four are yaks's own:
//!
//! - `stale`: a dok past its `stale_after` (its own, else its schema's);
//! - `uncited`: a source nothing in the farm cites;
//! - `similar-kinds`: dok kinds that differ only by plural or hyphens;
//! - `no-description` (yaks-2755): a dok with no `description:`, the index
//!   line `dok list` shows. Only doks: session notes and schemas are not.
//! - `stray-layout` (yaks-acc0): a file under `.yaks/` of a farm that moved to the
//!   branch root at schema 9. A writer who had not synced committed it at the old
//!   path, so no yaks command sees it until it is moved.
//! - `code-moved` and `code-gone` (yaks-6cf2): a dok cites `code:<path>` and the
//!   code repo has commits to that path since the dok was written, or the path
//!   is gone. Evidence, not a verdict: an agent reads the dok against the code
//!   and sets `verified:`. Needs a code checkout; without one the checks are
//!   skipped with one note ([`Report::notes`]), which is not a finding.

use crate::farm::{Farm, IssueKind};
use crate::links::{self, Link};
use crate::locate::{Place, Space};
use crate::{doks, gitrepo, schemas, sources, store};
use anyhow::Result;
use chrono::{DateTime, Days, Months, NaiveDate, Utc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The classes of finding, in the order a report lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    DokProblem,
    BrokenLink,
    SchemaMismatch,
    MergeConflict,
    Stale,
    Uncited,
    SimilarKinds,
    NoDescription,
    StrayLayout,
    CodeMoved,
    CodeGone,
}

impl Kind {
    /// The stable machine label, the same as `doctor`'s for the kinds it owns.
    pub fn code(self) -> &'static str {
        match self {
            Kind::DokProblem => IssueKind::DokProblem.code(),
            Kind::BrokenLink => IssueKind::BrokenLink.code(),
            Kind::SchemaMismatch => IssueKind::SchemaMismatch.code(),
            Kind::MergeConflict => IssueKind::MergeConflict.code(),
            Kind::Stale => "stale",
            Kind::Uncited => "uncited",
            Kind::SimilarKinds => "similar-kinds",
            Kind::NoDescription => "no-description",
            Kind::StrayLayout => "stray-layout",
            Kind::CodeMoved => "code-moved",
            Kind::CodeGone => "code-gone",
        }
    }

    fn heading(self) -> &'static str {
        match self {
            Kind::DokProblem => IssueKind::DokProblem.heading(),
            Kind::BrokenLink => IssueKind::BrokenLink.heading(),
            Kind::SchemaMismatch => IssueKind::SchemaMismatch.heading(),
            Kind::MergeConflict => IssueKind::MergeConflict.heading(),
            Kind::Stale => {
                "Stale dok (past its stale_after: check the facts again, then set verified:)"
            }
            Kind::Uncited => {
                "Uncited source (no dok or yak mentions sources/<id>/: cite it, or keep it \
on purpose with `yaks source mark`)"
            }
            Kind::SimilarKinds => {
                "Similar kinds (dok types that differ only by plural or hyphens: pick one)"
            }
            Kind::NoDescription => {
                "Dok with no description (`description:` is the index line `yaks dok list` \
shows: one sentence on when to read it)"
            }
            Kind::StrayLayout => {
                "Files left under `.yaks/` (the farm is at the branch root since schema 9: \
`git mv` each to the same path without `.yaks/`)"
            }
            Kind::CodeMoved => {
                "Code has changed since the dok was written or last verified (commits to a \
`code:` path it cites)"
            }
            Kind::CodeGone => {
                "Cited code is gone (a `code:` path it cites is not in the code repo)"
            }
        }
    }

    fn from_issue(k: IssueKind) -> Option<Kind> {
        match k {
            IssueKind::DokProblem => Some(Kind::DokProblem),
            IssueKind::BrokenLink => Some(Kind::BrokenLink),
            IssueKind::SchemaMismatch => Some(Kind::SchemaMismatch),
            IssueKind::MergeConflict => Some(Kind::MergeConflict),
            _ => None,
        }
    }
}

/// One thing to look at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub kind: Kind,
    pub message: String,
    /// What it concerns: dok paths, source directories or kind names.
    pub ids: Vec<String>,
}

/// What `check` found, and what it could not look at.
#[derive(Debug, Default)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// Information lines: not findings, and they never change the exit code.
    pub notes: Vec<String>,
}

/// Every finding of the farm, as of now. `code:` links resolve in the code
/// checkout of the current directory (see [`links::code_base`]).
pub fn run(farm: &Farm) -> Result<Vec<Finding>> {
    Ok(report(farm)?.findings)
}

/// [`run`], and the information lines too.
pub fn report(farm: &Farm) -> Result<Report> {
    let cwd = std::env::current_dir().ok();
    let code = links::code_base(farm.root(), cwd.as_deref());
    run_at(farm, Utc::now(), code)
}

/// Every finding of the farm, as of `now`, with `code` as the code checkout
/// (`None`: there is none to check `code:` links against).
pub fn run_at(farm: &Farm, now: DateTime<Utc>, code: Option<PathBuf>) -> Result<Report> {
    let root = farm.root();
    // `doctor` already sorts by kind then id, so its issues come grouped.
    let mut out: Vec<Finding> = farm
        .doctor(false)?
        .into_iter()
        .filter_map(|i| {
            Some(Finding {
                kind: Kind::from_issue(i.kind)?,
                message: i.message,
                ids: i.ids,
            })
        })
        .collect();
    let all_doks = doks::load_all(root)?;
    let all_schemas = schemas::load_all(root);
    out.extend(stale(root, &all_doks, &all_schemas, now));
    out.extend(uncited(root)?);
    out.extend(similar_kinds(&all_doks, &all_schemas));
    out.extend(no_description(&all_doks));
    out.extend(stray_layout(root));
    let (moved, notes) = code_changes(&Space::new(root, code), &all_doks);
    out.extend(moved);
    Ok(Report {
        findings: out,
        notes,
    })
}

/// `[{kind, message, ids}]`, the shape of `doctor --json`.
pub fn to_json(found: &[Finding]) -> Value {
    Value::Array(
        found
            .iter()
            .map(|f| json!({"kind": f.kind.code(), "message": f.message, "ids": f.ids}))
            .collect(),
    )
}

/// The text report: findings grouped under a heading per kind.
pub fn render(found: &[Finding]) -> String {
    if found.is_empty() {
        return "All clear: nothing to check up on.\n".to_string();
    }
    let noun = if found.len() == 1 {
        "finding"
    } else {
        "findings"
    };
    let mut s = format!("Found {} {noun}:\n", found.len());
    let mut last: Option<Kind> = None;
    for f in found {
        if last != Some(f.kind) {
            s.push_str(&format!("\n{}:\n", f.kind.heading()));
            last = Some(f.kind);
        }
        s.push_str(&format!("  {}\n", f.message));
    }
    if found
        .iter()
        .any(|f| matches!(f.kind, Kind::CodeMoved | Kind::CodeGone))
    {
        s.push_str(
            "\nTo clear a code finding: read the dok against the code, fix what drifted, then \
`yaks dok set <path> --set verified=<date>`.\n",
        );
    }
    s
}

// ---- stray layout --------------------------------------------------------

/// Files under `<root>/.yaks/` other than the stub the migration leaves
/// (`README.md`, `schema`) and the lock. Only a farm at the branch root has
/// them: for a farm still at `.yaks/` the root *is* that directory.
fn stray_layout(root: &Path) -> Vec<Finding> {
    let stub = root.join(".yaks");
    if !stub.is_dir() || !root.join("schema").is_file() {
        return Vec::new();
    }
    let mut found = Vec::new();
    let mut stack = vec![stub.clone()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let rel = path.strip_prefix(&stub).unwrap_or(&path);
            if matches!(
                rel.to_str(),
                Some("README.md" | "schema" | ".lock" | ".gitignore")
            ) {
                continue;
            }
            let rel = rel.to_string_lossy().replace('\\', "/");
            found.push(Finding {
                kind: Kind::StrayLayout,
                message: format!(".yaks/{rel} should be at {rel}"),
                ids: vec![format!(".yaks/{rel}")],
            });
        }
    }
    found.sort_by(|a, b| a.message.cmp(&b.message));
    found
}

// ---- stale ---------------------------------------------------------------

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

/// When the dok's file last changed on the farm branch.
fn last_commit(root: &Path, rel: &str) -> Option<DateTime<Utc>> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "-1", "--format=%cI", "--", rel])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let t = DateTime::parse_from_rfc3339(text.trim()).ok()?;
    Some(t.with_timezone(&Utc))
}

/// The day the dok's file last changed on the farm branch.
fn last_change(root: &Path, rel: &str) -> Option<NaiveDate> {
    Some(last_commit(root, rel)?.date_naive())
}

/// A dok's `verified:` as a date: a day, or the day of a timestamp.
fn verified_at(p: &doks::Dok) -> Option<VerifiedAt> {
    let v = p.str_field("verified")?;
    parse_date(&v)
        .map(VerifiedAt::Day)
        .or_else(|| store::parse_ts(&v).map(VerifiedAt::At))
}

#[derive(Debug, Clone, Copy)]
enum VerifiedAt {
    Day(NaiveDate),
    At(DateTime<Utc>),
}

/// Doks past their `stale_after`: the dok's own, else its schema's. A date
/// makes the dok stale once that day has passed; a duration counts from the
/// dok's `verified:` date, else from its last commit on the farm branch (a
/// dok with neither is skipped). A value that is neither is ignored.
fn stale(
    root: &Path,
    all_doks: &[doks::Dok],
    all_schemas: &[schemas::Schema],
    now: DateTime<Utc>,
) -> Vec<Finding> {
    let today = now.date_naive();
    let git = store::farm_branch(root).is_some();
    let mut out = Vec::new();
    for p in all_doks {
        let kind = p.kind();
        let (after, who) = match p.str_field("stale_after").filter(|s| !s.is_empty()) {
            Some(a) => (a, "it"),
            None => match all_schemas
                .iter()
                .find(|s| s.type_name == kind)
                .and_then(|s| s.stale_after.clone())
            {
                Some(a) => (a, "its type"),
                None => continue,
            },
        };
        let message = if let Some(date) = parse_date(&after) {
            (today > date).then(|| format!("{} was due to be checked by {after}", p.path))
        } else if let Some((months, days)) = crate::recur::parse_duration(&after) {
            let verified = verified_at(p).map(|v| match v {
                VerifiedAt::Day(d) => d,
                VerifiedAt::At(t) => t.date_naive(),
            });
            let from = match verified {
                Some(d) => Some((d, "verified")),
                None if git => last_change(root, &p.path).map(|d| (d, "last changed")),
                None => None,
            };
            from.and_then(|(from, how)| {
                let due = from
                    .checked_add_months(Months::new(months))?
                    .checked_add_days(Days::new(days.into()))?;
                (today > due).then(|| {
                    format!(
                        "{} was {how} {from}; {who} asks for a re-check by {due}",
                        p.path
                    )
                })
            })
        } else {
            None
        };
        if let Some(message) = message {
            out.push(Finding {
                kind: Kind::Stale,
                message,
                ids: vec![p.path.clone()],
            });
        }
    }
    out
}

// ---- uncited sources -----------------------------------------------------

/// The names in every `sources/<name>/` or `sources/<name>.md` written in
/// `text`, in a link, a `sources:` entry or plain prose.
fn source_refs(text: &str) -> Vec<&str> {
    const MARK: &str = "sources/";
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find(MARK) {
        let start = from + i;
        let id_start = start + MARK.len();
        from = id_start;
        // `resources/` and `my-sources/` are other words.
        if text[..start]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            continue;
        }
        let rest = &text[id_start..];
        let len = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
            .unwrap_or(rest.len());
        // `sources/<name>/...` (a file beside it, or the layout before schema
        // 10) or `sources/<name>.md` (its file, yaks-e83f).
        // Dots that end the run are a sentence's, never a name's
        // (`... see sources/<id>.md.`).
        let name = rest[..len].trim_end_matches('.');
        let len = name.len();
        if len > 0 && rest.as_bytes()[0].is_ascii_alphanumeric() {
            if let Some(stem) = name.strip_suffix(".md") {
                out.push(stem);
            } else if rest[len..].starts_with('/') {
                out.push(name);
            }
        }
    }
    out
}

/// Which sources are cited, and how they hang together by `part_of`
/// (`yaks check` and `yaks source list` both read it, so they cannot
/// disagree).
pub(crate) struct Citations {
    /// id -> the id it is `part_of`; empty for a source on its own (a
    /// `part_of` naming a source that is not there leaves the part on its own).
    pub parent: BTreeMap<String, String>,
    /// id -> its directory name (`<id>` or `<id>-<slug>`).
    dir_of: BTreeMap<String, String>,
    /// The top source of each family some dok or yak cites, or that is marked
    /// `superseded_by:` / `uncited:`.
    cited: BTreeSet<String>,
}

impl Citations {
    /// The root of `id`'s `part_of` chain.
    pub fn top(&self, mut id: String) -> String {
        for _ in 0..10 {
            match self.parent.get(&id) {
                Some(p) if !p.is_empty() => id = p.clone(),
                _ => break,
            }
        }
        id
    }

    /// Whether the family `id` heads is cited, or marked on purpose.
    pub fn is_cited(&self, id: &str) -> bool {
        self.cited.contains(id)
    }

    /// How many sources are saved with `part_of` somewhere under `id`.
    fn parts_of(&self, id: &str) -> usize {
        self.parent
            .keys()
            .filter(|c| c.as_str() != id && self.top((*c).clone()) == id)
            .count()
    }
}

/// Read the citations of the farm's sources (`all` is `sources::list`).
pub(crate) fn citations(root: &Path, all: &[sources::Source]) -> Citations {
    // A source is its `<date>-<sha8>` id; its directory may carry a slug
    // after it (yaks-7729), and a citation may name either, so everything
    // below is keyed by the id and the directory name is kept for display.
    let id_of = |dir: &str| sources::id_of(dir.rsplit('/').next().unwrap_or(dir)).to_string();
    let mut dir_of: BTreeMap<String, String> = BTreeMap::new();
    let mut parent: BTreeMap<String, String> = BTreeMap::new();
    let mut marked: BTreeSet<String> = BTreeSet::new();
    for s in all {
        let id = id_of(&s.dir);
        let part_of = s
            .fields
            .get("part_of")
            .and_then(|v| v.as_str().map(String::from))
            .and_then(|t| {
                source_refs(&t)
                    .first()
                    .map(|p| sources::id_of(p).to_string())
            })
            .filter(|p| *p != id);
        dir_of.insert(
            id.clone(),
            s.dir.rsplit('/').next().unwrap_or(&s.dir).to_string(),
        );
        parent.insert(id.clone(), part_of.unwrap_or_default());
        let set = |k: &str| {
            s.fields
                .get(k)
                .and_then(|v| v.as_str().map(|s| !s.is_empty()))
                .unwrap_or(false)
        };
        if set("superseded_by") || set("uncited") {
            marked.insert(id);
        }
    }
    let known: BTreeSet<String> = parent.keys().cloned().collect();
    for p in parent.values_mut() {
        if !p.is_empty() && !known.contains(p) {
            p.clear();
        }
    }
    let mut c = Citations {
        parent,
        dir_of,
        cited: BTreeSet::new(),
    };

    // Every dok and yak, whole (frontmatter, body and notes). The sources
    // themselves are not read: a source citing itself is no citation.
    let mut files: Vec<std::path::PathBuf> = doks::dok_paths(root)
        .iter()
        .map(|rel| root.join(rel))
        .collect();
    files.extend(store::yak_files(root));
    let mut cited = BTreeSet::new();
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for id in source_refs(&text) {
            cited.insert(c.top(sources::id_of(id).to_string()));
        }
    }
    for id in &marked {
        cited.insert(c.top(id.clone()));
    }
    c.cited = cited;
    c
}

/// Sources that no dok and no yak cites, as `sources/<id>`. A source and
/// its `part_of` children count as one: if any of them is cited, none is
/// listed. One whose `source.md` has `superseded_by:` or `uncited:` counts
/// as cited, with its parts.
fn uncited(root: &Path) -> Result<Vec<Finding>> {
    let all = sources::list(root);
    if all.is_empty() {
        return Ok(Vec::new());
    }
    let c = citations(root, &all);
    let mut out = Vec::new();
    for (id, p) in &c.parent {
        if !p.is_empty() || c.is_cited(id) {
            continue;
        }
        let parts = c.parts_of(id);
        let name = c.dir_of.get(id).unwrap_or(id);
        let mut message = format!("{}/{name} is cited by no dok or yak", sources::SOURCES_DIR);
        if parts > 0 {
            let noun = if parts == 1 { "part" } else { "parts" };
            message.push_str(&format!(" (nor are its {parts} {noun} saved with part_of)"));
        }
        out.push(Finding {
            kind: Kind::Uncited,
            message,
            ids: vec![format!("{}/{name}", sources::SOURCES_DIR)],
        });
    }
    Ok(out)
}

// ---- similar kinds -------------------------------------------------------

/// A kind name with case, hyphens and a plural ending dropped, so `project`,
/// `projects` and `work-projects` compare equal.
fn normalize_kind(t: &str) -> String {
    let t = t.to_lowercase().replace('-', "");
    if let Some(stem) = t.strip_suffix("ies") {
        format!("{stem}y")
    } else if t.ends_with('s') && !t.ends_with("ss") {
        t[..t.len() - 1].to_string()
    } else {
        t
    }
}

/// Kinds in use (and kinds with a schema) that look like one kind written
/// two ways. The yak schema is not a dok kind.
fn similar_kinds(all_doks: &[doks::Dok], all_schemas: &[schemas::Schema]) -> Vec<Finding> {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for p in all_doks {
        *count.entry(p.kind()).or_default() += 1;
    }
    let with_schema: BTreeSet<&str> = all_schemas
        .iter()
        .map(|s| s.type_name.as_str())
        .filter(|n| *n != schemas::YAK_TYPE)
        .collect();
    for n in &with_schema {
        count.entry(n.to_string()).or_default();
    }
    let mut groups: BTreeMap<String, Vec<&String>> = BTreeMap::new();
    for k in count.keys() {
        groups.entry(normalize_kind(k)).or_default().push(k);
    }
    let mut out: Vec<Finding> = groups
        .into_values()
        .filter(|g| g.len() > 1)
        .map(|g| {
            let parts: Vec<String> = g
                .iter()
                .map(|k| {
                    let n = count[*k];
                    let doks = format!("{n} {}", if n == 1 { "dok" } else { "doks" });
                    if with_schema.contains(k.as_str()) {
                        format!("`{k}` ({doks}, has a schema)")
                    } else {
                        format!("`{k}` ({doks})")
                    }
                })
                .collect();
            Finding {
                kind: Kind::SimilarKinds,
                message: format!("these look like the same kind of dok: {}", parts.join(", ")),
                ids: g.into_iter().cloned().collect(),
            }
        })
        .collect();
    out.sort_by(|a, b| a.ids.cmp(&b.ids));
    out
}

// ---- no description ------------------------------------------------------

/// A dok with no `description:` (absent, empty or not a scalar), one finding
/// per dok path. A file with no frontmatter at all is `doctor`'s `dok-problem`
/// already, so it is not listed twice. Only doks are scanned: session notes
/// and schemas live outside the doks roots.
fn no_description(all_doks: &[doks::Dok]) -> Vec<Finding> {
    all_doks
        .iter()
        .filter(|d| d.has_frontmatter && d.description().is_none())
        .map(|d| Finding {
            kind: Kind::NoDescription,
            message: format!(
                "{} has no description: (one sentence on when to read it)",
                d.path
            ),
            ids: vec![d.path.clone()],
        })
        .collect()
}

// ---- cited code ----------------------------------------------------------

/// A `code:` link a dok makes.
#[derive(Debug, Clone, PartialEq, Eq)]
struct CodeRef {
    path: String,
    rev: Option<String>,
}

/// The `code:<path>` links of a dok, once each: the inline links of its body
/// and the `code:` entries of its frontmatter `sources:` and `code:` lists
/// (an entry of `code:` may leave the `code:` prefix out). Read through the
/// link grammar and [`Space::resolve`], as `peek` does.
fn code_refs(space: &Space, dok: &doks::Dok) -> Vec<CodeRef> {
    let mut dests: Vec<String> = links::extract(&dok.body)
        .into_iter()
        .map(|f| f.dest)
        .collect();
    for (key, bare) in [("sources", false), ("code", true)] {
        let Some(v) = dok.fields.get(key) else {
            continue;
        };
        for item in v.as_list() {
            let Some(s) = item.as_str().map(str::trim) else {
                continue;
            };
            if s.starts_with("code:") {
                dests.push(s.to_string());
            } else if bare && !s.is_empty() {
                dests.push(format!("code:{s}"));
            }
        }
    }
    let mut out: Vec<CodeRef> = Vec::new();
    for d in dests {
        let link = links::parse(&d, links::From::Dok(&dok.path), |_| false);
        if !matches!(link, Link::Code { .. }) {
            continue;
        }
        if let Place::CodeFile { rel, rev, .. } = space.resolve(&link) {
            let r = CodeRef { path: rel, rev };
            if !out.contains(&r) {
                out.push(r);
            }
        }
    }
    out
}

/// Commit times (unix seconds) of the commits touching `path` in `code`,
/// all of history or just `<since>..HEAD`; `None` when git cannot say.
fn commit_times(code: &Path, since: Option<&str>, path: &str) -> Option<Vec<i64>> {
    let range = since.map(|s| format!("{s}..HEAD"));
    let mut cmd = std::process::Command::new("git");
    cmd.arg("--no-optional-locks")
        .arg("-C")
        .arg(code)
        .args(["log", "--format=%ct"])
        .args(range.as_deref())
        .args(["--", path])
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE");
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|l| l.trim().parse().ok())
            .collect(),
    )
}

fn commits(n: usize) -> String {
    format!("{n} commit{}", if n == 1 { "" } else { "s" })
}

/// Doks that cite `code:` paths the code repo has moved on from, and doks
/// citing a path that is gone, with the information lines to print (not
/// findings).
///
/// - A pinned link (`code:<path>@<sha>`): the commits to the path in
///   `<sha>..HEAD`, all of them, unless the dok's `verified:` is no earlier
///   than the newest (it was re-checked after the code last moved). A pin
///   names history, so a file since deleted is not "gone": the deleting
///   commit is one of the commits counted. A pin this checkout does not have
///   is skipped, with a note.
/// - An unpinned link: the commits to the path after the dok's `verified:`
///   (a date covers its whole day, UTC), else after the dok's last commit on
///   the farm branch; a dok with neither is skipped. A path that is not in
///   the code checkout is `code-gone`, whatever `verified:` says.
///
/// Any count above zero is reported. With no code checkout (or one with no
/// commit) nothing is checked and one note says so, if any dok cites code.
fn code_changes(space: &Space, all_doks: &[doks::Dok]) -> (Vec<Finding>, Vec<String>) {
    let cited: Vec<(&doks::Dok, Vec<CodeRef>)> = all_doks
        .iter()
        .map(|d| (d, code_refs(space, d)))
        .filter(|(_, r)| !r.is_empty())
        .collect();
    if cited.is_empty() {
        return (Vec::new(), Vec::new());
    }
    let code = space
        .code
        .as_deref()
        .filter(|c| gitrepo::commit_of(c, "HEAD").is_some());
    let Some(code) = code else {
        let n = cited.len();
        let note = format!(
            "{n} dok{} cite{} `code:` paths, not checked: no code checkout is reachable (run \
from one)",
            if n == 1 { "" } else { "s" },
            if n == 1 { "s" } else { "" },
        );
        return (Vec::new(), vec![note]);
    };
    let git = store::farm_branch(&space.farm).is_some();
    let mut out = Vec::new();
    let mut notes = Vec::new();
    for (dok, refs) in cited {
        let verified = verified_at(dok);
        // The last second `verified:` covers.
        let covers = verified.and_then(|v| match v {
            VerifiedAt::Day(d) => {
                Some(d.succ_opt()?.and_hms_opt(0, 0, 0)?.and_utc().timestamp() - 1)
            }
            VerifiedAt::At(t) => Some(t.timestamp()),
        });
        let mut dok_commit: Option<Option<DateTime<Utc>>> = None; // looked up once
        for r in refs {
            let code_path = format!("code:{}", r.path);
            let ids = vec![dok.path.clone(), code_path.clone()];
            let finding = |kind, message| Finding {
                kind,
                message,
                ids: ids.clone(),
            };
            if let Some(rev) = &r.rev {
                let Some(sha) = gitrepo::commit_of(code, rev) else {
                    notes.push(format!(
                        "{}: {code_path}@{rev} not checked: the code checkout has no commit {rev}",
                        dok.path
                    ));
                    continue;
                };
                let Some(times) = commit_times(code, Some(&sha), &r.path) else {
                    continue;
                };
                let newest = times.iter().max().copied();
                if newest.is_none_or(|n| covers.is_some_and(|c| n <= c)) {
                    continue;
                }
                out.push(finding(
                    Kind::CodeMoved,
                    format!(
                        "{} cites {code_path}@{rev}: {} to it since the pin",
                        dok.path,
                        commits(times.len())
                    ),
                ));
                continue;
            }
            if !code.join(&r.path).exists() {
                out.push(finding(
                    Kind::CodeGone,
                    format!(
                        "{} cites {code_path}, which is gone from the code repo",
                        dok.path
                    ),
                ));
                continue;
            }
            // The reference point: `verified:`, else the dok's last farm commit.
            let (after, since) = match (covers, verified) {
                (Some(c), Some(v)) => {
                    let day = match v {
                        VerifiedAt::Day(d) => d.to_string(),
                        VerifiedAt::At(t) => t.to_rfc3339(),
                    };
                    (c, format!("it was verified {day}"))
                }
                _ => {
                    let last = *dok_commit.get_or_insert_with(|| {
                        git.then(|| last_commit(&space.farm, &dok.path)).flatten()
                    });
                    match last {
                        Some(t) => (
                            t.timestamp(),
                            format!("the dok's last commit on {}", t.date_naive()),
                        ),
                        None => continue,
                    }
                }
            };
            let Some(times) = commit_times(code, None, &r.path) else {
                continue;
            };
            let n = times.iter().filter(|t| **t > after).count();
            if n > 0 {
                out.push(finding(
                    Kind::CodeMoved,
                    format!(
                        "{} cites {code_path}: {} to it since {since}",
                        dok.path,
                        commits(n)
                    ),
                ));
            }
        }
    }
    (out, notes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_file_left_under_the_old_layout_is_flagged_but_not_the_stub() {
        let root = std::env::temp_dir().join(format!("yaks-check-stray-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join(".yaks/hearth/yaks")).unwrap();
        std::fs::write(root.join("schema"), "9\n").unwrap();
        std::fs::write(root.join(".yaks/schema"), "9\n").unwrap();
        std::fs::write(root.join(".yaks/README.md"), "stub\n").unwrap();
        assert!(stray_layout(&root).is_empty(), "the stub alone is fine");
        std::fs::write(root.join(".yaks/hearth/yaks/hearth-ffff.md"), "x").unwrap();
        let found = stray_layout(&root);
        assert_eq!(found.len(), 1);
        assert_eq!(
            found[0].message,
            ".yaks/hearth/yaks/hearth-ffff.md should be at hearth/yaks/hearth-ffff.md"
        );
        // A farm still at `.yaks/` has no `schema` at its parent: not flagged.
        std::fs::remove_file(root.join("schema")).unwrap();
        assert!(stray_layout(&root).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_dok_without_a_usable_description_is_flagged_once_by_path() {
        let dok = |path: &str, text: &str| doks::Dok::parse(path, text);
        let all = [
            dok(
                "doks/ok.md",
                "---\ntitle: A\ndescription: When to read A.\n---\n",
            ),
            dok("doks/none.md", "---\ntitle: B\n---\n"),
            dok("doks/empty.md", "---\ntitle: C\ndescription: '  '\n---\n"),
            dok("doks/list.md", "---\ntitle: D\ndescription: [a, b]\n---\n"),
            // No frontmatter: `dok-problem` has it.
            dok("doks/bare.md", "just text\n"),
        ];
        let found = no_description(&all);
        let ids: Vec<&str> = found.iter().map(|f| f.ids[0].as_str()).collect();
        assert_eq!(ids, ["doks/none.md", "doks/empty.md", "doks/list.md"]);
        assert!(found.iter().all(|f| f.kind == Kind::NoDescription));
        assert_eq!(Kind::NoDescription.code(), "no-description");
    }

    #[test]
    fn kinds_normalize_by_plural_and_hyphen() {
        let n = normalize_kind;
        assert_eq!(n("project"), n("projects"));
        assert_eq!(n("work-project"), n("workprojects"));
        assert_eq!(n("Policy"), n("policies"));
        assert_ne!(n("class"), n("clas") + "x");
        assert_eq!(n("class"), "class");
        assert_ne!(n("topic"), n("howto"));
    }

    #[test]
    fn source_refs_find_ids_in_links_and_prose_only_at_a_word_edge() {
        let t = "see [x](../sources/2026-01-02-abcd1234/shot.png), sources/b-1/ and \
                 resources/nope/ and sources/no-slash and `sources/c.d_e/source.md`";
        assert_eq!(source_refs(t), ["2026-01-02-abcd1234", "b-1", "c.d_e"]);
    }

    /// A source's file named in prose at the end of a sentence still counts:
    /// the full stop is not part of the name (yaks-03c8 found it).
    #[test]
    fn source_refs_find_a_file_named_at_the_end_of_a_sentence() {
        let t = "As sources/2026-01-02-abcd1234.md says. See /sources/b-1-slug.md... \
                 and sources/c-2.md, then sources/d-3.mdx (not a source file).";
        assert_eq!(source_refs(t), ["2026-01-02-abcd1234", "b-1-slug", "c-2"]);
    }

    // ---- cited code ----

    /// A scratch code repo; `commit` makes one commit on `date` touching `file`.
    struct Code {
        dir: PathBuf,
    }

    impl Code {
        fn new(tag: &str) -> Code {
            let dir =
                std::env::temp_dir().join(format!("yaks-check-code-{tag}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            let c = Code { dir };
            c.git(&["init", "-q", "-b", "main"], "2020-01-01T00:00:00Z");
            c
        }

        fn git(&self, args: &[&str], date: &str) -> String {
            let out = std::process::Command::new("git")
                .arg("-C")
                .arg(&self.dir)
                .args(["-c", "user.name=t", "-c", "user.email=t@t"])
                .args(args)
                .env("GIT_COMMITTER_DATE", date)
                .env("GIT_AUTHOR_DATE", date)
                .output()
                .unwrap();
            assert!(out.status.success(), "git {args:?}: {out:?}");
            String::from_utf8_lossy(&out.stdout).trim().to_string()
        }

        /// Write `file` (a new line each time) and commit it at `date`; returns the sha.
        fn commit(&self, file: &str, date: &str) -> String {
            let p = self.dir.join(file);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            let mut text = std::fs::read_to_string(&p).unwrap_or_default();
            text.push_str(date);
            text.push('\n');
            std::fs::write(&p, text).unwrap();
            self.git(&["add", "-A"], date);
            self.git(&["commit", "-q", "-m", file], date);
            self.git(&["rev-parse", "HEAD"], date)
        }
    }

    impl Drop for Code {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    /// What `code_changes` says about one dok, as `kind: message` lines.
    fn moved(code: Option<&Code>, front: &str, body: &str) -> (Vec<String>, Vec<String>) {
        // The farm has no git here, so the "last farm commit" fallback is
        // exercised through the binary (tests/check.rs).
        let space = Space::new(Path::new("/nonexistent-farm"), code.map(|c| c.dir.clone()));
        let dok = doks::Dok::parse("doks/x.md", &format!("---\n{front}---\n\n{body}\n"));
        let (f, n) = code_changes(&space, &[dok]);
        (
            f.iter()
                .map(|f| format!("{}: {}", f.kind.code(), f.message))
                .collect(),
            n,
        )
    }

    #[test]
    fn a_pinned_link_counts_the_commits_to_its_path_since_the_pin() {
        let c = Code::new("pinned");
        let pin = c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        c.commit("src/b.rs", "2020-03-01T00:00:00Z"); // another path: not counted
        c.commit("src/a.rs", "2020-04-01T00:00:00Z");
        c.commit("src/a.rs", "2020-05-01T00:00:00Z");
        let body = format!("See [a](code:src/a.rs@{}).", &pin[..9]);
        let (found, notes) = moved(Some(&c), "title: X\n", &body);
        assert_eq!(
            found,
            [format!(
                "code-moved: doks/x.md cites code:src/a.rs@{}: 2 commits to it since the pin",
                &pin[..9]
            )]
        );
        assert!(notes.is_empty());
    }

    #[test]
    fn a_pinned_link_with_no_later_commit_to_its_path_is_quiet() {
        let c = Code::new("pinned-none");
        let pin = c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        c.commit("src/b.rs", "2020-03-01T00:00:00Z");
        let body = format!("See [a](code:src/a.rs@{}#L1).", &pin[..9]);
        assert_eq!(moved(Some(&c), "title: X\n", &body), (vec![], vec![]));
    }

    #[test]
    fn verified_after_the_newest_commit_clears_a_pinned_link_and_before_it_does_not() {
        let c = Code::new("pinned-verified");
        let pin = c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        c.commit("src/a.rs", "2020-04-01T12:00:00Z");
        let body = format!("[a](code:src/a.rs@{})", &pin[..9]);
        let found = |v: &str| moved(Some(&c), &format!("verified: {v}\n"), &body).0.len();
        assert_eq!(found("2020-03-31"), 1);
        // A day covers the whole day: a commit that day was seen.
        assert_eq!(found("2020-04-01"), 0);
        assert_eq!(found("2020-06-01"), 0);
        assert_eq!(found("2020-04-01T11:00:00Z"), 1);
        assert_eq!(found("2020-04-01T12:00:00Z"), 0);
    }

    #[test]
    fn an_unpinned_link_counts_commits_since_verified() {
        let c = Code::new("unpinned");
        c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        c.commit("src/a.rs", "2020-04-01T00:00:00Z");
        c.commit("src/a.rs", "2020-05-01T00:00:00Z");
        c.commit("src/b.rs", "2020-05-02T00:00:00Z");
        let (found, _) = moved(Some(&c), "verified: 2020-03-01\n", "[a](code:src/a.rs#L3)");
        assert_eq!(
            found,
            [
                "code-moved: doks/x.md cites code:src/a.rs: 2 commits to it since it was verified 2020-03-01"
            ]
        );
        let (found, _) = moved(Some(&c), "verified: 2020-04-15\n", "[a](code:src/a.rs)");
        assert_eq!(
            found,
            [
                "code-moved: doks/x.md cites code:src/a.rs: 1 commit to it since it was verified 2020-04-15"
            ]
        );
        // verified: newer than the code's last commit: no finding.
        assert!(
            moved(Some(&c), "verified: 2020-05-03\n", "[a](code:src/a.rs)")
                .0
                .is_empty()
        );
    }

    #[test]
    fn an_unpinned_link_with_neither_verified_nor_a_farm_commit_is_skipped() {
        let c = Code::new("unpinned-none");
        c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        assert_eq!(
            moved(Some(&c), "title: X\n", "[a](code:src/a.rs)"),
            (vec![], vec![])
        );
    }

    #[test]
    fn frontmatter_sources_and_code_lists_are_read_like_body_links() {
        let c = Code::new("front");
        c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        c.commit("src/b.rs", "2020-02-01T00:00:00Z");
        c.commit("src/c.rs", "2020-02-01T00:00:00Z");
        let front = "verified: 2020-01-01\nsources:\n  - code:src/a.rs\n  - sources/x/source.md\ncode:\n  - src/b.rs\n  - code:src/a.rs\n";
        let (found, _) = moved(
            Some(&c),
            front,
            "also [c](code:src/c.rs) and [a again](code:src/a.rs#L2)",
        );
        // Body links first, then the frontmatter's; each path once.
        let mut paths: Vec<&str> = found
            .iter()
            .map(|f| f.split(' ').nth(3).unwrap().trim_end_matches(':'))
            .collect();
        paths.sort();
        assert_eq!(
            paths,
            ["code:src/a.rs", "code:src/b.rs", "code:src/c.rs"],
            "{found:?}"
        );
    }

    #[test]
    fn no_code_checkout_is_one_note_and_no_finding() {
        let doks = [
            doks::Dok::parse(
                "doks/x.md",
                "---\nverified: 2020-01-01\n---\n[a](code:src/a.rs)\n",
            ),
            doks::Dok::parse(
                "doks/y.md",
                "---\nverified: 2020-01-01\n---\n[b](code:src/b.rs)\n",
            ),
        ];
        let (found, notes) = code_changes(&Space::new(Path::new("/nonexistent-farm"), None), &doks);
        assert!(found.is_empty());
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(
            notes[0].contains("2 doks cite") && notes[0].contains("no code checkout"),
            "{notes:?}"
        );
        // Nothing cites code: nothing to say.
        let plain = [doks::Dok::parse(
            "doks/z.md",
            "---\ntitle: Z\n---\nno links\n",
        )];
        let sp = Space::new(Path::new("/nonexistent-farm"), None);
        assert_eq!(code_changes(&sp, &plain), (vec![], vec![]));
    }

    #[test]
    fn a_removed_file_is_gone_unpinned_and_history_when_pinned() {
        let c = Code::new("gone");
        let pin = c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        c.git(&["rm", "-q", "src/a.rs"], "2020-03-01T00:00:00Z");
        c.git(&["commit", "-q", "-m", "rm"], "2020-03-01T00:00:00Z");
        let (found, _) = moved(Some(&c), "verified: 2021-01-01\n", "[a](code:src/a.rs)");
        assert_eq!(
            found,
            ["code-gone: doks/x.md cites code:src/a.rs, which is gone from the code repo"]
        );
        let body = format!("[a](code:src/a.rs@{})", &pin[..9]);
        let (found, _) = moved(Some(&c), "title: X\n", &body);
        assert_eq!(found.len(), 1);
        assert!(
            found[0].starts_with("code-moved:") && found[0].contains("1 commit to it"),
            "{found:?}"
        );
    }

    #[test]
    fn a_pin_the_checkout_does_not_have_is_a_note() {
        let c = Code::new("unknown-pin");
        c.commit("src/a.rs", "2020-02-01T00:00:00Z");
        let (found, notes) = moved(Some(&c), "title: X\n", "[a](code:src/a.rs@deadbeef)");
        assert!(found.is_empty());
        assert_eq!(notes.len(), 1, "{notes:?}");
    }

    #[test]
    fn the_code_kinds_have_stable_labels() {
        assert_eq!(Kind::CodeMoved.code(), "code-moved");
        assert_eq!(Kind::CodeGone.code(), "code-gone");
    }
}
