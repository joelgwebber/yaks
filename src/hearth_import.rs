//! `yaks import-hearth <repo>` (yaks-5550): copy a Hearth-layout repo's
//! knowledge into this farm.
//!
//! A Hearth data repo keeps its notes where a farm keeps them, under another
//! name, and speaks the same dok and source format (Hearth says wiki page):
//!
//! | In the repo | In the farm |
//! |---|---|
//! | `wiki/**` | `doks/**`, same paths |
//! | `raw/sources/<id>/` | `sources/<id>/`, byte for byte (but `source.md`'s `part_of` / `superseded_by`) |
//! | `schemas/*.md` | `schemas/` |
//! | `raw/sessions/**` | `sessions/**` (a top-level directory is read as a herd only when it holds `yaks/`, so `sessions/` is not one) |
//!
//! Everything else is left where it is: Hearth's harness side (`schedules/`,
//! `core/`, `instructions/`, `views/`, `.agents/`) is not knowledge, and
//! `notes/` or `projects/` are code, which links reach as `code:` links.
//!
//! Links are rewritten by byte range ([`crate::links::rewrite`]), so nothing
//! else in a dok changes: a `../` link (which the link grammar refuses)
//! becomes a farm-root link (`/doks/topics/x.md`, `/sources/<id>/x.png`), or
//! `code:<path>` when it points outside what is imported. Relative links
//! that stay below the dok, and every URL, are left as written. Frontmatter
//! references to `raw/sources/` and `raw/sessions/` (`sources:`) are rewritten
//! to the farm's names in place.
//!
//! The repo is only read. A file the farm already holds is skipped when it is
//! identical and a refusal when it differs (nothing is written then), so a
//! second run is a no-op. One commit holds the whole import.

use crate::doks::Dok;
use crate::farm::{Farm, Issue};
use crate::fields::Fields;
use crate::links::{self, Link};
use crate::store;
use anyhow::{Context, Result, bail};
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Top-level entries that belong to Hearth's harness, not to the knowledge.
const HARNESS: &[&str] = &[
    "schedules",
    "core",
    "instructions",
    "views",
    ".agents",
    "AGENTS.md",
    ".hearth-defaults.json",
];

/// Repo-relative prefixes of Hearth's folders and the farm folders they become.
const MAP: &[(&str, &str)] = &[
    // Hearth calls them wiki pages; the farm calls them doks.
    ("wiki", "doks"),
    ("raw/sources", "sources"),
    ("raw/sessions", "sessions"),
    ("schemas", "schemas"),
];

/// A pointer file git-lfs leaves when objects were not downloaded.
const LFS_MAGIC: &[u8] = b"version https://git-lfs.github.com/spec/v1";

