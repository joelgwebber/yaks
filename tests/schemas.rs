//! Schemas through the real binary (yaks-8e75): defaults on `dok new`,
//! warnings on writes and in doctor, `dok types`, table rows, and
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
        "---\ntype: Dok Type\ntitle: Appliance\nfields:\n  name: {type: text, required: true}\n  purchased: date\n  warranty: {type: text, default: none}\n---\n",
    );
    write(
        &f,
        "schemas/yak.md",
        "---\ntype: Dok Type\nfields:\n  due: date\ntags:\n  ui: The TUI\n---\n",
    );
    write(
        &f,
        "doks/platforms.md",
        "---\ntype: platform\ntitle: Platforms\n---\n\n| platform | kind |\n| --- | --- |\n| cursor | coding |\n| glean | enterprise |\n",
    );

    let types = f.run(&["dok", "types"]);
    assert!(
        types.contains("appliance (Appliance)")
            && types.contains("name: text, usually set")
            && types.contains("tag ui: The TUI"),
        "{types}"
    );

    f.run(&[
        "dok",
        "new",
        "doks/home/dishwasher.md",
        "--type",
        "appliance",
        "--set",
        "name=Dishwasher",
    ]);
    let text = std::fs::read_to_string(f.root().join("doks/home/dishwasher.md")).unwrap();
    assert!(
        text.contains("warranty: none\n") && text.contains("name: Dishwasher\n"),
        "{text}"
    );

    let rows: Value = serde_json::from_str(&f.run(&[
        "dok",
        "query",
        "--rows",
        "--where",
        "kind=enterprise",
        "--json",
    ]))
    .unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["platform"], "glean");
    assert_eq!(rows[0]["dok"], "doks/platforms.md");

    f.run(&["doctor"]);
    let id = f.create(&[]);
    f.run(&["update", &id, "--set", "due=soon"]);
    f.run(&[
        "dok",
        "set",
        "doks/home/dishwasher.md",
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
            "doks/home/dishwasher.md: purchased: expected a date, YYYY-MM-DD",
            format!("{id}: due: expected a date, YYYY-MM-DD").as_str()
        ]
    );
}

#[test]
fn dok_define_writes_a_schema_dok_types_reads_and_merges_into_it() {
    let f = Farm::new("define");
    let out = f.run(&[
        "dok",
        "define",
        "appliance",
        "--field",
        "name=text!",
        "--field",
        "purchased=date",
        "--description",
        "Things with plugs",
        "--stale-after",
        "P1Y",
    ]);
    assert!(
        out.starts_with("Created schemas/appliance.md (2 fields)"),
        "{out}"
    );
    let types = f.run(&["dok", "types"]);
    assert!(
        types.contains("appliance (Appliance): Things with plugs")
            && types.contains("name: text, usually set")
            && types.contains("purchased: date"),
        "{types}"
    );
    // A hand-written schema with prose and a comment: merging keeps both,
    // every field and its description, and only adds.
    write(
        &f,
        "schemas/pet.md",
        "---\ntype: Dok Type\ntitle: Pet\n# keep me\nfields:\n  name: {type: text, required: true}\n  born: {type: date, description: Day it arrived}\n---\n\nProse about pets.\n",
    );
    let out = f.run(&[
        "dok",
        "define",
        "pet",
        "--field",
        "name=text!",
        "--field",
        "born=date",
        "--field",
        "weight=number",
    ]);
    assert!(
        out.starts_with("Updated schemas/pet.md: added weight"),
        "{out}"
    );
    let text = std::fs::read_to_string(f.root().join("schemas/pet.md")).unwrap();
    assert_eq!(
        text,
        "---\ntype: Dok Type\ntitle: Pet\n# keep me\nfields:\n  name: {type: text, required: true}\n  born: {type: date, description: Day it arrived}\n  weight: number\n---\n\nProse about pets.\n"
    );
    assert!(
        f.run(&["dok", "define", "pet", "--field", "weight=number"])
            .starts_with("No changes")
    );

    // A different type is refused, with nothing written, until --replace; the
    // field's description stays.
    let err = f.fails(&[
        "dok",
        "define",
        "pet",
        "--field",
        "born=text",
        "--field",
        "age=number",
    ]);
    assert!(
        err.contains("born is date here, not text") && err.contains("--replace"),
        "{err}"
    );
    assert_eq!(
        std::fs::read_to_string(f.root().join("schemas/pet.md")).unwrap(),
        text
    );
    let out = f.run(&["dok", "define", "pet", "--field", "born=text", "--replace"]);
    assert!(out.contains("changed born"), "{out}");
    let now = std::fs::read_to_string(f.root().join("schemas/pet.md")).unwrap();
    assert!(
        now.contains("born: {type: text, description: Day it arrived}")
            && now.ends_with("Prose about pets.\n"),
        "{now}"
    );

    // Bad input is refused.
    for (args, why) in [
        (
            vec!["dok", "define", "note", "--field", "a=text"],
            "untyped",
        ),
        (
            vec!["dok", "define", "Bad", "--field", "a=text"],
            "not a kind name",
        ),
        (
            vec!["dok", "define", "x", "--field", "a=colour"],
            "not one yaks knows",
        ),
        (vec!["dok", "define", "x", "--field", "a"], "name=type"),
        (
            vec!["dok", "define", "x", "--stale-after", "soon"],
            "--stale-after",
        ),
        (vec!["dok", "define", "x"], "nothing to define"),
    ] {
        let err = f.fails(&args);
        assert!(err.contains(why), "{args:?}: {err}");
    }
    assert!(!f.root().join("schemas/x.md").exists());
}
