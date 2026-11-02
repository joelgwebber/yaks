//! Schemas through the real binary (yaks-8e75): defaults on `page new`,
//! warnings on writes and in doctor, `page types`, table rows, and
//! `schemas/yak.md` extending yaks.

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
        let dir = std::env::temp_dir().join(format!("yaks-schemas-{tag}-{}", std::process::id()));
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

fn write(f: &Farm, rel: &str, text: &str) {
    let p = f.root().join(rel);
    std::fs::create_dir_all(p.parent().unwrap()).unwrap();
    std::fs::write(p, text).unwrap();
}

#[test]
fn defaults_warnings_types_rows_and_yak_fields() {
    let f = Farm::new("all");
    write(
        &f,
        "schemas/appliance.md",
        "---\ntype: Page Type\ntitle: Appliance\nfields:\n  name: {type: text, required: true}\n  purchased: date\n  warranty: {type: text, default: none}\n---\n",
    );
    write(
        &f,
        "schemas/yak.md",
        "---\ntype: Page Type\nfields:\n  due: date\ntags:\n  ui: The TUI\n---\n",
    );
    write(
        &f,
        "wiki/platforms.md",
        "---\ntype: platform\ntitle: Platforms\n---\n\n| platform | kind |\n| --- | --- |\n| cursor | coding |\n| glean | enterprise |\n",
    );

    let types = f.run(&["page", "types"]);
    assert!(
        types.contains("appliance (Appliance)")
            && types.contains("name: text, usually set")
            && types.contains("tag ui: The TUI"),
        "{types}"
    );

    f.run(&[
        "page",
        "new",
        "wiki/home/dishwasher.md",
        "--type",
        "appliance",
        "--set",
        "name=Dishwasher",
    ]);
    let text = std::fs::read_to_string(f.root().join("wiki/home/dishwasher.md")).unwrap();
    assert!(
        text.contains("warranty: none\n") && text.contains("name: Dishwasher\n"),
        "{text}"
    );

    let rows: Value = serde_json::from_str(&f.run(&[
        "page",
        "query",
        "--rows",
        "--where",
        "kind=enterprise",
        "--json",
    ]))
    .unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["platform"], "glean");
    assert_eq!(rows[0]["page"], "wiki/platforms.md");

    f.run(&["doctor"]);
    let id = f.create(&[]);
    f.run(&["update", &id, "--set", "due=soon"]);
    f.run(&[
        "page",
        "set",
        "wiki/home/dishwasher.md",
        "--set",
        "purchased=spring",
    ]);
    let out = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&f.dir)
        .env("XDG_DATA_HOME", common::data_home(&f.dir))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .args(["doctor", "--json"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let msgs: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "schema-mismatch")
        .map(|i| i["message"].as_str().unwrap())
        .collect();
    // Doctor sorts by subject.
    assert_eq!(
        msgs,
        [
            "wiki/home/dishwasher.md: purchased: expected a date, YYYY-MM-DD",
            format!("{id}: due: expected a date, YYYY-MM-DD").as_str()
        ]
    );
}
