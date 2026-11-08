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
    /// A farm whose calendar is UTC, so a date-only `due:` and `+2d` mean the
    /// same day whatever zone the machine running the tests is in.
    fn new(tag: &str) -> Farm {
        Farm::in_zone(tag, "UTC")
    }

    /// A farm with `timezone: <zone>` in its `config.yaml` (yaks-ac6a).
    fn in_zone(tag: &str, zone: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-recur-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f.config(&format!("timezone: {zone}\n"));
        f
    }

    /// Append to the farm's `config.yaml`.
    fn config(&self, text: &str) {
        let path = PathBuf::from(self.run(&["path"]).trim()).join("config.yaml");
        let mut cur = std::fs::read_to_string(&path).unwrap_or_default();
        cur.push_str(text);
        std::fs::write(path, cur).unwrap();
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

/// Today as `--due +Nd` resolves it on a UTC farm.
fn local_days_ahead(n: i64) -> String {
    (Utc::now() + Duration::days(n))
        .format("%Y-%m-%d")
        .to_string()
}

#[test]
fn due_takes_relative_days_and_weeks_and_stores_a_date() {
    let f = Farm::new("relative");
    let id = f.create(&["--due", "+2d"]);
    assert_eq!(f.show(&id)["extra"]["due"], local_days_ahead(2));
    f.run(&["update", &id, "--due", "+1w"]);
    assert_eq!(f.show(&id)["extra"]["due"], local_days_ahead(7));
    f.run(&["update", &id, "--due", "+0d"]);
    assert_eq!(f.show(&id)["extra"]["due"], local_days_ahead(0));

    for bad in ["+d", "+2", "+2m", "-2d", "+2d3", "monday", "+1.5w"] {
        let flag = format!("--due={bad}");
        let out = f.cmd(&["update", &id, &flag]);
        assert!(!out.status.success(), "{bad} should be refused");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("+2d"),
            "{bad}: the error names the form: {err}"
        );
    }
    assert_eq!(f.show(&id)["extra"]["due"], local_days_ahead(0));
}

#[test]
fn snooze_moves_due_from_today_and_leaves_a_note() {
    let f = Farm::new("snooze");
    let a = f.create(&["--due", &days_ago(3)]);
    let b = f.create(&["--due", &days_ahead(1)]);
    assert!(f.run(&["list", "--due"]).contains("overdue"));

    f.run(&["snooze", &a, &b, "--to", "+1w"]);
    for id in [&a, &b] {
        let v = f.show(id);
        assert_eq!(v["extra"]["due"], local_days_ahead(7), "{id}");
        let note = v["notes"].as_array().unwrap().last().unwrap().clone();
        assert_eq!(note["meta"]["kind"], "snooze", "{note}");
        assert!(
            note["text"]
                .as_str()
                .unwrap()
                .contains(&format!("snoozed until {}", local_days_ahead(7))),
            "{note}"
        );
    }
    assert!(!f.run(&["list", "--due"]).contains("overdue"));

    // No `--to`: a day from today. An absolute date works too.
    f.run(&["snooze", &a]);
    assert_eq!(f.show(&a)["extra"]["due"], local_days_ahead(1));
    f.run(&["snooze", &a, "--to", "2031-01-02"]);
    assert_eq!(f.show(&a)["extra"]["due"], "2031-01-02");
}

