//! Session notes through the real binary (yaks-17f5, yaks-e58f): `session
//! touched`, `session note` (with a generated `## Saved`), `session list
//! --unreviewed` and `session promote`.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-sessions-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f
    }

    fn cmd(&self, args: &[&str], stdin: Option<&str>) -> std::process::Output {
        self.cmd_env(&[], args, stdin)
    }

    /// [`Farm::cmd`] with extra environment variables.
    fn cmd_env(
        &self,
        env: &[(&str, &str)],
        args: &[&str],
        stdin: Option<&str>,
    ) -> std::process::Output {
        let mut c = Command::cargo_bin("yaks").unwrap();
        c.current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .env_remove("DELTA_CURRENT_THREAD_ID")
            .env_remove("DELTA_THREAD_TITLE")
            .env_remove("YAKS_THREAD")
            .args(args);
        for (k, v) in env {
            c.env(k, v);
        }
        if let Some(s) = stdin {
            c.write_stdin(s);
        }
        c.output().unwrap()
    }

    fn run_in(&self, args: &[&str], stdin: Option<&str>) -> String {
        let out = self.cmd(args, stdin);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        self.run_in(args, None)
    }

    fn root(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

const BODY: &str = "## Summary

Built the parser and wrote up how it works.

## Turns

- good: the parser works
- partial: the docs came late

## Lessons

- `dok new` wanted --title; I had to guess the default
";

#[test]
fn a_session_note_cites_what_the_session_wrote_and_is_reviewed_once() {
    let f = Farm::new("flow");
    let v: Value = serde_json::from_str(&f.run(&["create", "Parser", "--json"])).unwrap();
    let id = v["id"].as_str().unwrap().to_string();
    f.run(&["shave", &id]);
    let out = f.run(&["update", &id, "--note", "Built the parser."]);
    let link = out.trim().rsplit(' ').next().unwrap().to_string();
    f.run(&[
        "dok",
        "new",
        "doks/topics/parsing.md",
        "--title",
        "Parsing",
        "--body",
        "How it works.",
    ]);

    let touched = f.run(&["session", "touched", "--since", "1h"]);
    assert!(
        touched.contains(&format!("- {id}: Parser (created, hairy -> shaving)")),
        "{touched}"
    );
    assert!(
        touched.contains(&format!("  - [Built the parser.]({link})")),
        "{touched}"
    );
    assert!(
        touched.contains("- created [/doks/topics/parsing.md](/doks/topics/parsing.md): Parsing"),
        "{touched}"
    );

    // A turn without a verdict is refused, and nothing is written.
    let bad = f.cmd(
        &["session", "note", "--title", "Parser", "--body", "-"],
        Some(&BODY.replace("- partial: the docs", "- the docs")),
    );
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("does not start with a verdict"));

    let out = f.run_in(
        &[
            "session",
            "note",
            "--title",
            "Parser: first pass",
            "--since",
            "1h",
            "--harness",
            "test",
            "--body",
            "-",
        ],
        Some(BODY),
    );
    let rel = out
        .lines()
        .next()
        .unwrap()
        .strip_prefix("Saved session note ")
        .unwrap()
        .to_string();
    assert!(
        rel.starts_with("sessions/") && rel.ends_with("-parser-first-pass.md"),
        "{rel}"
    );
    assert!(rel.contains("/tester-"), "{rel}");
    let text = std::fs::read_to_string(f.root().join(&rel)).unwrap();
    for want in [
        "type: session-review",
        "source: session-note",
        "actor: tester",
        "harness: test",
        "good: 1",
        "partial: 1",
        "## Saved",
        &format!("  - [Built the parser.]({link})"),
    ] {
        assert!(text.contains(want), "missing {want:?} in\n{text}");
    }
    assert!(text.find("## Saved").unwrap() < text.find("## Lessons").unwrap());
    // Its links resolve, and the commit names it.
    f.run(&["doctor"]);
    let log = std::process::Command::new("git")
        .arg("-C")
        .arg(f.root())
        .args(["log", "-1", "--format=%s%n%(trailers)"])
        .output()
        .unwrap();
    let log = String::from_utf8_lossy(&log.stdout);
    assert!(
        log.contains(&format!("sessions {rel}")) && log.contains("Yaks-Actor: tester"),
        "{log}"
    );

    let list = f.run(&["session", "list", "--unreviewed"]);
    assert!(
        list.starts_with(&rel) && list.contains("[good 1, partial 1]"),
        "{list}"
    );
    assert_eq!(
        f.run(&["session", "promote", &rel]).trim(),
        format!("Promoted {rel}")
    );
    assert!(
        f.run(&["session", "list", "--unreviewed"])
            .contains("No unreviewed session notes.")
    );
    assert!(
        f.run(&["session", "promote", &rel])
            .contains("already promoted")
    );
    assert!(f.run(&["session", "list"]).contains("promoted 20"));

    // The next note starts where this one ended: nothing new written.
    let later = f.run(&["session", "touched"]);
    assert_eq!(later, "- nothing\n");
    assert!(
        !f.cmd(&["session", "promote", "doks/topics/parsing.md"], None)
            .status
            .success()
    );
}

