//! `yaks init`: start a farm, or join another repository's, and install the
//! skills. Every farm is a **farm branch** (yaks-73f4): an orphan branch
//! holding only `.yaks/`, checked out once per machine in a worktree outside
//! every checkout ([`store::farm_home`]). Nothing is ever written into the
//! code checkout's tree but the skills.
//!
//! Two shapes:
//!
//! - **own farm** (no `--farm`): create the orphan branch and its worktree,
//!   or, when the branch already exists (locally or on `origin`), only add the
//!   worktree.
//! - **shared farm** (`--farm <checkout>`): record `git config yaks.farm` (and
//!   `yaks.herd`, from `--herd`) so discovery uses that other repository's farm
//!   branch (yaks-a99f). The other repository must already have one.
//!
//! Every step is idempotent and prints one line, done or already-done, so the
//! same command can be re-run, and a re-run with more flags adds only what is
//! missing. Nothing is ever changed silently: a setting that differs from
//! what is there is an error naming it.

use crate::branchfarm::{self, git, ref_exists};
use crate::{sheds, skills, store};
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Which bundled skills `--skills` installs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum SkillSet {
    /// No skills.
    None,
    /// yaks + yaks-tracker.
    Default,
    /// The default set plus the opt-in coordination group.
    Coordination,
}

/// The parsed `yaks init` flags.
pub struct Args {
    pub herd: Option<String>,
    pub kind: Option<String>,
    pub priority: Option<u8>,
    pub emacs: bool,
    pub branch: Option<String>,
    /// `--farm <checkout>`: use that repository's farm instead of starting one.
    pub farm: Option<String>,
    pub skills: Option<SkillSet>,
}

impl Args {
    fn farm_config(&self) -> store::InitConfig {
        let mut cfg = store::InitConfig::default();
        if let Some(p) = &self.herd {
            cfg.prefix = p.clone();
        }
        if let Some(k) = &self.kind {
            cfg.default_type = k.clone();
        }
        if let Some(p) = self.priority {
            cfg.default_priority = p;
        }
        if self.emacs {
            cfg.vim_mode = false;
        }
        cfg
    }
}

/// Prints one line per step and remembers whether anything changed.
struct Report {
    changed: bool,
}

impl Report {
    fn done(&mut self, line: impl AsRef<str>) {
        self.changed = true;
        println!("{}", line.as_ref());
    }
    fn already(&self, line: impl AsRef<str>) {
        println!("{}", line.as_ref());
    }
}

pub fn run(args: Args) -> Result<()> {
    let cwd = env::current_dir()?;
    let top = skills::git_toplevel(&cwd).with_context(|| {
        format!(
            "yaks init needs a git repository (run `git init` first): {} is not inside one. \
             A farm lives on a branch of it",
            cwd.display()
        )
    })?;
    if args.farm.is_some() {
        if args.branch.is_some() {
            bail!("--branch names this repository's own farm branch; it does not go with --farm");
        }
        if args.kind.is_some() || args.priority.is_some() || args.emacs {
            bail!(
                "--type, --priority and --emacs set a new farm's defaults; with --farm the farm \
                 already exists (edit its config.yaml)"
            );
        }
    }

    let mut rep = Report { changed: false };
    let farm_root = match &args.farm {
        Some(other) => ensure_shared(&top, other, args.herd.as_deref(), &mut rep)?,
        None => {
            // A repo that already uses another's farm is not started over.
            if let Some(other) = store::farm_repo(&top)? {
                bail!(
                    "this repository uses the farm of {} (`git config {}`); run \
                     `yaks init --farm {}` to re-check it, or `git config --unset {}` first to \
                     start its own",
                    other.display(),
                    store::FARM_REPO_KEY,
                    other.display(),
                    store::FARM_REPO_KEY
                );
            }
            ensure_branch_farm(&top, &args, &mut rep)?
        }
    };

    match args.skills.unwrap_or(SkillSet::Default) {
        SkillSet::None => println!("skills: none requested"),
        set => {
            let with: Vec<String> = match set {
                SkillSet::Coordination => vec!["coordination".to_string()],
                _ => Vec::new(),
            };
            let target = skills::resolve_target(None, false, &cwd);
            println!("skills: into {}", target.describe());
            for i in &skills::install(&target.dir, false, &skills::select(&with))? {
                if i.changed() {
                    rep.changed = true;
                }
                println!(
                    "  {}",
                    skills::render_installed(i, " (`yaks skills install --force` overwrites)")
                );
            }
        }
    }

    println!(
        "farm: {} (`yaks path` prints it from any checkout). Nothing is committed or pushed: \
         `yaks sync` publishes the farm, `yaks private` keeps it on this machine.",
        farm_root.display()
    );
    if rep.changed {
        println!("Create your first yak with: yaks create \"\u{2026}\"");
    } else {
        println!("Nothing to change: already set up this way.");
    }
    Ok(())
}

