//! `yaks init`: create a farm and, when asked, set up everything its mode and
//! skills imply.
//!
//! Two shapes, by whether any of `--mode` / `--skills` / `--path` is given:
//!
//! - **plain** (none): exactly the historical behaviour — an in-tree, committed
//!   (team) farm in the cwd, refusing to touch an existing one — followed by a
//!   hint that skills are not installed yet.
//! - **setup** (any given): the mode's full setup, then the skills via
//!   [`skills::install`] (the same function `yaks skills install` uses). Every
//!   step is idempotent and prints one line, done or already-done, so the same
//!   command can be re-run, and a re-run with more flags adds only what is
//!   missing. Nothing is ever converted silently: a different `--mode` on an
//!   existing farm is an error naming what would have to change.
//!
//! Private and pointer farms keep their files out of shared history through
//! `.git/info/exclude` (never `.gitignore`, which is committed and would leak
//! the farm's existence).

use crate::branchfarm::{self, git, ref_exists};
use crate::{sheds, skills, store};
use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Where the farm lives and whether it is shared through the repo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Farm in `.yaks/`, committed with the code.
    Team,
    /// Farm in `.yaks/`, kept out of git via `.git/info/exclude`.
    Private,
    /// A `.yaks` pointer file to a farm elsewhere (`--path`), kept out of git.
    Pointer,
    /// The farm on its own orphan branch, in a worktree at [`store::farm_home`].
    Branch,
}

impl Mode {
    fn word(self) -> &'static str {
        match self {
            Mode::Team => "team",
            Mode::Private => "private",
            Mode::Pointer => "pointer",
            Mode::Branch => "branch",
        }
    }
}

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
    pub mode: Option<Mode>,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub skills: Option<SkillSet>,
}

