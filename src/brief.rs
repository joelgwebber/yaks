//! `yaks brief`: print the worker brief for one yak.
//!
//! A coordinator hands a yak to a worker with a brief. Everything in it that is
//! a fact about the yak, the config or the farm is printed here, so it is
//! derived from the farm instead of filled in by hand (and a clause cannot be
//! forgotten). What the yak cannot know (spawn titles, how work lands in the
//! coordinator's checkout, which model) stays with the coordinator, who wraps
//! this text.
//!
//! The text is assembled from one small function per section, each taking the
//! [`Brief`] facts, so the differences between a published and a private
//! farm are explicit: [`Publication`] picks the finish steps (whether yak ids
//! may go in commit messages), [`yaks_dir`] decides whether commands carry
//! `YAKS_DIR`, and [`gate`] decides the evidence section. Read-only: the farm
//! is never written.

use crate::farm::Farm;
use crate::model::{Status, Task};
use crate::store::{self, Config};
use anyhow::{Result, bail};
use std::io::Write;
use std::path::Path;

/// Whether the farm's yak ids may reach git, which decides how a worker finishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Publication {
    /// The farm is a farm branch's worktree ([`store::farm_branch`], yaks-73f4):
    /// one farm per machine, which every checkout's discovery finds. The
    /// worker stages nothing of it; the yakherd commits it on the branch. Yak
    /// ids are fine in commit messages (the farm is in git, and may be shared).
    Published { branch: String },
    /// A farm marked private (`yaks private`), or one reached only through
    /// `--yaks-dir`/`YAKS_DIR`: keep yak ids out of git.
    Private,
}

/// Where the yak's verify command came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateSource {
    Yak,
    Config,
}

/// The yak's gate: its own `verify:`, else the config default for its labels
/// (the rule `yaks verify` applies).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Gate {
    pub command: String,
    pub source: GateSource,
}

/// Every fact the brief is built from.
pub struct Brief<'a> {
    pub task: &'a Task,
    pub actor: &'a str,
    /// The binary to run, as an absolute path.
    pub binary: &'a Path,
    pub publication: &'a Publication,
    /// The farm every command must name with `YAKS_DIR`, when there is one.
    pub yaks_dir: Option<&'a str>,
    pub gate: Option<&'a Gate>,
}

/// The yak's gate: its explicit `verify:` wins, else the config default for its
/// labels and herd (the id prefix).
pub fn gate(task: &Task, cfg: &Config) -> Option<Gate> {
    match &task.verify {
        Some(command) => Some(Gate {
            command: command.clone(),
            source: GateSource::Yak,
        }),
        None => cfg
            .resolve_verify(&task.labels, task.id.split('-').next())
            .map(|command| Gate {
                command,
                source: GateSource::Config,
            }),
    }
}

/// Detect whether the farm is published: a farm-branch worktree is
/// [`Publication::Published`] unless the repository marked it private (`yaks private`, so its yak ids never
/// reach a commit that could be pushed); anything else (a `YAKS_DIR` farm) is
/// private.
pub fn detect_publication(cwd: &Path, farm_root: &Path) -> Publication {
    match store::farm_branch(farm_root) {
        Some(_) if crate::privatefarm::is_private(cwd) => Publication::Private,
        Some(branch) => Publication::Published { branch },
        None => Publication::Private,
    }
}

/// The farm path to put on every command as `YAKS_DIR`, if any. An explicit
/// `--yaks-dir` always wins (made absolute against `cwd`, as discovery reads
/// it). A farm branch is found by discovery from every checkout of the
/// repository (and of any repository whose `yaks.farm` names it), so it needs
/// none; a farm that is not a farm branch (reached through `YAKS_DIR`) gets
/// its path, since no checkout's discovery finds it.
pub fn yaks_dir(explicit: Option<&str>, cwd: &Path, farm_root: &Path) -> Option<String> {
    match explicit {
        Some(dir) => Some(store::expand_path(cwd, dir).display().to_string()),
        None if store::farm_branch(farm_root).is_some() => None,
        None => Some(farm_root.display().to_string()),
    }
}

