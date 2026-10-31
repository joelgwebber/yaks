//! `yaks wait` through the real binary: a hit prints exactly what `inbox`
//! prints for the same flags and exits 0; a timeout exits 2 with nothing on
//! stdout; a background answer wakes a waiting process.

mod common;
use assert_cmd::Command;
use std::path::PathBuf;
use std::time::{Duration, Instant};

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-wait-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run("tester", &["init", "--no-skills"]);
        f
    }

    fn cmd(&self, actor: &str, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", actor)
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, actor: &str, args: &[&str]) -> String {
        let out = self.cmd(actor, args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn create(&self, title: &str) -> String {
        let out = self.run("tester", &["create", title, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        v["id"].as_str().unwrap().to_string()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

#[test]
fn timeout_exits_2_with_nothing_on_stdout() {
    let f = Farm::new("timeout");
    let id = f.create("Q");
    f.run("coord", &["ask", &id, "--note", "A or B?"]);
    // Something awaits a human, nothing an agent: `--for agent` times out.
    let out = f.cmd("agt", &["wait", "--for", "agent", "--timeout", "1s"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(
        out.stdout.is_empty(),
        "stdout: {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    let err = String::from_utf8(out.stderr).unwrap();
    assert_eq!(err.lines().count(), 1, "one-line stderr: {err:?}");
    assert!(err.contains("timed out after 1s"), "{err}");
}

#[test]
fn hit_prints_what_inbox_prints() {
    let f = Farm::new("hit");
    let id = f.create("Q");
    f.run("coord", &["ask", &id, "--note", "A or B?"]);
    for args in [&[][..], &["--for", "human"][..], &["--json"][..]] {
        let mut wait = vec!["wait", "--timeout", "0s"];
        wait.extend_from_slice(args);
        let mut inbox = vec!["inbox"];
        inbox.extend_from_slice(args);
        assert_eq!(f.run("agt", &wait), f.run("agt", &inbox), "{args:?}");
    }
}

#[test]
fn new_ignores_a_row_present_at_start() {
    let f = Farm::new("new");
    let id = f.create("Q");
    f.run("coord", &["ask", &id, "--note", "A or B?"]);
    let out = f.cmd("agt", &["wait", "--new", "--timeout", "1s"]);
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn a_background_answer_wakes_the_waiter() {
    let f = Farm::new("wake");
    let id = f.create("Q");
    f.run("coord", &["ask", &id, "--note", "A or B?"]);
    let bin = assert_cmd::cargo::cargo_bin("yaks");
    let start = Instant::now();
    let waiter = std::process::Command::new(bin)
        .current_dir(&f.dir)
        .env("XDG_DATA_HOME", common::data_home(&f.dir))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "agt")
        .args([
            "wait",
            "--for",
            "agent",
            "--timeout",
            "30s",
            "--interval",
            "1s",
        ])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_millis(300));
    f.run("joel", &["answer", &id, "--note", "B"]);
    let out = waiter.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(&id), "{stdout}");
    assert!(start.elapsed() < Duration::from_secs(10));
}
