//! `prs:` through the real binary (yaks-d695): `create --pr`,
//! `update --pr` / `--remove-pr`, and the `prs` lines in `show` / `show --json`.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

const P1: &str = "https://github.com/o/r/pull/1";
const P2: &str = "https://github.com/o/r/pull/2";

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-prs-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f
    }

    fn run(&self, args: &[&str]) -> String {
        let out = Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
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

    fn create(&self, args: &[&str]) -> String {
        let mut a = vec!["create", "A yak", "--json"];
        a.extend_from_slice(args);
        let v: Value = serde_json::from_str(&self.run(&a)).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn show_json(&self, id: &str) -> Value {
        serde_json::from_str(&self.run(&["show", id, "--json"])).unwrap()
    }

    fn file(&self, id: &str) -> String {
        std::fs::read_to_string(self.run(&["path", id]).trim()).unwrap()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

#[test]
fn no_prs_means_no_prs_key_in_json_or_text() {
    let f = Farm::new("none");
    let id = f.create(&[]);
    assert!(f.show_json(&id).get("prs").is_none());
    assert!(!f.run(&["show", &id]).contains("prs:"));
    assert!(!f.file(&id).contains("prs:"));
}

#[test]
fn update_adds_and_removes_prs_and_adding_twice_is_a_no_op() {
    let f = Farm::new("update");
    let id = f.create(&[]);
    f.run(&["update", &id, "--pr", P1, "--pr", P2]);
    assert_eq!(f.show_json(&id)["prs"], serde_json::json!([P1, P2]));
    assert!(f.file(&id).contains(&format!("prs:\n- {P1}\n- {P2}\n")));
    assert!(
        f.run(&["show", &id])
            .contains(&format!("prs:      {P1}, {P2}"))
    );

    // Re-adding an existing URL changes nothing (not even the file).
    let before = f.file(&id);
    f.run(&["update", &id, "--pr", P1]);
    assert_eq!(f.file(&id), before);

    f.run(&["update", &id, "--remove-pr", P1]);
    assert_eq!(f.show_json(&id)["prs"], serde_json::json!([P2]));
    f.run(&["update", &id, "--remove-pr", P2]);
    assert!(f.show_json(&id).get("prs").is_none());
    assert!(!f.file(&id).contains("prs:"));
}

#[test]
fn create_takes_repeatable_pr() {
    let f = Farm::new("create");
    let id = f.create(&["--pr", P1, "--pr", P2, "--pr", P1]);
    assert_eq!(f.show_json(&id)["prs"], serde_json::json!([P1, P2]));
}
