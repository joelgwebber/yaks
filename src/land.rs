//! `yaks land <sha>` (yaks-e5fb, yaks-3bcd): land a Delta worker's commit by SHA.
//!
//! Delta applies a returning worker's result to the coordinator's working tree
//! and may leave `MERGE_HEAD` set, but that applied tree is not a trustworthy
//! merge (stale files, reverted files, unfinished edits). The commit is. So this
//! reports the pending state; when the worker's base is still an ancestor of
//! HEAD, computes the correct merged tree (`git merge-tree --write-tree`) and
//! lands it as ONE single-parent squash commit on HEAD (the worker's message and
//! author plus a `Squashed-from:` trailer), else `git cherry-pick` (history was
//! rewritten, or the merge would conflict); refuses when the working tree holds
//! changes the worker's commit does not explain unless `--force-discard`;
//! clears the applied state; lands; and checks the result equals the computed
//! tree. History stays a straight line: no merge commits (yaks-2355). Never
//! pushes. git stays a subprocess; it needs git 2.38+ (2.40+ to verify a SHA
//! that is already an ancestor).
//!
//! It runs in the code checkout the cwd is in and needs no farm. This is the
//! Rust port of the retired `yaks-coordinating-delta/land.sh`, whose selftest
//! scenarios are the tests below.

use anyhow::Context;
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Exit code: a working tree holding changes the worker's commit does not explain.
const EXIT_UNEXPLAINED: i32 = 3;
/// Exit code: an untracked leftover that differs from the worker's file.
const EXIT_UNTRACKED: i32 = 4;
/// Exit code: the commit (or cherry-pick) itself failed.
const EXIT_COMMIT: i32 = 5;
/// Exit code: the worker's commit is an ancestor of HEAD but its change is not in HEAD.
const EXIT_DROPPED: i32 = 6;
/// Exit code: the worker's commit stages yak files under a branch farm.
const EXIT_YAK_FILES: i32 = 7;

/// What `yaks land` was asked.
#[derive(Debug, Clone)]
pub struct Opts {
    pub sha: String,
    /// Report what would happen and stop; touches nothing.
    pub dry_run: bool,
    /// Land even when the working tree holds changes the commit does not explain.
    pub force_discard: bool,
}

/// A refusal or failure: the message for stderr and the exit code.
#[derive(Debug)]
pub struct Refusal {
    pub code: i32,
    pub msg: String,
}

impl From<anyhow::Error> for Refusal {
    fn from(e: anyhow::Error) -> Self {
        Refusal {
            code: 1,
            msg: format!("{e:#}"),
        }
    }
}

type R<T> = Result<T, Refusal>;

fn die<T>(code: i32, msg: impl Into<String>) -> R<T> {
    Err(Refusal {
        code,
        msg: msg.into(),
    })
}

/// How a run ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// `--dry-run`, or stopped at a refusal that a dry run only reports.
    DryRun,
    /// The change is in HEAD (landed now, or already there), in this checkout.
    Landed(PathBuf),
}

/// The `yaks land` entry point: run in the cwd, print to stdout, return the exit code.
pub fn cli(opts: &Opts) -> i32 {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("land: {e}");
            return 1;
        }
    };
    let mut out = std::io::stdout().lock();
    match run(&cwd, opts, &mut out) {
        Ok(Outcome::Landed(top)) => {
            if let Some(line) = doctor_line(&top) {
                let _ = writeln!(out, "doctor: {line}");
            }
            0
        }
        Ok(Outcome::DryRun) => 0,
        Err(r) => {
            let _ = out.flush();
            eprintln!("land: {}", r.msg);
            r.code
        }
    }
}

/// The last line of `yaks doctor`, when a farm resolves from the checkout. The
/// farm may live elsewhere (on its own branch, yaks-73f4), so ask yaks, not the tree.
fn doctor_line(top: &Path) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let yaks = |args: &[&str]| Command::new(&exe).args(args).current_dir(top).output().ok();
    if !yaks(&["path"])?.status.success() {
        return None;
    }
    let out = yaks(&["doctor"])?;
    let last = |b: &[u8]| {
        String::from_utf8_lossy(b)
            .lines()
            .rfind(|l| !l.trim().is_empty())
            .map(str::to_string)
    };
    last(&out.stdout).or_else(|| last(&out.stderr))
}

struct Say<'a>(&'a mut dyn Write);

impl Say<'_> {
    fn say(&mut self, s: impl AsRef<str>) {
        let _ = writeln!(self.0, "{}", s.as_ref());
    }
}

/// git in one checkout. The `GIT_*` location variables are removed so a hook
/// or an outer git cannot point it somewhere else.
struct Git {
    dir: PathBuf,
}

