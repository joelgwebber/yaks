//! `yaks page` through the real binary (yaks-9af0): new, set, show, list,
//! query, and doctor's page checks.

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
        let dir = std::env::temp_dir().join(format!("yaks-pages-{tag}-{}", std::process::id()));
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
fn new_set_show_list_and_query() {
    let f = Farm::new("crud");
    f.run(&[
        "page",
        "new",
        "wiki/topics/heat-pump.md",
        "--type",
        "topic",
        "--set",
        "tags=[home, hvac]",
        "--body",
        "Notes.",
    ]);
    f.run(&[
        "page",
        "new",
        "yaks/wiki/design/links.md",
        "--title",
        "Link grammar",
    ]);
    let text = std::fs::read_to_string(f.root().join("wiki/topics/heat-pump.md")).unwrap();
    assert_eq!(
        text,
        "---\ntype: topic\ntitle: Heat pump\ntags:\n- home\n- hvac\n---\n\nNotes.\n"
    );

    f.run(&[
        "page",
        "set",
        "/wiki/topics/heat-pump.md",
        "--set",
        "description=One line.",
        "--unset",
        "tags",
    ]);
    let v: Value =
        serde_json::from_str(&f.run(&["page", "show", "wiki/topics/heat-pump.md", "--json"]))
            .unwrap();
    assert_eq!(v["fields"]["description"], "One line.");
    assert!(v["fields"].get("tags").is_none());
    assert_eq!(v["body"], "Notes.\n");

    let list = f.run(&["page", "list"]);
    assert!(
        list.contains("wiki/topics/heat-pump.md") && list.contains("yaks/wiki/design/links.md"),
        "{list}"
    );
    let herd_only = f.run(&["page", "list", "yaks/wiki"]);
    assert!(!herd_only.contains("heat-pump"), "{herd_only}");
    let q: Value =
        serde_json::from_str(&f.run(&["page", "query", "--type", "note", "--json"])).unwrap();
    assert_eq!(q.as_array().unwrap().len(), 1);
    assert_eq!(q[0]["herd"], "yaks");
}

#[test]
fn refusals_and_doctor() {
    let f = Farm::new("doctor");
    assert!(
        f.fails(&["page", "new", "wiki/Bad.md"])
            .contains("lowercase-kebab")
    );
    assert!(f.fails(&["page", "new", "notes/x.md"]).contains("wiki/"));
    assert!(
        f.fails(&["page", "show", "wiki/none.md"])
            .contains("no page")
    );
    f.run(&["doctor"]);
    let root = f.root();
    std::fs::create_dir_all(root.join("wiki")).unwrap();
    std::fs::write(root.join("wiki/loose.md"), "just text\n").unwrap();
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
    assert_eq!(v[0]["kind"], "page-problem");
    assert_eq!(v[0]["ids"][0], "wiki/loose.md");
}
