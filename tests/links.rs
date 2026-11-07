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

/// `refs <id> --incoming` also lists the doks and session notes that link the
/// yak or one of its notes (yaks-2e00), with path and line; a dok that only
/// links something else is left out, and the yak entries are unchanged.
#[test]
fn refs_incoming_lists_doks_and_session_notes() {
    let f = Farm::new("incoming");
    let id = f.create(&[]);
    let out = f.run(&["update", &id, "--note", "The finding."]);
    let link = out
        .trim()
        .strip_prefix(&format!("Updated {id}: note "))
        .unwrap()
        .to_string();
    let ts = link.strip_prefix(&format!("{id}#")).unwrap().to_string();
    let other = f.create(&["--parent", &id]);
    f.run(&[
        "dok",
        "new",
        "doks/topics/cites.md",
        "--body",
        &format!("Intro.\n\nSee [the yak]({id}).\n"),
    ]);
    f.run(&[
        "dok",
        "new",
        "doks/topics/elsewhere.md",
        "--body",
        &format!("Only [another]({other})."),
    ]);
    let root = f.root();
    std::fs::create_dir_all(root.join("sessions/2026")).unwrap();
    std::fs::write(
        root.join("sessions/2026/note.md"),
        format!("---\ntitle: A session\n---\nFound [it]({link}) today.\n"),
    )
    .unwrap();

    let v: Value = serde_json::from_str(&f.run(&["refs", &id, "--incoming", "--json"])).unwrap();
    let yaks: Vec<&str> = v["incoming"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["id"].as_str().unwrap())
        .collect();
    assert_eq!(yaks, [other.as_str()], "yak entries are unchanged: {v}");
    let docs = v["docs"].as_array().unwrap();
    let got: Vec<(&str, &str, u64, Option<&str>)> = docs
        .iter()
        .map(|d| {
            (
                d["path"].as_str().unwrap(),
                d["kind"].as_str().unwrap(),
                d["line"].as_u64().unwrap(),
                d["note"].as_str(),
            )
        })
        .collect();
    assert_eq!(
        got,
        [
            ("doks/topics/cites.md", "dok", 3, None),
            ("sessions/2026/note.md", "session", 1, Some(ts.as_str())),
        ],
        "{v}"
    );

    let text = f.run(&["refs", &id, "--incoming"]);
    assert!(
        text.contains("dok      doks/topics/cites.md:L3\n"),
        "{text}"
    );
    assert!(
        text.contains(&format!("session  sessions/2026/note.md:L1  {id}#{ts}\n")),
        "{text}"
    );
    assert!(!text.contains("elsewhere.md"), "{text}");
}

/// A dok's attachment directory is its sibling (`doks/x.md` + `doks/x/`,
/// yaks-db66): `attach` stores into it and links it downward, `mv` takes the
/// directory along and keeps the moved dok's relative link and any absolute
/// link from elsewhere pointing at it, and doctor stays clean.
#[test]
fn attach_to_a_dok_and_mv_it() {
    let f = Farm::new("dokattach");
    f.run(&["dok", "new", "doks/topics/a.md", "--body", "Intro."]);
    let shot = f.dir.join("my shot.png");
    std::fs::write(&shot, b"png").unwrap();
    let log = f.dir.join("build.log");
    std::fs::write(&log, b"log").unwrap();

    let out = f.run(&[
        "attach",
        "doks/topics/a.md",
        shot.to_str().unwrap(),
        "--caption",
        "The shot",
    ]);
    assert!(
        out.contains("attached my shot.png to doks/topics/a.md"),
        "{out}"
    );
    f.run(&["attach", "/doks/topics/a.md", log.to_str().unwrap()]);
    f.run(&[
        "attach",
        "doks/topics/a.md",
        log.to_str().unwrap(),
        "--name",
        "other",
        "--no-link",
    ]);
    let root = f.root();
    assert_eq!(
        std::fs::read(root.join("doks/topics/a/my shot.png")).unwrap(),
        b"png"
    );
    assert!(root.join("doks/topics/a/other.log").is_file());
    let text = std::fs::read_to_string(root.join("doks/topics/a.md")).unwrap();
    assert!(
        text.contains("Intro.\n\n![my shot](a/my%20shot.png \"The shot\")\n\n[build](a/build.log)"),
        "{text}"
    );
    assert!(!text.contains("other"), "--no-link adds no link: {text}");
    // The yak that cites an attachment by its farm-root path, and a dok that
    // names the dok itself.
    let id = f.create(&["--description", "See ![x](/doks/topics/a/build.log)."]);
    f.run(&["doctor"]);

    let out = f.run(&["mv", "doks/topics/a.md", "doks/design/renamed.md"]);
    assert!(
        out.contains("Moved doks/topics/a.md -> doks/design/renamed.md"),
        "{out}"
    );
    assert!(!root.join("doks/topics/a").exists(), "the directory moved");
    assert!(root.join("doks/design/renamed/my shot.png").is_file());
    assert!(root.join("doks/design/renamed/other.log").is_file());
    let moved = std::fs::read_to_string(root.join("doks/design/renamed.md")).unwrap();
    assert!(
        moved.contains("![my shot](renamed/my%20shot.png \"The shot\")")
            && moved.contains("[build](renamed/build.log)"),
        "{moved}"
    );
    assert!(f.file(&id).contains("(/doks/design/renamed/build.log)"));
    // `doctor` exits non-zero on any issue: this is the dead-link check.
    f.run(&["doctor"]);
}

/// `mv` will not drag other doks along with a sibling directory that is also
/// a namespace of doks, nor attach a markdown file there (it would read as a
/// dok).
#[test]
fn a_dok_directory_holding_doks_is_not_an_attachment_directory() {
    let f = Farm::new("dokattach-md");
    f.run(&["dok", "new", "doks/a.md", "--body", "A."]);
    f.run(&["dok", "new", "doks/a/child.md", "--body", "Child."]);
    let err = f.fails(&["mv", "doks/a.md", "doks/b.md"]);
    assert!(err.contains("doks/a/child.md"), "{err}");
    assert!(f.root().join("doks/a.md").is_file());

    let md = f.dir.join("notes.md");
    std::fs::write(&md, b"# n").unwrap();
    let err = f.fails(&["attach", "doks/a.md", md.to_str().unwrap()]);
    assert!(err.contains("dok"), "{err}");
    assert!(!f.root().join("doks/a/notes.md").exists());
    f.fails(&["attach", "doks/none.md", md.to_str().unwrap()]);
    f.fails(&["attach", "doks/a.md", "--help-not-a-file"]);
}
