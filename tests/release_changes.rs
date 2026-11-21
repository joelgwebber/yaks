//! `yaks release changes` (yaks-045c): what a release's commit range names,
//! computed from git and the farm, through the real binary on a temp repo with
//! a scratch farm and tags.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

struct Repo {
    dir: PathBuf,
    n: std::cell::Cell<u32>,
}

impl Repo {
    fn new(tag: &str) -> Repo {
        let dir = std::env::temp_dir().join(format!("yaks-release-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let r = Repo {
            dir,
            n: Default::default(),
        };
        r.run(&["init", "--no-skills"]);
        r
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

    fn git(&self, args: &[&str]) -> String {
        let out = std::process::Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@t"])
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .current_dir(&self.dir)
            .env_remove("GIT_DIR")
            .env_remove("GIT_WORK_TREE")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// A commit changing a file; returns its full sha.
    fn commit(&self, message: &str) -> String {
        self.commit_in("f.txt", message)
    }

    fn commit_in(&self, file: &str, message: &str) -> String {
        let n = self.n.get() + 1;
        self.n.set(n);
        std::fs::write(self.dir.join(file), format!("{n}\n")).unwrap();
        self.git(&["add", file]);
        self.git(&["commit", "-q", "-m", message]);
        self.git(&["rev-parse", "HEAD"])
    }

    fn create(&self, title: &str, kind: &str, parent: Option<&str>) -> String {
        let mut args = vec!["create", title, "--type", kind, "--json"];
        if let Some(p) = parent {
            args.extend(["--parent", p]);
        }
        let v: Value = serde_json::from_str(&self.run(&args)).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn changes(&self, args: &[&str]) -> String {
        let mut a = vec!["release", "changes"];
        a.extend(args);
        self.run(&a)
    }

    fn changes_json(&self, args: &[&str]) -> Value {
        let mut a = vec!["--json"];
        a.extend(args);
        serde_json::from_str(&self.changes(&a)).unwrap()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

/// The yak entry of `id` in a `--json` report.
fn yak<'a>(j: &'a Value, id: &str) -> &'a Value {
    j["yaks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|y| y["id"] == id)
        .unwrap_or_else(|| panic!("{id} not in {j}"))
}

struct Release {
    r: Repo,
    feat: String,
    by_note: String,
    umbrella: String,
    kid: String,
    gap: String,
    internal: String,
    open: String,
    note_sha: String,
}

/// Two releases: `v0.1.0` (an initial commit), then a range of commits that
/// exercises every group, tagged `v0.2.0`.
fn release(tag: &str) -> Release {
    let r = Repo::new(tag);
    r.commit("initial");
    r.git(&["tag", "v0.1.0"]);

    let feat = r.create("A feature", "feature", None);
    let by_note = r.create("Named by a note", "bug", None);
    let umbrella = r.create("Umbrella", "feature", None);
    let kid = r.create("Kid", "task", Some(&umbrella));
    let gap = r.create("Gap", "bug", None);
    let internal = r.create("Internal", "task", None);
    let open = r.create("Open", "task", None);

    r.run(&["shorn", &feat, "--change", "Adds the feature."]);
    r.run(&["shorn", &by_note, "--change", "Fixes the note case."]);
    r.run(&["shorn", &kid]);
    r.run(&["shorn", &umbrella, "--change", "Umbrella lands."]);
    r.run(&["shorn", &gap]);
    r.run(&["shorn", &internal, "--no-change"]);

    r.commit(&format!("{feat}: the feature"));
    // Named only by a `commit:` note, in a message that names no yak.
    let note_sha = r.commit("quiet fix");
    r.run(&[
        "update",
        &by_note,
        "--note",
        "landed quietly",
        "--meta",
        &format!("commit={note_sha}"),
    ]);
    r.commit(&format!("the kid\n\nRefs {kid}."));
    r.commit(&format!("{umbrella}: umbrella"));
    r.commit(&format!("the gap ({gap})"));
    r.commit(&format!("internal [[{internal}]]"));
    r.commit(&format!("partial work on {open}"));
    r.commit("stray work");
    r.commit("Bump version to 0.2.0.");
    r.git(&["tag", "v0.2.0"]);
    Release {
        r,
        feat,
        by_note,
        umbrella,
        kid,
        gap,
        internal,
        open,
        note_sha,
    }
}

#[test]
fn a_range_names_its_yaks_by_message_and_by_note_and_groups_them() {
    let s = release("groups");
    let text = s.r.changes(&["v0.1.0..v0.2.0"]);
    assert!(
        text.starts_with("Changes v0.1.0..v0.2.0: 9 commits, 7 yaks"),
        "{text}"
    );
    // Sections in order: Added (feature) before Fixed (bug).
    let (added, fixed) = (
        text.find("\nAdded\n").unwrap(),
        text.find("\nFixed\n").unwrap(),
    );
    assert!(added < fixed, "{text}");
    assert!(
        text.contains(&format!("- Adds the feature. ({}; ", s.feat)),
        "{text}"
    );
    assert!(
        text.contains(&format!("- Fixes the note case. ({}; ", s.by_note)),
        "{text}"
    );
    // The kid is listed under its umbrella, covered, not a gap.
    assert!(
        text.contains(&format!("- Umbrella lands. ({}; ", s.umbrella)),
        "{text}"
    );
    assert!(
        text.contains(&format!("  - Kid ({}, covered; ", s.kid)),
        "{text}"
    );
    assert!(
        text.contains(&format!("No user-facing change (1): {}", s.internal)),
        "{text}"
    );
    assert!(
        text.contains(&format!("Shorn with no change line (1)\n- {}: Gap", s.gap)),
        "{text}"
    );
    assert!(
        text.contains(&format!(
            "Named by commits but not shorn (1)\n- {} [hairy]: Open",
            s.open
        )),
        "{text}"
    );
    // The stray commit is listed; the version bump is not.
    assert!(text.contains("Commits naming no yak (1)"), "{text}");
    assert!(text.contains("stray work"), "{text}");
    assert!(!text.contains("Bump version"), "{text}");
}

#[test]
fn json_has_the_range_each_yak_and_the_commits_naming_none() {
    let s = release("json");
    let j = s.r.changes_json(&["v0.1.0..v0.2.0"]);
    assert_eq!(j["range"]["from"], "v0.1.0");
    assert_eq!(j["range"]["to"], "v0.2.0");
    let f = yak(&j, &s.feat);
    assert_eq!(f["title"], "A feature");
    assert_eq!(f["type"], "feature");
    assert_eq!(f["status"], "shorn");
    assert_eq!(f["change"], "Adds the feature.");
    assert_eq!(f["section"], "added");
    assert_eq!(
        f["commits"][0]["subject"],
        format!("{}: the feature", s.feat)
    );
    assert_eq!(f["covered_by"], Value::Null);
    // Named by a note only: its commit is the recorded one.
    let n = yak(&j, &s.by_note);
    assert_eq!(n["section"], "fixed");
    assert_eq!(n["commits"][0]["sha"], s.note_sha.as_str());
    assert_eq!(n["commits"][0]["subject"], "quiet fix");
    let k = yak(&j, &s.kid);
    assert_eq!(k["parent"], s.umbrella.as_str());
    assert_eq!(k["group"], "covered");
    assert_eq!(k["covered_by"], s.umbrella.as_str());
    assert_eq!(k["change"], Value::Null);
    assert_eq!(yak(&j, &s.gap)["group"], "gap");
    assert_eq!(yak(&j, &s.internal)["group"], "no_change");
    assert_eq!(yak(&j, &s.open)["group"], "unfinished");
    assert_eq!(yak(&j, &s.open)["status"], "hairy");
    assert_eq!(j["gaps"], serde_json::json!([s.gap]));
    let loose = j["commits_without_yak"].as_array().unwrap();
    assert_eq!(loose.len(), 1);
    assert_eq!(loose[0]["subject"], "stray work");
}

#[test]
fn check_exits_1_on_a_gap_after_printing_the_report_and_0_once_it_has_a_line() {
    let s = release("check");
    let out = s.r.cmd(&["release", "changes", "v0.2.0", "--check"]);
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("Shorn with no change line (1)"), "{text}");
    assert!(
        text.trim_end().ends_with("(add one: yaks update <id> --note \"<line>\" --meta kind=change, or record --no-change)"),
        "{text}"
    );
    // Without --check the same report exits 0.
    assert!(s.r.cmd(&["release", "changes", "v0.2.0"]).status.success());
    // JSON keeps stdout a document and still fails.
    let j =
        s.r.cmd(&["release", "changes", "v0.2.0", "--check", "--json"]);
    assert_eq!(j.status.code(), Some(1));
    serde_json::from_slice::<Value>(&j.stdout).unwrap();

    s.r.run(&[
        "update",
        &s.gap,
        "--note",
        "Closes the gap.",
        "--meta",
        "kind=change",
    ]);
    let out = s.r.cmd(&["release", "changes", "v0.2.0", "--check"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("check: ok"));
}

#[test]
fn a_single_tag_means_the_previous_release_tag_to_it() {
    let s = release("single");
    assert_eq!(
        s.r.changes(&["v0.2.0"]).replace("v0.2.0:", "x"),
        s.r.changes(&["v0.1.0..v0.2.0"]).replace("v0.2.0:", "x")
    );
    let j = s.r.changes_json(&["v0.2.0"]);
    assert_eq!(j["range"]["from"], "v0.1.0");
    // The first release runs from the start of history.
    let first = s.r.changes_json(&["v0.1.0"]);
    assert_eq!(first["range"]["from"], Value::Null);
    assert_eq!(first["commits"], 1);
    // A tag after it: the range is only the new commits.
    let next = s.r.create("Next", "feature", None);
    s.r.run(&["shorn", &next, "--change", "Next thing."]);
    s.r.commit(&format!("{next}: next"));
    s.r.git(&["tag", "v0.3.0"]);
    let j = s.r.changes_json(&["v0.3.0"]);
    assert_eq!(j["range"]["from"], "v0.2.0");
    assert_eq!(j["yaks"].as_array().unwrap().len(), 1);
}

#[test]
fn the_default_range_is_the_latest_release_tag_to_head() {
    let s = release("default");
    let before = s.r.changes_json(&[]);
    assert_eq!(before["range"]["from"], "v0.2.0");
    assert_eq!(before["range"]["to"], "HEAD");
    assert_eq!(before["commits"], 0);
    let late = s.r.create("Late", "bug", None);
    s.r.commit(&format!("{late}: late fix"));
    let after = s.r.changes_json(&[]);
    assert_eq!(after["commits"], 1);
    // An open yak a commit names is listed with its status, not as an entry.
    assert_eq!(yak(&after, &late)["group"], "unfinished");
    // A non-release tag is not a boundary.
    s.r.git(&["tag", "nightly"]);
    assert_eq!(s.r.changes_json(&[])["range"]["from"], "v0.2.0");
}

#[test]
fn a_merge_commit_does_not_name_a_yak() {
    let s = release("merge");
    let side = s.r.create("Side", "task", None);
    s.r.git(&["checkout", "-q", "-b", "side"]);
    s.r.commit_in("side.txt", "side work");
    s.r.git(&["checkout", "-q", "main"]);
    s.r.commit("main work");
    s.r.git(&[
        "merge",
        "--no-ff",
        "-q",
        "-m",
        &format!("Merge side ({side})"),
        "side",
    ]);
    let j = s.r.changes_json(&[]);
    assert!(j["yaks"].as_array().unwrap().is_empty(), "{j}");
    assert_eq!(j["commits"], 2);
}

#[test]
fn yaks_release_tags_overrides_the_glob() {
    let s = release("glob");
    s.r.git(&["config", "yaks.releaseTags", "rel-*"]);
    // No tag matches yet: the default range starts at the beginning.
    let j = s.r.changes_json(&[]);
    assert_eq!(j["range"]["from"], Value::Null);
    s.r.git(&["tag", "rel-1", "v0.2.0"]);
    let late = s.r.create("Late", "bug", None);
    s.r.commit(&format!("{late}: late fix"));
    s.r.git(&["tag", "rel-2"]);
    let j = s.r.changes_json(&[]);
    assert_eq!(j["range"]["from"], "rel-2");
    let j = s.r.changes_json(&["rel-2"]);
    assert_eq!(j["range"]["from"], "rel-1");
    assert_eq!(j["commits"], 1);
}

#[test]
fn it_needs_a_code_checkout() {
    let s = release("nocode");
    let farm = PathBuf::from(s.r.run(&["path"]).trim());
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&farm)
        .env("XDG_DATA_HOME", common::data_home(&s.r.dir))
        .env("YAKS_ACTOR", "tester")
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .args(["release", "changes"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("code repository"));
}