/// A session note's attachment directory is its sibling (yaks-db66): `attach`
/// stores into `sessions/.../<note>/` and links it, and doctor sees the link
/// resolve.
#[test]
fn attach_to_a_session_note() {
    let f = Farm::new("attach");
    let out = f.run_in(
        &[
            "session",
            "note",
            "--title",
            "Shots",
            "--harness",
            "test",
            "--body",
            "-",
        ],
        Some(BODY),
    );
    let rel = out
        .lines()
        .next()
        .unwrap()
        .strip_prefix("Saved session note ")
        .unwrap()
        .to_string();
    let shot = f.dir.join("screenshot.png");
    std::fs::write(&shot, b"png").unwrap();
    let out = f.run(&["attach", &rel, shot.to_str().unwrap()]);
    assert!(
        out.contains(&format!("attached screenshot.png to {rel}")),
        "{out}"
    );
    let root = f.root();
    let stem = rel.strip_suffix(".md").unwrap();
    assert_eq!(
        std::fs::read(root.join(stem).join("screenshot.png")).unwrap(),
        b"png"
    );
    let name = stem.rsplit('/').next().unwrap();
    let text = std::fs::read_to_string(root.join(&rel)).unwrap();
    assert!(
        text.trim_end()
            .ends_with(&format!("![screenshot]({name}/screenshot.png)")),
        "{text}"
    );
    assert!(text.starts_with("---\n"), "frontmatter kept: {text}");
    f.run(&["doctor"]);
    // The directory is not read as session notes, and a dead link is found.
    std::fs::remove_file(root.join(stem).join("screenshot.png")).unwrap();
    let bad = f.cmd(&["doctor"], None);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stdout).contains(&format!("{name}/screenshot.png")));
}

/// `$YAKS_THREAD` names the session note's `thread:` (yaks-f4ea), and wins over
/// Delta's thread id.
#[test]
fn a_session_note_records_yaks_thread_over_the_delta_thread_id() {
    let f = Farm::new("thread");
    let env = [
        ("YAKS_THREAD", "hearth-9"),
        ("DELTA_CURRENT_THREAD_ID", "delta-1"),
    ];
    let out = f.cmd_env(
        &env,
        &["session", "note", "--title", "T", "--body", "-"],
        Some(BODY),
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    let rel = stdout
        .lines()
        .next()
        .unwrap()
        .strip_prefix("Saved session note ")
        .unwrap();
    let text = std::fs::read_to_string(f.root().join(rel)).unwrap();
    assert!(text.contains("\nthread: hearth-9\n"), "{text}");
    assert!(!text.contains("delta-1"), "{text}");
}
