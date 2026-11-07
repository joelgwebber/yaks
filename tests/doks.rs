//! `yaks dok` through the real binary (yaks-9af0): new, set, show, list,
//! query, and doctor's dok checks.

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
        let dir = std::env::temp_dir().join(format!("yaks-doks-{tag}-{}", std::process::id()));
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
        "dok",
        "new",
        "doks/topics/heat-pump.md",
        "--type",
        "topic",
        "--set",
        "tags=[home, hvac]",
        "--body",
        "Notes.",
    ]);
    f.run(&[
        "dok",
        "new",
        "yaks/doks/design/links.md",
        "--title",
        "Link grammar",
    ]);
    let text = std::fs::read_to_string(f.root().join("doks/topics/heat-pump.md")).unwrap();
    assert_eq!(
        text,
        "---\ntype: topic\ntitle: Heat pump\ntags:\n- home\n- hvac\n---\n\nNotes.\n"
    );

    f.run(&[
        "dok",
        "set",
        "/doks/topics/heat-pump.md",
        "--set",
        "description=One line.",
        "--unset",
        "tags",
    ]);
    let v: Value =
        serde_json::from_str(&f.run(&["dok", "show", "doks/topics/heat-pump.md", "--json"]))
            .unwrap();
    assert_eq!(v["fields"]["description"], "One line.");
    assert!(v["fields"].get("tags").is_none());
    assert_eq!(v["body"], "Notes.\n");

    let list = f.run(&["dok", "list"]);
    assert!(
        list.contains("doks/topics/heat-pump.md") && list.contains("yaks/doks/design/links.md"),
        "{list}"
    );
    let herd_only = f.run(&["dok", "list", "yaks/doks"]);
    assert!(!herd_only.contains("heat-pump"), "{herd_only}");
    let q: Value =
        serde_json::from_str(&f.run(&["dok", "query", "--type", "note", "--json"])).unwrap();
    assert_eq!(q.as_array().unwrap().len(), 1);
    assert_eq!(q[0]["herd"], "yaks");
}

#[test]
fn refusals_and_doctor() {
    let f = Farm::new("doctor");
    assert!(
        f.fails(&["dok", "new", "doks/Bad.md"])
            .contains("lowercase-kebab")
    );
    assert!(f.fails(&["dok", "new", "notes/x.md"]).contains("doks/"));
    assert!(f.fails(&["dok", "show", "doks/none.md"]).contains("no dok"));
    f.run(&["doctor"]);
    let root = f.root();
    std::fs::create_dir_all(root.join("doks")).unwrap();
    std::fs::write(root.join("doks/loose.md"), "just text\n").unwrap();
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
    assert_eq!(v[0]["kind"], "dok-problem");
    assert_eq!(v[0]["ids"][0], "doks/loose.md");
}

