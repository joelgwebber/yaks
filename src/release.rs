//! `yaks release changes` (yaks-045c, design `/doks/design/changelog.md` §2
//! and §3): what a release's commit range turns up, computed from git and the
//! farm each time and never stored.
//!
//! A yak is *in* a range when a non-merge commit there names it: its message
//! holds the yak's id (a whole id that exists in the farm), or one of the
//! yak's notes records the commit in a `commit:` field (the rule of
//! [`crate::yakcommits`]). [`build`] is pure (tasks, commits and a reader of
//! each yak's recorded commits in, a [`Report`] out); the git side is
//! [`resolve_range`] and [`commits_in`], and [`cli`] ties them to the farm and
//! prints.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::changeline::{self, Change};
use crate::farm::Farm;
use crate::model::{Status, Task};
use crate::{json as json_out, refs, store, yakcommits};

/// The tags that are releases unless `git config yaks.releaseTags` says
/// otherwise.
pub const DEFAULT_TAGS: &str = "v*";
/// The `git config` key holding the release-tag glob.
pub const TAGS_KEY: &str = "yaks.releaseTags";

/// A commit that only bumps the version names no yak and is not a gap in the
/// trail (design §2): its subject starts with this.
const VERSION_BUMP: &str = "Bump version";

fn is_version_bump(subject: &str) -> bool {
    subject.starts_with(VERSION_BUMP)
}

/// A recorded sha is trusted as a prefix only from this many hex digits (the
/// length of `git log --abbrev`'s default), so a short accident matches nothing.
const MIN_SHA: usize = 7;

/// A resolved range: the commits reachable from `to` and not from `from`
/// (`None`: from the start of history).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Range {
    pub from: Option<String>,
    pub to: String,
}

impl Range {
    /// The argument for `git log`.
    fn spec(&self) -> String {
        match &self.from {
            Some(f) => format!("{f}..{}", self.to),
            None => self.to.clone(),
        }
    }

    fn label(&self) -> String {
        match &self.from {
            Some(f) => format!("{f}..{}", self.to),
            None => format!("start..{}", self.to),
        }
    }
}

/// One non-merge commit of a range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub sha: String,
    pub short: String,
    pub subject: String,
    /// Subject and body: what is scanned for yak ids.
    pub message: String,
}

/// Where a yak falls in the report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// Shorn with a change line: a changelog entry.
    Change,
    /// Shorn with `--no-change` (`section: none`).
    NoChange,
    /// Shorn with no change line, but an ancestor in the range has one.
    Covered,
    /// Shorn with no change line and nothing covering it: `--check` fails.
    Gap,
    /// Named by a commit but hairy, shaving or dead.
    Unfinished,
}

impl Group {
    fn as_str(self) -> &'static str {
        match self {
            Group::Change => "change",
            Group::NoChange => "no_change",
            Group::Covered => "covered",
            Group::Gap => "gap",
            Group::Unfinished => "unfinished",
        }
    }
}

/// A yak in the range, with the commits that put it there.
#[derive(Debug, Clone)]
pub struct Item {
    pub task: Task,
    pub commits: Vec<Commit>,
    pub change: Option<Change>,
    pub group: Group,
    /// The nearest ancestor that is a changelog entry of this range: it covers
    /// a yak with no line of its own, and a yak with one is listed under it.
    pub under: Option<String>,
}

/// What a range turns up.
#[derive(Debug, Clone)]
pub struct Report {
    pub range: Range,
    pub commits: usize,
    /// The yaks the range names, by id.
    pub items: Vec<Item>,
    /// Commits naming no yak, version bumps excluded.
    pub loose: Vec<Commit>,
}

impl Report {
    fn in_group(&self, g: Group) -> impl Iterator<Item = &Item> {
        self.items.iter().filter(move |i| i.group == g)
    }

    /// The shorn yaks `--check` fails on.
    pub fn gaps(&self) -> Vec<&Item> {
        self.in_group(Group::Gap).collect()
    }
}

