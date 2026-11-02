//! `yaks check` (yaks-c0ac): Hearth's nightly check minus what needs a model.
//! It reads the farm and reports what a person or an agent should look at,
//! and never edits.
//!
//! Seven kinds of finding. Four are `doctor`'s own issues, reused as they are
//! (`page-problem`, `broken-link`, `schema-mismatch`, `merge-conflict`); three
//! are ported from Hearth's `internal/check/scan.go`:
//!
//! - `stale`: a page past its `stale_after` (its own, else its schema's);
//! - `uncited`: a source nothing in the farm cites;
//! - `similar-kinds`: page kinds that differ only by plural or hyphens.

use crate::farm::{Farm, IssueKind};
use crate::{pages, schemas, sources, store};
use anyhow::Result;
use chrono::{DateTime, Days, Months, NaiveDate, Utc};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The classes of finding, in the order a report lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    PageProblem,
    BrokenLink,
    SchemaMismatch,
    MergeConflict,
    Stale,
    Uncited,
    SimilarKinds,
}

impl Kind {
    /// The stable machine label, the same as `doctor`'s for the kinds it owns.
    pub fn code(self) -> &'static str {
        match self {
            Kind::PageProblem => IssueKind::PageProblem.code(),
            Kind::BrokenLink => IssueKind::BrokenLink.code(),
            Kind::SchemaMismatch => IssueKind::SchemaMismatch.code(),
            Kind::MergeConflict => IssueKind::MergeConflict.code(),
            Kind::Stale => "stale",
            Kind::Uncited => "uncited",
            Kind::SimilarKinds => "similar-kinds",
        }
    }

    fn heading(self) -> &'static str {
        match self {
            Kind::PageProblem => IssueKind::PageProblem.heading(),
            Kind::BrokenLink => IssueKind::BrokenLink.heading(),
            Kind::SchemaMismatch => IssueKind::SchemaMismatch.heading(),
            Kind::MergeConflict => IssueKind::MergeConflict.heading(),
            Kind::Stale => {
                "Stale page (past its stale_after: check the facts again, then set verified:)"
            }
            Kind::Uncited => {
                "Uncited source (no page or yak mentions sources/<id>/: cite it, or set \
superseded_by: or uncited: in its source.md)"
            }
            Kind::SimilarKinds => {
                "Similar kinds (page types that differ only by plural or hyphens: pick one)"
            }
        }
    }

    fn from_issue(k: IssueKind) -> Option<Kind> {
        match k {
            IssueKind::PageProblem => Some(Kind::PageProblem),
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
    /// What it concerns: page paths, source directories or kind names.
    pub ids: Vec<String>,
}

/// Every finding of the farm, as of now.
pub fn run(farm: &Farm) -> Result<Vec<Finding>> {
    run_at(farm, Utc::now())
}

/// Every finding of the farm, as of `now`.
pub fn run_at(farm: &Farm, now: DateTime<Utc>) -> Result<Vec<Finding>> {
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
    let all_pages = pages::load_all(root)?;
    let all_schemas = schemas::load_all(root);
    out.extend(stale(root, &all_pages, &all_schemas, now));
    out.extend(uncited(root)?);
    out.extend(similar_kinds(&all_pages, &all_schemas));
    Ok(out)
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
    s
}

// ---- stale ---------------------------------------------------------------

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

/// The day the page's file last changed on the farm branch.
fn last_change(root: &Path, rel: &str) -> Option<NaiveDate> {
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
    Some(t.with_timezone(&Utc).date_naive())
}

/// Pages past their `stale_after`: the page's own, else its schema's. A date
/// makes the page stale once that day has passed; a duration counts from the
/// page's `verified:` date, else from its last commit on the farm branch (a
/// page with neither is skipped). A value that is neither is ignored.
fn stale(
    root: &Path,
    all_pages: &[pages::Page],
    all_schemas: &[schemas::Schema],
    now: DateTime<Utc>,
) -> Vec<Finding> {
    let today = now.date_naive();
    let git = store::farm_branch(root).is_some();
    let mut out = Vec::new();
    for p in all_pages {
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
            let verified = p.str_field("verified").and_then(|v| {
                parse_date(&v).or_else(|| store::parse_ts(&v).map(|t| t.date_naive()))
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

/// The ids in every `sources/<id>/` written in `text`, in a link, a
/// `sources:` entry or plain prose.
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
        if len > 0 && rest.as_bytes()[0].is_ascii_alphanumeric() && rest[len..].starts_with('/') {
            out.push(&rest[..len]);
        }
    }
    out
}

/// Sources that no page and no yak cites, as `sources/<id>`. A source and
/// its `part_of` children count as one: if any of them is cited, none is
/// listed. One whose `source.md` has `superseded_by:` or `uncited:` counts
/// as cited, with its parts.
fn uncited(root: &Path) -> Result<Vec<Finding>> {
    let all = sources::list(root);
    if all.is_empty() {
        return Ok(Vec::new());
    }
    let id_of = |dir: &str| dir.rsplit('/').next().unwrap_or(dir).to_string();
    let mut parent: BTreeMap<String, String> = BTreeMap::new(); // id -> the id it is part_of
    let mut marked: BTreeSet<String> = BTreeSet::new();
    for s in &all {
        let id = id_of(&s.dir);
        let part_of = s
            .fields
            .get("part_of")
            .and_then(|v| v.as_str().map(String::from))
            .and_then(|t| source_refs(&t).first().map(|p| p.to_string()))
            .filter(|p| *p != id);
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
    // A part_of naming a source that is not there leaves the part on its own.
    let known: BTreeSet<String> = parent.keys().cloned().collect();
    for p in parent.values_mut() {
        if !p.is_empty() && !known.contains(p) {
            p.clear();
        }
    }
    let top = |mut id: String| {
        for _ in 0..10 {
            match parent.get(&id) {
                Some(p) if !p.is_empty() => id = p.clone(),
                _ => break,
            }
        }
        id
    };

    // Every page and yak, whole (frontmatter, body and notes). The sources
    // themselves are not read: a source citing itself is no citation.
    let mut cited: BTreeSet<String> = BTreeSet::new();
    let mut files: Vec<std::path::PathBuf> = pages::page_paths(root)
        .iter()
        .map(|rel| root.join(rel))
        .collect();
    files.extend(store::yak_files(root));
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for id in source_refs(&text) {
            cited.insert(top(id.to_string()));
        }
    }
    for id in &marked {
        cited.insert(top(id.clone()));
    }

    let mut out = Vec::new();
    for (id, p) in &parent {
        if !p.is_empty() || cited.contains(id) {
            continue;
        }
        let parts = parent
            .keys()
            .filter(|c| *c != id && top((*c).clone()) == *id)
            .count();
        let mut message = format!("{}/{id} is cited by no page or yak", sources::SOURCES_DIR);
        if parts > 0 {
            let noun = if parts == 1 { "part" } else { "parts" };
            message.push_str(&format!(" (nor are its {parts} {noun} saved with part_of)"));
        }
        out.push(Finding {
            kind: Kind::Uncited,
            message,
            ids: vec![format!("{}/{id}", sources::SOURCES_DIR)],
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
/// two ways. The yak schema is not a page kind.
fn similar_kinds(all_pages: &[pages::Page], all_schemas: &[schemas::Schema]) -> Vec<Finding> {
    let mut count: BTreeMap<String, usize> = BTreeMap::new();
    for p in all_pages {
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
                    let pages = format!("{n} {}", if n == 1 { "page" } else { "pages" });
                    if with_schema.contains(k.as_str()) {
                        format!("`{k}` ({pages}, has a schema)")
                    } else {
                        format!("`{k}` ({pages})")
                    }
                })
                .collect();
            Finding {
                kind: Kind::SimilarKinds,
                message: format!(
                    "these look like the same kind of page: {}",
                    parts.join(", ")
                ),
                ids: g.into_iter().cloned().collect(),
            }
        })
        .collect();
    out.sort_by(|a, b| a.ids.cmp(&b.ids));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