#[test]
fn query_where_compares_dates_and_finds_missing_fields() {
    let f = Farm::new("ops");
    for (name, renews) in [("a", "2026-11-30"), ("b", "2027-02-01")] {
        f.run(&[
            "dok",
            "new",
            &format!("doks/topics/{name}.md"),
            "--set",
            &format!("renews={renews}"),
        ]);
    }
    f.run(&["dok", "new", "doks/topics/c.md"]);
    let paths = |args: &[&str]| -> Vec<String> {
        let mut a = vec!["dok", "query", "--json"];
        a.extend_from_slice(args);
        let v: Value = serde_json::from_str(&f.run(&a)).unwrap();
        v.as_array()
            .unwrap()
            .iter()
            .map(|d| d["path"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        paths(&["--where", "renews=<2026-12-01"]),
        ["doks/topics/a.md"]
    );
    assert_eq!(
        paths(&["--where", "renews=>=2026-12-01"]),
        ["doks/topics/b.md"]
    );
    assert_eq!(paths(&["--where", "renews="]), ["doks/topics/c.md"]);
    assert_eq!(
        paths(&["--where", "renews=!2026-11-30"]),
        ["doks/topics/b.md", "doks/topics/c.md"]
    );
    assert_eq!(paths(&["--where", "title=~B"]), ["doks/topics/b.md"]);
    assert_eq!(
        paths(&["--where", "renews=2026-11-30"]),
        ["doks/topics/a.md"]
    );
}

#[test]
fn row_set_and_delete_round_trip_through_query_rows() {
    let f = Farm::new("rows");
    let body = "Reading list.\n\n| name | author |\n| --- | --- |\n| dune | Herbert |\n| emma | Austen |\n\nAfter the table.\n";
    f.run(&[
        "dok",
        "new",
        "doks/topics/reading.md",
        "--type",
        "reading",
        "--body",
        body,
    ]);
    let file = f.root().join("doks/topics/reading.md");
    let rows = |f: &Farm| -> Value {
        serde_json::from_str(&f.run(&["dok", "query", "--rows", "--type", "reading", "--json"]))
            .unwrap()
    };

    f.run(&[
        "dok",
        "row",
        "set",
        "doks/topics/reading.md#ana",
        "--set",
        "author=Pipe|Author",
        "--set",
        "year=2001",
    ]);
    let v = rows(&f);
    let by = |v: &Value, name: &str| -> Value {
        v.as_array()
            .unwrap()
            .iter()
            .find(|r| r["name"] == name)
            .cloned()
            .unwrap_or(Value::Null)
    };
    assert_eq!(by(&v, "ana")["author"], "Pipe|Author");
    assert_eq!(by(&v, "ana")["year"], "2001");
    assert_eq!(by(&v, "dune")["year"], "");
    assert_eq!(
        v[0]["name"], "ana",
        "sorted table: the new row goes in place"
    );

    let out = f.run(&[
        "dok",
        "row",
        "set",
        "/doks/topics/reading.md#dune",
        "--set",
        "author=F. Herbert",
    ]);
    assert!(out.starts_with("Updated row"), "{out}");
    assert_eq!(by(&rows(&f), "dune")["author"], "F. Herbert");

    f.run(&["dok", "row", "delete", "doks/topics/reading.md#emma"]);
    assert_eq!(by(&rows(&f), "emma"), Value::Null);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(
        text.contains("\n\nReading list.\n\n| name | author | year |\n")
            && text.ends_with("|\n\nAfter the table.\n"),
        "{text}"
    );

    assert!(
        f.fails(&["dok", "row", "delete", "doks/topics/reading.md#emma"])
            .contains("no row")
    );
    f.run(&["dok", "new", "doks/topics/plain.md", "--body", "No table."]);
    assert!(
        f.fails(&["dok", "row", "set", "doks/topics/plain.md#a"])
            .contains("no markdown table")
    );
}

#[test]
fn stale_if_hash_is_refused_and_old_new_edits_chain_by_hash() {
    let f = Farm::new("hash");
    f.run(&[
        "dok",
        "new",
        "doks/topics/hashed.md",
        "--body",
        "alpha\nbeta\nbeta\n",
    ]);
    let shown: Value =
        serde_json::from_str(&f.run(&["dok", "show", "doks/topics/hashed.md", "--json"])).unwrap();
    let h0 = shown["hash"].as_str().unwrap().to_string();
    assert_eq!(h0.len(), 12);

    // A first writer moves the file on; the second, holding h0, is refused.
    f.run(&[
        "dok",
        "set",
        "doks/topics/hashed.md",
        "--if-hash",
        &h0,
        "--old",
        "alpha",
        "--new",
        "ALPHA",
    ]);
    let err = f.fails(&[
        "dok",
        "set",
        "doks/topics/hashed.md",
        "--if-hash",
        &h0,
        "--body",
        "stale overwrite",
    ]);
    assert!(
        err.contains("changed since it was read") && err.contains(&h0),
        "{err}"
    );
    let text = std::fs::read_to_string(f.root().join("doks/topics/hashed.md")).unwrap();
    assert!(
        text.contains("ALPHA") && !text.contains("stale overwrite"),
        "{text}"
    );

    // The printed hash is the file's hash, so a second edit chains on it.
    let out = f.run(&["dok", "show", "doks/topics/hashed.md"]);
    assert!(out.contains("ALPHA"));
    let printed = f.run(&[
        "dok",
        "set",
        "doks/topics/hashed.md",
        "--old",
        "ALPHA",
        "--new",
        "one",
    ]);
    let h1 = printed
        .lines()
        .find_map(|l| l.strip_prefix("hash: "))
        .unwrap()
        .to_string();
    let shown: Value =
        serde_json::from_str(&f.run(&["dok", "show", "doks/topics/hashed.md", "--json"])).unwrap();
    assert_eq!(shown["hash"], h1.as_str());
    let printed = f.run(&[
        "dok",
        "set",
        "doks/topics/hashed.md",
        "--if-hash",
        &h1,
        "--old",
        "one",
        "--new",
        "two",
    ]);
    assert!(
        printed.contains("Updated") && printed.contains("hash: "),
        "{printed}"
    );

    // --old must match exactly once; nothing is written otherwise.
    let err = f.fails(&[
        "dok",
        "set",
        "doks/topics/hashed.md",
        "--old",
        "beta",
        "--new",
        "x",
    ]);
    assert!(err.contains("matches 2 times"), "{err}");
    let err = f.fails(&[
        "dok",
        "set",
        "doks/topics/hashed.md",
        "--old",
        "gamma",
        "--new",
        "x",
    ]);
    assert!(err.contains("matches 0 times"), "{err}");
    let err = f.fails(&["dok", "set", "doks/topics/hashed.md", "--old", "two"]);
    assert!(err.contains("--old and --new go together"), "{err}");
    let text = std::fs::read_to_string(f.root().join("doks/topics/hashed.md")).unwrap();
    assert!(text.contains("two\nbeta\nbeta"), "{text}");
}
