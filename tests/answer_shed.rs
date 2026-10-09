//! `yaks answer <id>@<shed>`, driven through the real binary against a real git
//! repository with a linked worktree (a shed): the answer is written into the
//! SHED's copy of the yak and left uncommitted there; this checkout's farm is
//! untouched.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

struct Repo {
    dir: PathBuf,
    shed: PathBuf,
    id: String,
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Proc::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn run(dir: &Path, actor: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(dir)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", actor)
        .env_remove("YAKS_DIR")
        .args(args)
        .output()
        .unwrap()
}

fn yaks(dir: &Path, actor: &str, args: &[&str]) -> String {
    let out = run(dir, actor, args);
    assert!(
        out.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// A failing run: (stdout, stderr).
fn yaks_fails(dir: &Path, actor: &str, args: &[&str]) -> (String, String) {
    let out = run(dir, actor, args);
    assert!(!out.status.success(), "{args:?} should have failed");
    (
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

impl Repo {
    /// A repo with one committed yak, a worktree shed forked from it, and an
    /// ask made in the shed (uncommitted there).
    fn new(tag: &str) -> Repo {
        let base =
            std::env::temp_dir().join(format!("yaks-answer-shed-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("repo");
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        yaks(&dir, "main-actor", &["init"]);
        let v: Value =
            serde_json::from_str(&yaks(&dir, "main-actor", &["create", "alpha", "--json"]))
                .unwrap();
        let id = v["id"].as_str().unwrap().to_string();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "init"]);
        let shed = base.join("shed");
        git(
            &dir,
            &[
                "worktree",
                "add",
                "-q",
                "-b",
                "shed",
                shed.to_str().unwrap(),
            ],
        );
        yaks(
            &shed,
            "shed-worker",
            &["ask", &id, "--note", "Which database?"],
        );
        // The worker has committed its ask, so the answer is the only change.
        git(&shed, &["add", "-A"]);
        git(&shed, &["commit", "-q", "-m", "ask"]);
        Repo { dir, shed, id }
    }

    fn shed_file(&self) -> PathBuf {
        let p = yaks(&self.shed, "x", &["path", &self.id]);
        PathBuf::from(p.trim())
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

#[test]
fn answers_in_the_shed_copy_and_leaves_it_uncommitted() {
    let r = Repo::new("basic");
    let main_file = PathBuf::from(yaks(&r.dir, "x", &["path", &r.id]).trim());
    let main_before = std::fs::read_to_string(&main_file).unwrap();
    let head_before = git(&r.shed, &["rev-parse", "HEAD"]);
    assert!(git(&r.shed, &["status", "--porcelain"]).trim().is_empty());

    let out = yaks(
        &r.dir,
        "joel",
        &["answer", &format!("{}@shed", r.id), "--note", "Use sqlite."],
    );
    assert!(out.contains("shed shed-worker"), "{out}");
    assert!(out.contains("uncommitted"), "{out}");
    let shed_file = r.shed_file();
    assert!(out.contains(shed_file.to_str().unwrap()), "{out}");

    // The shed's copy: needs agent, attributed reply.
    let body = std::fs::read_to_string(&shed_file).unwrap();
    assert!(body.contains("needs: agent"), "{body}");
    assert!(body.contains("Use sqlite."), "{body}");
    assert!(body.contains("[joel]"), "{body}");

    // Still uncommitted there: modified, same HEAD, nothing staged by us.
    let status = git(&r.shed, &["status", "--porcelain"]);
    assert!(
        status.contains(shed_file.file_name().unwrap().to_str().unwrap()),
        "{status}"
    );
    assert!(
        git(&r.shed, &["diff"]).contains("Use sqlite."),
        "answer is an unstaged change"
    );
    assert_eq!(head_before, git(&r.shed, &["rev-parse", "HEAD"]));
    assert!(
        git(&r.shed, &["diff", "--cached", "--name-only"])
            .trim()
            .is_empty(),
        "answer must not stage anything"
    );

    // This checkout's copy is untouched.
    assert_eq!(main_before, std::fs::read_to_string(&main_file).unwrap());
    assert!(git(&r.dir, &["status", "--porcelain"]).trim().is_empty());

    // From main, the shed's ask now reads as answered.
    let j: Value =
        serde_json::from_str(&yaks(&r.dir, "x", &["inbox", "--sheds", "--json"])).unwrap();
    let sheds = j["sheds"].as_array().unwrap();
    assert_eq!(sheds.len(), 1, "{j}");
    assert_eq!(sheds[0]["id"], r.id.as_str());
    assert_eq!(sheds[0]["needs"], "agent");
}

#[test]
fn done_clears_needs_in_the_shed() {
    let r = Repo::new("done");
    yaks(
        &r.dir,
        "joel",
        &[
            "answer",
            &format!("{}@shed", r.id),
            "--note",
            "fine",
            "--done",
        ],
    );
    let body = std::fs::read_to_string(r.shed_file()).unwrap();
    assert!(!body.contains("needs:"), "{body}");
    assert!(body.contains("fine"), "{body}");
}

#[test]
fn plain_answer_without_at_is_unchanged() {
    let r = Repo::new("plain");
    yaks(&r.dir, "main-actor", &["ask", &r.id, "--note", "main q"]);
    yaks(&r.dir, "joel", &["answer", &r.id, "--note", "main a"]);
    let main_file = PathBuf::from(yaks(&r.dir, "x", &["path", &r.id]).trim());
    let body = std::fs::read_to_string(main_file).unwrap();
    assert!(
        body.contains("needs: agent") && body.contains("main a"),
        "{body}"
    );
    let shed = std::fs::read_to_string(r.shed_file()).unwrap();
    assert!(!shed.contains("main a"), "{shed}");
}

#[test]
fn unknown_shed_is_refused_and_lists_the_sheds() {
    let r = Repo::new("unknown");
    let (_, err) = yaks_fails(
        &r.dir,
        "joel",
        &["answer", &format!("{}@nope", r.id), "--note", "x"],
    );
    assert!(err.contains("no shed matches `nope`"), "{err}");
    assert!(err.contains("shed-worker"), "{err}");
}

#[test]
fn yak_absent_in_the_shed_is_refused_and_nothing_is_written() {
    let r = Repo::new("absent");
    let before = git(&r.shed, &["status", "--porcelain"]);
    let (_, err) = yaks_fails(&r.dir, "joel", &["answer", "zzzz-0000@shed", "--note", "x"]);
    assert!(
        err.contains("zzzz-0000") && err.contains("not found in shed"),
        "{err}"
    );
    assert_eq!(before, git(&r.shed, &["status", "--porcelain"]));
}

#[cfg(unix)]
#[test]
fn a_shed_sharing_our_farm_is_refused() {
    let r = Repo::new("shared");
    std::fs::remove_dir_all(r.shed.join(".yaks")).unwrap();
    std::os::unix::fs::symlink(r.dir.join(".yaks"), r.shed.join(".yaks")).unwrap();
    let (_, err) = yaks_fails(
        &r.dir,
        "joel",
        &["answer", &format!("{}@shed", r.id), "--note", "x"],
    );
    assert!(err.contains("plain `yaks answer"), "{err}");
}

#[test]
fn a_shed_with_no_farm_is_refused() {
    let r = Repo::new("nofarm");
    std::fs::rename(
        r.shed.join(".yaks"),
        r.shed.parent().unwrap().join("moved-yaks"),
    )
    .unwrap();
    let (_, err) = yaks_fails(
        &r.dir,
        "joel",
        &["answer", &format!("{}@shed", r.id), "--note", "x"],
    );
    assert!(err.contains("no farm"), "{err}");
}
