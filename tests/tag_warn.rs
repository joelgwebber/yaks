//! Unknown-tag warnings (yaks-bcd7): `create --tags`, `update --add-tag` and
//! `bulk --add-tag` say, on stderr, when a tag is not among the farm's known
//! tags (`schemas/yak.md` `tags:`). Advice only: the write still happens.

mod common;
use assert_cmd::Command;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-tagwarn-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f
    }

    /// (stdout, stderr); every call must succeed.
    fn run(&self, args: &[&str]) -> (String, String) {
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
        (
            String::from_utf8(out.stdout).unwrap(),
            String::from_utf8(out.stderr).unwrap(),
        )
    }

    fn known_tags(&self, tags: &str) {
        let root = PathBuf::from(self.run(&["path"]).0.trim());
        std::fs::create_dir_all(root.join("schemas")).unwrap();
        std::fs::write(
            root.join("schemas/yak.md"),
            format!("---\ntype: Dok Type\ntags:\n{tags}---\n"),
        )
        .unwrap();
    }

    fn id(&self, args: &[&str]) -> String {
        let mut a = vec!["create", "A yak", "--json"];
        a.extend_from_slice(args);
        let v: serde_json::Value = serde_json::from_str(&self.run(&a).0).unwrap();
        v["id"].as_str().unwrap().to_string()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

const KNOWN: &str = "  ui: The TUI\n  docs: Documentation\n  cli: Commands and flags\n";

#[test]
fn create_warns_on_an_unknown_tag_and_still_creates() {
    let f = Farm::new("create");
    f.known_tags(KNOWN);
    let (out, err) = f.run(&["create", "A yak", "--tags", "dox,ui"]);
    assert!(out.starts_with("Created "), "{out}");
    assert_eq!(
        err.trim(),
        "warning: tag \"dox\" is not among this farm's known tags (schemas/yak.md); known: docs, ui, cli",
        "nearest first"
    );
}

#[test]
fn known_tags_are_silent_however_they_are_spelled() {
    let f = Farm::new("known");
    f.known_tags(KNOWN);
    let (_, err) = f.run(&["create", "A yak", "--tags", "ui, docs"]);
    assert_eq!(err, "");
    let id = f.id(&[]);
    let (_, err) = f.run(&["update", &id, "--add-tag", "cli,ui"]);
    assert_eq!(err, "");
}

#[test]
fn update_and_bulk_warn_on_an_unknown_added_tag() {
    let f = Farm::new("update");
    f.known_tags(KNOWN);
    let id = f.id(&["--tags", "ui"]);
    let (_, err) = f.run(&["update", &id, "--add-tag", "wibble"]);
    assert!(err.contains("tag \"wibble\" is not among"), "{err}");
    let shown = f.run(&["show", &id]).0;
    assert!(shown.contains("wibble"), "still written: {shown}");
    // Removing an unknown tag is not a write of one.
    let (_, err) = f.run(&["update", &id, "--remove-tag", "wibble"]);
    assert_eq!(err, "");
    let (_, err) = f.run(&["bulk", "--tag", "ui", "--add-tag", "zzz"]);
    assert!(err.contains("tag \"zzz\" is not among"), "{err}");
}

#[test]
fn at_most_five_known_tags_are_named() {
    let f = Farm::new("five");
    f.known_tags("  a: a\n  b: b\n  c: c\n  d: d\n  e: e\n  f: f\n  g: g\n");
    let (_, err) = f.run(&["create", "A yak", "--tags", "x"]);
    assert!(err.ends_with("known: a, b, c, d, e\n"), "{err}");
}

#[test]
fn a_farm_without_known_tags_is_silent() {
    let f = Farm::new("none");
    let (_, err) = f.run(&["create", "A yak", "--tags", "anything"]);
    assert_eq!(err, "");
    // A schema with no `tags:` map is silent too.
    let root = PathBuf::from(f.run(&["path"]).0.trim());
    std::fs::create_dir_all(root.join("schemas")).unwrap();
    std::fs::write(root.join("schemas/yak.md"), "---\ntype: Dok Type\n---\n").unwrap();
    let (_, err) = f.run(&["create", "A yak", "--tags", "anything"]);
    assert_eq!(err, "");
}
