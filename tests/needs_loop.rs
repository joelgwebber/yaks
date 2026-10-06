//! The ask -> answer -> pickup hand-off, driven through the real binary: the
//! `needs: agent` state, the two-section `inbox` (and `--for`, `--json`), `next`
//! marking, and the derived `replied` flag.

use assert_cmd::Command;
use std::path::PathBuf;

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-needs-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let f = Farm { dir };
        f.run("tester", &["init"]);
        f
    }

    fn cmd(&self, actor: &str, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", actor)
            .args(args)
            .output()
            .unwrap()
    }

    /// Run as `actor`; must succeed; returns stdout.
    fn run(&self, actor: &str, args: &[&str]) -> String {
        let out = self.cmd(actor, args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    /// Run as `actor`; must fail; returns stderr.
    fn fail(&self, actor: &str, args: &[&str]) -> String {
        let out = self.cmd(actor, args);
        assert!(!out.status.success(), "{args:?} unexpectedly succeeded");
        String::from_utf8(out.stderr).unwrap()
    }

    fn create(&self, title: &str) -> String {
        let out = self.run("tester", &["create", title, "--json"]);
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn json(&self, args: &[&str]) -> Vec<serde_json::Value> {
        serde_json::from_str::<Vec<serde_json::Value>>(&self.run("tester", args)).unwrap()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn whole_loop_ask_reply_answer_next_pickup() {
    let f = Farm::new("loop");
    let id = f.create("Q");

    // Ask: awaiting a human, out of `next`.
    f.run("coord", &["ask", &id, "--note", "A or B?"]);
    let inbox = f.run("tester", &["inbox"]);
    assert!(
        inbox.contains("Awaiting a human:") && inbox.contains(&id),
        "{inbox}"
    );
    assert!(!inbox.contains("replied"), "{inbox}");
    assert!(f.run("tester", &["next"]).contains("No yaks ready"));

    // A plain-note reply from someone else: still `needs: human`, but flagged.
    f.run("joel", &["update", &id, "--note", "B"]);
    let inbox = f.run("tester", &["inbox"]);
    assert!(inbox.contains("replied"), "{inbox}");
    let j = f.json(&["inbox", "--json"]);
    assert_eq!(
        (j[0]["needs"].as_str(), j[0]["replied"].as_bool()),
        (Some("human"), Some(true))
    );

    // Answer: awaiting an agent; both sections are right, `--for` narrows.
    f.run("joel", &["answer", &id, "--note", "Confirmed: B"]);
    let inbox = f.run("tester", &["inbox"]);
    assert!(inbox.contains("Answered, awaiting an agent"), "{inbox}");
    assert!(!inbox.contains("Awaiting a human"), "{inbox}");
    assert!(
        f.run("tester", &["inbox", "--for", "human"])
            .contains("nothing awaiting a human")
    );
    assert!(f.run("tester", &["inbox", "--for", "agent"]).contains(&id));
    assert!(f.json(&["inbox", "--for", "human", "--json"]).is_empty());
    let j = f.json(&["inbox", "--for", "agent", "--json"]);
    assert_eq!(
        (j[0]["needs"].as_str(), j[0]["replied"].as_bool()),
        (Some("agent"), Some(false))
    );

    // `next` lists it, marked answered.
    let next = f.run("tester", &["next"]);
    assert!(next.contains(&id) && next.contains("answered"), "{next}");

    // Pickup clears it, attributed; nothing else did.
    f.run("agt", &["pickup", &id, "--note", "Doing B"]);
    assert!(f.run("tester", &["inbox"]).contains("Inbox empty"));
    let log = f.run("tester", &["log", "--json"]);
    assert!(
        log.contains("picked up") && log.contains("\"agt\""),
        "{log}"
    );
    let next = f.run("tester", &["next"]);
    assert!(next.contains(&id) && !next.contains("answered"), "{next}");
}

#[test]
fn answer_edge_cases_through_the_cli() {
    let f = Farm::new("edges");
    let id = f.create("E");

    // No `needs`: today's behaviour, the note is recorded, nothing is set.
    f.run("joel", &["answer", &id, "--note", "fyi"]);
    assert!(f.run("tester", &["inbox"]).contains("Inbox empty"));

    // --done clears instead of handing off.
    f.run("coord", &["ask", &id, "--note", "q"]);
    f.run("joel", &["answer", &id, "--done", "--note", "no follow-up"]);
    assert!(f.run("tester", &["inbox"]).contains("Inbox empty"));

    // A second answer on `needs: agent` keeps the state.
    f.run("coord", &["ask", &id, "--note", "q2"]);
    f.run("joel", &["answer", &id, "--note", "one"]);
    let out = f.run("joel", &["answer", &id, "--note", "two"]);
    assert!(out.contains("still needs agent"), "{out}");
    assert_eq!(f.json(&["inbox", "--json"])[0]["needs"], "agent");

    // Ask on `needs: agent` flips back to human and says so.
    let out = f.run("coord", &["ask", &id, "--note", "follow-up"]);
    assert!(out.contains("awaiting a human again"), "{out}");
    assert_eq!(f.json(&["inbox", "--json"])[0]["needs"], "human");
}

#[test]
fn pickup_errors_name_the_state() {
    let f = Farm::new("pickup");
    let id = f.create("P");
    let e = f.fail("agt", &["pickup", &id]);
    assert!(e.contains("no needs block set"), "{e}");
    f.run("coord", &["ask", &id, "--note", "q"]);
    let e = f.fail("agt", &["pickup", &id]);
    assert!(e.contains("needs: human"), "{e}");
    assert_eq!(
        f.json(&["inbox", "--json"])[0]["needs"],
        "human",
        "refusal changed nothing"
    );
    let e = f.fail("agt", &["pickup", "nope-0000"]);
    assert!(e.contains("not found"), "{e}");
}

/// Answering a SHORN yak (not in `next`) must stay findable until picked up:
/// the original failure mode.
#[test]
fn answered_shorn_yak_stays_in_the_inbox_until_picked_up() {
    let f = Farm::new("shorn");
    let id = f.create("S");
    f.run("coord", &["shave", &id]);
    f.run("coord", &["shorn", &id]);
    f.run("coord", &["ask", &id, "--note", "post-merge q"]);
    f.run("joel", &["answer", &id, "--note", "do X"]);
    assert!(f.run("tester", &["inbox", "--for", "agent"]).contains(&id));
    f.run("agt", &["pickup", &id]);
    assert!(f.run("tester", &["inbox"]).contains("Inbox empty"));
}

/// A `needs` change is not a status move: no `moved:` entry is written.
#[test]
fn needs_changes_do_not_write_moved_entries() {
    let f = Farm::new("moved");
    let id = f.create("M");
    f.run("coord", &["ask", &id, "--note", "q"]);
    f.run("joel", &["answer", &id, "--note", "a"]);
    f.run("agt", &["pickup", &id]);
    let log = f.run("tester", &["log", "--json"]);
    assert!(!log.contains("moved:"), "{log}");
}