/// Compute the report. `recorded` gives the shas a yak's notes record.
pub fn build(
    range: Range,
    tasks: &[Task],
    commits: &[Commit],
    recorded: &dyn Fn(&Task) -> Vec<String>,
) -> Report {
    let known: HashSet<&str> = tasks.iter().map(|t| t.id.as_str()).collect();
    // yak id -> indices into `commits` that name it.
    let mut named: HashMap<&str, Vec<usize>> = HashMap::new();
    for (i, c) in commits.iter().enumerate() {
        for f in refs::scan_text(&c.message, |t| known.contains(t)) {
            let id = known.get(f.id.as_str()).copied().unwrap_or_default();
            named.entry(id).or_default().push(i);
        }
    }
    for t in tasks {
        for sha in recorded(t) {
            let sha = sha.to_ascii_lowercase();
            if sha.len() < MIN_SHA || !sha.bytes().all(|b| b.is_ascii_hexdigit()) {
                continue;
            }
            for (i, c) in commits.iter().enumerate() {
                if c.sha.starts_with(&sha) {
                    named.entry(t.id.as_str()).or_default().push(i);
                }
            }
        }
    }
    let mut claimed = vec![false; commits.len()];
    let by_id: HashMap<&str, &Task> = tasks.iter().map(|t| (t.id.as_str(), t)).collect();
    let mut items: Vec<Item> = Vec::new();
    for (id, mut idx) in named {
        idx.sort_unstable();
        idx.dedup();
        for &i in &idx {
            claimed[i] = true;
        }
        let task = by_id[id];
        let change = changeline::latest(task);
        let group = match (task.status, &change) {
            (Status::Shorn, Some(c)) if c.is_none() => Group::NoChange,
            (Status::Shorn, Some(_)) => Group::Change,
            (Status::Shorn, None) => Group::Gap,
            _ => Group::Unfinished,
        };
        items.push(Item {
            task: task.clone(),
            commits: idx.iter().map(|&i| commits[i].clone()).collect(),
            change,
            group,
            under: None,
        });
    }
    items.sort_by(|a, b| a.task.id.cmp(&b.task.id));
    // Families: the nearest ancestor that is an entry of this range covers a
    // shorn yak with no line, and is the one an entry with a line nests under.
    let entries: HashSet<String> = items
        .iter()
        .filter(|i| i.group == Group::Change)
        .map(|i| i.task.id.clone())
        .collect();
    for item in &mut items {
        if !matches!(item.group, Group::Change | Group::Gap) {
            continue;
        }
        let mut seen: HashSet<&str> = HashSet::new();
        let mut at = item.task.parent.as_deref();
        while let Some(p) = at.filter(|p| seen.insert(p)) {
            if entries.contains(p) {
                item.under = Some(p.to_string());
                break;
            }
            at = by_id.get(p).and_then(|t| t.parent.as_deref());
        }
        if item.group == Group::Gap && item.under.is_some() {
            item.group = Group::Covered;
        }
    }
    let loose = commits
        .iter()
        .zip(&claimed)
        .filter(|(c, claimed)| !**claimed && !is_version_bump(&c.subject))
        .map(|(c, _)| c.clone())
        .collect();
    Report {
        range,
        commits: commits.len(),
        items,
        loose,
    }
}

// ---- git -----------------------------------------------------------------

fn git(repo: &Path, args: &[&str]) -> Result<std::process::Output> {
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .context("running git")
}

/// The release-tag glob of the repository at `repo`.
pub fn tag_glob(repo: &Path) -> String {
    store::repo_config(repo, TAGS_KEY)
        .filter(|g| !g.is_empty())
        .unwrap_or_else(|| DEFAULT_TAGS.to_string())
}

/// The nearest release tag reachable from `rev`, if any.
fn nearest_tag(repo: &Path, glob: &str, rev: &str) -> Option<String> {
    let out = git(
        repo,
        &["describe", "--tags", "--abbrev=0", "--match", glob, rev],
    )
    .ok()
    .filter(|o| o.status.success())?;
    let tag = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!tag.is_empty()).then_some(tag)
}

/// Resolve the range argument: `A..B`; a single `T` (the release before `T`,
/// the nearest release tag from `T`'s first parent, up to `T`); or nothing
/// (the latest release tag reachable from `HEAD`, up to `HEAD`).
pub fn resolve_range(repo: &Path, arg: Option<&str>) -> Result<Range> {
    let glob = tag_glob(repo);
    Ok(match arg {
        None => Range {
            from: nearest_tag(repo, &glob, "HEAD"),
            to: "HEAD".into(),
        },
        Some(a) if a.contains("...") => bail!("{a:?}: use A..B, not A...B"),
        Some(a) if a.contains("..") => {
            let (from, to) = a.split_once("..").unwrap_or_default();
            Range {
                from: (!from.is_empty()).then(|| from.to_string()),
                to: if to.is_empty() { "HEAD" } else { to }.to_string(),
            }
        }
        Some(t) => Range {
            from: nearest_tag(repo, &glob, &format!("{t}^")),
            to: t.to_string(),
        },
    })
}

