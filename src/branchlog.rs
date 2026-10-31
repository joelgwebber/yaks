//! `yaks commits`: the commits on the branches a yak's notes record
//! (yaks-4728, design yaks-a3fc).
//!
//! A worker pushes its branch and records it in a note (`--meta kind=checkpoint
//! --meta branch=<b> --meta commit=<sha>`, optionally `remote=` and `base=`).
//! This reads those notes and asks the code repository what the branch holds:
//! `git log <base>..<remote>/<branch>`, with each recorded checkpoint marked.
//!
//! Fetching is explicit: only `fetch = true` touches the network. Otherwise the
//! remote-tracking ref that is already there is used, which is only as fresh as
//! the last fetch. A failure on one branch is reported on that branch and never
//! stops the others.

use crate::farm::{CommitRef, git_commit_ref, git_commits};
use crate::store::NoteEntry;
use std::path::Path;
use std::process::Command;

/// Where a recorded checkpoint commit is relative to the fetched branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointState {
    /// In `base..branch`: listed with the branch's commits and marked there.
    Listed,
    /// Reachable from the branch but already in the base (landed or merged).
    InBase,
    /// Not reachable from the branch: rewritten, or never pushed.
    NotOnBranch,
}

impl CheckpointState {
    /// The JSON `state` value.
    pub fn as_str(self) -> &'static str {
        match self {
            CheckpointState::Listed => "on_branch",
            CheckpointState::InBase => "in_base",
            CheckpointState::NotOnBranch => "not_on_branch",
        }
    }
}

/// A commit a note recorded for this branch.
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// The sha as the note wrote it.
    pub commit: String,
    pub ts: String,
    pub actor: Option<String>,
    pub state: CheckpointState,
}

/// One recorded branch of a yak, and what the code repository says about it.
#[derive(Debug, Clone)]
pub struct BranchCommits {
    pub branch: String,
    pub remote: String,
    /// `<remote>/<branch>`, the remote-tracking ref read.
    pub tracking: String,
    /// The base the log runs from, once resolved.
    pub base: Option<String>,
    /// `git fetch` ran and succeeded for this branch.
    pub fetched: bool,
    /// `--fetch` was asked and failed (no such remote branch, offline, ...).
    pub fetch_error: Option<String>,
    /// Why there is no log: no tracking ref, or no base.
    pub error: Option<String>,
    /// `base..tracking`, newest first.
    pub commits: Vec<CommitRef>,
    pub checkpoints: Vec<Checkpoint>,
}

impl BranchCommits {
    /// Whether `c` is a listed commit a checkpoint note recorded.
    pub fn is_checkpoint(&self, c: &CommitRef) -> bool {
        self.checkpoints
            .iter()
            .any(|k| k.state == CheckpointState::Listed && matches_sha(c, &k.commit))
    }
}

fn matches_sha(c: &CommitRef, recorded: &str) -> bool {
    recorded.len() >= 7 && c.sha.starts_with(&recorded.to_ascii_lowercase())
}

/// The branches recorded in `notes` (a note with a `branch` key, except the
/// `landed` notes `yaks land` writes, whose branch is where the work landed),
/// each looked up in the code repository at `top`.
pub fn gather(top: &Path, notes: &[NoteEntry], fetch: bool) -> Vec<BranchCommits> {
    let mut groups: Vec<((String, String), Vec<&NoteEntry>)> = Vec::new();
    for n in notes {
        if n.kind() == Some("landed") {
            continue;
        }
        let Some(branch) = n.get("branch").map(str::trim).filter(|b| !b.is_empty()) else {
            continue;
        };
        let remote = n
            .get("remote")
            .map(str::trim)
            .filter(|r| !r.is_empty())
            .unwrap_or("origin");
        let key = (remote.to_string(), branch.to_string());
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, v)) => v.push(n),
            None => groups.push((key, vec![n])),
        }
    }
    groups
        .into_iter()
        .map(|((remote, branch), notes)| one(top, &remote, &branch, &notes, fetch))
        .collect()
}

