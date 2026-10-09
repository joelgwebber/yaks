//! `yaks inbox --sheds`, driven through the real binary against a real git
//! repository with a linked worktree (a shed): open asks in the shed's own
//! farm that the main farm lacks are listed with the shed's name and the
//! question text; asks identical in both are not repeated; a shed with no farm
//! of its own adds nothing.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

struct Repo {
    dir: PathBuf,
    shed: PathBuf,
}

fn git(dir: &Path, args: &[&str]) {
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
}

fn yaks(dir: &Path, actor: &str, args: &[&str]) -> String {
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(dir)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", actor)
        .env_remove("YAKS_DIR")
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

impl Repo {
    /// A repo with two committed yaks (`a`, `b`); `pre` runs on the main farm
    /// before the commit, then a worktree shed is forked from it.
    fn new(tag: &str, pre: impl FnOnce(&Path, &str, &str)) -> Repo {
        let base =
            std::env::temp_dir().join(format!("yaks-inbox-sheds-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let dir = base.join("repo");
        std::fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        yaks(&dir, "main-actor", &["init"]);
        let make = |t: &str| -> String {
            let v: Value =
                serde_json::from_str(&yaks(&dir, "main-actor", &["create", t, "--json"])).unwrap();
            v["id"].as_str().unwrap().to_string()
        };
        let (a, b) = (make("alpha"), make("beta"));
        pre(&dir, &a, &b);
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
        Repo { dir, shed }
    }

    fn inbox(&self, args: &[&str]) -> String {
        let mut a = vec!["inbox"];
        a.extend_from_slice(args);
        yaks(&self.dir, "main-actor", &a)
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

#[test]
fn shed_ask_is_listed_with_shed_name_and_question_only_with_the_flag() {
    let r = Repo::new("basic", |_, _, _| {});
    // The shed asks about a yak; the main farm has no such ask.
    let id = {
        let v: Value = serde_json::from_str(&yaks(&r.shed, "x", &["list", "--json"])).unwrap();
        v[0]["id"].as_str().unwrap().to_string()
    };
    yaks(
        &r.shed,
        "shed-worker",
        &["ask", &id, "--note", "Which database should we use?"],
    );

    let plain = r.inbox(&[]);
    assert!(plain.contains("Inbox empty"), "{plain}");
    assert!(!plain.contains("database"), "{plain}");

    let out = r.inbox(&["--sheds"]);
    assert!(
        out.contains("Inbox empty"),
        "main inbox comes first:\n{out}"
    );
    assert!(out.contains("Shed shed-worker"), "{out}");
    assert!(out.contains(&id), "{out}");
    assert!(out.contains("needs:human"), "{out}");
    assert!(out.contains("Which database should we use?"), "{out}");
    // The shed's path is shown too.
    assert!(
        out.contains(r.shed.file_name().unwrap().to_str().unwrap()),
        "{out}"
    );
}

#[test]
fn an_ask_identical_in_both_farms_is_listed_once_and_a_changed_one_is_not_hidden() {
    let r = Repo::new("same", |dir, a, b| {
        yaks(dir, "asker", &["ask", a, "--note", "same question"]);
        yaks(dir, "asker", &["ask", b, "--note", "old question"]);
    });
    // The shed leaves `a` alone and re-asks `b` with new text.
    let (a, b) = {
        let v: Value = serde_json::from_str(&yaks(&r.dir, "x", &["inbox", "--json"])).unwrap();
        let mut ids: Vec<String> = v
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["id"].as_str().unwrap().into())
            .collect();
        ids.sort();
        (ids[0].clone(), ids[1].clone())
    };
    yaks(
        &r.shed,
        "shed-worker",
        &["ask", &b, "--note", "new question"],
    );

    let out = r.inbox(&["--sheds"]);
    let main_part = out.split("Shed ").next().unwrap();
    assert!(main_part.contains(&a) && main_part.contains(&b), "{out}");
    let shed_part = &out[main_part.len()..];
    assert!(shed_part.contains("new question"), "{out}");
    assert!(!shed_part.contains("same question"), "{out}");
    assert!(!shed_part.contains(&a), "identical ask repeated:\n{out}");
    assert_eq!(
        out.matches("same question").count(),
        0,
        "question text is only shown for shed rows:\n{out}"
    );

    let j: Value = serde_json::from_str(&r.inbox(&["--sheds", "--json"])).unwrap();
    assert_eq!(j["inbox"].as_array().unwrap().len(), 2);
    let sheds = j["sheds"].as_array().unwrap();
    assert_eq!(sheds.len(), 1, "{j}");
    assert_eq!(sheds[0]["shed"], "shed-worker");
    assert_eq!(sheds[0]["id"], b.as_str());
    assert_eq!(sheds[0]["needs"], "human");
    assert_eq!(sheds[0]["question"], "new question");
    assert!(sheds[0]["path"].as_str().unwrap().ends_with("shed"));
}

#[test]
fn a_shed_that_forked_before_our_later_note_does_not_repeat_the_ask() {
    // The common case behind a stale-looking shed: the ask was committed, the
    // shed forked, then THIS farm moved on (a note; a coordinator's update).
    // The shed's copy says nothing we do not already have.
    let r = Repo::new("moved-on", |dir, a, _| {
        yaks(
            dir,
            "asker",
            &["ask", a, "--note", "forked-before question"],
        );
    });
    let a = {
        let v: Value = serde_json::from_str(&yaks(&r.dir, "x", &["inbox", "--json"])).unwrap();
        v[0]["id"].as_str().unwrap().to_string()
    };
    yaks(
        &r.dir,
        "main-actor",
        &["update", &a, "--note", "a later note here"],
    );
    let out = r.inbox(&["--sheds"]);
    assert!(out.contains("No open asks in other sheds"), "{out}");
    // A reply in the shed after the same ask IS news.
    yaks(
        &r.shed,
        "shed-worker",
        &["update", &a, "--note", "reply from the shed"],
    );
    let out = r.inbox(&["--sheds"]);
    assert!(out.contains("reply from the shed"), "{out}");
}

#[test]
fn for_filter_and_json_shape() {
    let r = Repo::new("for", |_, _, _| {});
    let v: Value = serde_json::from_str(&yaks(&r.shed, "x", &["list", "--json"])).unwrap();
    let id = v[0]["id"].as_str().unwrap().to_string();
    yaks(&r.shed, "shed-worker", &["ask", &id, "--note", "q?"]);

    // Without --sheds the JSON is still the flat array.
    let flat: Value = serde_json::from_str(&r.inbox(&["--json"])).unwrap();
    assert!(flat.is_array(), "{flat}");

    // The shed ask is awaiting a human: --for agent drops it, --for human keeps it.
    let agent = r.inbox(&["--sheds", "--for", "agent"]);
    assert!(!agent.contains("q?"), "{agent}");
    assert!(agent.contains("No open asks in other sheds."), "{agent}");
    let human = r.inbox(&["--sheds", "--for", "human"]);
    assert!(human.contains("q?"), "{human}");
    let j: Value =
        serde_json::from_str(&r.inbox(&["--sheds", "--for", "agent", "--json"])).unwrap();
    assert!(j["sheds"].as_array().unwrap().is_empty(), "{j}");
}

#[test]
fn a_shed_with_no_farm_adds_nothing() {
    let r = Repo::new("nofarm", |_, _, _| {});
    let v: Value = serde_json::from_str(&yaks(&r.shed, "x", &["list", "--json"])).unwrap();
    let id = v[0]["id"].as_str().unwrap().to_string();
    yaks(&r.shed, "shed-worker", &["ask", &id, "--note", "hidden?"]);
    // Discovery finds no farm for the shed once its `.yaks` is gone (what a
    // private farm looks like from outside).
    std::fs::rename(
        r.shed.join(".yaks"),
        r.shed.parent().unwrap().join("moved-yaks"),
    )
    .unwrap();
    let out = r.inbox(&["--sheds"]);
    assert!(!out.contains("hidden?"), "{out}");
    assert!(out.contains("No open asks in other sheds."), "{out}");
}

#[cfg(unix)]
#[test]
fn a_shed_sharing_our_farm_adds_nothing() {
    let r = Repo::new("shared", |dir, a, _| {
        yaks(
            dir,
            "asker",
            &["ask", a, "--note", "only in the shared farm"],
        );
    });
    // The shed's `.yaks` is a symlink to the main farm: the same farm.
    std::fs::remove_dir_all(r.shed.join(".yaks")).unwrap();
    std::os::unix::fs::symlink(r.dir.join(".yaks"), r.shed.join(".yaks")).unwrap();
    let out = r.inbox(&["--sheds"]);
    assert!(
        !out.contains("only in the shared farm"),
        "question text is only shown for shed rows:\n{out}"
    );
    assert!(out.contains("No open asks in other sheds."), "{out}");
}

#[test]
fn inbox_sheds_never_writes_into_the_shed() {
    let r = Repo::new("ro", |_, _, _| {});
    let v: Value = serde_json::from_str(&yaks(&r.shed, "x", &["list", "--json"])).unwrap();
    let id = v[0]["id"].as_str().unwrap().to_string();
    yaks(&r.shed, "shed-worker", &["ask", &id, "--note", "q?"]);
    let snapshot = |p: &Path| {
        let out = Proc::new("git")
            .args([
                "-C",
                p.to_str().unwrap(),
                "status",
                "--porcelain=v2",
                "--untracked-files=all",
            ])
            .output()
            .unwrap();
        let mut s = String::from_utf8_lossy(&out.stdout).to_string();
        for e in walk(&p.join(".yaks")) {
            s.push_str(&format!("{} {:?}\n", e.0.display(), e.1));
        }
        s
    };
    let before = snapshot(&r.shed);
    r.inbox(&["--sheds"]);
    assert_eq!(before, snapshot(&r.shed));
}

fn walk(dir: &Path) -> Vec<(PathBuf, std::time::SystemTime)> {
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walk(&p));
            } else if let Ok(m) = e.metadata().and_then(|m| m.modified()) {
                out.push((p, m));
            }
        }
    }
    out.sort();
    out
}