/// The non-merge commits of `range`, newest first.
pub fn commits_in(repo: &Path, range: &Range) -> Result<Vec<Commit>> {
    let out = git(
        repo,
        &[
            "log",
            "--no-merges",
            "--format=%H%x1f%h%x1f%s%x1f%B%x1e",
            &range.spec(),
            "--",
        ],
    )?;
    if !out.status.success() {
        bail!(
            "git log {} failed: {}",
            range.spec(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .split('\x1e')
        .filter_map(|rec| {
            let mut f = rec.trim_start_matches('\n').splitn(4, '\x1f');
            Some(Commit {
                sha: f.next().filter(|s| !s.is_empty())?.to_string(),
                short: f.next()?.to_string(),
                subject: f.next()?.to_string(),
                message: f.next().unwrap_or_default().to_string(),
            })
        })
        .collect())
}

// ---- output --------------------------------------------------------------

fn commit_json(c: &Commit) -> Value {
    json!({"sha": c.sha, "short": c.short, "subject": c.subject})
}

fn item_json(i: &Item) -> Value {
    let t = &i.task;
    json!({
        "id": t.id,
        "title": t.title,
        "type": t.kind,
        "status": t.status.as_str(),
        "parent": t.parent,
        "group": i.group.as_str(),
        "change": i.change.as_ref().map(|c| &c.text),
        "section": i.change.as_ref().map(|c| &c.section),
        "covered_by": if i.group == Group::Covered { i.under.as_ref() } else { None },
        "commits": i.commits.iter().map(commit_json).collect::<Vec<_>>(),
    })
}

/// The report as JSON: the resolved range, every yak it names (`group` says
/// which of `change`, `no_change`, `covered`, `gap`, `unfinished`), the ids
/// `--check` fails on, and the commits naming no yak.
pub fn report_json(r: &Report) -> Value {
    json!({
        "range": {"from": r.range.from, "to": r.range.to},
        "commits": r.commits,
        "yaks": r.items.iter().map(item_json).collect::<Vec<_>>(),
        "gaps": r.gaps().iter().map(|i| &i.task.id).collect::<Vec<_>>(),
        "commits_without_yak": r.loose.iter().map(commit_json).collect::<Vec<_>>(),
    })
}

fn shas(i: &Item) -> String {
    i.commits
        .iter()
        .map(|c| c.short.as_str())
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_case(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// One entry and, indented under it, the yaks nested in it.
fn entry_text(r: &Report, i: &Item, depth: usize, out: &mut String) {
    let pad = "  ".repeat(depth);
    let line = i.change.as_ref().map(|c| c.text.as_str());
    let (head, how) = match (i.group, line) {
        (Group::Change, Some(l)) => (l, String::new()),
        _ => (i.task.title.as_str(), ", covered".to_string()),
    };
    out.push_str(&format!(
        "{pad}- {head} ({}{how}; {})\n",
        i.task.id,
        shas(i)
    ));
    for child in r
        .items
        .iter()
        .filter(|c| c.under.as_deref() == Some(i.task.id.as_str()))
    {
        entry_text(r, child, depth + 1, out);
    }
}

/// The report as text.
pub fn report_text(r: &Report) -> String {
    let mut out = format!(
        "Changes {}: {} commit{}, {} yak{}\n",
        r.range.label(),
        r.commits,
        if r.commits == 1 { "" } else { "s" },
        r.items.len(),
        if r.items.len() == 1 { "" } else { "s" },
    );
    // Keep a Changelog's order; a hand-written section none of them follows.
    let mut sections: Vec<&str> = changeline::SECTIONS.to_vec();
    let mut extra: Vec<&str> = r
        .in_group(Group::Change)
        .filter_map(|i| i.change.as_ref().map(|c| c.section.as_str()))
        .filter(|s| !changeline::SECTIONS.contains(s))
        .collect();
    extra.sort_unstable();
    extra.dedup();
    sections.extend(extra);
    for s in sections {
        let top: Vec<&Item> = r
            .in_group(Group::Change)
            .filter(|i| i.under.is_none())
            .filter(|i| i.change.as_ref().is_some_and(|c| c.section == s))
            .collect();
        if top.is_empty() {
            continue;
        }
        out.push_str(&format!("\n{}\n", title_case(s)));
        for i in top {
            entry_text(r, i, 0, &mut out);
        }
    }
    let none: Vec<&Item> = r.in_group(Group::NoChange).collect();
    if !none.is_empty() {
        let ids: Vec<&str> = none.iter().map(|i| i.task.id.as_str()).collect();
        out.push_str(&format!(
            "\nNo user-facing change ({}): {}\n",
            none.len(),
            ids.join(", ")
        ));
    }
    let gaps = r.gaps();
    if !gaps.is_empty() {
        out.push_str(&format!("\nShorn with no change line ({})\n", gaps.len()));
        for i in gaps {
            out.push_str(&format!(
                "- {}: {} ({})\n",
                i.task.id,
                i.task.title,
                shas(i)
            ));
        }
    }
    let open: Vec<&Item> = r.in_group(Group::Unfinished).collect();
    if !open.is_empty() {
        out.push_str(&format!(
            "\nNamed by commits but not shorn ({})\n",
            open.len()
        ));
        for i in open {
            out.push_str(&format!(
                "- {} [{}]: {} ({})\n",
                i.task.id,
                i.task.status.as_str(),
                i.task.title,
                shas(i)
            ));
        }
    }
    if !r.loose.is_empty() {
        out.push_str(&format!("\nCommits naming no yak ({})\n", r.loose.len()));
        for c in &r.loose {
            out.push_str(&format!("- {} {}\n", c.short, c.subject));
        }
    }
    out
}

/// Run `yaks release changes` from `cwd`: print the report, return the exit
/// status (1 with `check` when a shorn yak in the range has no change line and
/// no ancestor's line covers it).
pub fn cli(farm: &Farm, cwd: &Path, range: Option<&str>, json: bool, check: bool) -> Result<i32> {
    let Some(repo) = farm.code_repo(cwd) else {
        bail!("release changes reads a release from the code repository: run it in your checkout");
    };
    let range = resolve_range(&repo, range)?;
    let commits = commits_in(&repo, &range)?;
    let tasks = store::load(farm.root(), &crate::farm::EVERY)?;
    let space = crate::locate::Space::new(farm.root(), Some(repo.clone()));
    let recorded = |t: &Task| -> Vec<String> {
        if !t.body.contains("commit") {
            return Vec::new();
        }
        yakcommits::collect(&store::parse_notes(&t.body), &|l| space.read_url(l))
            .into_iter()
            .map(|r| r.commit)
            .collect()
    };
    let report = build(range, &tasks, &commits, &recorded);
    let gaps = report.gaps();
    if json {
        json_out::print(&report_json(&report))?;
    } else {
        print!("{}", report_text(&report));
    }
    if check && !gaps.is_empty() {
        let line = format!(
            "check failed: {} shorn yak{} {} no change line (add one: yaks update <id> --note \"<line>\" --meta kind=change, or record --no-change)",
            gaps.len(),
            if gaps.len() == 1 { "" } else { "s" },
            if gaps.len() == 1 { "has" } else { "have" },
        );
        // JSON keeps stdout one document.
        if json {
            eprintln!("{line}");
        } else {
            println!("\n{line}");
        }
        return Ok(1);
    }
    if check && !json {
        println!("\ncheck: ok, every shorn yak in the range has a change line or is covered");
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(
        id: &str,
        status: Status,
        kind: &str,
        parent: Option<&str>,
        change: Option<&[(&str, &str)]>,
    ) -> Task {
        let mut body = String::new();
        if let Some(meta) = change {
            let text = format!("line of {id}");
            body = store::append_note_meta(&body, "2026-01-01T00:00:00Z", None, meta, &text);
        }
        Task {
            id: id.into(),
            title: format!("title {id}"),
            kind: kind.into(),
            priority: 3,
            status,
            created: None,
            updated: None,
            parent: parent.map(str::to_string),
            tags: vec![],
            depends_on: vec![],
            source: None,
            prs: vec![],
            needs: None,
            verify: None,
            extra: Default::default(),
            body,
        }
    }

    fn commit(n: u32, subject: &str, body: &str) -> Commit {
        let sha = format!("{n:07x}{}", "0".repeat(33));
        Commit {
            short: sha[..7].to_string(),
            sha,
            subject: subject.into(),
            message: format!("{subject}\n\n{body}"),
        }
    }

    const LINE: &[(&str, &str)] = &[("kind", "change")];
    const NONE: &[(&str, &str)] = &[("kind", "change"), ("section", "none")];

    fn range() -> Range {
        Range {
            from: Some("v1".into()),
            to: "v2".into(),
        }
    }

    #[test]
    fn groups_gaps_coverage_and_loose_commits() {
        let tasks = vec![
            task("y-umbr", Status::Shorn, "feature", None, Some(LINE)),
            task("y-kid1", Status::Shorn, "task", Some("y-umbr"), None),
            task("y-kid2", Status::Shorn, "task", Some("y-kid1"), Some(LINE)),
            task("y-grand", Status::Shorn, "task", Some("y-kid1"), None),
            task("y-gap", Status::Shorn, "bug", None, None),
            task("y-int", Status::Shorn, "task", None, Some(NONE)),
            task("y-open", Status::Shaving, "task", None, None),
            task("y-dead", Status::Dead, "task", None, None),
            task("y-outside", Status::Shorn, "task", None, None),
            task("y-nokid", Status::Shorn, "task", Some("y-int"), None),
        ];
        let commits = vec![
            commit(1, "Bump version to 2", ""),
            commit(2, "umbrella (y-umbr)", ""),
            commit(3, "kid1: y-kid1 and y-kid2", ""),
            commit(4, "grand", "Refs: y-grand."),
            commit(5, "the gap", "y-gap"),
            commit(6, "internal", "[[y-int]]"),
            commit(7, "y-open partial; y-dead revert", ""),
            commit(8, "stray", "mentions y-nokid and y-ghost"),
            commit(9, "no yak at all", "y-gapped and y-ghost are no yaks"),
        ];
        let r = build(range(), &tasks, &commits, &|_| vec![]);
        let g = |id: &str| r.items.iter().find(|i| i.task.id == id).unwrap();
        assert_eq!(g("y-umbr").group, Group::Change);
        assert_eq!(g("y-kid1").group, Group::Covered);
        assert_eq!(g("y-kid1").under.as_deref(), Some("y-umbr"));
        // A child with its own line nests under the nearest entry above it.
        assert_eq!(g("y-kid2").group, Group::Change);
        assert_eq!(g("y-kid2").under.as_deref(), Some("y-umbr"));
        // A grandchild is covered by the nearest ancestor that is an entry,
        // skipping a parent that has no line of its own.
        assert_eq!(g("y-grand").group, Group::Covered);
        assert_eq!(g("y-grand").under.as_deref(), Some("y-umbr"));
        assert_eq!(g("y-gap").group, Group::Gap);
        assert_eq!(g("y-int").group, Group::NoChange);
        // A --no-change parent covers nothing.
        assert_eq!(g("y-nokid").group, Group::Gap);
        assert_eq!(g("y-open").group, Group::Unfinished);
        assert_eq!(g("y-dead").group, Group::Unfinished);
        assert!(r.items.iter().all(|i| i.task.id != "y-outside"));
        let gaps: Vec<&str> = r.gaps().iter().map(|i| i.task.id.as_str()).collect();
        assert_eq!(gaps, ["y-gap", "y-nokid"]);
        // The bump is ignored; the commit naming only a prefix-lookalike and
        // an unknown id names no yak.
        let loose: Vec<&str> = r.loose.iter().map(|c| c.subject.as_str()).collect();
        assert_eq!(loose, ["no yak at all"]);
        assert_eq!(g("y-kid1").commits.len(), 1);
        assert_eq!(g("y-kid2").commits[0].subject, "kid1: y-kid1 and y-kid2");
    }

    #[test]
    fn a_recorded_commit_names_a_yak_by_prefix() {
        let tasks = vec![
            task("y-a", Status::Shorn, "task", None, Some(LINE)),
            task("y-b", Status::Shorn, "task", None, Some(LINE)),
        ];
        let commits = vec![commit(0xabc, "landed work", ""), commit(0xdef, "other", "")];
        let rec = |t: &Task| match t.id.as_str() {
            // A prefix of the full sha, and one too short to trust.
            "y-a" => vec![commits[0].sha[..9].to_uppercase()],
            _ => vec!["000d".to_string()],
        };
        let r = build(range(), &tasks, &commits, &rec);
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].task.id, "y-a");
        assert_eq!(r.loose.len(), 1);
    }

    #[test]
    fn text_nests_families_and_orders_sections() {
        let tasks = vec![
            task("y-fix", Status::Shorn, "bug", None, Some(LINE)),
            task("y-umbr", Status::Shorn, "feature", None, Some(LINE)),
            task("y-kid", Status::Shorn, "task", Some("y-umbr"), None),
        ];
        let commits = vec![commit(1, "x", "y-fix y-umbr y-kid")];
        let text = report_text(&build(range(), &tasks, &commits, &|_| vec![]));
        let added = text.find("Added").unwrap();
        let fixed = text.find("Fixed").unwrap();
        assert!(added < fixed, "{text}");
        assert!(text.contains("- line of y-umbr (y-umbr; 0000001"), "{text}");
        assert!(
            text.contains("  - title y-kid (y-kid, covered; 0000001"),
            "{text}"
        );
        assert!(!text.contains("Changed\n"), "{text}");
    }

    #[test]
    fn version_bumps_are_a_named_rule() {
        assert!(is_version_bump("Bump version to 0.0.40."));
        assert!(!is_version_bump("Fix: bump version parsing"));
    }
}