fn one(top: &Path, remote: &str, branch: &str, notes: &[&NoteEntry], fetch: bool) -> BranchCommits {
    let tracking = format!("{remote}/{branch}");
    let mut out = BranchCommits {
        branch: branch.to_string(),
        remote: remote.to_string(),
        tracking: tracking.clone(),
        base: None,
        fetched: false,
        fetch_error: None,
        error: None,
        commits: Vec::new(),
        checkpoints: Vec::new(),
    };
    if remote.starts_with('-') || branch.starts_with('-') {
        out.error = Some("not a usable remote or branch name".to_string());
        return out;
    }
    let full = format!("refs/remotes/{tracking}");
    if fetch {
        let refspec = format!("+refs/heads/{branch}:{full}");
        match git(top, &["fetch", "--quiet", remote, &refspec], true) {
            Ok(_) => out.fetched = true,
            Err(e) => out.fetch_error = Some(e),
        }
    }
    if git(
        top,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{full}^{{commit}}"),
        ],
        false,
    )
    .is_err()
    {
        out.error = Some(if fetch {
            format!("no remote-tracking ref {tracking}")
        } else {
            format!(
                "no remote-tracking ref {tracking} here yet (it has never been fetched; try --fetch)"
            )
        });
        return out;
    }
    // The latest note naming a base wins; else the remote's default branch.
    let want = notes
        .iter()
        .rev()
        .find_map(|n| n.get("base").map(str::trim).filter(|b| !b.is_empty()));
    let Some(base) = resolve_base(top, remote, want) else {
        out.error = Some(match want {
            Some(b) => format!("base {b} is not a ref in this repository"),
            None => "no base: no origin/HEAD and no `main` here (a note can name one with `--meta base=<ref>`)".to_string(),
        });
        return out;
    };
    match git_commits(top, &[&format!("{base}..{full}"), "--"]) {
        Ok(c) => out.commits = c,
        Err(e) => out.error = Some(e.to_string()),
    }
    out.base = Some(base);
    for n in notes {
        let Some(commit) = n.get("commit").map(str::trim).filter(|c| !c.is_empty()) else {
            continue;
        };
        let resolved = git_commit_ref(top, commit).map(|c| c.sha);
        let state = if out.commits.iter().any(|c| matches_sha(c, commit))
            || resolved
                .as_ref()
                .is_some_and(|r| out.commits.iter().any(|c| &c.sha == r))
        {
            CheckpointState::Listed
        } else if resolved
            .as_ref()
            .is_some_and(|r| git(top, &["merge-base", "--is-ancestor", r, &full], false).is_ok())
        {
            CheckpointState::InBase
        } else {
            CheckpointState::NotOnBranch
        };
        out.checkpoints.push(Checkpoint {
            commit: commit.to_string(),
            ts: n.ts.clone(),
            actor: n.actor.clone(),
            state,
        });
    }
    out
}

/// The base ref to log from: the note's own (as given, else under `remote`),
/// else what the remote's `HEAD` points at (`origin/main`), else a local `main`.
fn resolve_base(top: &Path, remote: &str, want: Option<&str>) -> Option<String> {
    let exists = |r: &str| {
        git(
            top,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                &format!("{r}^{{commit}}"),
            ],
            false,
        )
        .is_ok()
    };
    if let Some(b) = want {
        return [b.to_string(), format!("{remote}/{b}")]
            .into_iter()
            .find(|r| !r.starts_with('-') && exists(r));
    }
    ["origin", remote]
        .iter()
        .find_map(|r| {
            git(
                top,
                &[
                    "symbolic-ref",
                    "--quiet",
                    "--short",
                    &format!("refs/remotes/{r}/HEAD"),
                ],
                false,
            )
            .ok()
            .filter(|b| exists(b))
        })
        .or_else(|| exists("main").then(|| "main".to_string()))
}

/// Run git in `top`, returning trimmed stdout, or git's own message on failure.
/// `network` keeps git from stopping to ask for a password.
fn git(top: &Path, args: &[&str], network: bool) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(top)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE");
    if network {
        cmd.env("GIT_TERMINAL_PROMPT", "0");
    }
    let out = cmd.output().map_err(|e| format!("running git: {e}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        let msg = String::from_utf8_lossy(&out.stderr);
        Err(msg
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("; "))
    }
}
