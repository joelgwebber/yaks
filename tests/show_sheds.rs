//! `yaks show <id> --sheds`, driven through the real binary against a real git
//! repository with linked worktrees (sheds): this checkout's copy first, then
//! one block per shed whose copy differs (status, `needs:`, the note entries
//! it has that ours lacks); identical sheds and sheds lacking the yak are only
//! counted; a yak that exists only in a shed is shown with `--sheds` and still
//! exits 1 without it.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

struct Repo {
    dir: PathBuf,
    sheds: Vec<PathBuf>,
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

impl Repo {
    /// A repo with one committed yak (`alpha`) and `names.len()` worktree
    /// sheds forked from that commit.
    fn new(tag: &str, names: &[&str]) -> (Repo, String) {
        let base =
            std::env::temp_dir().join(format!("yaks-show-sheds-{tag}-{}", std::process::id()));
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
        let sheds = names
            .iter()
            .map(|n| {
                let p = base.join(n);
                git(
                    &dir,
                    &["worktree", "add", "-q", "-b", n, p.to_str().unwrap()],
                );
                p
            })
            .collect();
        (Repo { dir, sheds }, id)
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(self.dir.parent().unwrap());
    }
}

#[test]
fn a_shed_with_a_note_and_a_move_shows_exactly_those_and_identical_sheds_are_counted() {
    let (r, id) = Repo::new("diff", &["changed", "twin"]);
    let changed = &r.sheds[0];
    yaks(changed, "shed-worker", &["shave", &id]);
    yaks(
        changed,
        "shed-worker",
        &[
            "update",
            &id,
            "--note",
            "first line of shed note\nsecond line",
        ],
    );

    let plain = yaks(&r.dir, "main-actor", &["show", &id]);
    assert!(!plain.contains("Sheds:"), "{plain}");

    let out = yaks(&r.dir, "main-actor", &["show", &id, "--sheds"]);
    // This checkout's copy comes first, unchanged.
    assert!(out.starts_with(&plain), "{out}");
    let sheds = &out[plain.len()..];
    assert!(sheds.contains("Sheds:"), "{out}");
    assert!(sheds.contains("shed-worker"), "{out}");
    assert!(sheds.contains("changed"), "shed path shown:\n{out}");
    assert!(sheds.contains("status: shaving"), "{out}");
    assert!(sheds.contains("first line of shed note"), "{out}");
    assert!(!sheds.contains("second line"), "first line only:\n{out}");
    assert!(
        sheds.contains("shed-worker  moved: hairy -> shaving"),
        "{out}"
    );
    // The twin is only counted.
    assert!(
        sheds.contains("same in 1 shed; absent in 0; no farm of its own in 0 sheds"),
        "{out}"
    );
    assert!(!sheds.contains("twin"), "{out}");

    let j: Value = serde_json::from_str(&yaks(
        &r.dir,
        "main-actor",
        &["show", &id, "--sheds", "--json"],
    ))
    .unwrap();
    assert_eq!(j["id"], id.as_str());
    let arr = j["sheds"].as_array().unwrap();
    assert_eq!(arr.len(), 2, "{j}");
    let c = arr.iter().find(|s| s["shed"] == "shed-worker").unwrap();
    assert_eq!(c["status"], "shaving");
    assert_eq!(c["same"], false);
    assert_eq!(c["absent"], false);
    assert!(c["needs"].is_null());
    assert!(c["path"].as_str().unwrap().ends_with("changed"));
    let notes = c["new_notes"].as_array().unwrap();
    assert_eq!(notes.len(), 2, "claim transition + the note: {j}");
    assert_eq!(notes[1]["actor"], "shed-worker");
    assert_eq!(notes[1]["text"], "first line of shed note\nsecond line");
    let t = arr.iter().find(|s| s["same"] == true).unwrap();
    assert_eq!(t["status"], "hairy");
    assert_eq!(t["new_notes"].as_array().unwrap().len(), 0);
    // Without the flag the object has no `sheds` key.
    let plain: Value =
        serde_json::from_str(&yaks(&r.dir, "main-actor", &["show", &id, "--json"])).unwrap();
    assert!(plain.get("sheds").is_none(), "{plain}");
}

#[test]
fn a_shed_asking_shows_its_needs() {
    let (r, id) = Repo::new("needs", &["asker"]);
    yaks(
        &r.sheds[0],
        "shed-worker",
        &["ask", &id, "--note", "Which db?"],
    );
    let out = yaks(&r.dir, "main-actor", &["show", &id, "--sheds"]);
    assert!(out.contains("needs:  human"), "{out}");
    assert!(out.contains("asked: needs human"), "{out}");
}

#[test]
fn a_yak_only_in_a_shed_is_shown_with_sheds_and_still_exits_1_without() {
    let (r, _) = Repo::new("only", &["maker", "bare"]);
    let v: Value = serde_json::from_str(&yaks(
        &r.sheds[0],
        "shed-worker",
        &["create", "shed-born", "--json"],
    ))
    .unwrap();
    let id = v["id"].as_str().unwrap();

    let plain = run(&r.dir, "main-actor", &["show", id]);
    assert_eq!(plain.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&plain.stderr).contains("no such task"));

    let out = run(&r.dir, "main-actor", &["show", id, "--sheds"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{text}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(text.contains("not in this checkout"), "{text}");
    assert!(text.contains("shed-born"), "title shown:\n{text}");
    // A new yak has no notes, so the shed is named by its branch.
    assert!(text.contains("  maker  "), "{text}");
    assert!(text.contains("status: hairy"), "{text}");
    assert!(text.contains("absent in 1"), "{text}");

    let j: Value = serde_json::from_str(&yaks(
        &r.dir,
        "main-actor",
        &["show", id, "--sheds", "--json"],
    ))
    .unwrap();
    assert_eq!(j["sheds"].as_array().unwrap().len(), 2, "{j}");

    // Absent everywhere still fails, flag or not.
    let none = run(&r.dir, "main-actor", &["show", "nope-0000", "--sheds"]);
    assert_eq!(none.status.code(), Some(1));
}
