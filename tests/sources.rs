//! Sources and promotion through the real binary (yaks-2c42).

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
        let dir = std::env::temp_dir().join(format!("yaks-sources-{tag}-{}", std::process::id()));
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
fn source_add_dedups_and_lists() {
    let f = Farm::new("add");
    let file = f.dir.join("paper.md");
    std::fs::write(&file, "# Paper\n\ntext\n").unwrap();
    let p = file.to_str().unwrap();
    let out = f.run(&[
        "source",
        "add",
        p,
        "--url",
        "https://example.test/paper",
        "--title",
        "The paper",
    ]);
    assert!(out.starts_with("Saved sources/"), "{out}");
    assert!(
        f.run(&["source", "add", p])
            .starts_with("Already saved: sources/")
    );
    let v: Value = serde_json::from_str(&f.run(&["source", "list", "--json"])).unwrap();
    assert_eq!(v.as_array().unwrap().len(), 1);
    assert_eq!(v[0]["title"], "The paper");
    assert_eq!(v[0]["resource"], "https://example.test/paper");
    let dir = f.root().join(v[0]["dir"].as_str().unwrap());
    assert_eq!(
        std::fs::read_to_string(dir.join("extracted.md")).unwrap(),
        "# Paper\n\ntext\n"
    );
    // A path inside the farm is refused.
    let inside = dir.join("paper.md");
    assert!(
        f.fails(&["source", "add", inside.to_str().unwrap()])
            .contains("already in the farm")
    );
}

#[test]
fn promote_moves_an_attachment_and_rewrites_its_links() {
    let f = Farm::new("promote");
    let id = f.create(&[]);
    let note = f.dir.join("finding.md");
    std::fs::write(&note, "Learned a thing.\n").unwrap();
    let shot = f.dir.join("my shot.png");
    std::fs::write(&shot, "png").unwrap();
    f.run(&["attach", &id, note.to_str().unwrap()]);
    f.run(&["attach", &id, shot.to_str().unwrap()]);
    f.run(&[
        "update",
        &id,
        "--note",
        &format!("See [it]({id}/finding.md)."),
    ]);

    let out = f.run(&[
        "promote",
        &format!("{id}/finding.md"),
        "--dok",
        "doks/topics/finding.md",
    ]);
    assert!(out.contains("-> doks/topics/finding.md"), "{out}");
    let dok = std::fs::read_to_string(f.root().join("doks/topics/finding.md")).unwrap();
    assert!(
        dok.starts_with("---\ntype: note\ntitle: Finding\n---\n"),
        "{dok}"
    );
    let out = f.run(&["promote", &format!("{id}/my shot.png")]);
    assert!(
        out.contains("-> sources/") && out.contains("/my-shot.png"),
        "{out}"
    );

    let yak = f.file(&id);
    assert!(yak.contains("[it](/doks/topics/finding.md)"), "{yak}");
    assert!(yak.contains("[finding](/doks/topics/finding.md)"), "{yak}");
    assert!(
        yak.contains("](/sources/") && !yak.contains(&format!("{id}/my shot.png")),
        "{yak}"
    );
    let v: Value = serde_json::from_str(&f.run(&["source", "list", "--json"])).unwrap();
    assert_eq!(v[0]["resource"], format!("{id}/my shot.png"));
    f.run(&["doctor"]);
}