#[test]
fn snooze_refuses_finished_yaks_and_bad_dates_and_does_the_rest() {
    let f = Farm::new("snooze-refuse");
    let open = f.create(&["--due", &days_ago(1)]);
    let done = f.create(&["--due", &days_ago(1)]);
    f.run(&["shorn", &done]);

    let out = f.cmd(&["snooze", &done, &open, "--to", "+2d"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains(&done) && err.contains("shorn"), "{err}");
    assert_eq!(f.show(&open)["extra"]["due"], local_days_ahead(2));
    assert_eq!(f.show(&done)["extra"]["due"], days_ago(1));

    let out = f.cmd(&["snooze", &open, "--to", "tomorrow"]);
    assert!(!out.status.success());
    assert_eq!(f.show(&open)["extra"]["due"], local_days_ahead(2));
    let out = f.cmd(&["snooze", "nope-0000"]);
    assert!(!out.status.success());
}

fn needs_of(f: &Farm, id: &str) -> Option<String> {
    f.show(id)["needs"].as_str().map(String::from)
}

fn inbox_ids(f: &Farm, who: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(&f.run(&["inbox", "--for", who, "--json"])).unwrap();
    v.as_array()
        .unwrap()
        .iter()
        .map(|y| y["id"].as_str().unwrap().to_string())
        .collect()
}

#[test]
fn wake_agent_puts_the_regrown_yak_in_the_agent_inbox_every_round() {
    let f = Farm::new("wake-agent");
    let id = f.create(&["--every", "P1D", "--wake", "agent", "--due", &days_ago(3)]);
    f.run(&["shorn", &id]);
    assert_eq!(needs_of(&f, &id), None);

    let out = f.run(&["regrow-due"]);
    assert!(out.contains("awaiting agent"), "{out}");
    assert_eq!(f.show(&id)["status"], "hairy");
    assert_eq!(needs_of(&f, &id).as_deref(), Some("agent"));
    assert_eq!(inbox_ids(&f, "agent"), vec![id.clone()]);
    let inbox: Value =
        serde_json::from_str(&f.run(&["inbox", "--for", "agent", "--json"])).unwrap();
    assert_eq!(inbox[0]["why"], "requested");
    assert!(f.run(&["doctor"]).contains("clean") || f.cmd(&["doctor"]).status.success());

    // The agent takes it on and does it; next round it wakes again.
    f.run(&["pickup", &id, "--note", "on it"]);
    assert_eq!(needs_of(&f, &id), None);
    assert!(inbox_ids(&f, "agent").is_empty());
    f.run(&["shave", &id]);
    f.run(&["shorn", &id]);
    f.run(&["update", &id, "--due", &days_ago(2), "--set", "regrown="]);
    f.run(&["regrow-due"]);
    assert_eq!(needs_of(&f, &id).as_deref(), Some("agent"));
    assert_eq!(inbox_ids(&f, "agent"), vec![id.clone()]);
    assert!(
        f.cmd(&["doctor", "--strict"]).status.success(),
        "no needs drift after a second round"
    );
}

#[test]
fn wake_human_asks_and_answer_done_clears_it() {
    let f = Farm::new("wake-human");
    let id = f.create(&["--every", "P1W", "--wake", "human", "--due", &days_ago(10)]);
    f.run(&["shorn", &id]);
    let out = f.run(&["regrow-due"]);
    assert!(out.contains("awaiting human"), "{out}");
    assert_eq!(needs_of(&f, &id).as_deref(), Some("human"));
    assert_eq!(inbox_ids(&f, "human"), vec![id.clone()]);
    assert!(f.cmd(&["doctor"]).status.success());

    f.run(&["answer", &id, "--done", "--note", "looked, fine"]);
    assert_eq!(needs_of(&f, &id), None);
    assert!(inbox_ids(&f, "human").is_empty());
}

#[test]
fn without_wake_regrow_only_reopens_and_an_open_question_is_left_alone() {
    let f = Farm::new("wake-none");
    let plain = f.create(&["--every", "P1D", "--due", &days_ago(3)]);
    let asked = f.create(&["--every", "P1D", "--wake", "agent", "--due", &days_ago(3)]);
    f.run(&["ask", &asked, "--note", "a question from last round"]);
    f.run(&["shorn", &plain]);
    f.run(&["shorn", &asked]);

    f.run(&["regrow-due"]);
    assert_eq!(needs_of(&f, &plain), None);
    assert_eq!(
        needs_of(&f, &asked).as_deref(),
        Some("human"),
        "an unanswered question is not clobbered"
    );
    let notes = f.show(&asked)["notes"].as_array().unwrap().clone();
    assert!(
        !notes.iter().any(|n| n["meta"]["kind"] == "request"),
        "no request written over an open question"
    );
}

#[test]
fn wake_is_checked_and_a_bad_one_holds_the_yak() {
    let f = Farm::new("wake-bad");
    let id = f.create(&["--every", "P1D", "--due", &days_ago(3)]);
    assert!(!f.cmd(&["update", &id, "--wake", "nobody"]).status.success());
    f.run(&["update", &id, "--wake", "agent"]);
    assert_eq!(f.show(&id)["extra"]["wake"], "agent");
    f.run(&["update", &id, "--wake", ""]);
    assert!(f.show(&id)["extra"].get("wake").is_none());

    f.run(&["update", &id, "--set", "wake=robot"]);
    f.run(&["shorn", &id]);
    let out = f.run(&["regrow-due"]);
    assert!(out.contains("Held") && out.contains("wake"), "{out}");
    assert_eq!(f.show(&id)["status"], "shorn");
}

#[test]
fn dry_run_says_who_would_be_woken_and_writes_nothing() {
    let f = Farm::new("wake-dry");
    let id = f.create(&["--every", "P1D", "--wake", "agent", "--due", &days_ago(3)]);
    f.run(&["shorn", &id]);
    let before = f.commits();
    let out = f.run(&["regrow-due", "--dry-run", "--json"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["regrew"][0]["wake"], "agent");
    assert_eq!(f.commits(), before);
    assert_eq!(needs_of(&f, &id), None);
}

// ---- the farm's time zone (yaks-ac6a) --------------------------------------

#[test]
fn a_datetime_due_keeps_its_offset_and_a_daily_yak_keeps_its_wall_clock_across_dst() {
    let f = Farm::in_zone("dst", "America/New_York");
    // 19:30 New York on Oct 31 (EDT). Every day since then is 19:30 there, in
    // EDT before Nov 1 and EST after: never 18:30.
    let id = f.create(&["--every", "P1D", "--due", "2026-10-31T19:30:00-04:00"]);
    assert_eq!(
        f.show(&id)["extra"]["due"],
        "2026-10-31T19:30:00-04:00",
        "the offset it was given is kept, not rewritten to Z"
    );
    f.run(&["shorn", &id]);
    let out = f.run(&["regrow-due"]);
    assert!(out.contains(&format!("Regrown: {id}")), "{out}");
    let v = f.show(&id);
    let due = v["extra"]["due"].as_str().unwrap();
    // The real clock decides the occurrence, so only the wall clock and an
    // offset New York has are fixed.
    assert!(
        due.contains("T19:30:00-04:00") || due.contains("T19:30:00-05:00"),
        "{due}"
    );
    let body = v["description"].as_str().unwrap();
    assert!(body.contains("T19:30:00-0"), "{body}");
}

#[test]
fn a_zone_decides_what_today_is_for_relative_and_date_only_dues() {
    use chrono_tz::Pacific::Auckland;
    use chrono_tz::Pacific::Honolulu;
    // Auckland and Honolulu are 23 hours apart (22 in their summers), so their
    // calendar days differ for most of any day: each farm gets its own date.
    for (zone, tz) in [
        ("Pacific/Auckland", Auckland),
        ("Pacific/Honolulu", Honolulu),
    ] {
        let f = Farm::in_zone(&format!("today-{}", zone.replace('/', "-")), zone);
        let today = Utc::now().with_timezone(&tz).date_naive();
        let id = f.create(&["--due", "+2d"]);
        let want = (today + Duration::days(2)).format("%Y-%m-%d").to_string();
        assert_eq!(f.show(&id)["extra"]["due"], want, "{zone}");
    }
}

#[test]
fn an_unknown_zone_warns_once_and_doctor_reports_it_and_the_machine_zone_is_used() {
    let f = Farm::in_zone("badzone", "Mars/Olympus");
    let out = f.cmd(&["list"]);
    assert!(out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(err.matches("timezone:").count(), 1, "{err}");
    assert!(
        err.contains("warning:") && err.contains("Mars/Olympus"),
        "{err}"
    );
    // Work goes on: a relative due still resolves (on the machine's zone).
    let id = f.create(&["--due", "+1d"]);
    assert!(f.show(&id)["extra"]["due"].as_str().is_some());
    let doctor = f.cmd(&["doctor"]);
    let text = String::from_utf8_lossy(&doctor.stdout);
    assert!(
        text.contains("Bad timezone") && text.contains("Mars/Olympus"),
        "{text}"
    );
    // `--json` stays clean on stderr, as for the other warnings.
    let json = f.cmd(&["list", "--json"]);
    assert!(!String::from_utf8_lossy(&json.stderr).contains("timezone"));
}