impl Args {
    fn farm_config(&self, prefix: Option<&String>) -> store::InitConfig {
        let mut cfg = store::InitConfig::default();
        if let Some(p) = prefix {
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

pub fn run(args: Args) -> Result<()> {
    if args.mode.is_none() && args.skills.is_none() && args.path.is_none() && args.branch.is_none()
    {
        plain(&args)
    } else {
        setup(&args)
    }
}

// -- plain ------------------------------------------------------------------

fn plain(args: &Args) -> Result<()> {
    let cfg = args.farm_config(args.herd.as_ref());
    let cwd = env::current_dir()?;
    let root = cwd.join(".yaks");
    match store::init(&root, &cfg)? {
        store::InitOutcome::AlreadyExists => {
            eprintln!(
                "error: {} already exists \u{2014} leaving it untouched.",
                root.display()
            );
            std::process::exit(1);
        }
        store::InitOutcome::Created => {
            println!("Initialized empty yaks farm in {}", root.display());
            println!(
                "  herd {}  \u{b7}  default type {}  \u{b7}  default priority {}",
                cfg.prefix, cfg.default_type, cfg.default_priority
            );
            println!("Create your first yak with: yaks create --title \"\u{2026}\"");
            // Skills are optional, so tell people they still have to do it.
            if skills::git_toplevel(&cwd).is_some() {
                println!(
                    "Skills are not installed. Run `yaks init --skills default` to add them \
                     (safe to re-run; it leaves this farm alone), or `yaks skills install`."
                );
            } else {
                println!(
                    "Skills are not installed. Run `yaks skills install` to add them \
                     (this directory is not a git repo, so that goes to ~/.agents/skills)."
                );
            }
            Ok(())
        }
    }
}

// -- setup ------------------------------------------------------------------

/// What `./.yaks` is before init acts.
#[derive(Debug, PartialEq, Eq)]
enum Existing {
    Nothing,
    /// A farm directory; `private` when `.git/info/exclude` lists it.
    Farm {
        private: bool,
    },
    /// A pointer file.
    Pointer,
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

fn setup(args: &Args) -> Result<()> {
    let cwd = env::current_dir()?;
    let top = skills::git_toplevel(&cwd);
    // A branch farm belongs to the repository, so an in-tree farm is looked for
    // at the git top-level; every other mode works in the cwd.
    let farm_here = match (args.mode, &top) {
        (Some(Mode::Branch), Some(top)) => top.join(".yaks"),
        _ => cwd.join(".yaks"),
    };

    if args.path.is_some() && args.mode == Some(Mode::Branch) {
        let home = top.as_deref().map_or_else(
            || "its farm_home".to_string(),
            |t| store::farm_home(t).display().to_string(),
        );
        bail!(
            "--path only applies to `--mode pointer`; `--mode branch` puts the farm's worktree \
             at its farm home ({home}), not at a path you choose"
        );
    }
    if args.path.is_some() && args.mode != Some(Mode::Pointer) {
        bail!("--path only applies to `--mode pointer`");
    }
    if args.branch.is_some() && args.mode != Some(Mode::Branch) {
        bail!("--branch only applies to `--mode branch`");
    }
    if args.mode == Some(Mode::Pointer) && args.path.is_none() {
        bail!("`--mode pointer` needs `--path <dir>`: the directory that holds the farm");
    }

    let existing = existing_kind(&farm_here, top.as_deref())?;
    // Only `--skills` given: it asserts no mode, so an existing farm keeps its
    // own; a fresh one is a team farm.
    let mode = args.mode.unwrap_or(match existing {
        Existing::Farm { private: true } => Mode::Private,
        Existing::Pointer => Mode::Pointer,
        _ => Mode::Team,
    });
    refuse_conversion(mode, &existing, &farm_here)?;

    let skill_set = args.skills.unwrap_or(SkillSet::Default);
    let needs_git = |why: &str| -> Result<PathBuf> {
        top.clone().with_context(|| {
            format!(
                "{why} needs a git repository (run `git init` first): {} is not inside one",
                cwd.display()
            )
        })
    };
    let top_for_exclude = match mode {
        Mode::Team => None,
        Mode::Branch => Some(needs_git("`--mode branch`")?),
        m => Some(needs_git(&format!("`--mode {}`", m.word()))?),
    };
    let top_for_skills = match skill_set {
        SkillSet::None => None,
        _ => Some(top.clone().with_context(|| {
            format!(
                "installing skills project-locally needs a git repository (run `git init` first): \
                 {} is not inside one. `yaks skills install` installs into ~/.agents/skills instead",
                cwd.display()
            )
        })?),
    };

    let mut rep = Report { changed: false };
    println!("yaks init: {} mode", mode.word());

    // The farm, and where discovery will find it.
    // The exclude line goes in first, so the farm never shows up in `git status`.
    let farm_root: PathBuf = match mode {
        Mode::Branch => {
            let top = top_for_exclude.as_ref().expect("branch requires git");
            ensure_branch_farm(top, args, &mut rep)?
        }
        Mode::Team => {
            let cfg = args.farm_config(args.herd.as_ref());
            ensure_farm(&farm_here, &cfg, args, true, &mut rep)?
        }
        Mode::Private => {
            let top = top_for_exclude.as_ref().expect("private requires git");
            ensure_excluded(top, &exclude_line(top, &cwd, ".yaks"), "farm", &mut rep)?;
            let cfg = args.farm_config(args.herd.as_ref());
            ensure_farm(&farm_here, &cfg, args, true, &mut rep)?
        }
        Mode::Pointer => {
            let top = top_for_exclude.as_ref().expect("pointer requires git");
            ensure_excluded(top, &exclude_line(top, &cwd, ".yaks"), "pointer", &mut rep)?;
            ensure_pointer(&cwd, &farm_here, args, &mut rep)?
        }
    };

    // Skills, through the shared install.
    if let Some(top) = &top_for_skills {
        let with: Vec<String> = match skill_set {
            SkillSet::Coordination => vec!["coordination".to_string()],
            _ => Vec::new(),
        };
        let target = skills::resolve_target(None, false, &cwd);
        println!("skills: into {}", target.describe());
        let chosen = skills::select(&with);
        let installed = skills::install(&target.dir, false, &chosen)?;
        for i in &installed {
            if i.wrote {
                rep.changed = true;
            }
            println!(
                "  {}",
                skills::render_installed(i, " (`yaks skills install --force` overwrites)")
            );
        }
        if matches!(mode, Mode::Private | Mode::Pointer) {
            for s in &chosen {
                let line = format!("/.agents/skills/{}", s.name);
                ensure_excluded(top, &line, "skill", &mut rep)?;
            }
        }
    } else {
        println!("skills: none requested");
    }

    // Closing notes.
    match mode {
        Mode::Team => println!(
            "note: .yaks/ (and .agents/skills/, if installed) are ordinary project files \
             \u{2014} commit them to share them."
        ),
        Mode::Branch => println!(
            "note: the farm is the git branch above, checked out outside every checkout; \
             nothing is committed or pushed. `yaks sync` publishes it, `yaks private` \
             keeps it local. .agents/skills/ (if installed) are ordinary project files."
        ),
        Mode::Private | Mode::Pointer => println!(
            "note: a Delta or other worktree worker runs in a different checkout, where \
             discovery stops at that checkout's git top-level and will not find this farm. \
             Put YAKS_DIR={} in its brief.",
            farm_root.display()
        ),
    }
    if rep.changed {
        println!("Create your first yak with: yaks create --title \"\u{2026}\"");
    } else {
        println!("Nothing to change: already set up this way.");
    }
    Ok(())
}

/// Classify the `.yaks` entry in the cwd.
fn existing_kind(farm_here: &Path, top: Option<&Path>) -> Result<Existing> {
    if farm_here.is_dir() {
        let private = match top {
            Some(top) => {
                let cwd = farm_here.parent().unwrap_or(top);
                let line = exclude_line(top, cwd, ".yaks");
                read_exclude(top)?.lines().any(|l| l.trim() == line)
            }
            None => false,
        };
        Ok(Existing::Farm { private })
    } else if farm_here.is_file() {
        Ok(Existing::Pointer)
    } else {
        Ok(Existing::Nothing)
    }
}

/// A different `--mode` on an existing farm is never a silent conversion.
fn refuse_conversion(mode: Mode, existing: &Existing, farm_here: &Path) -> Result<()> {
    let here = farm_here.display();
    let msg = match (existing, mode) {
        (Existing::Nothing, _)
        | (Existing::Farm { private: false }, Mode::Team)
        | (Existing::Farm { private: true }, Mode::Private)
        | (Existing::Pointer, Mode::Pointer) => return Ok(()),
        // The remains of a farm (`.lock`, `.gitignore`) are not one.
        (Existing::Farm { .. }, Mode::Branch) if store::only_leftovers(farm_here) => {
            return Ok(());
        }
        (Existing::Farm { .. }, Mode::Branch) => format!(
            "{here} is an in-tree farm; `--mode branch` starts a NEW farm on its own branch. \
             To move this one onto a branch, keeping its history, run `yaks migrate --to-branch`"
        ),
        (Existing::Pointer, Mode::Branch) => format!(
            "{here} is a pointer file to a farm elsewhere; `--mode branch` would have to \
             replace it. Remove the pointer (and its line in .git/info/exclude) first if the \
             farm should live on a branch instead"
        ),
        (Existing::Farm { private: false }, Mode::Private) => format!(
            "{here} is a team farm (not listed in .git/info/exclude); `--mode private` \
             would have to add `.yaks` to .git/info/exclude and, if the farm is already \
             committed, remove it from the index (`git rm -r --cached .yaks`)"
        ),
        (Existing::Farm { private: true }, Mode::Team) => format!(
            "{here} is a private farm (listed in .git/info/exclude); `--mode team` would \
             have to remove that line from .git/info/exclude so the farm can be committed"
        ),
        (Existing::Farm { .. }, Mode::Pointer) => format!(
            "{here} is a farm directory; `--mode pointer` would have to move it to the \
             `--path` location and replace it with a pointer file"
        ),
        (Existing::Pointer, m) => format!(
            "{here} is a pointer file; `--mode {}` would have to replace it with a farm \
             directory (and drop its line from .git/info/exclude)",
            m.word()
        ),
    };
    bail!("{msg}. init never converts a farm silently; do that by hand, or drop `--mode`.")
}

/// Create the farm at `root` if absent, else leave it alone — and refuse flags
/// that would contradict its existing settings (init does not rewrite config).
/// `check_herd` is false for pointer farms, where `--herd` names the pointer's
/// herd rather than the farm's default.
fn ensure_farm(
    root: &Path,
    cfg: &store::InitConfig,
    args: &Args,
    check_herd: bool,
    rep: &mut Report,
) -> Result<PathBuf> {
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
            if check_herd {
                if let Some(h) = args.herd.as_ref().filter(|h| **h != have.prefix) {
                    clashes.push(format!("--herd {h} (farm herd is {})", have.prefix));
                }
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
    Ok(root.to_path_buf())
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
    if store::farm_branch(&top.join(".yaks")).is_some() {
        bail!(
            "{} is itself the farm's worktree; run `yaks init --mode branch` in a code checkout",
            top.display()
        );
    }
    if git(top, &["symbolic-ref", "--short", "-q", "HEAD"]).is_ok_and(|b| b == branch) {
        bail!("`{branch}` is the branch you are on, not a farm; pick another with --branch");
    }
    let cfg = args.farm_config(args.herd.as_ref());
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
                 Run `yaks init --mode branch` in its main checkout: {main}"
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
    ensure_farm(&root, &cfg, args, true, rep)?;
    Ok(root)
}

/// Pointer mode: the farm at `--path` (created if absent), then the pointer
/// file `./.yaks` naming it. The farm comes first so a pointer never dangles.
fn ensure_pointer(cwd: &Path, pointer: &Path, args: &Args, rep: &mut Report) -> Result<PathBuf> {
    let path = args.path.as_deref().expect("validated by setup");
    let target = store::expand_path(cwd, path);
    // `path:` may name the farm itself or a directory containing `.yaks`;
    // a new farm goes in `<dir>/.yaks` unless the dir is itself named `.yaks`.
    let root = match store::resolve_farm_root(&target) {
        Ok(root) => root,
        Err(_) if target.is_file() => {
            bail!("--path {path} is a file, not a directory");
        }
        Err(_) if target.file_name().is_some_and(|n| n == ".yaks") => target.clone(),
        Err(_) => target.join(".yaks"),
    };
    // `--herd` is this repo's herd inside the (possibly shared) farm; a farm
    // created here starts with it as its default.
    let cfg = args.farm_config(args.herd.as_ref());
    let root = ensure_farm(&root, &cfg, args, false, rep)?;
    if let Some(h) = &args.herd {
        if !store::read_config(&root).known_herds().contains(h) {
            store::declare_herds(&root, std::slice::from_ref(h))?;
            rep.done(format!(
                "farm: declared herd {h} in {}",
                root.join("config.yaml").display()
            ));
        }
    }

    if pointer.is_file() {
        let (have_path, have_herd) = store::parse_pointer(pointer)?;
        let mut clashes = Vec::new();
        if have_path != path {
            clashes.push(format!("path: {have_path} (you asked for {path})"));
        }
        match (&have_herd, &args.herd) {
            (Some(a), Some(b)) if a != b => {
                clashes.push(format!("herd: {a} (you asked for {b})"));
            }
            _ => {}
        }
        if !clashes.is_empty() {
            bail!(
                "{} already points elsewhere \u{2014} {}. init will not repoint it; edit or \
                 remove the file.",
                pointer.display(),
                clashes.join("; ")
            );
        }
        if have_herd.is_none() {
            if let Some(h) = &args.herd {
                let mut text = fs::read_to_string(pointer)?;
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                text.push_str(&format!("herd: {h}\n"));
                fs::write(pointer, text)
                    .with_context(|| format!("writing {}", pointer.display()))?;
                rep.done(format!(
                    "pointer: added `herd: {h}` to {}",
                    pointer.display()
                ));
                return Ok(root);
            }
        }
        rep.already(format!(
            "pointer: {} -> {path} already in place, unchanged",
            pointer.display()
        ));
    } else {
        let mut text = format!("path: {path}\n");
        if let Some(h) = &args.herd {
            text.push_str(&format!("herd: {h}\n"));
        }
        fs::write(pointer, text).with_context(|| format!("writing {}", pointer.display()))?;
        rep.done(format!("pointer: wrote {} -> {path}", pointer.display()));
    }
    Ok(root)
}

// -- .git/info/exclude ---------------------------------------------------------

/// The exclude pattern for `<dir>/<name>`, anchored at the git top-level.
fn exclude_line(top: &Path, dir: &Path, name: &str) -> String {
    let rel = dir.strip_prefix(top).unwrap_or(Path::new(""));
    let mut line = String::from("/");
    for part in rel.components() {
        line.push_str(&part.as_os_str().to_string_lossy());
        line.push('/');
    }
    line.push_str(name);
    line
}

/// `<git dir>/info/exclude` for the repo whose top-level is `top`. A linked
/// worktree's `.git` is a file, so ask git where the shared exclude lives.
fn exclude_file(top: &Path) -> Result<PathBuf> {
    let git = top.join(".git");
    if git.is_dir() {
        return Ok(git.join("info").join("exclude"));
    }
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(top)
        .args(["rev-parse", "--git-path", "info/exclude"])
        .output()
        .context("running `git rev-parse --git-path info/exclude`")?;
    if !out.status.success() {
        bail!(
            "cannot locate .git/info/exclude for {}: {}",
            top.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(top.join(String::from_utf8_lossy(&out.stdout).trim()))
}

fn read_exclude(top: &Path) -> Result<String> {
    let file = exclude_file(top)?;
    match fs::read_to_string(&file) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e).with_context(|| format!("reading {}", file.display())),
    }
}

/// Append `line` to `.git/info/exclude` unless it is already there. Creates
/// the file (and `info/`) if missing; never rewrites any other line.
fn ensure_excluded(top: &Path, line: &str, what: &str, rep: &mut Report) -> Result<()> {
    let file = exclude_file(top)?;
    let mut text = read_exclude(top)?;
    if text.lines().any(|l| l.trim() == line) {
        rep.already(format!(
            "exclude: `{line}` already in .git/info/exclude ({what})"
        ));
        return Ok(());
    }
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    }
    if !text.is_empty() && !text.ends_with('\n') {
        text.push('\n');
    }
    text.push_str(line);
    text.push('\n');
    fs::write(&file, text).with_context(|| format!("writing {}", file.display()))?;
    rep.done(format!(
        "exclude: added `{line}` to .git/info/exclude ({what})"
    ));
    Ok(())
}