/// What an import did, or on a dry run would do.
#[derive(Debug, Default)]
pub struct Report {
    pub repo: PathBuf,
    /// The repo's HEAD, when it is a git repository.
    pub head: Option<String>,
    /// The imported folders have uncommitted changes (they are copied as they
    /// are on disk).
    pub dirty: bool,
    pub dry_run: bool,
    pub pages: usize,
    /// Other files under Hearth's `wiki/` (images a dok links to).
    pub wiki_files: usize,
    pub sources: usize,
    pub source_files: usize,
    pub schemas: usize,
    pub sessions: usize,
    pub bytes: u64,
    /// Files already in the farm, identical.
    pub identical: usize,
    /// Farm paths written (or to write).
    pub written: Vec<String>,
    /// Farm paths that exist and differ: the import refuses and writes nothing.
    pub conflicts: Vec<String>,
    /// Links rewritten in dok bodies.
    pub links_rewritten: usize,
    /// Frontmatter references rewritten.
    pub fields_rewritten: usize,
    /// Links that now read `code:`: (document, was, now, exists in the repo).
    pub to_code: Vec<(String, String, String, bool)>,
    /// `../` links that could not be mapped: (document, link, why).
    pub unresolved: Vec<(String, String, String)>,
    /// Lines of prose that still name a `raw/` path: (document, line).
    pub prose: Vec<(String, usize)>,
    /// Files that are git-lfs pointers, not the content.
    pub lfs_pointers: Vec<String>,
    /// Top-level entries not imported, with why.
    pub skipped: Vec<(String, String)>,
    /// Doctor's findings after a real import.
    pub doctor: Option<Vec<Issue>>,
    pub commit: CommitState,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum CommitState {
    #[default]
    DryRun,
    /// Everything was already in the farm.
    Nothing,
    Committed(String),
    /// Written, but the farm is not a git branch, so there is nothing to
    /// commit to.
    NotBranch,
}

/// Where a file goes and how its content is made.
enum Content {
    /// The repo file's bytes as they are.
    Copy(PathBuf),
    Text(String),
}

struct Planned {
    rel: String,
    content: Content,
}

/// Link counters for one import, filled while documents are converted.
#[derive(Default)]
struct Acc {
    links: usize,
    fields: usize,
    to_code: Vec<(String, String, String, bool)>,
    unresolved: Vec<(String, String, String)>,
    prose: Vec<(String, usize)>,
}

/// Import the Hearth-layout repo at `repo` into `farm`.
pub fn run(farm: &Farm, repo: &Path, dry_run: bool) -> Result<Report> {
    let repo = repo
        .canonicalize()
        .with_context(|| format!("{}: no such directory", repo.display()))?;
    if !MAP.iter().any(|(from, _)| repo.join(from).is_dir()) {
        bail!(
            "{} has none of wiki/, raw/sources/, raw/sessions/ or schemas/: it is not a Hearth-layout repo",
            repo.display()
        );
    }
    let root = farm.root();
    let mut r = Report {
        head: git_out(&repo, &["rev-parse", "HEAD"]),
        dirty: git_out(
            &repo,
            &[
                "status",
                "--porcelain",
                "--",
                "wiki",
                "raw/sources",
                "raw/sessions",
                "schemas",
            ],
        )
        .is_some_and(|s| !s.is_empty()),
        repo: repo.clone(),
        dry_run,
        ..Report::default()
    };

    // Held across the writes and the commit, so no other command's per-change
    // commit can interleave. `lock_uncommitted`: this command makes its own.
    let _lock = if dry_run {
        None
    } else {
        if let Some((head, what)) = crate::changes::pending_operation(root)? {
            bail!(
                "a {what} is in progress in the farm ({head} is set); finish it before importing"
            );
        }
        Some(store::lock_uncommitted(root)?)
    };

    let acc = RefCell::new(Acc::default());
    let plan = plan(&repo, &mut r, &acc)?;
    let acc = acc.into_inner();
    r.links_rewritten = acc.links;
    r.fields_rewritten = acc.fields;
    r.to_code = acc.to_code;
    r.unresolved = acc.unresolved;
    r.prose = acc.prose;
    r.skipped = skipped(&repo);

    // Decide everything before writing anything.
    let mut todo: Vec<&Planned> = Vec::new();
    for p in &plan {
        match existing(root, p)? {
            Existing::Identical => r.identical += 1,
            Existing::Differs => r.conflicts.push(p.rel.clone()),
            Existing::Absent => todo.push(p),
        }
    }
    r.written = todo.iter().map(|p| p.rel.clone()).collect();
    if !r.conflicts.is_empty() || dry_run {
        return Ok(r);
    }

    for p in &todo {
        let dest = root.join(&p.rel);
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
        }
        match &p.content {
            Content::Copy(from) => {
                std::fs::copy(from, &dest)
                    .with_context(|| format!("copying {}", from.display()))?;
            }
            Content::Text(t) => store::write::atomic_write(&dest, t)?,
        }
    }
    r.commit = commit(root, &r)?;
    r.doctor = Some(farm.doctor(false)?);
    Ok(r)
}

enum Existing {
    Absent,
    Identical,
    Differs,
}

fn existing(root: &Path, p: &Planned) -> Result<Existing> {
    let dest = root.join(&p.rel);
    if !dest.exists() {
        return Ok(Existing::Absent);
    }
    let have = std::fs::read(&dest).with_context(|| format!("reading {}", dest.display()))?;
    let same = match &p.content {
        Content::Text(t) => have == t.as_bytes(),
        Content::Copy(from) => {
            have == std::fs::read(from).with_context(|| format!("reading {}", from.display()))?
        }
    };
    Ok(if same {
        Existing::Identical
    } else {
        Existing::Differs
    })
}

