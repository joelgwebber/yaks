//! `yaks tick` through the real binary (yaks-5e8d): regrow what is due, then
//! one report of what needs someone.

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
        let dir = std::env::temp_dir().join(format!("yaks-tick-{tag}-{}", std::process::id()));
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

    fn create(&self, title: &str, args: &[&str]) -> String {
        let mut a = vec!["create", title, "--json"];
        a.extend_from_slice(args);
        let v: Value = serde_json::from_str(&self.run(&a)).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn tick_json(&self, args: &[&str]) -> Value {
        let mut a = vec!["tick", "--json"];
        a.extend_from_slice(args);
        serde_json::from_str(&self.run(&a)).unwrap()
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
        String::from_utf8(out.stdout).unwrap().trim().parse().unwrap()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn day(offset: i64) -> String {
    (Utc::now() + Duration::days(offset))
        .format("%Y-%m-%d")
        .to_string()
}

fn ids(v: &Value) -> Vec<String> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|y| y["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn an_empty_farm_says_nothing_needs_you() {
    let f = Farm::new("empty");
    assert_eq!(f.run(&["tick"]), "tick: nothing needs you.\n");
    let v = f.tick_json(&[]);
    assert_eq!(v["quiet"], true);
    assert_eq!(v["regrew"].as_array().unwrap().len(), 0);
}

#[test]
fn tick_regrows_then_reports_and_a_second_tick_regrows_nothing() {
    let f = Farm::new("round");
    let weekly = f.create("Sweep the queue", &["--every", "P1W", "--wake", "agent", "--due", &day(-9)]);
    f.run(&["shorn", &weekly]);
    let late = f.create("Late one", &["--due", &day(-2)]);
    let soon = f.create("Soon one", &["--due", &day(2)]);
    let far = f.create("Far one", &["--due", &day(30)]);
    let asked = f.create("Needs a decision", &[]);
    f.run(&["ask", &asked, "--note", "which one?"]);

    let text = f.run(&["tick"]);
    assert!(text.starts_with("tick: 1 regrew, 1 overdue, 1 due soon, 1 awaiting you, 1 awaiting an agent."), "{text}");
    for needle in [
        "Regrew:",
        &weekly,
        "awaiting agent",
        "Overdue:",
        &late,
        "Due within 3d:",
        &soon,
        "Awaiting you (`yaks answer <id>`):",
        &asked,
        "Awaiting an agent (`yaks pickup <id>`):",
    ] {
        assert!(text.contains(needle), "{needle} missing from:\n{text}");
    }
    assert!(!text.contains(&far), "a yak due in a month is not due soon:\n{text}");

    // Idempotent: the regrown yak is hairy and waiting now, so nothing regrows again.
    let before = f.commits();
    let v = f.tick_json(&[]);
    assert_eq!(v["regrew"].as_array().unwrap().len(), 0);
    assert_eq!(ids(&v["awaiting_agent"]), vec![weekly.clone()]);
    assert_eq!(ids(&v["awaiting_human"]), vec![asked]);
    assert_eq!(ids(&v["overdue"]), vec![late]);
    assert_eq!(ids(&v["due_soon"]), vec![soon]);
    assert_eq!(f.commits(), before, "a second tick writes nothing");
}

#[test]
fn dry_run_reports_what_would_regrow_and_writes_nothing() {
    let f = Farm::new("dry");
    let id = f.create("Weekly", &["--every", "P1W", "--wake", "human", "--due", &day(-9)]);
    f.run(&["shorn", &id]);
    let before = f.commits();

    let text = f.run(&["tick", "--dry-run"]);
    assert!(text.starts_with("tick: 1 would regrow."), "{text}");
    assert!(text.contains("Would regrow:") && text.contains("(dry run"), "{text}");
    let v = f.tick_json(&["--dry-run"]);
    assert_eq!(v["dry_run"], true);
    assert_eq!(v["regrew"][0]["id"], id.as_str());
    assert_eq!(f.commits(), before);
    assert_eq!(f.run(&["show", &id, "--json"]).contains("\"status\": \"shorn\""), true);

    // The real tick then does what the dry run said.
    f.run(&["tick"]);
    let shown: Value = serde_json::from_str(&f.run(&["show", &id, "--json"])).unwrap();
    assert_eq!(shown["status"], "hairy");
    assert_eq!(shown["needs"], "human");
}

#[test]
fn a_held_yak_is_reported_and_within_widens_the_window() {
    let f = Farm::new("held");
    let bad = f.create("Bad wake", &["--every", "P1D", "--due", &day(-3)]);
    f.run(&["update", &bad, "--set", "wake=robot"]);
    f.run(&["shorn", &bad]);
    let later = f.create("Next week", &["--due", &day(6)]);

    let text = f.run(&["tick"]);
    assert!(text.contains("1 held"), "{text}");
    assert!(text.contains("Held (could not regrow):") && text.contains(&bad), "{text}");
    assert!(!text.contains(&later), "{text}");

    let wide = f.run(&["tick", "--within", "1w"]);
    assert!(wide.contains("Due within 1w:") && wide.contains(&later), "{wide}");
    assert_eq!(ids(&f.tick_json(&["--within", "1w"])["due_soon"]), vec![later]);
}

#[test]
fn a_bad_within_is_refused_before_anything_is_written() {
    let f = Farm::new("within");
    let id = f.create("Weekly", &["--every", "P1W", "--due", &day(-9)]);
    f.run(&["shorn", &id]);
    let before = f.commits();
    for bad in ["soon", "-1d", "3", "d"] {
        let flag = format!("--within={bad}");
        let out = f.cmd(&["tick", &flag]);
        assert!(!out.status.success(), "{bad}");
        assert!(String::from_utf8_lossy(&out.stderr).contains("--within"), "{bad}");
    }
    assert_eq!(f.commits(), before);
}

#[test]
fn findings_are_counted_not_listed() {
    let f = Farm::new("findings");
    let done = f.create("Done without a note", &[]);
    f.run(&["shave", &done]);
    f.run(&["shorn", &done]);
    // A page nobody cites is a `check` finding but not a doctor one.
    f.run(&["dok", "new", "doks/lonely.md", "--set", "stale_after=P1D", "--set", "verified=2020-01-01"]);
    let v = f.tick_json(&[]);
    let n = v["findings"].as_u64().unwrap();
    assert!(n >= 1, "{v}");
    let text = f.run(&["tick"]);
    assert!(text.contains("check finding") && text.contains("run `yaks check`"), "{text}");
}