impl Git {
    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new("git");
        c.arg("-C")
            .arg(&self.dir)
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE");
        c
    }

    fn raw(&self, args: &[&str]) -> anyhow::Result<Output> {
        self.cmd(args)
            .output()
            .with_context(|| format!("running git {}", args.join(" ")))
    }

    /// Run with `input` on stdin and extra environment.
    fn raw_in(&self, args: &[&str], envs: &[(&str, &str)], input: &str) -> anyhow::Result<Output> {
        let mut child = self
            .cmd(args)
            .envs(envs.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("running git {}", args.join(" ")))?;
        child
            .stdin
            .take()
            .context("git stdin")?
            .write_all(input.as_bytes())
            .context("writing to git")?;
        Ok(child.wait_with_output()?)
    }

    fn success(&self, args: &[&str]) -> bool {
        self.raw(args).is_ok_and(|o| o.status.success())
    }

    /// Trimmed stdout when git succeeds.
    fn text(&self, args: &[&str]) -> Option<String> {
        let o = self.raw(args).ok()?;
        o.status
            .success()
            .then(|| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// Trimmed stdout; a failure is an error carrying git's stderr.
    fn must(&self, args: &[&str]) -> R<String> {
        let o = self.raw(args)?;
        if !o.status.success() {
            return die(
                1,
                format!(
                    "git {}: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&o.stderr).trim()
                ),
            );
        }
        Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
    }

    /// NUL-separated path list from `args` (a command run with `-z`).
    fn paths(&self, args: &[&str]) -> R<Vec<String>> {
        let o = self.raw(args)?;
        if !o.status.success() {
            return die(
                1,
                format!(
                    "git {}: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&o.stderr).trim()
                ),
            );
        }
        Ok(String::from_utf8_lossy(&o.stdout)
            .split('\0')
            .filter(|p| !p.is_empty())
            .map(str::to_string)
            .collect())
    }
}

/// Where the worker's change stands relative to a revision that already has the
/// worker's commit as an ancestor.
enum Presence {
    Present,
    /// Merging the change onto `rev` again would still change the tree.
    Missing,
    /// The merge conflicts (later commits edited the same lines); lines named.
    Conflicts(Vec<String>),
    /// git could not answer (too old for `merge-tree --merge-base`, or failed).
    Unknown(String),
}

/// Is the change `parent..full` already in `rev`'s tree? A three-way merge with the
/// worker's own base as the merge base changes nothing exactly when it is. (Without
/// `--merge-base` git would pick `full` itself as the base, and merging an
/// ancestor trivially returns `rev`'s tree: that check can never fail.)
fn presence(g: &Git, rev: &str, parent: &str, full: &str) -> R<Presence> {
    let base = format!("--merge-base={parent}");
    let o = g.raw(&[
        "merge-tree",
        "--write-tree",
        "--name-only",
        &base,
        rev,
        full,
    ])?;
    let stdout = String::from_utf8_lossy(&o.stdout).to_string();
    Ok(match o.status.code() {
        Some(0) => {
            let merged = stdout.lines().next().unwrap_or("").trim().to_string();
            let have = g.must(&["rev-parse", &format!("{rev}^{{tree}}")])?;
            if merged == have {
                Presence::Present
            } else {
                Presence::Missing
            }
        }
        Some(1) => Presence::Conflicts(
            stdout
                .lines()
                .filter(|l| l.starts_with("CONFLICT"))
                .map(str::to_string)
                .collect(),
        ),
        _ => Presence::Unknown(String::from_utf8_lossy(&o.stderr).trim().to_string()),
    })
}

/// The oldest commit after `full` (at most 50) whose tree lacks the change.
fn first_drop(g: &Git, parent: &str, full: &str) -> Option<String> {
    let range = format!("{full}..HEAD");
    let list = g.text(&[
        "rev-list",
        "--reverse",
        "--ancestry-path",
        "-n",
        "50",
        &range,
    ])?;
    list.lines()
        .find(|c| !matches!(presence(g, c, parent, full), Ok(Presence::Present)))
        .map(|c| {
            let short = g
                .text(&["rev-parse", "--short", c])
                .unwrap_or(c.to_string());
            let subject = g.text(&["log", "-1", "--format=%s", c]).unwrap_or_default();
            format!("{short} {subject}")
        })
}

/// Shallow boundaries only change an answer about a history that reaches one:
/// true when some commit listed in the shallow file is an ancestor of HEAD or of
/// the worker commit. A boundary elsewhere (a pinned commit Delta imported)
/// cannot, so it must not trigger the note.
fn shallow_matters(g: &Git, full: &str) -> bool {
    let Some(file) = g.text(&["rev-parse", "--git-path", "shallow"]) else {
        return false;
    };
    let file = g.dir.join(file);
    let Ok(text) = std::fs::read_to_string(file) else {
        return false;
    };
    text.lines()
        .map(str::trim)
        .filter(|c| !c.is_empty())
        .any(|c| {
            g.success(&["merge-base", "--is-ancestor", c, "HEAD"])
                || g.success(&["merge-base", "--is-ancestor", c, full])
        })
}

/// The paths `git status` reports changed or untracked, a rename counting both names.
fn dirty_paths(g: &Git) -> R<BTreeSet<String>> {
    let mut set = BTreeSet::new();
    let mut it = g
        .paths(&["status", "--porcelain=v1", "-z", "--untracked-files=all"])?
        .into_iter();
    while let Some(entry) = it.next() {
        let (xy, path) = entry.split_at(3.min(entry.len()));
        set.insert(path.to_string());
        if xy.contains('R') || xy.contains('C') {
            if let Some(orig) = it.next() {
                set.insert(orig);
            }
        }
    }
    Ok(set)
}

/// Land `opts.sha` in the checkout containing `dir`, narrating to `out`.
pub fn run(dir: &Path, opts: &Opts, out: &mut dyn Write) -> R<Outcome> {
    let mut s = Say(out);
    let g = Git {
        dir: dir.to_path_buf(),
    };
    if !g.success(&["rev-parse", "--git-dir"]) {
        return die(1, "not inside a git repository");
    }
    let Some(top) = g.text(&["rev-parse", "--show-toplevel"]) else {
        return die(1, "not inside a git working tree");
    };
    // Everything below is relative to the checkout's root, wherever the cwd was.
    let top = PathBuf::from(top);
    let g = Git { dir: top.clone() };

    let Some(full) = g.text(&[
        "rev-parse",
        "--verify",
        "-q",
        &format!("{}^{{commit}}", opts.sha),
    ]) else {
        return die(1, format!("no such commit: {}", opts.sha));
    };
    let short = g.must(&["rev-parse", "--short", &full])?;
    let subject = g.must(&["log", "-1", "--format=%s", &full])?;
    if !g.success(&["rev-parse", "--verify", "-q", "HEAD"]) {
        return die(1, "no HEAD");
    }

    let mut pending = Vec::new();
    if g.success(&["rev-parse", "-q", "--verify", "MERGE_HEAD"]) {
        pending.push("merge");
    }
    if g.success(&["rev-parse", "-q", "--verify", "CHERRY_PICK_HEAD"]) {
        pending.push("cherry-pick");
    }
    let had_pending = !pending.is_empty();
    s.say(format!("worker commit: {short} {subject}"));
    s.say(format!(
        "pending state: {}",
        if had_pending {
            pending.join("+")
        } else {
            "none".into()
        }
    ));
    if shallow_matters(&g, &full) {
        s.say("note: this repository is shallow (Delta marks the commits it imports from another machine so); the ancestry and merge answers below are lower bounds: a missing parent can make a real ancestor look like 'not an ancestor'. 'git fetch --unshallow origin' (or local) fixes it; Delta may mark more later.");
    }

    let parents = g.must(&["rev-list", "--parents", "-n", "1", &full])?;
    let Some(parent) = parents.split_whitespace().nth(1).map(str::to_string) else {
        if g.text(&["rev-parse", "--is-shallow-repository"]).as_deref() == Some("true") {
            return die(
                1,
                "the worker commit has no parent here because this repository is shallow; git fetch --unshallow origin (or local), then retry",
            );
        }
        return die(1, "the worker commit has no parent");
    };
    let parent_short = g.must(&["rev-parse", "--short", &parent])?;
    let worker_paths: BTreeSet<String> = g
        .paths(&[
            "diff-tree",
            "--no-renames",
            "--name-only",
            "-r",
            "-z",
            &parent,
            &full,
        ])?
        .into_iter()
        .collect();

    // yaks-7bfd: a worker never stages yak files under a branch farm (the farm
    // lives on its own branch, so HEAD tracks no `.yaks/`): they would land in code.
    let yak_files: Vec<&String> = worker_paths
        .iter()
        .filter(|p| p.starts_with(".yaks/"))
        .collect();
    if !yak_files.is_empty()
        && g.text(&["ls-tree", "-r", "--name-only", "HEAD", "--", ".yaks"])
            .is_some_and(|t| t.is_empty())
    {
        let list: Vec<String> = yak_files.iter().map(|p| format!("  {p}")).collect();
        return die(
            EXIT_YAK_FILES,
            format!(
                "the worker commit touches yak files, but HEAD tracks no `.yaks/` (the farm lives on its own branch); a worker must never stage them under a branch farm:\n{}",
                list.join("\n")
            ),
        );
    }

    // yaks-3bcd: a worker commit that is already an ancestor of HEAD is only
    // "landed" when its change is really in HEAD's tree. A commit made while Delta
    // held a pending merge can be a merge with the worker as a parent that dropped it.
    if g.success(&["merge-base", "--is-ancestor", &full, "HEAD"]) {
        match presence(&g, "HEAD", &parent, &full)? {
            Presence::Present => {
                s.say(format!(
                    "worker commit {short} is already an ancestor of HEAD and its change is in HEAD's tree"
                ));
                if had_pending {
                    s.say("note: the pending state is still set; nothing was cleared (finish or abort it yourself)");
                }
                let head = g.must(&["log", "--oneline", "-1"])?;
                s.say(format!("landed: {head}"));
                return Ok(Outcome::Landed(top));
            }
            Presence::Missing => {
                let culprit = first_drop(&g, &parent, &full)
                    .map(|c| format!("\nthe first commit after it that lacks the change: {c}"))
                    .unwrap_or_default();
                return die(
                    EXIT_DROPPED,
                    format!(
                        "worker commit {short} is an ancestor of HEAD but its change is missing from HEAD (a merge that dropped it, or a later revert); it was NOT landed{culprit}"
                    ),
                );
            }
            Presence::Conflicts(lines) => {
                let list: Vec<String> = lines.iter().map(|l| format!("  {l}")).collect();
                return die(
                    EXIT_DROPPED,
                    format!(
                        "worker commit {short} is an ancestor of HEAD but its change cannot be confirmed in HEAD: later commits edited the same lines:\n{}\ncompare `git diff {parent_short} {short}` with HEAD by hand",
                        list.join("\n")
                    ),
                );
            }
            Presence::Unknown(why) => {
                return die(
                    EXIT_DROPPED,
                    format!(
                        "worker commit {short} is an ancestor of HEAD, but git could not check that its change is in HEAD (`git merge-tree --merge-base` needs git 2.40+): {why}"
                    ),
                );
            }
        }
    }

    let mut squash = g.success(&["merge-base", "--is-ancestor", &parent, "HEAD"]);
    if squash {
        s.say(format!(
            "worker base {parent_short} is an ancestor of HEAD: squashing onto HEAD"
        ));
    } else {
        s.say(format!(
            "worker base {parent_short} is NOT an ancestor of HEAD (history was rewritten): cherry-pick"
        ));
    }

    let mut tree = String::new();
    if squash {
        let o = g.raw(&["merge-tree", "--write-tree", "--name-only", "HEAD", &full])?;
        let stdout = String::from_utf8_lossy(&o.stdout).to_string();
        match o.status.code() {
            Some(0) => {
                tree = stdout.lines().next().unwrap_or("").trim().to_string();
                s.say(format!(
                    "correct merged tree: {}",
                    tree.chars().take(12).collect::<String>()
                ));
            }
            Some(1) => {
                s.say("the merge would conflict, so landing by cherry-pick instead (you resolve the files named here, then 'git cherry-pick --continue'):");
                for l in stdout.lines().filter(|l| l.starts_with("CONFLICT")) {
                    s.say(format!("  {l}"));
                }
                squash = false;
            }
            _ => {
                return die(
                    1,
                    format!(
                        "git merge-tree --write-tree failed (it needs git 2.38+): {}",
                        String::from_utf8_lossy(&o.stderr).trim()
                    ),
                );
            }
        }
    }
    let mode = if squash { "squash" } else { "cherry-pick" };

    let dirty = dirty_paths(&g)?;
    let unexplained: Vec<&String> = dirty.difference(&worker_paths).collect();
    if !unexplained.is_empty() {
        s.say("changes in the working tree that the worker's commit does NOT explain:");
        for p in &unexplained {
            s.say(format!("  {p}"));
        }
        s.say("(your own uncommitted edits, or Delta's applied state reverting or resurrecting files)");
        if !opts.force_discard {
            if opts.dry_run {
                s.say("dry run: stopping here");
                return Ok(Outcome::DryRun);
            }
            return die(
                EXIT_UNEXPLAINED,
                "refusing to continue; commit or stash your edits, or rerun with --force-discard",
            );
        }
        s.say("--force-discard: tracked changes will be discarded");
    }
    if opts.dry_run {
        s.say(format!(
            "dry run: would clear the applied state and land {short} by {mode}"
        ));
        return Ok(Outcome::DryRun);
    }

    if had_pending {
        let _ = g.raw(&["merge", "--abort"]);
        let _ = g.raw(&["cherry-pick", "--abort"]);
    }
    // The tree and index back to HEAD (the applied state may be staged or not).
    if !g.success(&["reset", "-q", "--hard", "HEAD"]) {
        return die(1, "could not restore tracked files");
    }
    // Untracked leftovers of the applied state: remove only if identical to the worker's file.
    for p in &dirty {
        let path = top.join(p);
        if !path.exists() || g.success(&["ls-files", "--error-unmatch", "--", p]) {
            continue;
        }
        let theirs = g
            .raw(&["show", &format!("{full}:{p}")])
            .ok()
            .filter(|o| o.status.success());
        match theirs {
            Some(o) if std::fs::read(&path).is_ok_and(|mine| mine == o.stdout) => {
                let _ = std::fs::remove_file(&path);
            }
            _ => {
                return die(
                    EXIT_UNTRACKED,
                    format!(
                        "untracked {p} is not identical to the worker's version; resolve it by hand"
                    ),
                );
            }
        }
    }

    if squash && g.text(&["rev-parse", "HEAD^{tree}"]).as_deref() == Some(tree.as_str()) {
        // Landed before (a squash is a different commit, so the ancestor check above
        // cannot see it): an empty commit would only be noise.
        s.say("HEAD's tree already equals the computed merge: nothing to commit");
    } else if squash {
        let message = squash_message(&g, &full)?;
        let author = g.must(&["log", "-1", "--format=%an%n%ae%n%aI", &full])?;
        let mut a = author.lines();
        let (name, email, date) = (
            a.next().unwrap_or(""),
            a.next().unwrap_or(""),
            a.next().unwrap_or(""),
        );
        // The computed tree, committed once on top of HEAD as the worker (author and date kept).
        let o = g.raw_in(
            &["commit-tree", &tree, "-p", "HEAD"],
            &[
                ("GIT_AUTHOR_NAME", name),
                ("GIT_AUTHOR_EMAIL", email),
                ("GIT_AUTHOR_DATE", date),
            ],
            &message,
        )?;
        if !o.status.success() {
            return die(
                EXIT_COMMIT,
                format!(
                    "commit-tree failed: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                ),
            );
        }
        let new = String::from_utf8_lossy(&o.stdout).trim().to_string();
        if !g.success(&["merge", "--ff-only", "-q", &new]) {
            return die(
                EXIT_COMMIT,
                format!("could not fast-forward to the squash commit {new}"),
            );
        }
        if g.text(&["rev-parse", "HEAD^{tree}"]).as_deref() == Some(tree.as_str()) {
            s.say("result is identical to the computed merge");
        } else {
            s.say("WARNING: result differs from the computed merge tree");
        }
    } else {
        if !g.success(&["cherry-pick", &full]) {
            return die(
                EXIT_COMMIT,
                "cherry-pick stopped with conflicts; resolve, then 'git cherry-pick --continue', then run yaks doctor",
            );
        }
        s.say(format!("cherry-picked {short} onto HEAD"));
    }
    let head = g.must(&["log", "--oneline", "-1"])?;
    s.say(format!("landed: {head}"));
    Ok(Outcome::Landed(top))
}

/// The worker's message with `Squashed-from: <full sha>` as a trailer.
fn squash_message(g: &Git, full: &str) -> R<String> {
    let body = g.must(&["log", "-1", "--format=%B", full])?;
    let trailer = format!("Squashed-from: {full}");
    let input = format!("{body}\n");
    let o = g.raw_in(
        &["interpret-trailers", "--no-divider", "--trailer", &trailer],
        &[],
        &input,
    )?;
    if o.status.success() {
        return Ok(String::from_utf8_lossy(&o.stdout).to_string());
    }
    Ok(format!("{body}\n\n{trailer}\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch git repository (the old selftest's `setup`): `root`, then
    /// `base` (a.txt, c.txt), a `worker` branch with one commit changing a.txt and
    /// adding new/f.txt, and `main` checked out at `base`.
    struct Repo {
        dir: PathBuf,
        root: String,
        base: String,
        w: String,
    }

    impl Drop for Repo {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    fn temp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "yaks-land-{tag}-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// git in `dir` with a fixed identity (a CI runner has no global one).
    fn git(dir: &Path, args: &[&str]) -> String {
        let o = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .env_remove("GIT_INDEX_FILE")
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8_lossy(&o.stdout).trim().to_string()
    }

    fn write(dir: &Path, rel: &str, text: &str) {
        let p = dir.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn read(dir: &Path, rel: &str) -> String {
        std::fs::read_to_string(dir.join(rel))
            .unwrap()
            .trim()
            .to_string()
    }

    fn setup(tag: &str) -> Repo {
        setup_with(tag, &[])
    }

    /// `setup`, the worker commit also adding each of `extra` (path, text).
    fn setup_with(tag: &str, extra: &[(&str, &str)]) -> Repo {
        let d = temp(tag);
        git(&d, &["init", "-q", "-b", "main"]);
        // The repo's own identity, for the commits `yaks land` itself makes.
        git(&d, &["config", "user.name", "t"]);
        git(&d, &["config", "user.email", "t@t"]);
        write(&d, "root.txt", "root\n");
        git(&d, &["add", "-A"]);
        git(&d, &["commit", "-q", "-m", "root"]);
        let root = git(&d, &["rev-parse", "HEAD"]);
        write(&d, "a.txt", "a\n");
        write(&d, "c.txt", "c\n");
        git(&d, &["add", "-A"]);
        git(&d, &["commit", "-q", "-m", "base"]);
        let base = git(&d, &["rev-parse", "HEAD"]);
        git(&d, &["checkout", "-q", "-b", "worker"]);
        write(&d, "a.txt", "a2\n");
        write(&d, "new/f.txt", "n\n");
        for (p, t) in extra {
            write(&d, p, t);
        }
        git(&d, &["add", "-A"]);
        git(&d, &["commit", "-q", "-m", "worker change"]);
        let w = git(&d, &["rev-parse", "HEAD"]);
        git(&d, &["checkout", "-q", "main"]);
        Repo {
            dir: d,
            root,
            base,
            w,
        }
    }

    fn land(dir: &Path, sha: &str, dry_run: bool, force_discard: bool) -> (R<Outcome>, String) {
        let mut out = Vec::new();
        let r = run(
            dir,
            &Opts {
                sha: sha.to_string(),
                dry_run,
                force_discard,
            },
            &mut out,
        );
        (r, String::from_utf8_lossy(&out).to_string())
    }

    fn code(r: &R<Outcome>) -> i32 {
        match r {
            Ok(_) => 0,
            Err(e) => e.code,
        }
    }

    fn msg(r: &R<Outcome>) -> String {
        match r {
            Ok(_) => String::new(),
            Err(e) => e.msg.clone(),
        }
    }

    fn nparents(d: &Path) -> usize {
        git(d, &["rev-list", "--parents", "-n", "1", "HEAD"])
            .split_whitespace()
            .count()
            - 1
    }

    // 1. applied state with a stale untracked leftover; squash onto HEAD
    #[test]
    fn squashes_onto_a_moved_head_and_clears_the_applied_state() {
        let r = setup("squash");
        let d = &r.dir;
        write(d, "b.txt", "b\n");
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "main moves"]);
        let moved = git(d, &["rev-parse", "HEAD"]);
        // Delta-like: the worker's tree applied, MERGE_HEAD set, nothing staged
        // (a.txt modified, new/f.txt an untracked leftover).
        git(d, &["merge", "-q", "--no-commit", "--no-ff", &r.w]);
        git(d, &["read-tree", "HEAD"]);
        assert!(d.join(".git/MERGE_HEAD").exists(), "setup: MERGE_HEAD");

        let (res, out) = land(d, &r.w, false, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert!(out.contains("pending state: merge"), "{out}");
        assert_eq!(nparents(d), 1, "one single-parent commit (no merge commit)");
        assert_eq!(git(d, &["rev-parse", "HEAD^"]), moved);
        let body = git(d, &["log", "-1", "--format=%B"]);
        assert!(body.starts_with("worker change"), "{body}");
        assert!(
            body.lines().any(|l| l == format!("Squashed-from: {}", r.w)),
            "{body}"
        );
        assert_eq!(read(d, "a.txt"), "a2");
        assert_eq!(read(d, "new/f.txt"), "n");
        assert_eq!(read(d, "b.txt"), "b");
        assert!(!d.join(".git/MERGE_HEAD").exists(), "MERGE_HEAD left");
        assert_eq!(git(d, &["status", "--porcelain"]), "");
        assert!(out.contains("identical to the computed merge"), "{out}");
        assert!(out.contains("landed: "), "{out}");
    }

    #[test]
    fn the_squash_keeps_the_workers_author_and_date() {
        let r = setup("author");
        let d = &r.dir;
        // Re-author the worker commit.
        git(d, &["checkout", "-q", "worker"]);
        let o = Command::new("git")
            .arg("-C")
            .arg(d)
            .args([
                "commit",
                "-q",
                "--amend",
                "--no-edit",
                "--author=Wendy <w@w>",
                "--date=2020-01-02T03:04:05+00:00",
            ])
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .output()
            .unwrap();
        assert!(o.status.success());
        let w = git(d, &["rev-parse", "HEAD"]);
        git(d, &["checkout", "-q", "main"]);
        let (res, out) = land(d, &w, false, false);
        assert_eq!(code(&res), 0, "{out}");
        assert_eq!(
            git(d, &["log", "-1", "--format=%an <%ae> %aI"]),
            "Wendy <w@w> 2020-01-02T03:04:05Z"
        );
        assert_eq!(git(d, &["log", "-1", "--format=%cn"]), "t");
    }

    // 2. an unexplained uncommitted edit is refused, and kept
    #[test]
    fn an_unexplained_edit_is_refused_and_kept() {
        let r = setup("unexplained");
        write(&r.dir, "c.txt", "mine\n");
        let before = git(&r.dir, &["rev-parse", "HEAD"]);
        let (res, out) = land(&r.dir, &r.w, false, false);
        assert_eq!(code(&res), EXIT_UNEXPLAINED, "{res:?}\n{out}");
        assert_eq!(read(&r.dir, "c.txt"), "mine", "edit preserved");
        assert!(out.contains("  c.txt"), "names the file: {out}");
        assert_eq!(git(&r.dir, &["rev-parse", "HEAD"]), before);
    }

    // 3. --force-discard drops it and lands
    #[test]
    fn force_discard_drops_the_edit_and_lands() {
        let r = setup("force");
        write(&r.dir, "c.txt", "mine\n");
        let (res, out) = land(&r.dir, &r.w, false, true);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert_eq!(read(&r.dir, "c.txt"), "c");
        assert_eq!(read(&r.dir, "a.txt"), "a2");
        assert!(out.contains("--force-discard"), "{out}");
    }

    // 4. dry run changes nothing
    #[test]
    fn a_dry_run_changes_nothing() {
        let r = setup("dry");
        write(&r.dir, "c.txt", "mine\n");
        let before = git(&r.dir, &["rev-parse", "HEAD"]);
        let (res, out) = land(&r.dir, &r.w, true, false);
        assert_eq!(res.unwrap(), Outcome::DryRun, "{out}");
        assert!(out.contains("dry run: stopping here"), "{out}");
        assert_eq!(git(&r.dir, &["rev-parse", "HEAD"]), before);
        assert_eq!(read(&r.dir, "c.txt"), "mine");
        assert!(
            !out.contains("repository is shallow"),
            "no shallow note in a full repository: {out}"
        );

        // A clean tree: the dry run says what it would do and still changes nothing.
        git(&r.dir, &["checkout", "-q", "--", "."]);
        let (res, out) = land(&r.dir, &r.w, true, false);
        assert_eq!(res.unwrap(), Outcome::DryRun, "{out}");
        assert!(
            out.contains(&format!(
                "dry run: would clear the applied state and land {} by squash",
                &r.w[..7]
            )),
            "{out}"
        );
        assert_eq!(git(&r.dir, &["rev-parse", "HEAD"]), before);
    }

    // 5. worker base no longer an ancestor: cherry-pick
    #[test]
    fn a_rewritten_base_lands_by_cherry_pick() {
        let r = setup("rewritten");
        let d = &r.dir;
        git(d, &["reset", "-q", "--hard", &r.root]);
        write(d, "a.txt", "a\n");
        write(d, "c.txt", "c\n");
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "base (rewritten)"]);
        assert!(
            !Command::new("git")
                .arg("-C")
                .arg(d)
                .args(["merge-base", "--is-ancestor", &r.base, "HEAD"])
                .status()
                .unwrap()
                .success()
        );
        let (res, out) = land(d, &r.w, false, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert!(out.contains("cherry-pick"), "{out}");
        assert_eq!(read(d, "a.txt"), "a2");
        assert_eq!(nparents(d), 1);
    }

    // The merge would conflict: say so, land by cherry-pick, which stops for the user.
    #[test]
    fn a_conflicting_merge_falls_back_to_cherry_pick_and_names_the_files() {
        let r = setup("conflict");
        let d = &r.dir;
        write(d, "a.txt", "main edit\n");
        git(d, &["commit", "-q", "-am", "main edits a"]);
        let (res, out) = land(d, &r.w, false, false);
        assert_eq!(code(&res), EXIT_COMMIT, "{res:?}\n{out}");
        assert!(out.contains("the merge would conflict"), "{out}");
        assert!(out.contains("CONFLICT") && out.contains("a.txt"), "{out}");
        assert!(
            d.join(".git/CHERRY_PICK_HEAD").exists(),
            "left for the user to resolve"
        );
    }

    // Landing the same worker twice must not add an empty commit.
    #[test]
    fn landing_the_same_worker_twice_adds_no_empty_commit() {
        let r = setup("twice");
        let (res, out) = land(&r.dir, &r.w, false, false);
        assert_eq!(code(&res), 0, "{out}");
        let once = git(&r.dir, &["rev-parse", "HEAD"]);
        let (res, out) = land(&r.dir, &r.w, false, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert!(out.contains("nothing to commit"), "{out}");
        assert!(out.contains("landed: "), "{out}");
        assert_eq!(git(&r.dir, &["rev-parse", "HEAD"]), once);
    }

    // 6. a shallow repository gets the note (yaks-a398)
    #[test]
    fn a_shallow_repository_gets_the_note() {
        let r = setup("shallow");
        let c = temp("shallow-clone");
        let url = format!("file://{}", r.dir.display());
        git(&c, &["clone", "-q", "--depth", "1", &url, "c"]);
        let cd = c.join("c");
        git(&cd, &["fetch", "-q", "--depth", "2", "origin", "worker"]);
        assert_eq!(git(&cd, &["rev-parse", "--is-shallow-repository"]), "true");
        let (res, out) = land(&cd, &r.w, true, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert!(out.contains("repository is shallow"), "{out}");
        let _ = std::fs::remove_dir_all(&c);
    }

    // 7. a shallow boundary that no history reaches does not trigger the note
    #[test]
    fn an_unrelated_shallow_boundary_does_not_trigger_the_note() {
        let r = setup("shallow-other");
        let d = &r.dir;
        git(d, &["branch", "other", &r.root]);
        git(d, &["checkout", "-q", "other"]);
        write(d, "o.txt", "o\n");
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "other tip"]);
        git(d, &["checkout", "-q", "main"]);
        let c = temp("shallow-other-clone");
        let url = format!("file://{}", d.display());
        git(&c, &["clone", "-q", &url, "c"]);
        let cd = c.join("c");
        git(&cd, &["fetch", "-q", "--depth", "1", "origin", "other"]);
        assert_eq!(git(&cd, &["rev-parse", "--is-shallow-repository"]), "true");
        let (res, out) = land(&cd, &r.w, true, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert!(!out.contains("repository is shallow"), "{out}");
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn an_untracked_leftover_that_differs_from_the_workers_file_stops_the_land() {
        let r = setup("untracked");
        // Not the worker's content, at a path the worker adds.
        write(&r.dir, "new/f.txt", "someone else's\n");
        let (res, out) = land(&r.dir, &r.w, false, false);
        assert_eq!(code(&res), EXIT_UNTRACKED, "{res:?}\n{out}");
        assert!(msg(&res).contains("new/f.txt"), "{res:?}");
        assert_eq!(read(&r.dir, "new/f.txt"), "someone else's");
    }

    #[test]
    fn runs_from_a_subdirectory_of_the_checkout() {
        let r = setup("subdir");
        write(&r.dir, "sub/x.txt", "x\n");
        git(&r.dir, &["add", "-A"]);
        git(&r.dir, &["commit", "-q", "-m", "sub"]);
        git(&r.dir, &["read-tree", "HEAD"]);
        let (res, out) = land(&r.dir.join("sub"), &r.w, false, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert_eq!(read(&r.dir, "a.txt"), "a2");
    }

    #[test]
    fn an_unknown_commit_and_a_non_repository_are_errors() {
        let r = setup("unknown");
        let (res, _) = land(&r.dir, "deadbeefdeadbeef", false, false);
        assert_eq!(code(&res), 1);
        assert!(msg(&res).contains("no such commit"), "{res:?}");
        let plain = temp("nongit");
        let (res, _) = land(&plain, "HEAD", false, false);
        assert!(msg(&res).contains("not inside a git"), "{res:?}");
        let _ = std::fs::remove_dir_all(&plain);
    }

    // yaks-3bcd: an ancestor SHA that really is in HEAD still says landed ...
    #[test]
    fn an_already_landed_ancestor_says_landed_and_adds_no_commit() {
        let r = setup("ancestor-ok");
        let d = &r.dir;
        git(d, &["merge", "-q", "--ff-only", &r.w]);
        write(d, "later.txt", "later\n");
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "later"]);
        let head = git(d, &["rev-parse", "HEAD"]);
        let (res, out) = land(d, &r.w, false, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert!(out.contains("already an ancestor of HEAD"), "{out}");
        assert!(out.contains("landed: "), "{out}");
        assert_eq!(git(d, &["rev-parse", "HEAD"]), head, "no new commit");
    }

    // ... and the evil merge (the worker is a parent, its change was dropped) is refused.
    #[test]
    fn the_evil_merge_that_dropped_the_workers_change_is_refused() {
        let r = setup("evil");
        let d = &r.dir;
        write(d, "b.txt", "b\n");
        git(d, &["add", "-A"]);
        git(d, &["commit", "-q", "-m", "main moves"]);
        // A commit made while the merge was pending, with the tree reset to main's:
        // a merge commit with the worker as a parent that lacks the worker's change.
        git(d, &["merge", "-q", "--no-commit", "--no-ff", &r.w]);
        git(d, &["checkout", "-q", "HEAD", "--", "."]);
        git(d, &["clean", "-fdq"]);
        git(d, &["commit", "-q", "-m", "evil merge"]);
        let evil = git(d, &["rev-parse", "HEAD"]);
        assert_eq!(nparents(d), 2, "setup: a merge commit");
        assert_eq!(read(d, "a.txt"), "a", "setup: the change is absent");
        // The literal check yaks-3bcd described cannot see it:
        assert_eq!(
            git(d, &["merge-tree", "--write-tree", "HEAD", &r.w]),
            git(d, &["rev-parse", "HEAD^{tree}"]),
            "setup: merge-tree without --merge-base is vacuous here"
        );

        let (res, out) = land(d, &r.w, false, false);
        assert_eq!(code(&res), EXIT_DROPPED, "{res:?}\n{out}");
        assert!(msg(&res).contains("change is missing from HEAD"), "{res:?}");
        assert!(
            msg(&res).contains(&evil[..7]),
            "names the dropping commit: {res:?}"
        );
        assert!(!out.contains("landed: "), "{out}");
        assert_eq!(
            git(d, &["rev-parse", "HEAD"]),
            evil,
            "nothing was committed"
        );
        // A dry run refuses too.
        let (res, _) = land(d, &r.w, true, false);
        assert_eq!(code(&res), EXIT_DROPPED);
    }

    #[test]
    fn an_ancestor_whose_change_was_edited_over_later_cannot_be_confirmed() {
        let r = setup("ancestor-conflict");
        let d = &r.dir;
        git(d, &["merge", "-q", "--ff-only", &r.w]);
        write(d, "a.txt", "a3\n");
        git(d, &["commit", "-q", "-am", "edit it again"]);
        let (res, out) = land(d, &r.w, false, false);
        assert_eq!(code(&res), EXIT_DROPPED, "{res:?}\n{out}");
        assert!(msg(&res).contains("cannot be confirmed"), "{res:?}");
        assert!(msg(&res).contains("a.txt"), "{res:?}");
    }

    // yaks-7bfd: yak files in a worker commit under a branch farm are refused.
    #[test]
    fn a_worker_commit_with_yak_files_is_refused_when_head_tracks_none() {
        let r = setup_with("yakfiles", &[(".yaks/yaks/yaks/yaks-1.md", "x\n")]);
        let before = git(&r.dir, &["rev-parse", "HEAD"]);
        let (res, out) = land(&r.dir, &r.w, false, false);
        assert_eq!(code(&res), EXIT_YAK_FILES, "{res:?}\n{out}");
        assert!(msg(&res).contains(".yaks/yaks/yaks/yaks-1.md"), "{res:?}");
        assert_eq!(git(&r.dir, &["rev-parse", "HEAD"]), before);
        assert_eq!(read(&r.dir, "a.txt"), "a");
        // A dry run refuses too.
        let (res, _) = land(&r.dir, &r.w, true, false);
        assert_eq!(code(&res), EXIT_YAK_FILES);
    }

    #[test]
    fn yak_files_land_when_head_tracks_a_yaks_directory() {
        // A repo whose HEAD tracks `.yaks/` (a farm branch's own history, or a
        // project that commits a fixture farm): the worker's yak files are fine.
        let d = temp("yakfiles-tracked");
        git(&d, &["init", "-q", "-b", "main"]);
        git(&d, &["config", "user.name", "t"]);
        git(&d, &["config", "user.email", "t@t"]);
        write(&d, ".yaks/config.yaml", "schema: 1\n");
        write(&d, "a.txt", "a\n");
        git(&d, &["add", "-A"]);
        git(&d, &["commit", "-q", "-m", "base"]);
        git(&d, &["checkout", "-q", "-b", "worker"]);
        write(&d, ".yaks/yaks/yaks/yaks-1.md", "x\n");
        git(&d, &["add", "-A"]);
        git(&d, &["commit", "-q", "-m", "worker"]);
        let w = git(&d, &["rev-parse", "HEAD"]);
        git(&d, &["checkout", "-q", "main"]);
        let (res, out) = land(&d, &w, false, false);
        assert_eq!(code(&res), 0, "{res:?}\n{out}");
        assert!(d.join(".yaks/yaks/yaks/yaks-1.md").is_file());
        let _ = std::fs::remove_dir_all(&d);
    }
}