/// `--farm <checkout>`: point this repository at another's farm branch. The
/// other repository's farm must exist (it is resolved now, bootstrapping its
/// worktree if needed, so a typo fails here rather than at the next command).
fn ensure_shared(top: &Path, other: &str, herd: Option<&str>, rep: &mut Report) -> Result<PathBuf> {
    let dir = store::expand_path(&env::current_dir()?, other);
    let other_top = skills::git_toplevel(&dir)
        .with_context(|| format!("--farm {other}: {} is not a git checkout", dir.display()))?;
    let other_top = sheds::canonical(&other_top);
    if sheds::host(&other_top).map(|h| h.git_dir) == sheds::host(top).map(|h| h.git_dir) {
        bail!("--farm {other} is this same repository; run `yaks init` without --farm");
    }
    // Resolving from the other checkout finds (or bootstraps) its farm.
    let root =
        store::discover_bootstrapping(&other_top, None, store::farm_home, |l| println!("{l}"))
            .with_context(|| {
                format!("--farm {other}: that repository has no farm; run `yaks init` there first")
            })?
            .root;
    if let Some(have) = store::farm_repo(top)? {
        if sheds::canonical(&have) != other_top {
            bail!(
                "this repository already uses the farm of {} (`git config {}`); init will not \
                 repoint it. Change it with `git config {} <checkout>`",
                have.display(),
                store::FARM_REPO_KEY,
                store::FARM_REPO_KEY
            );
        }
        rep.already(format!(
            "config: {} = {} already set, unchanged",
            store::FARM_REPO_KEY,
            other_top.display()
        ));
    } else {
        git(
            top,
            &[
                "config",
                store::FARM_REPO_KEY,
                &other_top.display().to_string(),
            ],
        )?;
        rep.done(format!(
            "config: set {} = {}",
            store::FARM_REPO_KEY,
            other_top.display()
        ));
    }
    if let Some(h) = herd {
        match store::repo_config(top, "yaks.herd") {
            Some(have) if have == h => {
                rep.already(format!("config: yaks.herd = {h} already set, unchanged"))
            }
            Some(have) => bail!(
                "this repository's herd is already {have} (`git config yaks.herd`); init will \
                 not change it. Change it with `git config yaks.herd {h}`"
            ),
            None => {
                git(top, &["config", "yaks.herd", h])?;
                rep.done(format!("config: set yaks.herd = {h}"));
            }
        }
        if !store::read_config(&root)
            .known_herds()
            .iter()
            .any(|k| k == h)
        {
            store::declare_herds(&root, &[h.to_string()])?;
            rep.done(format!(
                "farm: declared herd {h} in {}",
                root.join("config.yaml").display()
            ));
        }
    }
    Ok(root)
}

/// Create the farm at `root` if absent, else leave it alone — and refuse flags
/// that would contradict its existing settings (init does not rewrite config).
fn ensure_farm(root: &Path, cfg: &store::InitConfig, args: &Args, rep: &mut Report) -> Result<()> {
    match store::init(root, cfg)? {
        store::InitOutcome::Created => rep.done(format!(
            "farm: created {} (herd {}, type {}, priority {})",
            root.display(),
            cfg.prefix,
            cfg.default_type,
            cfg.default_priority
        )),
        store::InitOutcome::AlreadyExists => {
            let have = store::read_config(root);
            let mut clashes = Vec::new();
            if let Some(h) = args.herd.as_ref().filter(|h| **h != have.prefix) {
                clashes.push(format!("--herd {h} (farm herd is {})", have.prefix));
            }
            if let Some(k) = args.kind.as_ref().filter(|k| **k != have.default_type) {
                clashes.push(format!(
                    "--type {k} (farm default is {})",
                    have.default_type
                ));
            }
            if let Some(p) = args.priority.filter(|p| *p != have.default_priority) {
                clashes.push(format!(
                    "--priority {p} (farm default is {})",
                    have.default_priority
                ));
            }
            if args.emacs && have.vim_mode {
                clashes.push("--emacs (farm uses vim keybindings)".to_string());
            }
            if !clashes.is_empty() {
                bail!(
                    "{} already exists with different settings: {}. init does not rewrite an \
                     existing farm's config.yaml; edit it by hand.",
                    root.display(),
                    clashes.join(", ")
                );
            }
            rep.already(format!(
                "farm: {} already exists, unchanged",
                root.display()
            ));
        }
    }
    Ok(())
}