// ------------------------------------------------------------------ planning

/// Every file to import, in farm-path order, with its content made. Fills the
/// counts in `r`.
fn plan(repo: &Path, r: &mut Report, acc: &RefCell<Acc>) -> Result<Vec<Planned>> {
    let mut out = Vec::new();
    let mut source_dirs = BTreeSet::new();
    for (from, to) in MAP {
        for rel in walk(&repo.join(from)) {
            let src_rel = format!("{from}/{rel}");
            let farm_rel = format!("{to}/{rel}");
            let src = repo.join(&src_rel);
            let size = std::fs::metadata(&src).map(|m| m.len()).unwrap_or(0);
            r.bytes += size;
            if size < 200 && std::fs::read(&src).is_ok_and(|b| b.starts_with(LFS_MAGIC)) {
                r.lfs_pointers.push(src_rel.clone());
            }
            let is_md = rel.ends_with(".md");
            let content = match *to {
                "doks" | "sessions" if is_md => {
                    r.pages += usize::from(*to == "doks");
                    r.sessions += usize::from(*to == "sessions");
                    Content::Text(convert_dok(repo, &src_rel, &farm_rel, &src, acc)?)
                }
                "doks" => {
                    r.wiki_files += 1;
                    Content::Copy(src)
                }
                "sessions" => {
                    r.sessions += 1;
                    Content::Copy(src)
                }
                "schemas" => {
                    r.schemas += usize::from(is_md);
                    Content::Copy(src)
                }
                _ => {
                    // sources/<id>/...
                    r.source_files += 1;
                    if let Some((id, _)) = rel.split_once('/') {
                        source_dirs.insert(id.to_string());
                    }
                    if rel.ends_with("/source.md") && rel.matches('/').count() == 1 {
                        Content::Text(convert_source_md(&farm_rel, &src, acc)?)
                    } else {
                        Content::Copy(src)
                    }
                }
            };
            out.push(Planned {
                rel: farm_rel,
                content,
            });
        }
    }
    r.sources = source_dirs.len();
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(out)
}

/// Files below `dir` as `/`-separated paths from it, sorted; dot files and
/// directories are left out.
fn walk(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![(String::new(), dir.to_path_buf())];
    while let Some((rel, d)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else {
            continue;
        };
        for e in rd.flatten() {
            let Some(name) = e.file_name().to_str().map(String::from) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            let child = if rel.is_empty() {
                name
            } else {
                format!("{rel}/{name}")
            };
            let path = e.path();
            if path.is_dir() {
                stack.push((child, path));
            } else if path.is_file() {
                out.push(child);
            }
        }
    }
    out.sort();
    out
}

