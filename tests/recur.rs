//! Due dates and recurring yaks through the real binary (yaks-2fc5): `--due`
//! and `--every`, `list --due`, overdue, and `regrow-due`.

mod common;
use assert_cmd::Command;
use chrono::{Duration, Utc};
use serde_json::Value;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-recur-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f
    }

    fn cmd(&self, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        let out = self.cmd(args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn create(&self, args: &[&str]) -> String {
        let mut a = vec!["create", "Check the salt", "--json"];
        a.extend_from_slice(args);
        let v: Value = serde_json::from_str(&self.run(&a)).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn show(&self, id: &str) -> Value {
        serde_json::from_str(&self.run(&["show", id, "--json"])).unwrap()
    }

    fn commits(&self) -> usize {
        let farm = self.run(&["path"]);
        let out = std::process::Command::new("git")
            .arg("-C")
            .arg(farm.trim())
            .args(["rev-list", "--count", "HEAD"])
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap();
        String::from_utf8(out.stdout)
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn days_ago(n: i64) -> String {
    (Utc::now() - Duration::days(n))
        .format("%Y-%m-%d")
        .to_string()
}

fn days_ahead(n: i64) -> String {
    (Utc::now() + Duration::days(n))
        .format("%Y-%m-%d")
        .to_string()
}

#[test]
fn a_weekly_yak_regrows_once_for_the_latest_occurrence_and_then_rests() {
    let f = Farm::new("weekly");
    let due = days_ago(14);
    let id = f.create(&["--every", "P1W", "--due", &due]);
    f.run(&["shorn", &id]);

    let dry = f.run(&["regrow-due", "--dry-run"]);
    assert!(dry.contains("Would regrow"), "{dry}");
    assert_eq!(f.show(&id)["status"], "shorn", "a dry run writes nothing");

    // Occurrences: due+7 (a week ago) and due+14 (today). One regrow covers
    // both: the latest is today, one skipped, and the next deadline is a week on.
    let before = f.commits();
    let out = f.run(&["regrow-due"]);
    assert!(out.contains(&format!("Regrown: {id}")), "{out}");
    assert!(out.contains("1 skipped"), "{out}");
    assert_eq!(f.commits(), before + 1, "one commit per yak");

    let v = f.show(&id);
    assert_eq!(v["status"], "hairy");
    assert_eq!(v["extra"]["due"], days_ahead(7));
    assert_eq!(v["extra"]["every"], "P1W");
    assert!(v["extra"]["regrown"].as_str().unwrap().ends_with('Z'));
    let body = v["description"].as_str().unwrap();
    assert!(body.contains("kind: regrow"), "{body}");
    assert!(
        body.contains(&format!(
            "regrew for the {} occurrence (1 skipped)",
            days_ago(0)
        )),
        "{body}"
    );

    // Idempotent: the new due is in the future, and a hairy yak is not a candidate.
    assert_eq!(f.run(&["regrow-due"]).trim(), "Nothing to regrow.");
    f.run(&["shorn", &id]);
    assert_eq!(
        f.run(&["regrow-due"]).trim(),
        "Nothing to regrow.",
        "shorn again before the next occurrence: it waits"
    );
}

#[test]
fn due_lists_sorts_marks_overdue_and_refuses_bad_values() {
    let f = Farm::new("due");
    let late = f.create(&["--due", &days_ago(3)]);
    let soon = f.create(&["--due", &days_ahead(3)]);
    let _none = f.create(&[]);

    let out = f.run(&["list", "--due"]);
    let rows: Vec<&str> = out.lines().collect();
    assert_eq!(rows.len(), 2, "{out}");
    assert!(
        rows[0].contains(&late) && rows[0].contains("overdue"),
        "{out}"
    );
    assert!(
        rows[1].contains(&soon) && !rows[1].contains("overdue"),
        "{out}"
    );

    let before = f.run(&["list", "--due", "--before", &days_ago(0)]);
    assert!(
        before.contains(&late) && !before.contains(&soon),
        "{before}"
    );

    assert_eq!(f.show(&late)["overdue"], true);
    assert!(f.show(&soon).get("overdue").is_none());

    // A shorn yak is not overdue.
    f.run(&["shorn", &late]);
    assert!(!f.run(&["list", "--due"]).contains("overdue"));

    for args in [
        vec!["create", "x", "--due", "soon"],
        vec!["create", "x", "--every", "weekly"],
        vec!["update", &late, "--every", "P0D"],
        vec!["list", "--due", "--before", "soon"],
    ] {
        let out = f.cmd(&args);
        assert!(!out.status.success(), "{args:?} should be refused");
    }
    // `--set` is the unchecked escape hatch: `doctor` is what flags it.
    f.run(&["update", &late, "--set", "due=tomorrow"]);
    // Clearing is allowed.
    f.run(&["update", &late, "--due", ""]);
    assert!(f.show(&late)["extra"].get("due").is_none());
}

#[test]
fn dead_yaks_never_regrow_and_a_shorn_parent_reopens_with_its_child() {
    let f = Farm::new("family");
    let parent = f.create(&[]);
    let kid = f.create(&["--parent", &parent, "--every", "P1D", "--due", &days_ago(5)]);
    let dead = f.create(&["--every", "P1D", "--due", &days_ago(5)]);
    f.run(&["shorn", &kid]);
    f.run(&["shorn", &parent]);
    f.run(&["slaughter", &dead]);

    let out = f.run(&["regrow-due"]);
    assert!(
        out.contains(&format!("Regrown: {parent} (parent of {kid})")),
        "{out}"
    );
    assert_eq!(f.show(&kid)["status"], "hairy");
    assert_eq!(f.show(&parent)["status"], "hairy");
    assert_eq!(f.show(&dead)["status"], "dead");
}
