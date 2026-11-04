//! Find or create by `source:` (yaks-bdae): `list --source <url>` and
//! `create --key <url>`, through the real binary.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

const PR1: &str = "https://github.com/o/r/pull/1";
const PR2: &str = "https://github.com/o/r/pull/2";

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-key-{tag}-{}", std::process::id()));
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

    fn key(&self, title: &str, url: &str) -> Value {
        serde_json::from_str(&self.run(&["create", title, "--key", url, "--json"])).unwrap()
    }

    fn ids(&self, args: &[&str]) -> Vec<String> {
        let mut a = vec!["list", "--json"];
        a.extend_from_slice(args);
        let v: Value = serde_json::from_str(&self.run(&a)).unwrap();
        v.as_array()
            .unwrap()
            .iter()
            .map(|y| y["id"].as_str().unwrap().to_string())
            .collect()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

#[test]
fn create_key_creates_once_and_then_finds() {
    let f = Farm::new("once");
    let first = f.key("Review o/r#1", PR1);
    assert_eq!(first["created"], true);
    assert_eq!(first["source"], PR1);

    // Same key, even with a different title: the existing yak, nothing new.
    let again = f.key("A different title", PR1);
    assert_eq!(again["created"], false);
    assert_eq!(again["id"], first["id"]);
    assert_eq!(again["title"], "Review o/r#1");
    assert_eq!(f.ids(&[]).len(), 1);

    // A different key is a different yak.
    let other = f.key("Review o/r#2", PR2);
    assert_eq!(other["created"], true);
    assert_ne!(other["id"], first["id"]);
    assert_eq!(f.ids(&[]).len(), 2);
}

#[test]
fn create_key_finds_shorn_and_dead_yaks_and_never_resurrects_them() {
    let f = Farm::new("states");
    let a = f.key("a", PR1)["id"].as_str().unwrap().to_string();
    let b = f.key("b", PR2)["id"].as_str().unwrap().to_string();
    f.run(&["shorn", &a]);
    f.run(&["slaughter", &b]);

    let a2 = f.key("a again", PR1);
    let b2 = f.key("b again", PR2);
    assert_eq!(
        (a2["created"].as_bool(), a2["id"].as_str()),
        (Some(false), Some(a.as_str()))
    );
    assert_eq!(
        (b2["created"].as_bool(), b2["id"].as_str()),
        (Some(false), Some(b.as_str()))
    );
    assert_eq!(a2["status"], "shorn");
    assert_eq!(
        b2["status"], "dead",
        "a dead yak stays dead: the caller decides"
    );
    assert_eq!(f.ids(&["--all"]).len(), 2);
}

#[test]
fn create_key_text_output_says_whether_it_created() {
    let f = Farm::new("text");
    let out = f.run(&["create", "T", "--key", PR1]);
    assert!(out.starts_with("Created "), "{out}");
    let out = f.run(&["create", "T", "--key", PR1]);
    assert!(out.starts_with("Exists "), "{out}");
}

#[test]
fn create_key_conflicts_with_source_and_refuses_an_empty_key() {
    let f = Farm::new("conflict");
    assert!(
        !f.cmd(&["create", "T", "--key", PR1, "--source", PR2])
            .status
            .success()
    );
    assert!(!f.cmd(&["create", "T", "--key", ""]).status.success());
    assert!(f.ids(&["--all"]).is_empty());
}

#[test]
fn list_source_matches_exactly_and_composes_with_all() {
    let f = Farm::new("list");
    let a = f.key("a", PR1)["id"].as_str().unwrap().to_string();
    let b = f.key("b", PR2)["id"].as_str().unwrap().to_string();
    f.run(&["create", "no source"]);
    f.run(&["slaughter", &b]);

    assert_eq!(f.ids(&["--source", PR1]), vec![a.clone()]);
    // A longer URL that merely starts with PR1 is not a match.
    let pr10 = "https://github.com/o/r/pull/10";
    assert!(f.ids(&["--source", pr10]).is_empty());
    f.key("ten", pr10);
    assert_eq!(f.ids(&["--source", PR1]), vec![a]);
    // Dead yaks only with --all, like every other filter.
    assert!(f.ids(&["--source", PR2]).is_empty());
    assert_eq!(f.ids(&["--source", PR2, "--all"]), vec![b]);
    // `next` shares the filters.
    assert!(f.run(&["next", "--source", PR1]).contains("a"));
}

#[test]
fn concurrent_keyed_creates_make_exactly_one_yak() {
    let f = Farm::new("race");
    let outs: Vec<std::process::Output> = std::thread::scope(|s| {
        let hs: Vec<_> = (0..6)
            .map(|i| {
                let f = &f;
                s.spawn(move || f.cmd(&["create", &format!("racer {i}"), "--key", PR1, "--json"]))
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    let mut created = 0;
    let mut ids = std::collections::BTreeSet::new();
    for out in &outs {
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        created += usize::from(v["created"] == true);
        ids.insert(v["id"].as_str().unwrap().to_string());
    }
    assert_eq!(created, 1, "exactly one caller creates");
    assert_eq!(ids.len(), 1, "everyone is told the same yak");
    assert_eq!(f.ids(&["--all"]).len(), 1);
}