/// What of the repo's top level is not imported, and why.
fn skipped(repo: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let top_imported: BTreeSet<&str> = MAP
        .iter()
        .map(|(from, _)| from.split('/').next().unwrap())
        .collect();
    let mut names: Vec<String> = std::fs::read_dir(repo)
        .map(|rd| {
            rd.flatten()
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    for name in names {
        if name == ".git" || name == ".gitattributes" || name == ".gitignore" || name == ".hearth" {
            continue;
        }
        if HARNESS.contains(&name.as_str()) {
            out.push((name, "Hearth's harness side".to_string()));
        } else if name == "raw" {
            let mut sub: Vec<String> = std::fs::read_dir(repo.join("raw"))
                .map(|rd| {
                    rd.flatten()
                        .filter_map(|e| e.file_name().to_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            sub.sort();
            for s in sub {
                if s != "sources" && s != "sessions" && !s.starts_with('.') {
                    out.push((
                        format!("raw/{s}"),
                        "not knowledge the farm reads".to_string(),
                    ));
                }
            }
        } else if !top_imported.contains(name.as_str()) && !name.starts_with('.') {
            out.push((
                name,
                "stays in the repo; links to it become code: links".to_string(),
            ));
        }
    }
    out
}

// ------------------------------------------------------------ converting text

/// A dok or session review: body links and frontmatter references rewritten.
fn convert_dok(
    repo: &Path,
    src_rel: &str,
    farm_rel: &str,
    src: &Path,
    acc: &RefCell<Acc>,
) -> Result<String> {
    let text =
        std::fs::read_to_string(src).with_context(|| format!("reading {}", src.display()))?;
    let mut page = Dok::parse(farm_rel, &text);
    let (body, links) = links::rewrite(&page.body, |dest| {
        map_link(repo, src_rel, farm_rel, dest, &mut acc.borrow_mut())
    });
    let fields = rewrite_fields(&mut page.fields, |_| true);
    {
        let mut a = acc.borrow_mut();
        a.links += links;
        a.fields += fields;
        for (i, line) in body.lines().enumerate() {
            if line.contains("raw/sources/") || line.contains("raw/sessions/") {
                a.prose.push((farm_rel.to_string(), i + 1));
            }
        }
    }
    if links == 0 && fields == 0 {
        return Ok(text);
    }
    page.body = body;
    Ok(if page.has_frontmatter {
        page.render()
    } else {
        page.body
    })
}

/// A source's `source.md`: `part_of` and `superseded_by` name sources by their
/// repo path; the rest is Hearth's format and stays as it is.
fn convert_source_md(farm_rel: &str, src: &Path, acc: &RefCell<Acc>) -> Result<String> {
    let text =
        std::fs::read_to_string(src).with_context(|| format!("reading {}", src.display()))?;
    let mut page = Dok::parse(farm_rel, &text);
    let n = rewrite_fields(&mut page.fields, |k| k == "part_of" || k == "superseded_by");
    if n == 0 {
        return Ok(text);
    }
    acc.borrow_mut().fields += n;
    Ok(page.render())
}

/// In every frontmatter key `keep` accepts, rename the repo's `raw/sources/`
/// and `raw/sessions/` paths to the farm's, keeping each entry's formatting.
/// Returns how many references changed.
fn rewrite_fields(fields: &mut Fields, keep: impl Fn(&str) -> bool) -> usize {
    let keys: Vec<String> = fields
        .keys()
        .filter(|k| keep(k))
        .map(String::from)
        .collect();
    let mut total = 0;
    for k in keys {
        let Some(lines) = fields.raw(&k) else {
            continue;
        };
        let mut n = 0;
        let new: Vec<String> = lines
            .iter()
            .map(|l| {
                let (s, c) = rename_paths(l);
                n += c;
                s
            })
            .collect();
        if n > 0 {
            fields.set_raw(&k, new);
            total += n;
        }
    }
    total
}

/// `raw/sources/` -> `sources/` and `raw/sessions/` -> `sessions/` where the
/// path starts (after a space, quote, bracket or comma), not inside a longer
/// path such as `/tmp/raw/sources/x`.
fn rename_paths(line: &str) -> (String, usize) {
    let mut out = String::with_capacity(line.len());
    let mut n = 0;
    let mut rest = line;
    let mut prev: Option<char> = None;
    while !rest.is_empty() {
        let starts = prev.is_none_or(|c| matches!(c, ' ' | '"' | '\'' | '[' | ',' | '('));
        let hit = starts
            .then(|| {
                [("raw/sources/", "sources/"), ("raw/sessions/", "sessions/")]
                    .into_iter()
                    .find(|(from, _)| rest.starts_with(from))
            })
            .flatten();
        if let Some((from, to)) = hit {
            out.push_str(to);
            rest = &rest[from.len()..];
            prev = Some('/');
            n += 1;
            continue;
        }
        let c = rest.chars().next().unwrap();
        out.push(c);
        rest = &rest[c.len_utf8()..];
        prev = Some(c);
    }
    (out, n)
}

/// The new destination for the link `dest` written in the document at repo
/// path `src_rel` (now at farm path `farm_rel`), or `None` to leave it.
///
/// Only a `../` link changes: the link grammar refuses it, and what it
/// climbs to is mapped to where that file is now (see [`MAP`]), or to a
/// `code:` link when it lies outside what is imported.
fn map_link(
    repo: &Path,
    src_rel: &str,
    farm_rel: &str,
    dest: &str,
    acc: &mut Acc,
) -> Option<String> {
    let link = links::parse(dest, links::From::Dok(farm_rel), |_| false);
    if !matches!(link, Link::Upward(_)) {
        return None;
    }
    let (body, frag) = match dest.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (dest, None),
    };
    let decoded = links::percent_decode(body);
    if !decoded.split('/').any(|s| s == "..") {
        return None;
    }
    // Resolve against the document's folder in the repo.
    let mut parts: Vec<&str> = src_rel.split('/').collect();
    parts.pop();
    for seg in decoded.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    acc.unresolved.push((
                        farm_rel.to_string(),
                        dest.to_string(),
                        "climbs out of the repo".to_string(),
                    ));
                    return None;
                }
            }
            s => parts.push(s),
        }
    }
    let target = parts.join("/");
    let (new, code) = match MAP.iter().find_map(|(from, to)| {
        let rest = target.strip_prefix(from)?;
        (rest.is_empty() || rest.starts_with('/')).then(|| format!("{to}{rest}"))
    }) {
        Some(farm_path) => (
            format!("/{}", links::percent_encode_path(&farm_path)),
            false,
        ),
        None => (
            format!("code:{}", links::percent_encode_path(&target)),
            true,
        ),
    };
    let new = match frag {
        Some(f) => format!("{new}#{f}"),
        None => new,
    };
    if code {
        acc.to_code.push((
            farm_rel.to_string(),
            dest.to_string(),
            new.clone(),
            repo.join(&target).exists(),
        ));
    }
    Some(new)
}