/// Quote a value for a POSIX shell only when it needs it.
fn sh_quote(s: &str) -> String {
    let plain = !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._-:=+@%,~".contains(c));
    if plain {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// The environment prefix of every yaks command.
fn env_prefix(b: &Brief) -> String {
    let mut p = format!("YAKS_ACTOR={}", sh_quote(b.actor));
    if let Some(dir) = b.yaks_dir {
        p.push_str(&format!(" YAKS_DIR={}", sh_quote(dir)));
    }
    p
}

/// The whole brief, sections separated by a blank line.
pub fn render(b: &Brief) -> String {
    let sections = [
        intro(b),
        commands(b),
        start(b),
        evidence(b),
        forbidden(),
        notes(b),
        finish(b),
        blocked(b),
        final_message(b),
    ];
    sections.join("\n\n") + "\n"
}

fn intro(b: &Brief) -> String {
    format!(
        "You are worker `{actor}`. You own exactly ONE yak: `{id}` - \"{title}\". It is already `shaving`.\n\
         Do not shave, shear, regrow or edit any other yak.",
        actor = b.actor,
        id = b.task.id,
        title = b.task.title,
    )
}

fn commands(b: &Brief) -> String {
    let mut s = format!(
        "Commands\n\
         - Run every yaks command as `{prefix} {binary} <args>`; each terminal call is a fresh shell, so set it every time. Below, `yaks <args>` means that.\n\
         - Use that binary, or the one the coordinator names; never npx or an installer.",
        prefix = env_prefix(b),
        binary = sh_quote(&b.binary.display().to_string()),
    );
    if b.yaks_dir.is_some() {
        s.push_str("\n- `YAKS_DIR` names the farm this yak lives in; without it a command opens another farm.");
    }
    if b.yaks_dir.is_none() {
        s.push_str(
            "\n- The farm is one copy per machine, on its own branch, shared by every checkout. `yaks path` (no arguments) prints it; never create or stage a `.yaks/` in your own checkout.",
        );
    }
    s
}

fn start(b: &Brief) -> String {
    let id = &b.task.id;
    format!(
        "Start\n\
         1. `yaks show {id}` and read every note. The scope, the judge and the evidence contract are in the claim note. Anything it puts out of scope or does not decide is not yours to decide: `yaks ask {id}`.\n\
         2. In your first note (`yaks update {id} --note ...`) record `date -u` and the output of `yaks path {id}`."
    )
}

fn evidence(b: &Brief) -> String {
    let id = &b.task.id;
    let gate = match b.gate {
        Some(g) => {
            let from = match g.source {
                GateSource::Yak => "the yak's `verify:`",
                GateSource::Config => "the config `verify:` default for this yak",
            };
            format!(
                "- Gate: `{}` ({from}). `yaks verify {id}` runs it and records PASS/FAIL as a note.",
                g.command
            )
        }
        None => "- This yak has no `verify:` command and the config has no default for it: the evidence is what the claim note names. Record each command you ran and its real output in a note.".to_string(),
    };
    format!(
        "Evidence (the coordinator re-runs it and rejects what it cannot reproduce)\n\
         {gate}\n\
         - A file someone should look at (screenshot, TUI frame, log): `yaks attach {id} <file>`.\n\
         - Paste real output in notes; claim nothing unobserved."
    )
}

fn forbidden() -> String {
    "Forbidden: edits to Cargo.toml, Cargo.lock, .gitignore or config; path dependencies; `git push`; installs; history rewrites.".to_string()
}

fn notes(b: &Brief) -> String {
    let id = &b.task.id;
    format!(
        "Notes\n\
         - Append progress as you go: `yaks update {id} --note \"...\"`.\n\
         - Record every choice a reviewer could argue about as its own note starting `Decision:`, naming the alternatives you rejected.\n\
         - Write a multi-line note as markdown with real line breaks, piped on stdin (`yaks update {id} --note - <<'EOF'` ... `EOF`, the quoted `'EOF'` keeps backticks and `$` literal), never as one long argument and never inside `$( )` or a double-quoted string, where backticks and `$` in the note run as commands.\n\
         - Notes render as CommonMark: a single newline joins lines into one paragraph. Put a blank line between every block (paragraph, list, fence), and parallel facts on list lines (`- `)."
    )
}

/// The finish steps: the part that depends on whether the farm is published.
fn finish(b: &Brief) -> String {
    let id = &b.task.id;
    let proof = if b.gate.is_some() {
        format!("`yaks verify {id}` has recorded a PASS (run it from your checkout's root)")
    } else {
        "the evidence the claim note names is in a note".to_string()
    };
    let shear = format!(
        "1. Finish only after the evidence exists: {proof}, and your summary is in a note (`yaks shorn` takes no note): short, the shape of the fix and the evidence (linked or attached), not a bare \"done\" and not a design doc.\n\
         2. `yaks shorn {id}` (there is no `shear` subcommand)."
    );
    match b.publication {
        Publication::Published { branch } => format!(
            "Finish (the yakherd commits the farm on branch `{branch}`, never you)\n\
             {shear}\n\
             3. ONE commit of your own source files only: `git add -A -- <your files>`. Never stage anything under `.yaks/`. Message `{id}: <what changed>`.\n\
             4. `git show --stat` must list every file you expect and nothing under `.yaks/`."
        ),
        Publication::Private => format!(
            "Finish (private farm: yak ids stay out of git)\n\
             {shear}\n\
             3. ONE commit of your own source files only: `git add -A -- <your files>`. Never add anything from the farm. Write the message in plain English; no yak id and not the word \"yaks\" in it, in code or in comments.\n\
             4. `git show --stat` must list every file you expect."
        ),
    }
}

fn blocked(b: &Brief) -> String {
    format!(
        "Blocked, or a decision needed: `yaks ask {id} --note \"<question>\"`, leave your edits in the working tree (never revert them, never park them in $TMPDIR), say which in your final message, and return. Never answer your own ask.\n\
         When you are woken with the answer, or start a new session, `yaks inbox --for agent` lists it: read it, then `yaks pickup {id} --note \"<what you will do>\"` clears it.",
        id = b.task.id
    )
}

fn final_message(b: &Brief) -> String {
    let gate = if b.gate.is_some() {
        "the gate command and its last 15 lines"
    } else {
        "the evidence commands and their last 15 lines"
    };
    format!(
        "Final message\n\
         - the commit SHA and `git show --stat`\n\
         - {gate}\n\
         - start and finish `date -u`\n\
         - the `Decision:` notes, one line each\n\
         - the docs-parity grep you ran\n\
         - anything surprising"
    )
}

/// The stderr warning for a yak the coordinator has not claimed.
pub fn claim_warning(task: &Task) -> Option<String> {
    (task.status != Status::Shaving).then(|| {
        format!(
            "warning: {id} is {status}, not shaving: the coordinator must claim it (`yaks shave {id}`) before the worker starts; the brief says it is already shaving",
            id = task.id,
            status = task.status.as_str()
        )
    })
}

/// A name safe to put in a shell command and in a note stamp.
fn check_actor(actor: &str) -> Result<()> {
    if actor.is_empty()
        || actor
            .chars()
            .any(|c| c.is_whitespace() || "'\"[]\\$`;&|<>()".contains(c))
    {
        bail!("invalid worker name {actor:?}: use a short name such as `sheds-1`");
    }
    Ok(())
}

/// What one `yaks brief` invocation asks for.
pub struct Request<'a> {
    pub id: &'a str,
    pub actor: &'a str,
    /// `--yaks-dir`, as typed.
    pub explicit_dir: Option<&'a str>,
    pub cwd: &'a Path,
    /// The running executable.
    pub binary: &'a Path,
}

/// `yaks brief`: write the brief to `out`, any claim warning to `err`; return
/// the exit code. An unknown yak writes nothing to `out` and returns 1.
pub fn run(farm: &Farm, req: &Request, out: &mut impl Write, err: &mut impl Write) -> Result<i32> {
    check_actor(req.actor)?;
    let Some(show) = farm.show(req.id)? else {
        writeln!(err, "no such task: {}", req.id)?;
        return Ok(1);
    };
    let task = &show.task;
    let publication = detect_publication(req.cwd, farm.root());
    let dir = yaks_dir(req.explicit_dir, req.cwd, farm.root());
    let gate = gate(task, &farm.config());
    let text = render(&Brief {
        task,
        actor: req.actor,
        binary: req.binary,
        publication: &publication,
        yaks_dir: dir.as_deref(),
        gate: gate.as_ref(),
    });
    if let Some(w) = claim_warning(task) {
        writeln!(err, "{w}")?;
    }
    out.write_all(text.as_bytes())?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Status;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SEQ: AtomicU64 = AtomicU64::new(0);

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
            labels: vec![],
            depends_on: vec![],
            source: None,
            needs: None,
            verify: None,
            extra: Vec::new(),
            body: String::new(),
        }
    }

    fn git(repo: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}: {out:?}");
    }

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "yaks-brief-{tag}-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&d).unwrap();
        d.canonicalize().unwrap()
    }

    /// A farm directory with a config and one saved yak.
    fn make_farm(root: &Path, config: &str, t: &Task) {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join("config.yaml"), config).unwrap();
        std::fs::write(root.join("schema"), format!("{}\n", store::SCHEMA)).unwrap();
        store::write::save(root, t).unwrap();
    }

    /// A code repo whose farm is the orphan branch `yaks`, checked out beside
    /// it: `(code checkout, farm)`.
    fn branch_repo(config: &str, t: &Task) -> (PathBuf, Farm) {
        let base = tmp("branch");
        let code = base.join("code");
        std::fs::create_dir_all(&code).unwrap();
        git(&code, &["init", "-q", "-b", "main"]);
        git(&code, &["commit", "-q", "--allow-empty", "-m", "init"]);
        let wt = base.join("farm");
        git(
            &code,
            &[
                "worktree",
                "add",
                "-q",
                "--orphan",
                "-b",
                "yaks",
                wt.to_str().unwrap(),
            ],
        );
        make_farm(&wt.join(".yaks"), config, t);
        let farm = Farm::open(&code).unwrap_or_else(|_| panic!("open farm from {code:?}"));
        (code, farm)
    }

    fn facts<'a>(
        t: &'a Task,
        publication: &'a Publication,
        dir: Option<&'a str>,
        gate: Option<&'a Gate>,
    ) -> Brief<'a> {
        Brief {
            task: t,
            actor: "demo-1",
            binary: Path::new("/opt/yaks"),
            publication,
            yaks_dir: dir,
            gate,
        }
    }

    fn run_brief(farm: &Farm, repo: &Path, id: &str, dir: Option<&str>) -> (i32, String, String) {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(
            farm,
            &Request {
                id,
                actor: "demo-1",
                explicit_dir: dir,
                cwd: repo,
                binary: Path::new("/opt/yaks"),
            },
            &mut out,
            &mut err,
        )
        .unwrap();
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    fn published_yaks() -> Publication {
        Publication::Published {
            branch: "yaks".into(),
        }
    }

    #[test]
    fn every_brief_names_the_yak_the_actor_prefix_and_the_final_message() {
        let t = task("yak-0001", Status::Shaving);
        let text = render(&facts(&t, &Publication::Private, None, None));
        assert!(text.contains("You are worker `demo-1`"));
        assert!(text.contains("`yak-0001` - \"title yak-0001\". It is already `shaving`."));
        assert!(text.contains("YAKS_ACTOR=demo-1 /opt/yaks <args>"));
        assert!(text.contains("`Decision:`"), "the decision-note rule");
        assert!(text.contains("<<'EOF'"), "multi-line notes go on stdin");
        assert!(text.contains("Final message"));
        assert!(text.contains("Do not shave, shear, regrow or edit any other yak."));
    }

    #[test]
    fn a_private_brief_keeps_yak_ids_out_of_git() {
        let t = task("yak-0001", Status::Shaving);
        let text = render(&facts(&t, &Publication::Private, Some("/farm/.yaks"), None));
        assert!(text.contains("Finish (private farm"));
        assert!(text.contains("no yak id and not the word \"yaks\""));
        assert!(!text.contains("Message `yak-0001"));
        assert!(text.contains("YAKS_ACTOR=demo-1 YAKS_DIR=/farm/.yaks /opt/yaks <args>"));
        assert!(text.contains("`YAKS_DIR` names the farm"));
    }

    #[test]
    fn a_branch_brief_never_stages_the_farm_and_keeps_ids_in_messages() {
        let t = task("yak-0001", Status::Shaving);
        let publication = published_yaks();
        let text = render(&facts(&t, &publication, None, None));
        assert!(text.contains("Finish (the yakherd commits the farm on branch `yaks`"));
        assert!(text.contains("Never stage anything under `.yaks/`"));
        assert!(
            text.contains("Message `yak-0001: <what changed>`"),
            "ids are fine: the farm is in git"
        );
        assert!(text.contains("on its own branch") && text.contains("`yaks path` (no arguments)"));
        assert!(!text.contains("YAKS_DIR"));
    }

    #[test]
    fn run_prints_a_branch_brief_with_no_yaks_dir() {
        let t = task("yak-0001", Status::Shaving);
        let (code, farm) = branch_repo("herd: yak\n", &t);
        assert_eq!(detect_publication(&code, farm.root()), published_yaks());
        assert_eq!(yaks_dir(None, &code, farm.root()), None);
        let (rc, out, err) = run_brief(&farm, &code, "yak-0001", None);
        assert_eq!(rc, 0);
        assert!(err.is_empty(), "no warning for a shaving yak: {err}");
        assert!(out.contains("Finish (the yakherd commits the farm"));
        assert!(!out.contains("YAKS_DIR"));
    }

    #[test]
    fn a_private_branch_farm_gets_the_private_finish_but_no_yaks_dir() {
        let t = task("yak-0001", Status::Shaving);
        let (code, farm) = branch_repo("herd: yak\n", &t);
        git(&code, &["config", "yaks.private", "true"]);
        assert_eq!(detect_publication(&code, farm.root()), Publication::Private);
        let (_, out, _) = run_brief(&farm, &code, "yak-0001", None);
        assert!(out.contains("Finish (private farm"), "{out}");
        assert!(!out.contains("YAKS_DIR"), "discovery finds a branch farm");
    }

    #[test]
    fn a_yaks_dir_farm_is_private_and_named_on_every_command() {
        let dir = tmp("nogit");
        let t = task("yak-0001", Status::Shaving);
        let root = dir.join(".yaks");
        make_farm(&root, "herd: yak\n", &t);
        assert_eq!(detect_publication(&dir, &root), Publication::Private);
        assert_eq!(
            yaks_dir(None, &dir, &root),
            Some(root.display().to_string())
        );
    }

    #[test]
    fn an_explicit_yaks_dir_wins_and_is_made_absolute() {
        let t = task("yak-0001", Status::Shaving);
        let (code, farm) = branch_repo("herd: yak\n", &t);
        let (_, out, _) = run_brief(&farm, &code, "yak-0001", Some("../other/.yaks"));
        let want = code.join("../other/.yaks");
        assert!(
            out.contains(&format!("YAKS_DIR={}", want.display())),
            "{out}"
        );
        assert!(
            out.contains("Finish (the yakherd commits"),
            "the publication is unchanged"
        );
    }

    #[test]
    fn the_brief_says_how_an_answered_ask_is_picked_up() {
        let t = task("yak-0001", Status::Shaving);
        let text = render(&facts(&t, &Publication::Private, None, None));
        assert!(text.contains("`yaks ask yak-0001"));
        assert!(text.contains("Never answer your own ask."));
        assert!(text.contains("`yaks inbox --for agent`"));
        assert!(text.contains("`yaks pickup yak-0001"));
    }

    #[test]
    fn the_evidence_section_follows_the_gate() {
        let t = task("yak-0001", Status::Shaving);
        let g = Gate {
            command: "cargo test".into(),
            source: GateSource::Yak,
        };
        let with = render(&facts(&t, &Publication::Private, None, Some(&g)));
        assert!(with.contains("Gate: `cargo test` (the yak's `verify:`)"));
        assert!(with.contains("`yaks verify yak-0001` runs it"));
        assert!(with.contains("the gate command and its last 15 lines"));
        assert!(with.contains("`yaks verify yak-0001` has recorded a PASS"));
        let none = render(&facts(&t, &Publication::Private, None, None));
        assert!(none.contains("no `verify:` command"));
        assert!(none.contains("the evidence commands and their last 15 lines"));
        assert!(none.contains("the evidence the claim note names is in a note"));
        assert!(!none.contains("has recorded a PASS"));
        let cfg = Gate {
            command: "make check".into(),
            source: GateSource::Config,
        };
        assert!(
            render(&facts(&t, &Publication::Private, None, Some(&cfg)))
                .contains("the config `verify:` default for this yak")
        );
    }

    #[test]
    fn a_shell_unsafe_binary_path_is_quoted() {
        assert_eq!(sh_quote("/opt/yaks"), "/opt/yaks");
        assert_eq!(sh_quote("/My Tools/yaks"), "'/My Tools/yaks'");
        assert_eq!(sh_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn a_worker_name_that_is_not_shell_safe_is_refused() {
        assert!(check_actor("sheds-1").is_ok());
        for bad in ["", "two words", "a;b", "$x", "a'b", "[x]"] {
            assert!(check_actor(bad).is_err(), "{bad:?} should be refused");
        }
    }

    #[test]
    fn only_an_unclaimed_yak_gets_the_claim_warning() {
        assert!(claim_warning(&task("yak-0001", Status::Shaving)).is_none());
        let w = claim_warning(&task("yak-0001", Status::Hairy)).unwrap();
        assert!(w.contains("yak-0001 is hairy, not shaving") && w.contains("yaks shave yak-0001"));
    }

    #[test]
    fn a_yak_verify_wins_over_the_config_default() {
        let mut t = task("yak-0001", Status::Shaving);
        let dir = tmp("cfg");
        std::fs::write(dir.join("config.yaml"), "herd: yak\n").unwrap();
        let cfg = store::read_config(&dir);
        assert_eq!(gate(&t, &cfg), None);
        t.verify = Some("cargo test".into());
        let g = gate(&t, &cfg).unwrap();
        assert_eq!(
            (g.command.as_str(), g.source),
            ("cargo test", GateSource::Yak)
        );
    }

    #[test]
    fn the_config_verify_default_becomes_the_gate() {
        let t = task("yak-0001", Status::Shaving);
        let cfg = "herd: yak\nverify:\n  default: make check\n";
        let (code, farm) = branch_repo(cfg, &t);
        let (_, out, _) = run_brief(&farm, &code, "yak-0001", None);
        assert!(
            out.contains("Gate: `make check` (the config `verify:` default"),
            "{out}"
        );
    }

    #[test]
    fn an_unknown_id_prints_nothing_on_stdout_and_exits_nonzero() {
        let t = task("yak-0001", Status::Shaving);
        let (code, farm) = branch_repo("herd: yak\n", &t);
        let (rc, out, err) = run_brief(&farm, &code, "yak-9999", None);
        assert_eq!(rc, 1);
        assert!(out.is_empty());
        assert!(err.contains("no such task: yak-9999"));
    }

    #[test]
    fn an_unclaimed_yak_still_prints_with_a_warning_on_stderr() {
        let t = task("yak-0001", Status::Hairy);
        let (code, farm) = branch_repo("herd: yak\n", &t);
        let (rc, out, err) = run_brief(&farm, &code, "yak-0001", None);
        assert_eq!(rc, 0);
        assert!(out.contains("You are worker `demo-1`"));
        assert!(err.contains("warning: yak-0001 is hairy, not shaving"));
    }
}
