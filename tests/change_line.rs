//! `yaks shorn --change` / `--no-change` (yaks-bcb8): the change line is a
//! `kind: change` note written with the shear, driven through the real binary.

mod common;
use assert_cmd::Command;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-change-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.cmd(&["init", "--no-skills"]);
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

    fn fail(&self, args: &[&str]) -> String {
        let out = self.cmd(args);
        assert!(!out.status.success(), "{args:?} unexpectedly succeeded");
        String::from_utf8(out.stderr).unwrap()
    }

    fn create(&self, title: &str, kind: &str) -> String {
        let out = self.run(&["create", title, "--type", kind, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    /// The yak's notes with `kind: change`, as `(section, text)`.
    fn changes(&self, id: &str) -> Vec<(Option<String>, String)> {
        let v: serde_json::Value =
            serde_json::from_str(&self.run(&["show", id, "--json"])).unwrap();
        v["notes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|n| n["meta"]["kind"] == "change")
            .map(|n| {
                (
                    n["meta"]["section"].as_str().map(str::to_string),
                    n["text"].as_str().unwrap().trim().to_string(),
                )
            })
            .collect()
    }

    fn status(&self, id: &str) -> String {
        let v: serde_json::Value =
            serde_json::from_str(&self.run(&["show", id, "--json"])).unwrap();
        v["status"].as_str().unwrap().to_string()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

#[test]
fn shorn_change_writes_a_kind_change_note_with_no_stored_section() {
    let f = Farm::new("line");
    let id = f.create("Thing", "feature");
    f.run(&["shorn", &id, "--change", "Adds the thing."]);
    assert_eq!(f.status(&id), "shorn");
    assert_eq!(f.changes(&id), vec![(None, "Adds the thing.".to_string())]);
}

#[test]
fn shorn_change_section_is_stored_only_when_given() {
    let f = Farm::new("section");
    let id = f.create("Hole", "bug");
    f.run(&[
        "shorn",
        &id,
        "--change",
        "Closes a hole.",
        "--section",
        "security",
    ]);
    assert_eq!(
        f.changes(&id),
        vec![(Some("security".into()), "Closes a hole.".to_string())]
    );
    let other = f.create("Bad", "task");
    f.fail(&["shorn", &other, "--change", "x", "--section", "bogus"]);
    f.fail(&["shorn", &other, "--section", "fixed"]);
    assert_eq!(f.status(&other), "hairy");
}

#[test]
fn shorn_no_change_writes_section_none() {
    let f = Farm::new("none");
    let (a, b) = (f.create("A", "task"), f.create("B", "task"));
    f.run(&["shorn", &a, &b, "--no-change"]);
    for id in [&a, &b] {
        assert_eq!(f.status(id), "shorn");
        assert_eq!(
            f.changes(id),
            vec![(Some("none".into()), "No user-facing change.".to_string())]
        );
    }
}

#[test]
fn change_and_no_change_conflict_and_several_ids_refuse_change() {
    let f = Farm::new("refuse");
    let (a, b) = (f.create("A", "task"), f.create("B", "task"));
    f.fail(&["shorn", &a, "--change", "x", "--no-change"]);
    let err = f.fail(&["shorn", &a, &b, "--change", "one line"]);
    assert!(err.contains("one yak"), "{err}");
    f.fail(&["shorn", &a, "--change", "  "]);
    // Refused before anything moved or was written.
    for id in [&a, &b] {
        assert_eq!(f.status(id), "hairy");
        assert!(f.changes(id).is_empty());
    }
}

#[test]
fn a_refused_shear_writes_no_change_note() {
    let f = Farm::new("kids");
    let parent = f.create("Parent", "task");
    f.run(&["create", "Kid", "--parent", &parent]);
    f.cmd(&["shorn", &parent, "--change", "never"]);
    assert_eq!(f.status(&parent), "hairy");
    assert!(f.changes(&parent).is_empty());
}

#[test]
fn a_change_line_for_a_yak_already_shorn_is_not_dropped_silently() {
    let f = Farm::new("again");
    let id = f.create("Done", "task");
    f.run(&["shorn", &id]);
    let out = f.cmd(&["shorn", &id, "--change", "late line"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("--meta kind=change"), "{err}");
    assert!(f.changes(&id).is_empty());
}
