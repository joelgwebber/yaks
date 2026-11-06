//! Links through the real binary (yaks-101d): `peek`, `refs` on a dok,
//! `mv` rewriting links, and doctor's broken-link check.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

#[allow(dead_code)] // shared helpers; each test file uses some
impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-links-{tag}-{}", std::process::id()));
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

#[allow(dead_code)] // shared helpers; each test file uses some
impl Farm {
    fn root(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
    }

    fn fails(&self, args: &[&str]) -> String {
        let out = Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .args(args)
            .output()
            .unwrap();
        assert!(!out.status.success(), "{args:?} should fail");
        String::from_utf8(out.stderr).unwrap()
    }
}

#[test]
fn peek_refs_and_mv() {
    let f = Farm::new("flow");
    let id = f.create(&[]);
    f.run(&[
        "dok",
        "new",
        "doks/topics/a.md",
        "--body",
        "Intro.\n\n## Setup\n\nSee [b](sub/b.md).\n",
    ]);
    f.run(&[
        "dok",
        "new",
        "doks/topics/sub/b.md",
        "--body",
        "Back to [a](/doks/topics/a.md#setup).",
    ]);
    f.run(&["update", &id, "--note", "Notes in [a](/doks/topics/a.md)."]);

    let peek = f.run(&["peek", "/doks/topics/a.md#setup"]);
    assert_eq!(
        peek,
        "── doks/topics/a.md#setup\n## Setup\n\nSee [b](sub/b.md).\n"
    );

    let inc: Value =
        serde_json::from_str(&f.run(&["refs", "doks/topics/a.md", "--incoming", "--json"]))
            .unwrap();
    let from: Vec<&str> = inc["incoming"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["from"].as_str().unwrap())
        .collect();
    assert_eq!(from, [id.as_str(), "doks/topics/sub/b.md"]);

    let out = f.run(&["mv", "doks/topics/a.md", "doks/design/a.md"]);
    assert!(
        out.contains("Moved doks/topics/a.md -> doks/design/a.md"),
        "{out}"
    );
    let root = f.root();
    assert!(!root.join("doks/topics/a.md").exists());
    let moved = std::fs::read_to_string(root.join("doks/design/a.md")).unwrap();
    assert!(
        moved.contains("[b](/doks/topics/sub/b.md)"),
        "its own relative link keeps its target: {moved}"
    );
    let b = std::fs::read_to_string(root.join("doks/topics/sub/b.md")).unwrap();
    assert!(b.contains("[a](/doks/design/a.md#setup)"), "{b}");
    assert!(f.file(&id).contains("[a](/doks/design/a.md)"));
    f.run(&["doctor"]);
}

#[test]
fn doctor_reports_broken_and_upward_links() {
    let f = Farm::new("broken");
    f.run(&[
        "dok",
        "new",
        "doks/a.md",
        "--body",
        "[gone](/doks/none.md) and [up](../x.md) and [example](path)",
    ]);
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&f.dir)
        .env("XDG_DATA_HOME", common::data_home(&f.dir))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let msgs: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "broken-link")
        .map(|i| i["message"].as_str().unwrap())
        .collect();
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(
        msgs[0].contains("/doks/none.md") && msgs[1].contains("../x.md"),
        "{msgs:?}"
    );
}

/// One note of a yak, by its timestamp (yaks-c5bd): `update --note` prints
/// the link, `peek` reads just that note, and doctor reports one that leads
/// nowhere.
#[test]
fn a_note_link_from_update_peeks_and_is_checked() {
    let f = Farm::new("note");
    let id = f.create(&[]);
    let out = f.run(&["update", &id, "--note", "The finding."]);
    let link = out
        .trim()
        .strip_prefix(&format!("Updated {id}: note "))
        .unwrap_or_else(|| panic!("no note link in {out:?}"))
        .to_string();
    assert!(link.starts_with(&format!("{id}#20")), "{link}");
    let peek = f.run(&["peek", &link]);
    assert!(peek.ends_with("\n\nThe finding.\n"), "{peek}");
    assert!(peek.starts_with(&format!("── {link}\n▸ ")), "{peek}");

    let other = f.create(&[]);
    f.run(&["update", &other, "--note", &format!("See [it]({link}).")]);
    f.run(&["doctor"]);
    f.run(&[
        "update",
        &other,
        "--note",
        &format!("And [nothing]({id}#2001-01-01T00:00:00Z)."),
    ]);
    f.fails(&["doctor"]);
    f.fails(&["peek", &format!("{id}#2001-01-01T00:00:00Z")]);
}