// -------------------------------------------------------------------- commit

fn git_out(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Commit exactly the paths this import wrote, on the farm branch. A path
/// list on stdin keeps the command line short and takes names literally.
fn commit(root: &Path, r: &Report) -> Result<CommitState> {
    if r.written.is_empty() {
        return Ok(CommitState::Nothing);
    }
    if store::farm_branch(root).is_none() {
        return Ok(CommitState::NotBranch);
    }
    let actor = crate::actor::resolve_detail(None)
        .map(|(n, _)| n)
        .unwrap_or_default();
    let name = r
        .repo
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "repo".to_string());
    let short = r.head.as_deref().map(|h| &h[..h.len().min(7)]);
    let mut msg = format!(
        "yaks: import {} doks, {} sources, {} schemas, {} sessions from {name}{}",
        r.pages,
        r.sources,
        r.schemas,
        r.sessions,
        short.map(|s| format!(" @ {s}")).unwrap_or_default()
    );
    msg.push_str(&format!(
        "\n\nImported by `yaks import-hearth` from {}{}.\n{} links rewritten to farm paths, {} turned into code: links.",
        r.repo.display(),
        r.head
            .as_deref()
            .map(|h| format!(" (HEAD {h})"))
            .unwrap_or_default(),
        r.links_rewritten,
        r.to_code.len()
    ));
    if !actor.is_empty() {
        msg.push_str(&format!(
            "\n\n{}: {actor}",
            crate::autocommit::ACTOR_TRAILER
        ));
    }
    let mut paths = Vec::new();
    for p in &r.written {
        paths.extend_from_slice(p.as_bytes());
        paths.push(0);
    }
    let ident = crate::autocommit::identity_env(root, &actor);
    let run = |args: &[&str]| -> Result<()> {
        let mut c = Command::new("git");
        c.arg("-C")
            .arg(root)
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env("GIT_LITERAL_PATHSPECS", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in &ident {
            c.env(k, v);
        }
        let mut child = c.spawn().context("running git")?;
        child.stdin.take().unwrap().write_all(&paths)?;
        let out = child.wait_with_output()?;
        if !out.status.success() {
            bail!(
                "git {}: {} (the import is written in the farm; `yaks commit` commits it)",
                args[0],
                String::from_utf8_lossy(&out.stderr).trim()
            );
        }
        Ok(())
    };
    run(&["add", "--pathspec-from-file=-", "--pathspec-file-nul"])?;
    run(&[
        "commit",
        "--only",
        "-q",
        "--no-verify",
        "-m",
        &msg,
        "--pathspec-from-file=-",
        "--pathspec-file-nul",
    ])?;
    Ok(CommitState::Committed(
        git_out(root, &["rev-parse", "--short", "HEAD"]).unwrap_or_default(),
    ))
}

// -------------------------------------------------------------------- report

impl Report {
    /// The text `yaks import-hearth` prints.
    pub fn render(&self) -> String {
        use std::fmt::Write;
        let mut s = String::new();
        let verb = if self.dry_run {
            "would import"
        } else {
            "imported"
        };
        let head = self
            .head
            .as_deref()
            .map(|h| format!(" (HEAD {})", &h[..h.len().min(7)]))
            .unwrap_or_default();
        let _ = writeln!(s, "{verb} from {}{head}", self.repo.display());
        if self.dirty {
            let _ = writeln!(
                s,
                "note: the imported folders have uncommitted changes; they are copied as they are on disk"
            );
        }
        let mib = self.bytes as f64 / (1024.0 * 1024.0);
        let _ = writeln!(
            s,
            "  wiki/ -> doks/  {} doks, {} other files",
            self.pages, self.wiki_files
        );
        let _ = writeln!(
            s,
            "  sources/    {} sources, {} files",
            self.sources, self.source_files
        );
        let _ = writeln!(s, "  schemas/    {} schemas", self.schemas);
        let _ = writeln!(s, "  sessions/   {} files", self.sessions);
        let _ = writeln!(s, "  {mib:.1} MiB in all");
        let _ = writeln!(
            s,
            "{} file(s) {}, {} already in the farm and identical",
            self.written.len(),
            if self.dry_run { "to write" } else { "written" },
            self.identical
        );
        if !self.conflicts.is_empty() {
            let _ = writeln!(
                s,
                "\nREFUSED: {} file(s) exist in the farm with other content; nothing was written:",
                self.conflicts.len()
            );
            for c in &self.conflicts {
                let _ = writeln!(s, "  {c}");
            }
        }
        let _ = writeln!(
            s,
            "\nlinks: {} rewritten in dok bodies, {} frontmatter reference(s) renamed, {} now code: links",
            self.links_rewritten,
            self.fields_rewritten,
            self.to_code.len()
        );
        for (doc, was, now, exists) in &self.to_code {
            let missing = if *exists { "" } else { "  (not in the repo)" };
            let _ = writeln!(s, "  {doc}: ({was}) -> ({now}){missing}");
        }
        for (doc, link, why) in &self.unresolved {
            let _ = writeln!(s, "  left: {doc}: ({link}) {why}");
        }
        if !self.prose.is_empty() {
            let _ = writeln!(
                s,
                "\nprose naming a raw/ path (not a link, left as written): {} line(s)",
                self.prose.len()
            );
            for (doc, line) in self.prose.iter().take(10) {
                let _ = writeln!(s, "  {doc}:{line}");
            }
            if self.prose.len() > 10 {
                let _ = writeln!(s, "  ... and {} more", self.prose.len() - 10);
            }
        }
        if !self.lfs_pointers.is_empty() {
            let _ = writeln!(
                s,
                "\nwarning: {} file(s) are git-lfs pointers, not their content (run `git lfs pull` in the repo and import again):",
                self.lfs_pointers.len()
            );
            for p in self.lfs_pointers.iter().take(10) {
                let _ = writeln!(s, "  {p}");
            }
        }
        if !self.skipped.is_empty() {
            let _ = writeln!(s, "\nnot imported:");
            for (name, why) in &self.skipped {
                let _ = writeln!(s, "  {name}: {why}");
            }
        }
        match &self.doctor {
            None => {
                let _ = writeln!(s, "\ndoctor: not run (nothing was written)");
            }
            Some(issues) if issues.is_empty() => {
                let _ = writeln!(s, "\ndoctor: all clear");
            }
            Some(issues) => {
                let _ = writeln!(s, "\ndoctor: {} issue(s) after the import", issues.len());
                let mut kinds: Vec<&str> = issues.iter().map(|i| i.kind.code()).collect();
                kinds.dedup();
                for k in kinds {
                    let of: Vec<&Issue> = issues.iter().filter(|i| i.kind.code() == k).collect();
                    let _ = writeln!(s, "  {k}: {}", of.len());
                    for i in of.iter().take(5) {
                        let _ = writeln!(s, "    {}", i.message);
                    }
                    if of.len() > 5 {
                        let _ = writeln!(s, "    ... and {} more", of.len() - 5);
                    }
                }
            }
        }
        match &self.commit {
            CommitState::DryRun => {
                let _ = writeln!(
                    s,
                    "\n(dry run: nothing written; run without --dry-run to import)"
                );
            }
            CommitState::Nothing => {
                let _ = writeln!(s, "\nnothing to import: the farm already holds all of it");
            }
            CommitState::Committed(sha) => {
                let _ = writeln!(
                    s,
                    "\ncommitted {sha} on the farm branch (one commit); the repo is untouched"
                );
            }
            CommitState::NotBranch => {
                let _ = writeln!(
                    s,
                    "\nwritten, not committed: the farm is not a git branch (`yaks commit` when it is)"
                );
            }
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_with(files: &[&str]) -> PathBuf {
        let d = std::env::temp_dir().join(format!("yaks-hi-unit-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        for f in files {
            let p = d.join(f);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, "x").unwrap();
        }
        d
    }

    #[test]
    fn up_links_map_to_the_farm_or_to_code() {
        let repo = repo_with(&["projects/app/README.md"]);
        let mut acc = Acc::default();
        let mut m = |doc: &str, farm: &str, dest: &str| map_link(&repo, doc, farm, dest, &mut acc);
        let (d, f) = ("wiki/topics/a.md", "doks/topics/a.md");
        assert_eq!(
            m(d, f, "../projects/b.md#intro").as_deref(),
            Some("/doks/projects/b.md#intro")
        );
        assert_eq!(
            m(d, f, "../../raw/sources/2026-01-01-abcd1234/shot%201.png").as_deref(),
            Some("/sources/2026-01-01-abcd1234/shot%201.png")
        );
        assert_eq!(
            m(d, f, "../../schemas/x.md").as_deref(),
            Some("/schemas/x.md")
        );
        assert_eq!(
            m(d, f, "../../projects/app/README.md").as_deref(),
            Some("code:projects/app/README.md")
        );
        assert_eq!(m(d, f, "../../../way/out.md"), None);
        // Left as written: below the dok, farm-root, URLs, anchors.
        for keep in [
            "b.md",
            "sub/b.md#x",
            "/wiki/x.md",
            "https://x.org/../y",
            "#top",
        ] {
            assert_eq!(m(d, f, keep), None, "{keep}");
        }
        // A session review sits in raw/sessions and lands in sessions/.
        assert_eq!(
            m(
                "raw/sessions/2026/10/s.md",
                "sessions/2026/10/s.md",
                "../../../sources/2026-01-01-abcd1234/source.md"
            )
            .as_deref(),
            Some("/sources/2026-01-01-abcd1234/source.md")
        );
        assert_eq!(acc.to_code.len(), 1);
        assert!(acc.to_code[0].3, "the target exists in the repo");
        assert_eq!(acc.unresolved.len(), 1);
    }

    #[test]
    fn repo_paths_in_frontmatter_are_renamed_in_place() {
        assert_eq!(
            rename_paths("  - raw/sources/2026-10-26-c27296b6/source.md"),
            ("  - sources/2026-10-26-c27296b6/source.md".to_string(), 1)
        );
        assert_eq!(
            rename_paths("sources: [raw/sources/a/source.md, \"raw/sessions/b.md\"]"),
            (
                "sources: [sources/a/source.md, \"sessions/b.md\"]".to_string(),
                2
            )
        );
        // Inside a longer path it is not the repo's folder.
        assert_eq!(
            rename_paths("resource: /tmp/raw/sources/x"),
            ("resource: /tmp/raw/sources/x".to_string(), 0)
        );
    }
}
