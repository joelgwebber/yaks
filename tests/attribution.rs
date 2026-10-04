//! Attribution of status moves: every move (`shave`, `shorn`, `regrow`,
//! `slaughter`, `revive`) appends a `moved: <from> -> <to>` entry stamped with
//! the resolved actor, so a yak file alone says who moved it and when. One test
//! per actor-resolution path, plus the shave+shorn pair and `yaks log`.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// A throwaway farm (`yaks init`) plus a global git config naming a human, so
/// the git-user fallback is deterministic. `run` starts every command with the
/// whole attribution environment cleared; `env` adds what a test pins.
struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-attr-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("gitconfig"), "[user]\n\tname = Git Human\n").unwrap();
        let farm = Farm { dir };
        farm.run(&[], &["init"]);
        farm
    }

    fn run(&self, env: &[(&str, &str)], args: &[&str]) -> String {
        let mut cmd = Command::cargo_bin("yaks").unwrap();
        cmd.current_dir(&self.dir)
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("GIT_CONFIG_GLOBAL", self.dir.join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("YAKS_ACTOR")
            .env_remove("DELTA_THREAD_TITLE")
            .env_remove("DELTA_CURRENT_THREAD_ID");
        for (k, v) in env {
            cmd.env(k, v);
        }
        let out = cmd.args(args).output().unwrap();
        assert!(
            out.status.success(),
            "command {args:?} failed: {:?}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn create(&self, title: &str) -> String {
        let out = self.run(&[], &["create", title, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    /// The yak's file text, wherever its status put it.
    fn file(&self, id: &str) -> String {
        let path = self.run(&[], &["path", id]);
        std::fs::read_to_string(Path::new(path.trim())).unwrap()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The marker line `▸ <ts> [<actor>]` followed by the transition text.
fn has_move(file: &str, actor: &str, from: &str, to: &str) -> bool {
    let lines: Vec<&str> = file.lines().collect();
    lines.windows(2).any(|w| {
        w[0].starts_with("\u{25b8} ")
            && w[0].ends_with(&format!(" [{actor}]"))
            && w[1] == format!("moved: {from} -> {to}")
    })
}

#[test]
fn move_actor_from_the_flag() {
    let f = Farm::new("flag");
    let id = f.create("flag");
    // The flag beats every other source.
    let env = [
        ("YAKS_ACTOR", "from-env"),
        ("DELTA_THREAD_TITLE", "A thread"),
    ];
    f.run(&env, &["shave", &id, "--as", "from-flag"]);
    assert!(has_move(&f.file(&id), "from-flag", "hairy", "shaving"));
}

#[test]
fn move_actor_from_yaks_actor() {
    let f = Farm::new("env");
    let id = f.create("env");
    let env = [
        ("YAKS_ACTOR", "from-env"),
        ("DELTA_THREAD_TITLE", "A thread"),
    ];
    f.run(&env, &["shave", &id]);
    assert!(has_move(&f.file(&id), "from-env", "hairy", "shaving"));
}

#[test]
fn move_actor_from_delta_thread_title_then_id() {
    let f = Farm::new("delta");
    let id = f.create("delta");
    let env = [
        ("DELTA_THREAD_TITLE", "Fix the thing"),
        ("DELTA_CURRENT_THREAD_ID", "t-123"),
    ];
    f.run(&env, &["shave", &id]);
    assert!(has_move(
        &f.file(&id),
        "delta:Fix the thing",
        "hairy",
        "shaving"
    ));
    f.run(&[("DELTA_CURRENT_THREAD_ID", "t-123")], &["shorn", &id]);
    assert!(has_move(&f.file(&id), "delta:t-123", "shaving", "shorn"));
}

#[test]
fn move_actor_falls_back_to_the_git_user() {
    let f = Farm::new("git");
    let id = f.create("git");
    f.run(&[], &["shave", &id]);
    assert!(has_move(&f.file(&id), "Git Human", "hairy", "shaving"));
}

#[test]
fn every_moving_command_takes_as() {
    let f = Farm::new("every");
    let id = f.create("every");
    f.run(&[], &["regrow", &id]); // already hairy: no entry, no error
    f.run(&[], &["shave", &id, "--as", "a"]);
    f.run(&[], &["shorn", &id, "--as", "b"]);
    f.run(&[], &["regrow", &id, "--as", "c"]);
    f.run(&[], &["slaughter", &id, "--as", "d"]);
    f.run(&[], &["revive", &id, "--as", "e"]);
    let file = f.file(&id);
    assert!(has_move(&file, "a", "hairy", "shaving"));
    assert!(has_move(&file, "b", "shaving", "shorn"));
    assert!(has_move(&file, "c", "shorn", "hairy"));
    assert!(has_move(&file, "d", "hairy", "dead"));
    assert!(has_move(&file, "e", "dead", "hairy"));
    assert_eq!(file.matches("moved: ").count(), 5, "{file}");
}

#[test]
fn shave_then_shorn_leave_two_attributed_entries_and_log_lists_them() {
    let f = Farm::new("log");
    let id = f.create("pair");
    f.run(&[], &["shave", &id, "--as", "alice"]);
    f.run(&[], &["update", &id, "--note", "halfway", "--as", "bob"]);
    f.run(&[], &["shorn", &id, "--as", "bob"]);
    let file = f.file(&id);
    assert!(has_move(&file, "alice", "hairy", "shaving"));
    assert!(has_move(&file, "bob", "shaving", "shorn"));
    assert_eq!(file.matches("moved: ").count(), 2, "{file}");

    let log = f.run(&[], &["log", "--status", "shorn"]);
    assert!(log.contains("moved: hairy -> shaving"), "{log}");
    assert!(log.contains("moved: shaving -> shorn"), "{log}");
    assert!(log.contains("halfway"), "{log}");
    // `--by` filters moves like notes.
    let by_alice = f.run(&[], &["log", "--status", "shorn", "--by", "alice"]);
    assert!(by_alice.contains("moved: hairy -> shaving"), "{by_alice}");
    assert!(!by_alice.contains("shaving -> shorn"), "{by_alice}");
}

#[test]
fn slaughter_family_attributes_every_yak_moved() {
    let f = Farm::new("family");
    let p = f.create("p");
    let out = f.run(&[], &["create", "c", "--parent", &p, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let c = v["id"].as_str().unwrap().to_string();
    f.run(&[], &["slaughter", &p, "--family", "--as", "reaper"]);
    assert!(has_move(&f.file(&p), "reaper", "hairy", "dead"));
    assert!(has_move(&f.file(&c), "reaper", "hairy", "dead"));
}

#[test]
fn a_file_without_move_entries_still_works_and_doctor_accepts_the_marker() {
    let f = Farm::new("doctor");
    let id = f.create("doc");
    f.run(&[], &["shave", &id, "--as", "a"]);
    f.run(&[], &["shorn", &id, "--as", "a"]);
    // A yak moved by an older binary has no entries; a plain regrow after
    // stripping them must still parse and move.
    let path = PathBuf::from(f.run(&[], &["path", &id]).trim());
    let text = std::fs::read_to_string(&path).unwrap();
    let stripped: String = text.split("\n\n---\n\u{25b8} ").next().unwrap().to_string();
    std::fs::write(&path, format!("{stripped}\n")).unwrap();
    f.run(&[], &["regrow", &id, "--as", "z"]);
    assert!(has_move(&f.file(&id), "z", "shorn", "hairy"));
    f.run(&[], &["doctor"]);
}
