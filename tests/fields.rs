//! Other frontmatter fields through the real binary (yaks-1f34): `update
//! --set key=value` / `--unset key`, how they are written, `show --json`'s
//! parsed `extra`, and the refusals.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-fields-{tag}-{}", std::process::id()));
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
fn set_writes_scalars_lists_and_maps_and_json_reads_them_back() {
    let f = Farm::new("set");
    let id = f.create(&[]);
    f.run(&[
        "update",
        &id,
        "--set",
        "due=2026-11-05",
        "--set",
        "commit=[3f2a9c1, e99754d]",
        "--set",
        "owner={team: web}",
    ]);
    let file = f.file(&id);
    assert!(
        file.contains("due: 2026-11-05\ncommit:\n- 3f2a9c1\n- e99754d\nowner: {team: web}\n---"),
        "{file}"
    );
    let v = f.show_json(&id);
    assert_eq!(v["extra"]["due"], "2026-11-05");
    assert_eq!(
        v["extra"]["commit"],
        serde_json::json!(["3f2a9c1", "e99754d"])
    );
    assert_eq!(v["extra"]["owner"]["team"], "web");
}

#[test]
fn set_rewrites_one_field_and_leaves_hand_written_ones_alone() {
    let f = Farm::new("keep");
    let id = f.create(&[]);
    let path = f.run(&["path", &id]).trim().to_string();
    let hand = "sources:\n  - id: file   # by hand\n    resource: \"/tmp/x.png\"\n";
    let text = f
        .file(&id)
        .replacen("\n---\n", &format!("\n{hand}---\n"), 1);
    std::fs::write(&path, &text).unwrap();
    f.run(&["update", &id, "--set", "due=2026-11-05"]);
    f.run(&["update", &id, "--set", "due=2026-11-06"]);
    let after = f.file(&id);
    assert!(after.contains(hand), "{after}");
    assert_eq!(after.matches("due:").count(), 1, "{after}");
    assert_eq!(
        f.show_json(&id)["extra"]["sources"][0]["resource"],
        "/tmp/x.png"
    );
    f.run(&["update", &id, "--unset", "due", "--set", "sources="]);
    let cleared = f.file(&id);
    assert!(
        !cleared.contains("due:") && !cleared.contains("sources:"),
        "{cleared}"
    );
}

#[test]
fn set_refuses_built_in_and_malformed_keys_and_writes_nothing() {
    let f = Farm::new("refuse");
    let id = f.create(&[]);
    let before = f.file(&id);
    for bad in [
        &["--set", "title=x"][..],
        &["--set", "9x=1"],
        &["--set", "noequals"],
        &["--unset", "tags"],
        &["--set", "labels=x"],
    ] {
        let mut a = vec!["update", id.as_str()];
        a.extend_from_slice(bad);
        let out = Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&f.dir)
            .env("XDG_DATA_HOME", common::data_home(&f.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .args(&a)
            .output()
            .unwrap();
        assert!(!out.status.success(), "{bad:?} should fail");
    }
    assert_eq!(f.file(&id), before);
}
