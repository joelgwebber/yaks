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
//!
//! `yaks release dok` (yaks-c708, design §4) renders a report as the changes
//! block of a release dok, `doks/releases/<version>.md`, and rewrites only
//! that block on a rerun: [`block`], [`plan`] and [`apply`]. `yaks check`
//! compares each release dok's block with what [`block`] would write now
//! ([`stale_block`]).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use crate::changeline::{self, Change};
use crate::doks::{self, Dok};
use crate::farm::Farm;
use crate::fields::Value as Fv;
use crate::locate::Space;
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

    /// The range as `resolve_range` reads it back: `A..B`, `..B` from the
    /// start of history. What a release dok's `range:` field holds.
    fn arg(&self) -> String {
        format!("{}..{}", self.from.as_deref().unwrap_or_default(), self.to)
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

/// The sections of a report in Keep a Changelog's order, then any
/// hand-written section none of them follows (sorted).
fn section_order(r: &Report) -> Vec<&str> {
    let mut sections: Vec<&str> = changeline::SECTIONS.to_vec();
    let mut extra: Vec<&str> = r
        .in_group(Group::Change)
        .filter_map(|i| i.change.as_ref().map(|c| c.section.as_str()))
        .filter(|s| !changeline::SECTIONS.contains(s))
        .collect();
    extra.sort_unstable();
    extra.dedup();
    sections.extend(extra);
    sections
}

/// The entries of section `s` that nest under no other entry.
fn top_entries<'a>(r: &'a Report, s: &str) -> Vec<&'a Item> {
    r.in_group(Group::Change)
        .filter(|i| i.under.is_none())
        .filter(|i| i.change.as_ref().is_some_and(|c| c.section == s))
        .collect()
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
    let sections = section_order(r);
    for s in sections {
        let top = top_entries(r, s);
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

/// The report for `range` of `repo`, over `tasks`: the commits are read from
/// git, and a yak's recorded `commit:` notes resolve through `space`.
fn report_in(tasks: &[Task], space: &Space, repo: &Path, range: Range) -> Result<Report> {
    let commits = commits_in(repo, &range)?;
    let recorded = |t: &Task| -> Vec<String> {
        if !t.body.contains("commit") {
            return Vec::new();
        }
        yakcommits::collect(&store::parse_notes(&t.body), &|l| space.read_url(l))
            .into_iter()
            .map(|r| r.commit)
            .collect()
    };
    Ok(build(range, tasks, &commits, &recorded))
}

/// Run `yaks release changes` from `cwd`: print the report, return the exit
/// status (1 with `check` when a shorn yak in the range has no change line and
/// no ancestor's line covers it).
pub fn cli(farm: &Farm, cwd: &Path, range: Option<&str>, json: bool, check: bool) -> Result<i32> {
    let Some(repo) = farm.code_repo(cwd) else {
        bail!("release changes reads a release from the code repository: run it in your checkout");
    };
    let range = resolve_range(&repo, range)?;
    let tasks = store::load(farm.root(), &crate::farm::EVERY)?;
    let space = Space::new(farm.root(), Some(repo.clone()));
    let report = report_in(&tasks, &space, &repo, range)?;
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

// ---- release doks --------------------------------------------------------

/// The line opening the generated block of a release dok.
pub const BLOCK_START: &str = "<!-- yaks:changes -->";
/// The line closing it.
pub const BLOCK_END: &str = "<!-- /yaks:changes -->";
/// The kind of a release dok (its schema is `schemas/release.md`).
pub const DOK_KIND: &str = "release";

/// The placeholder a new release dok holds for the person cutting the
/// release to replace with headlines.
const HEADLINES: &str = "<!-- Headlines: what matters in this release, with a screenshot if there is one (pin images with @<sha>). -->";

/// The schema of release doks, written the first time one is.
pub fn schema() -> crate::schemas::Define {
    let field = |name: &str, kind: &str, required: bool| crate::schemas::FieldSpec {
        name: name.into(),
        kind: kind.into(),
        required,
    };
    crate::schemas::Define {
        kind: DOK_KIND.into(),
        fields: vec![
            field("version", "text", true),
            field("tag", "text", false),
            field("date", "date", false),
            field("range", "text", false),
            field("yanked", "bool", false),
        ],
        title: Some("Release".into()),
        description: Some(format!(
            "One release: its headlines, then its changes between {BLOCK_START} and {BLOCK_END}, which `yaks release dok` rewrites from the yaks"
        )),
        stale_after: None,
        replace: false,
    }
}

/// The farm-relative path of the release dok of `version`.
pub fn dok_path(version: &str) -> String {
    format!("{}/releases/{version}.md", doks::DOKS_DIR)
}

/// A yak's file, as the farm-root link a markdown host follows.
fn yak_link(id: &str) -> String {
    format!("/{}/yaks/{id}.md", store::herd_of(id))
}

fn linked(text: &str, id: &str) -> String {
    format!("{text} ([{id}]({}))", yak_link(id))
}

/// One entry of the block, and the yaks nested in it as sub-bullets: a yak
/// with a line of its own shows it, a covered yak shows its title.
fn block_entry(r: &Report, i: &Item, depth: usize, out: &mut String) {
    let text = match (i.group, &i.change) {
        (Group::Change, Some(c)) => c.text.trim(),
        _ => i.task.title.as_str(),
    };
    out.push_str(&format!(
        "{}- {}\n",
        "  ".repeat(depth),
        linked(text, &i.task.id)
    ));
    let mut kids: Vec<&Item> = r
        .items
        .iter()
        .filter(|c| c.under.as_deref() == Some(i.task.id.as_str()))
        .collect();
    kids.sort_by_key(|k| oldest_first(k));
    for child in kids {
        block_entry(r, child, depth + 1, out);
    }
}

/// The order entries read in within a section: the oldest yak first (ids are
/// random, so their order says nothing), the id breaking a tie.
fn oldest_first(i: &Item) -> (Option<String>, String) {
    (i.task.created.clone(), i.task.id.clone())
}

/// The generated block of a release dok, markers included: the change lines
/// by section in Keep a Changelog's order (families nested), then the shorn
/// yaks with no line under `## Not described`. Open and dead yaks, yaks
/// recorded `--no-change` and commits naming no yak are for `yaks release
/// changes` and `yaks check`, not for readers of the release. Nothing in it
/// varies with the commits' hashes, so rewriting it twice changes nothing.
pub fn block(r: &Report) -> String {
    let mut out = format!("{BLOCK_START}\n");
    let mut any = false;
    for s in section_order(r) {
        let top = top_entries(r, s);
        if top.is_empty() {
            continue;
        }
        any = true;
        out.push_str(&format!("\n## {}\n\n", title_case(s)));
        let mut top = top;
        top.sort_by_key(|i| oldest_first(i));
        for i in top {
            block_entry(r, i, 0, &mut out);
        }
    }
    let gaps = r.gaps();
    if !gaps.is_empty() {
        any = true;
        out.push_str("\n## Not described\n\n");
        let mut gaps = gaps;
        gaps.sort_by_key(|i| oldest_first(i));
        for i in gaps {
            out.push_str(&format!(
                "- {} (no change line)\n",
                linked(&i.task.title, &i.task.id)
            ));
        }
    }
    if !any {
        out.push_str("\n_No change lines in this range._\n");
    }
    out.push_str(&format!("\n{BLOCK_END}\n"));
    out
}

/// The byte span of the block in `body`, markers included (the end marker's
/// line break too).
fn block_span(body: &str) -> Option<(usize, usize)> {
    let start = body.find(BLOCK_START)?;
    let end = start + body[start..].find(BLOCK_END)? + BLOCK_END.len();
    let end = if body[end..].starts_with('\n') {
        end + 1
    } else {
        end
    };
    Some((start, end))
}

/// What a release dok's own fields are derived from.
#[derive(Debug, Clone)]
pub struct Derived {
    pub title: String,
    pub version: String,
    /// The tag, when `version` is one.
    pub tag: Option<String>,
    /// The tag's date, else today (the farm's zone).
    pub date: String,
    /// The resolved range, `A..B`.
    pub range: String,
}

/// The dok to write for `d`: a new one (heading, headlines placeholder,
/// block), or `existing` with its derived fields (`range`, `tag`, and `date`
/// once tagged) brought up to date and its block rewritten, nothing else
/// touched. Does not write.
pub fn dok_for(root: &Path, existing: Option<Dok>, d: &Derived, block: &str) -> Result<Dok> {
    let path = dok_path(&d.version);
    let Some(mut dok) = existing else {
        let mut fields = vec![
            (
                "description".to_string(),
                Fv::Str(format!(
                    "What changed in {}: the headlines, then every change by section",
                    d.version
                )),
            ),
            (
                "created".into(),
                Fv::Plain(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()),
            ),
            ("version".into(), Fv::Plain(d.version.clone())),
        ];
        if let Some(t) = &d.tag {
            fields.push(("tag".into(), Fv::Plain(t.clone())));
        }
        fields.push(("date".into(), Fv::Plain(d.date.clone())));
        fields.push(("range".into(), Fv::Plain(d.range.clone())));
        let body = format!("# {}\n\n{HEADLINES}\n\n{block}", d.title);
        return doks::new_dok(root, &path, Some(DOK_KIND), Some(&d.title), &fields, &body);
    };
    if dok.kind() != DOK_KIND {
        bail!(
            "{} is a {} dok, not a release dok; move it aside to write the release",
            dok.path,
            dok.kind()
        );
    }
    let Some((a, b)) = block_span(&dok.body) else {
        bail!(
            "{} has no generated block: put the lines {BLOCK_START} and {BLOCK_END} where it goes, then rerun",
            dok.path
        );
    };
    dok.body = format!("{}{block}{}", &dok.body[..a], &dok.body[b..]);
    dok.fields.set("range", &Fv::Plain(d.range.clone()));
    if let Some(t) = &d.tag {
        dok.fields.set("tag", &Fv::Plain(t.clone()));
    }
    // An untagged release's date is when it was started; the tag's is when it
    // happened.
    if d.tag.is_some() || !dok.fields.contains("date") {
        dok.fields.set("date", &Fv::Plain(d.date.clone()));
    }
    Ok(dok)
}

/// Whether `version` is a tag of `repo`.
fn tag_exists(repo: &Path, version: &str) -> bool {
    git(
        repo,
        &[
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/tags/{version}^{{commit}}"),
        ],
    )
    .is_ok_and(|o| o.status.success())
}

/// The calendar day (in `zone`) of the commit `rev` names.
fn commit_day(repo: &Path, rev: &str, zone: crate::zone::Zone) -> Option<String> {
    let out = git(repo, &["log", "-1", "--format=%cI", rev, "--"]).ok()?;
    let when =
        chrono::DateTime::parse_from_rfc3339(String::from_utf8_lossy(&out.stdout).trim()).ok()?;
    Some(zone.date_of(when.with_timezone(&chrono::Utc)).to_string())
}

/// Run `yaks release dok`: write (or with `dry_run` print) the release dok of
/// `version`, over `range` or, without one, the version's own range (a tag:
/// the release before it up to it; not a tag yet: the latest release tag up
/// to `HEAD`).
pub fn dok_cli(
    farm: &Farm,
    cwd: &Path,
    version: &str,
    range: Option<&str>,
    dry_run: bool,
) -> Result<()> {
    let Some(repo) = farm.code_repo(cwd) else {
        bail!("release dok reads a release from the code repository: run it in your checkout");
    };
    let path = dok_path(version);
    if let Some(why) = doks::path_problem(&path) {
        bail!("{version}: not usable as a release dok name ({why})");
    }
    let tagged = tag_exists(&repo, version);
    let range = match (range, tagged) {
        (Some(r), _) => resolve_range(&repo, Some(r))?,
        (None, true) => resolve_range(&repo, Some(version))?,
        (None, false) => resolve_range(&repo, None)?,
    };
    let tasks = store::load(farm.root(), &crate::farm::EVERY)?;
    let space = Space::new(farm.root(), Some(repo.clone()));
    let report = report_in(&tasks, &space, &repo, range)?;
    let zone = farm.zone();
    let config = farm.config();
    let derived = Derived {
        title: match config.prefix_declared {
            true => format!("{} {version}", config.prefix),
            false => version.to_string(),
        },
        version: version.to_string(),
        tag: tagged.then(|| version.to_string()),
        date: tagged
            .then(|| commit_day(&repo, version, zone))
            .flatten()
            .unwrap_or_else(|| zone.today().to_string()),
        range: report.range.arg(),
    };
    let new_block = block(&report);
    let make = |existing: Option<Dok>| dok_for(farm.root(), existing, &derived, &new_block);
    if dry_run {
        let existing = farm.dok(&path)?;
        let verb = if existing.is_some() {
            "update"
        } else {
            "create"
        };
        let dok = make(existing)?;
        eprintln!(
            "dry run: would {verb} {path} (range {}); nothing written",
            derived.range
        );
        print!("{}", dok.render());
        return Ok(());
    }
    let saved = farm.release_dok_save(&path, &make)?;
    let gaps = report.gaps().len();
    println!(
        "{} {path} (range {})",
        match saved {
            Saved::Created => "Created",
            Saved::Updated => "Updated",
            Saved::Unchanged => "Already up to date:",
        },
        derived.range
    );
    if saved == Saved::Created {
        println!(
            "Replace the headlines placeholder above the block with what matters in this release."
        );
    }
    if gaps > 0 {
        println!(
            "{gaps} shorn yak{} {} no change line (listed under \"Not described\"): add one with `yaks update <id> --note \"<line>\" --meta kind=change`, then rerun.",
            if gaps == 1 { "" } else { "s" },
            if gaps == 1 { "has" } else { "have" },
        );
    }
    Ok(())
}

/// What [`dok_cli`] did to the dok.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Saved {
    Created,
    Updated,
    Unchanged,
}

/// Release doks whose generated block no longer matches their yaks, for
/// `yaks check`: `(dok path, version)` of each, and a note when the doks could
/// not be checked (no code checkout). Each dok is compared over the `range:`
/// it records. A release dok with no `range:`, or no block, is skipped: there
/// is nothing a rerun would be comparing against.
pub fn stale_blocks(
    farm_root: &Path,
    code: Option<&Path>,
    all_doks: &[Dok],
) -> (Vec<(String, String)>, Option<String>) {
    let releases: Vec<&Dok> = all_doks
        .iter()
        .filter(|d| {
            d.kind() == DOK_KIND && block_span(&d.body).is_some() && d.str_field("range").is_some()
        })
        .collect();
    if releases.is_empty() {
        return (Vec::new(), None);
    }
    let code = code.filter(|c| crate::gitrepo::commit_of(c, "HEAD").is_some());
    let Some(code) = code else {
        let n = releases.len();
        return (
            Vec::new(),
            Some(format!(
                "{n} release dok{} not checked against {}: no code checkout is reachable (run from one)",
                if n == 1 { "" } else { "s" },
                if n == 1 { "its yaks" } else { "their yaks" },
            )),
        );
    };
    let Ok(tasks) = store::load(farm_root, &crate::farm::EVERY) else {
        return (Vec::new(), None);
    };
    let space = Space::new(farm_root, Some(code.to_path_buf()));
    let mut stale = Vec::new();
    for d in releases {
        let range = d.str_field("range").unwrap_or_default();
        let Ok(range) = resolve_range(code, Some(&range)) else {
            continue;
        };
        let Ok(report) = report_in(&tasks, &space, code, range) else {
            continue;
        };
        let (Some((a, b)), now) = (block_span(&d.body), block(&report)) else {
            continue;
        };
        if d.body[a..b] != now {
            let version = d.str_field("version").unwrap_or_else(|| {
                let stem = d.path.rsplit('/').next().unwrap_or(&d.path);
                stem.strip_suffix(".md").unwrap_or(stem).to_string()
            });
            stale.push((d.path.clone(), version));
        }
    }
    (stale, None)
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

    #[test]
    fn a_release_dok_without_a_code_checkout_is_a_note_not_a_finding() {
        let dok = Dok::parse(
            "doks/releases/v1.md",
            &format!("---\ntype: release\nrange: v0..v1\n---\n{BLOCK_START}\nold\n{BLOCK_END}\n"),
        );
        let (stale, note) = stale_blocks(Path::new("/nonexistent-farm"), None, &[dok]);
        assert!(stale.is_empty());
        assert!(note.unwrap().contains("1 release dok not checked"));
        // No release dok: nothing to say.
        assert_eq!(
            stale_blocks(Path::new("/nonexistent-farm"), None, &[]),
            (vec![], None)
        );
    }

    #[test]
    fn the_block_lists_entries_nested_families_and_gaps_only() {
        let tasks = vec![
            task("y-fix", Status::Shorn, "bug", None, Some(LINE)),
            task("y-umbr", Status::Shorn, "feature", None, Some(LINE)),
            task("y-kid", Status::Shorn, "task", Some("y-umbr"), None),
            task("y-gap", Status::Shorn, "task", None, None),
            task("y-int", Status::Shorn, "task", None, Some(NONE)),
            task("y-open", Status::Shaving, "task", None, None),
        ];
        let commits = vec![commit(
            1,
            "x",
            "y-fix y-umbr y-kid y-gap y-int y-open stray",
        )];
        let b = block(&build(range(), &tasks, &commits, &|_| vec![]));
        let want = "<!-- yaks:changes -->\n\
\n## Added\n\n- line of y-umbr ([y-umbr](/y/yaks/y-umbr.md))\n  - title y-kid ([y-kid](/y/yaks/y-kid.md))\n\
\n## Fixed\n\n- line of y-fix ([y-fix](/y/yaks/y-fix.md))\n\
\n## Not described\n\n- title y-gap ([y-gap](/y/yaks/y-gap.md)) (no change line)\n\
\n<!-- /yaks:changes -->\n";
        assert_eq!(b, want);
    }

    #[test]
    fn only_the_block_between_the_markers_is_replaced() {
        let d = Derived {
            title: "t".into(),
            version: "v1".into(),
            tag: None,
            date: "2026-01-01".into(),
            range: "v0..HEAD".into(),
        };
        let text = format!(
            "---\ntype: release\nversion: v1\ndate: 2025-01-01\nrange: old\nnote: kept\n---\n# T\n\nprose\n\n{BLOCK_START}\nold\n{BLOCK_END}\n\ntail\n"
        );
        let dok = Dok::parse("doks/releases/v1.md", &text);
        let out = dok_for(Path::new("/x"), Some(dok), &d, "NEW\n").unwrap();
        assert!(
            out.body.starts_with("# T\n\nprose\n\nNEW\n\ntail\n"),
            "{}",
            out.body
        );
        assert_eq!(out.str_field("range").as_deref(), Some("v0..HEAD"));
        // Untagged: the date it already had stays; a hand field stays.
        assert_eq!(out.str_field("date").as_deref(), Some("2025-01-01"));
        assert_eq!(out.str_field("note").as_deref(), Some("kept"));
    }
}
