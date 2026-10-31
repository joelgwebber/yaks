//! `--run-on` (yaks-ef3b) through the real binary: `list --run-on` keeps the
//! yaks whose description or a note has lines markdown joins into one
//! paragraph, and leaves well-formed ones out.

mod common;
use assert_cmd::Command;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new() -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-run-on-{}", std::process::id()));
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

    fn create(&self, title: &str, description: &str) -> String {
        let out = self.run(&["create", title, "--description", description, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        v["id"].as_str().unwrap().to_string()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

#[test]
fn list_run_on_keeps_only_joined_lines() {
    let f = Farm::new();
    let joined = f.create("Joined", "first line\nsecond line");
    let clean = f.create("Clean", "para one\n\n- a\n- b");
    let in_note = f.create("In a note", "Clean description.");
    f.run(&["update", &in_note, "--note", "one\ntwo"]);

    let out = f.run(&["list", "--run-on"]);
    assert!(out.contains(&joined), "{out}");
    assert!(out.contains(&in_note), "{out}");
    assert!(!out.contains(&clean), "{out}");

    // It composes with the other filters (AND).
    let out = f.run(&["list", "--run-on", "--search", "note"]);
    assert!(out.contains(&in_note) && !out.contains(&joined), "{out}");
}
