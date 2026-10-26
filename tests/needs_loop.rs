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

    /// The yak's file, read through `yaks path <id>`.
    fn body(&self, id: &str) -> String {
        let path = self.run("tester", &["path", id]);
        std::fs::read_to_string(path.trim()).unwrap()
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
    assert!(inbox.contains("Awaiting an agent"), "{inbox}");
    assert!(inbox.contains("answered (needs:agent)"), "{inbox}");
    assert!(!inbox.contains("Awaiting a human"), "{inbox}");
    assert!(
        f.run("tester", &["inbox", "--for", "human"])
            .contains("nothing awaiting a human")
    );
    assert!(f.run("tester", &["inbox", "--for", "agent"]).contains(&id));
    assert!(f.json(&["inbox", "--for", "human", "--json"]).is_empty());
    let j = f.json(&["inbox", "--for", "agent", "--json"]);
    assert_eq!(
        (
            j[0]["needs"].as_str(),
            j[0]["replied"].as_bool(),
            j[0]["why"].as_str()
        ),
        (Some("agent"), Some(false), Some("answered"))
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

/// `request` hands a yak with no question to an agent: the inbox marks it
/// `requested` (text and JSON `why`), `next` too, and `pickup` clears it. On a
/// yak awaiting a human it refuses, pointing at `answer`.
#[test]
fn request_hands_to_agent_through_the_inbox() {
    let f = Farm::new("request");
    let id = f.create("R");
    let out = f.run("joel", &["request", &id, "--note", "tidy the body"]);
    assert!(out.contains("needs agent (awaiting pickup"), "{out}");
    let inbox = f.run("tester", &["inbox", "--for", "agent"]);
    assert!(
        inbox.contains("Awaiting an agent") && inbox.contains("requested (needs:agent)"),
        "{inbox}"
    );
    let j = f.json(&["inbox", "--json"]);
    assert_eq!(
        (j[0]["needs"].as_str(), j[0]["why"].as_str()),
        (Some("agent"), Some("requested"))
    );
    let next = f.run("tester", &["next"]);
    assert!(next.contains("requested, awaiting pickup"), "{next}");
    let out = f.run("joel", &["request", &id, "--note", "and summarize"]);
    assert!(out.contains("still needs agent"), "{out}");
    f.run("agt", &["pickup", &id, "--note", "on it"]);
    assert!(f.run("tester", &["inbox"]).contains("Inbox empty"));

    f.run("coord", &["ask", &id, "--note", "q"]);
    let e = f.fail("joel", &["request", &id, "--note", "x"]);
    assert!(
        e.contains("awaiting a human") && e.contains("answer"),
        "{e}"
    );
    assert_eq!(
        f.json(&["inbox", "--json"])[0]["why"],
        serde_json::Value::Null
    );
}

/// `request --kind tidy` writes the kind's instruction (first line names it)
/// under the marker, then the extra note after a blank line; neither `--kind`
/// nor `--note` is an error.
#[test]
fn request_kind_writes_the_instruction_then_the_note() {
    let f = Farm::new("kind");
    let id = f.create("K");
    let e = f.fail("joel", &["request", &id]);
    assert!(e.contains("--kind") && e.contains("--note"), "{e}");

    f.run(
        "joel",
        &["request", &id, "--kind", "tidy", "--note", "esp. the plan"],
    );
    let body = f.body(&id);
    let want = "requested: needs agent\n\nTidy request: change only the line structure";
    assert!(body.contains(want), "{body}");
    assert!(body.contains(&format!("`yaks path {id}`")), "{body}");
    assert!(body.contains("pickup"), "{body}");
    assert!(body.trim_end().ends_with("\n\nesp. the plan"), "{body}");
    assert!(
        f.run("tester", &["inbox", "--for", "agent"])
            .contains("requested"),
    );

    f.run("joel", &["request", &id, "--kind", "summarize"]);
    let body = f.body(&id);
    assert!(
        body.contains("requested: needs agent\n\nSummarize request: append one note"),
        "{body}"
    );
}

/// `bulk --request tidy` is a dry run without `--commit` (nothing queued),
/// queues on every match with it, and skips (and reports) a yak awaiting a
/// human without failing.
#[test]
fn bulk_request_dry_runs_then_queues_and_skips_awaiting_human() {
    let f = Farm::new("bulkreq");
    let a = f.create("A");
    let b = f.create("B");
    let h = f.create("H");
    f.run("tester", &["update", &a, &b, &h, "--add-label", "chore"]);
    f.run("coord", &["ask", &h, "--note", "q?"]);

    let out = f.run("joel", &["bulk", "--label", "chore", "--request", "tidy"]);
    assert!(out.contains("would update 3 yaks"), "{out}");
    assert!(out.contains("mutation: request tidy"), "{out}");
    assert!(out.contains("awaiting a human: request skipped"), "{out}");
    assert!(out.contains("dry run"), "{out}");
    assert!(f.json(&["inbox", "--for", "agent", "--json"]).is_empty());

    let out = f.run(
        "joel",
        &[
            "bulk",
            "--label",
            "chore",
            "--request",
            "tidy",
            "--note",
            "x",
            "--commit",
        ],
    );
    assert!(out.contains(&format!("Requested {a}: tidy")), "{out}");
    assert!(out.contains(&format!("Requested {b}: tidy")), "{out}");
    assert!(
        out.contains(&format!("skipped 1 awaiting a human (answer instead): {h}")),
        "{out}"
    );
    let agent = f.json(&["inbox", "--for", "agent", "--json"]);
    let mut ids: Vec<&str> = agent.iter().map(|j| j["id"].as_str().unwrap()).collect();
    ids.sort();
    let mut want = vec![a.as_str(), b.as_str()];
    want.sort();
    assert_eq!(ids, want);
    assert_eq!(agent[0]["why"].as_str(), Some("requested"));
    let body = f.body(&a);
    assert!(
        body.contains("requested: needs agent\n\nTidy request:") && body.contains("\n\nx"),
        "{body}"
    );
    // Still awaiting the human, untouched.
    assert_eq!(
        f.json(&["inbox", "--for", "human", "--json"])[0]["id"].as_str(),
        Some(h.as_str())
    );
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
