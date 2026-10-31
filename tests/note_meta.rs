//! The note-metadata surface through the real binary (yaks-b8cd): `update
//! --meta`, the `kind` every conventional command writes, `show --json` /
//! `log --json`, and `doctor`'s needs-drift check.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-meta-{tag}-{}", std::process::id()));
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

    fn fail(&self, args: &[&str]) -> String {
        let out = self.cmd(args);
        assert!(!out.status.success(), "{args:?} unexpectedly succeeded");
        String::from_utf8(out.stderr).unwrap()
    }

    fn create(&self, title: &str) -> String {
        let v: Value = serde_json::from_str(&self.run(&["create", title, "--json"])).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn notes(&self, id: &str) -> Vec<Value> {
        let v: Value = serde_json::from_str(&self.run(&["show", id, "--json"])).unwrap();
        v["notes"].as_array().cloned().unwrap_or_default()
    }

    fn body(&self, id: &str) -> String {
        std::fs::read_to_string(self.run(&["path", id]).trim()).unwrap()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

#[test]
fn update_meta_writes_lines_under_the_marker_and_show_json_lists_them() {
    let f = Farm::new("update");
    let id = f.create("M");
    assert!(f.notes(&id).is_empty());
    f.run(&[
        "update",
        &id,
        "--note",
        "saved",
        "--meta",
        "kind=checkpoint",
        "--meta",
        "commit=47076b0",
        "--meta",
        "msg=a=b: c",
    ]);
    let body = f.body(&id);
    assert!(
        body.contains("kind: checkpoint\ncommit: 47076b0\nmsg: 'a=b: c'\n---\n\nsaved")
            || body.contains("kind: checkpoint\ncommit: 47076b0\nmsg: \"a=b: c\"\n---\n\nsaved"),
        "{body}"
    );
    let notes = f.notes(&id);
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0]["text"], "saved");
    assert_eq!(notes[0]["actor"], "tester");
    assert_eq!(notes[0]["meta"]["kind"], "checkpoint");
    assert_eq!(notes[0]["meta"]["commit"], "47076b0");
    assert_eq!(notes[0]["meta"]["msg"], "a=b: c");
    // Key order is file order.
    let keys: Vec<&str> = notes[0]["meta"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["kind", "commit", "msg"]);

    // A stdin note takes metadata too.
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&f.dir)
        .env("XDG_DATA_HOME", common::data_home(&f.dir))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .args(["update", &id, "--note", "-", "--meta", "kind=decision"])
        .write_stdin("from stdin\n")
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(f.notes(&id)[1]["meta"]["kind"], "decision");

    // `log --json` carries the same metadata; a plain note has `{}`.
    f.run(&["update", &id, "--note", "plain"]);
    let log: Vec<Value> = serde_json::from_str(&f.run(&["log", "--json"])).unwrap();
    assert_eq!(log[0]["meta"]["commit"], "47076b0");
    assert_eq!(log[2]["meta"], serde_json::json!({}));
}

#[test]
fn update_meta_rejects_a_bad_key_a_missing_equals_and_a_missing_note() {
    let f = Farm::new("bad");
    let id = f.create("B");
    let e = f.fail(&["update", &id, "--note", "x", "--meta", "no-equals"]);
    assert!(e.contains("expected key=value"), "{e}");
    let e = f.fail(&["update", &id, "--note", "x", "--meta", "bad key=1"]);
    assert!(e.contains("not a valid key"), "{e}");
    let e = f.fail(&["update", &id, "--note", "x", "--meta", "1a=1"]);
    assert!(e.contains("not a valid key"), "{e}");
    let e = f.fail(&["update", &id, "--title", "t", "--meta", "kind=x"]);
    assert!(e.contains("needs a note"), "{e}");
    assert!(f.notes(&id).is_empty(), "a rejected update writes nothing");
}

#[test]
fn conventional_commands_write_kinds_and_verify_writes_the_result() {
    let f = Farm::new("kinds");
    let id = f.create("K");
    f.run(&["ask", &id, "--note", "which?"]);
    f.run(&["answer", &id, "--note", "this one"]);
    f.run(&["pickup", &id]);
    f.run(&["shave", &id]);
    f.run(&["update", &id, "--verify", "true"]);
    f.run(&["verify", &id]);
    f.run(&["update", &id, "--verify", "false"]);
    let _ = f.cmd(&["verify", &id]);
    let notes = f.notes(&id);
    let got: Vec<(&str, &str)> = notes
        .iter()
        .map(|n| {
            (
                n["meta"]["kind"].as_str().unwrap_or("-"),
                n["text"].as_str().unwrap().lines().next().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            ("ask", "asked: needs human"),
            ("answer", "this one"),
            ("pickup", "picked up"),
            ("move", "moved: hairy -> shaving"),
            ("verify", "verify: `true` -> PASS (exit 0)"),
            ("verify", "verify: `false` -> FAIL (exit 1)"),
        ]
    );
    assert_eq!(notes[0]["meta"]["needs"], "human");
    assert_eq!(notes[3]["meta"]["from"], "hairy");
    assert_eq!(notes[3]["meta"]["to"], "shaving");
    assert_eq!(notes[4]["meta"]["result"], "pass");
    assert_eq!(notes[5]["meta"]["result"], "fail");
    f.run(&["request", &id, "--note", "tidy"]);
    assert_eq!(f.notes(&id)[6]["meta"]["kind"], "request");
}

#[test]
fn doctor_flags_needs_drift_against_the_latest_kinded_note() {
    let f = Farm::new("drift");
    let id = f.create("D");
    f.run(&["ask", &id, "--note", "q"]);
    assert!(f.run(&["doctor"]).contains("All clear"));

    // Hand-edit the frontmatter `needs:` (the first match) to disagree with the ask note.
    let path = f.run(&["path", &id]);
    let path = path.trim();
    let text = std::fs::read_to_string(path).unwrap();
    std::fs::write(path, text.replacen("needs: human", "needs: elsewhere", 1)).unwrap();
    let out = f.cmd(&["doctor"]);
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("Needs drift") && stdout.contains("leaves needs: human"),
        "{stdout}"
    );
    let json = f.cmd(&["doctor", "--json"]);
    let json = String::from_utf8(json.stdout).unwrap();
    assert!(json.contains("needs-drift"), "{json}");
}
