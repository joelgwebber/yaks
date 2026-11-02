//! `yaks find` through the real binary (yaks-bdcb): each kind, `--scope`,
//! `--kind`, `--json`, `--regex`, exit codes, and that binary files are skipped.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-find-{tag}-{}", std::process::id()));
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
            "{args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn root(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
    }

    fn put(&self, rel: &str, bytes: &[u8]) {
        let p = self.root().join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, bytes).unwrap();
    }

    /// `find` with `args`, as `(exit code, stdout)`.
    fn find(&self, args: &[&str]) -> (i32, String) {
        let mut a = vec!["find"];
        a.extend_from_slice(args);
        let out = self.cmd(&a);
        (
            out.status.code().unwrap(),
            String::from_utf8(out.stdout).unwrap(),
        )
    }

    fn find_json(&self, args: &[&str]) -> Vec<Value> {
        let mut a = args.to_vec();
        a.push("--json");
        let (_, out) = self.find(&a);
        serde_json::from_str::<Value>(&out)
            .unwrap()
            .as_array()
            .unwrap()
            .clone()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn paths(hits: &[Value]) -> Vec<String> {
    let mut v: Vec<String> = hits
        .iter()
        .map(|h| h["path"].as_str().unwrap().to_string())
        .collect();
    v.dedup();
    v
}

/// A farm with one thing of each kind that mentions "quokka", plus decoys.
fn stocked(tag: &str) -> (Farm, String) {
    let f = Farm::new(tag);
    let out = f.run(&[
        "create",
        "Zoo census",
        "--description",
        "Count the quokka herd.",
        "--json",
    ]);
    let id = serde_json::from_str::<Value>(&out).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    f.run(&["update", &id, "--note", "Saw a Quokka by the gate."]);
    f.put(
        "wiki/topics/marsupials.md",
        b"---\ntype: topic\ntitle: Marsupials\ntags: [quokka-facts]\n---\n\n# Marsupials\n\nThe quokka smiles.\n",
    );
    f.put(
        "web/wiki/field-notes.md",
        b"---\ntitle: Field notes\n---\nNothing here but the QUOKKA.\n",
    );
    f.put(
        "sources/2026-01-01-aaaaaaaa/source.md",
        b"---\ntitle: Animal almanac\nmedia_type: text/html\n---\nCaptured page.\n",
    );
    f.put(
        "sources/2026-01-01-aaaaaaaa/extracted.md",
        b"Chapter one.\nQuokkas live on Rottnest.\n",
    );
    f.put(
        "sessions/2026/chat.md",
        b"---\ntitle: Zoo chat\n---\nwe talked about a quokka\n",
    );
    (f, id)
}

#[test]
fn finds_each_kind_with_path_line_and_title() {
    let (f, id) = stocked("kinds");
    let (code, out) = f.find(&["quokka"]);
    assert_eq!(code, 0);
    let ypath = format!("yak/yaks/{id}.md");
    // Group headers carry the title; match lines are `path:line: text`.
    assert!(out.contains(&format!("{ypath}  Zoo census\n")), "{out}");
    assert!(
        out.contains(&format!("{ypath}:")) && out.contains("Count the quokka herd."),
        "{out}"
    );
    assert!(out.contains("Saw a Quokka by the gate."), "{out}");
    assert!(
        out.contains("wiki/topics/marsupials.md  Marsupials\n"),
        "{out}"
    );
    assert!(
        out.contains("wiki/topics/marsupials.md:4: tags: [quokka-facts]"),
        "{out}"
    );
    assert!(
        out.contains("wiki/topics/marsupials.md:9: The quokka smiles."),
        "{out}"
    );
    assert!(
        out.contains("web/wiki/field-notes.md:4: Nothing here but the QUOKKA."),
        "{out}"
    );
    assert!(
        out.contains("sources/2026-01-01-aaaaaaaa/extracted.md  Animal almanac\n"),
        "{out}"
    );
    assert!(
        out.contains("sources/2026-01-01-aaaaaaaa/extracted.md:2: Quokkas live on Rottnest."),
        "{out}"
    );
    assert!(out.contains("sessions/2026/chat.md  Zoo chat\n"), "{out}");
    assert!(
        out.contains("sessions/2026/chat.md:4: we talked about a quokka"),
        "{out}"
    );
    // A file with several matches is one group: its header appears once.
    assert_eq!(out.matches(&format!("{ypath}  ")).count(), 1, "{out}");
}

#[test]
fn json_has_path_kind_title_line_text() {
    let (f, id) = stocked("json");
    let hits = f.find_json(&["quokka"]);
    let h = hits
        .iter()
        .find(|h| h["path"] == "sessions/2026/chat.md")
        .unwrap();
    assert_eq!(h["kind"], "sessions");
    assert_eq!(h["title"], "Zoo chat");
    assert_eq!(h["line"], 4);
    assert_eq!(h["text"], "we talked about a quokka");
    let kinds: Vec<&str> = hits.iter().map(|h| h["kind"].as_str().unwrap()).collect();
    for k in ["yaks", "pages", "sources", "sessions"] {
        assert!(kinds.contains(&k), "{k} missing: {kinds:?}");
    }
    let y = hits.iter().find(|h| h["kind"] == "yaks").unwrap();
    assert_eq!(y["path"], format!("yak/yaks/{id}.md"));
    assert_eq!(y["title"], "Zoo census");
}

#[test]
fn kind_flag_limits_what_is_searched() {
    let (f, _) = stocked("kindflag");
    for (kind, only) in [
        ("pages", "pages"),
        ("sources", "sources"),
        ("sessions", "sessions"),
        ("yaks", "yaks"),
    ] {
        let hits = f.find_json(&["quokka", "--kind", kind]);
        assert!(!hits.is_empty(), "{kind}");
        assert!(hits.iter().all(|h| h["kind"] == only), "{kind}: {hits:?}");
    }
    let two = f.find_json(&["quokka", "--kind", "pages", "--kind", "sessions"]);
    assert!(
        two.iter()
            .all(|h| h["kind"] == "pages" || h["kind"] == "sessions")
    );
    assert!(
        two.iter().any(|h| h["kind"] == "pages") && two.iter().any(|h| h["kind"] == "sessions")
    );
    assert!(
        !f.cmd(&["find", "quokka", "--kind", "bogus"])
            .status
            .success()
    );
}

#[test]
fn scope_keeps_a_directory_or_a_path_prefix() {
    let (f, _) = stocked("scope");
    assert_eq!(
        paths(&f.find_json(&["quokka", "--scope", "wiki"])),
        ["wiki/topics/marsupials.md"]
    );
    assert_eq!(
        paths(&f.find_json(&["quokka", "--scope", "wiki/topics/marsu"])),
        ["wiki/topics/marsupials.md"]
    );
    assert_eq!(
        paths(&f.find_json(&["quokka", "--scope", "web"])),
        ["web/wiki/field-notes.md"]
    );
    // A directory name is not a bare string prefix: `we` is not `web`.
    assert_eq!(f.find(&["quokka", "--scope", "we"]).0, 1);
    let two = paths(&f.find_json(&["quokka", "--scope", "web", "--scope", "sessions"]));
    assert_eq!(two, ["web/wiki/field-notes.md", "sessions/2026/chat.md"]);
    assert_eq!(f.find(&["quokka", "--scope", "nothing/here"]).0, 1);
}

#[test]
fn case_is_ignored_unless_asked() {
    let (f, _) = stocked("case");
    let loose = f.find_json(&["QUOKKA", "--kind", "pages"]);
    assert_eq!(loose.len(), 3);
    let exact = f.find_json(&["QUOKKA", "--kind", "pages", "--case-sensitive"]);
    assert_eq!(paths(&exact), ["web/wiki/field-notes.md"]);
    assert_eq!(f.find(&["quokka", "-i", "--kind", "pages"]).0, 0);
}

#[test]
fn regex_is_opt_in() {
    let (f, _) = stocked("regex");
    // Without --regex the pattern is literal text.
    assert_eq!(f.find(&["quokka.*smiles"]).0, 1);
    let hits = f.find_json(&["quokka.*smiles", "--regex"]);
    assert_eq!(paths(&hits), ["wiki/topics/marsupials.md"]);
    let hits = f.find_json(&["^quokkas?\\b", "--regex", "--kind", "sources"]);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["line"], 2);
    let out = f.cmd(&["find", "(", "--regex"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("invalid pattern"));
}

#[test]
fn exit_one_without_a_match_and_empty_json_array() {
    let (f, _) = stocked("none");
    let (code, out) = f.find(&["no-such-text-anywhere"]);
    assert_eq!((code, out.as_str()), (1, ""));
    let (code, out) = f.find(&["no-such-text-anywhere", "--json"]);
    assert_eq!((code, out.trim()), (1, "[]"));
}

#[test]
fn binary_files_and_originals_are_skipped() {
    let (f, _) = stocked("binary");
    // The original of a source and a binary `.md` hold the text, but are not searched.
    f.put(
        "sources/2026-01-01-aaaaaaaa/original.html",
        b"<p>needle-in-original</p>",
    );
    f.put(
        "sources/2026-01-01-aaaaaaaa/original.pdf",
        b"%PDF needle-in-original\0\x01\x02",
    );
    f.put(
        "sources/2026-01-01-bbbbbbbb/extracted.md",
        b"needle-in-original\0binary",
    );
    f.put("wiki/blob.md", b"needle-in-original \xff\xfe not utf-8");
    f.put("sessions/raw.md", b"needle-in-original\0");
    assert_eq!(f.find(&["needle-in-original"]).0, 1);
}

#[test]
fn sessions_is_a_reserved_herd_name() {
    let f = Farm::new("reserved");
    let out = f.cmd(&["create", "x", "--herd", "sessions"]);
    assert!(!out.status.success());
    // And a `sessions/` directory is never read as a herd, so its markdown is
    // not a yak even with a `yaks/` inside.
    f.put(
        "sessions/yaks/sessions-0001.md",
        b"---\nid: sessions-0001\ntitle: Not a yak\n---\nneedle\n",
    );
    let hits = f.find_json(&["needle"]);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["kind"], "sessions");
    assert!(!f.run(&["list", "--json"]).contains("sessions-0001"));
}