/// Branch mode: the farm on an orphan branch, in a worktree at
/// [`store::farm_home`], found by discovery from every checkout of the
/// repository. Reuses the branch when it exists (a worktree is added if none
/// has it: [`branchfarm::bootstrap`]), else starts a new orphan branch holding
/// only `.yaks/`. Nothing is committed or pushed. Returns the farm root.
///
/// Every refusal is checked before the first change.
fn ensure_branch_farm(top: &Path, args: &Args, rep: &mut Report) -> Result<PathBuf> {
    let configured = store::configured_farm_branch(top);
    let branch = args.branch.clone().unwrap_or_else(|| configured.clone());
    if git(top, &["check-ref-format", "--branch", &branch]).is_err() {
        bail!("`{branch}` is not a valid branch name");
    }
    // Run inside the farm branch's own worktree (made by hand with `git
    // worktree add --orphan`, so still unborn), init fills it in. The branch
    // you are on is otherwise never a farm: a code branch has history.
    if git(top, &["symbolic-ref", "--short", "-q", "HEAD"]).is_ok_and(|b| b == branch)
        && ref_exists(top, &format!("refs/heads/{branch}"))
        && git(top, &["cat-file", "-e", &format!("{branch}:.yaks")]).is_err()
    {
        bail!("`{branch}` is the branch you are on, not a farm; pick another with --branch");
    }
    let cfg = args.farm_config();
    let local = ref_exists(top, &format!("refs/heads/{branch}"));
    let remote = ref_exists(top, &format!("refs/remotes/origin/{branch}"));
    let existing_wt = sheds::worktree_of_branch(top, &branch).filter(|w| w.is_dir());

    let worktree = if let Some(wt) = existing_wt {
        rep.already(format!(
            "branch: `{branch}` is checked out at {}, unchanged",
            wt.display()
        ));
        wt
    } else {
        // Creating a worktree is refused where it would be invisible to the
        // other checkouts of the repository (a Delta or --shared clone).
        let host = sheds::host(top).context("reading the repository")?;
        if host.via_alternates {
            let main = sheds::host_checkout(&host).map_or_else(
                || host.git_dir.display().to_string(),
                |p| p.display().to_string(),
            );
            bail!(
                "this checkout borrows another repository's objects (a Delta or --shared clone); a \
                 farm worktree made here would be invisible to that repository's other checkouts. \
                 Run `yaks init` in its main checkout: {main}"
            );
        }
        let wt = store::farm_home(top);
        if fs::symlink_metadata(&wt).is_ok() {
            bail!(
                "{} (the farm home) already exists; remove it, or add the worktree elsewhere \
                 yourself: `git -C {} worktree add <path> {branch}`",
                wt.display(),
                top.display()
            );
        }
        if local || remote {
            let tip = if local {
                branch.clone()
            } else {
                format!("origin/{branch}")
            };
            if git(top, &["cat-file", "-e", &format!("{tip}:.yaks")]).is_err() {
                bail!(
                    "the branch `{branch}` exists but holds no .yaks/, so it is not a farm; pick \
                     another name with --branch"
                );
            }
            let made = branchfarm::bootstrap(top, &branch, |_| wt.clone())?;
            rep.done(format!(
                "branch: added the worktree for the existing branch `{branch}` at {}",
                made.display()
            ));
            made
        } else {
            if let Some(parent) = wt.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            // Forget the record of a worktree whose directory was deleted, or
            // git refuses to check the (unborn) branch out again.
            git(top, &["worktree", "prune"])?;
            let wt_s = wt.to_string_lossy().into_owned();
            git(
                top,
                &["worktree", "add", "-q", "--orphan", "-b", &branch, &wt_s],
            )
            .context(
                "creating the orphan farm worktree (`git worktree add --orphan` needs git 2.42+)",
            )?;
            let wt = sheds::canonical(&wt);
            rep.done(format!(
                "branch: created the orphan branch `{branch}` at {}",
                wt.display()
            ));
            wt
        }
    };

    // Discovery must find the branch: record it unless the repository already
    // resolves to it (the default `yaks`, or an earlier setting).
    if configured != branch {
        git(top, &["config", "yaks.branch", &branch])?;
        rep.done(format!("config: set yaks.branch = {branch}"));
    }

    let root = worktree.join(".yaks");
    ensure_farm(&root, &cfg, args, rep)?;
    Ok(root)
}
