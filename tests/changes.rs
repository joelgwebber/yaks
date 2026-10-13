//! `yaks changes <shed>` through the real binary: a worker shed's per-yak
//! changes with actors, and `resolve`'s candidate list (exit 1) for a name that
//! matches no shed.

use assert_cmd::Command;
use std::path::{Path, PathBuf};

fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args([
            "-c",
            "commit.gpgsign=false",
            "-c",
            "init.defaultBranch=main",
        ])
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn yaks(dir: &Path, actor: &str, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(dir)
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", actor)
        .env_remove("YAKS_DIR")
        .args(args)
        .output()
        .unwrap()
}

fn ok(dir: &Path, actor: &str, args: &[&str]) -> String {
    let out = yaks(dir, actor, args);
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// A committed repo with two yaks (`first` shaving, `second` hairy) and a
/// worktree shed `wt` on branch `work`.
fn repo(tag: &str) -> (PathBuf, PathBuf, String, String) {
    let base = std::env::temp_dir().join(format!("yaks-changes-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let t = base.join("repo");
    std::fs::create_dir_all(&t).unwrap();
    let t = t.canonicalize().unwrap();
    git(&t, &["init", "-q"]);
    ok(&t, "coord", &["init"]);
    let id = |title: &str| {
        let v: serde_json::Value =
            serde_json::from_str(&ok(&t, "coord", &["create", title, "--json"])).unwrap();
        v["id"].as_str().unwrap().to_string()
    };
    let (first, second) = (id("first"), id("second"));
    ok(&t, "coord", &["shave", &first]);
    git(&t, &["add", "-A"]);
    git(&t, &["commit", "-q", "-m", "init"]);
    let wt = t.parent().unwrap().join("wt");
    git(
        &t,
        &["worktree", "add", "-q", "-b", "work", wt.to_str().unwrap()],
    );
    (t, wt, first, second)
}

#[test]
fn a_shed_that_created_moved_and_noted_is_listed_per_yak_with_actors() {
    let (t, wt, first, second) = repo("per-yak");
    ok(
        &wt,
        "w-1",
        &["update", &first, "--note", "worker one\nsecond line"],
    );
    ok(&wt, "w-2", &["update", &first, "--note", "worker two"]);
    ok(&wt, "w-2", &["shave", &second]);
    let new: serde_json::Value =
        serde_json::from_str(&ok(&wt, "w-1", &["create", "made here", "--json"])).unwrap();
    let new = new["id"].as_str().unwrap();

    let out = ok(&t, "me", &["changes", "work"]);
    assert!(
        out.contains(&format!("  {first}  first  [shaving]\n")),
        "{out}"
    );
    assert!(out.contains("  w-1  worker one\n"), "{out}");
    assert!(out.contains("  w-2  worker two\n"), "{out}");
    assert!(!out.contains("second line"), "{out}");
    assert!(
        out.contains(&format!("  {second}  second  [hairy -> shaving]\n")),
        "{out}"
    );
    assert!(out.contains("  w-2  moved: hairy -> shaving\n"), "{out}");
    assert!(
        out.contains(&format!("  {new}  made here  [new, hairy]\n")),
        "{out}"
    );
    assert!(
        !out.contains("coord"),
        "the coordinator's claim is before the fork: {out}"
    );

    let j: Vec<serde_json::Value> =
        serde_json::from_str(&ok(&t, "me", &["changes", "w-1", "--json"])).unwrap();
    assert_eq!(j.len(), 3, "{j:?}");
    let actors: Vec<_> = j.iter().find(|y| y["id"] == first.as_str()).unwrap()["notes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["actor"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(actors, ["w-1", "w-2"]);

    // From the shed, `main` is the primary checkout: nothing of its own yet.
    let out = ok(&wt, "me", &["changes", "main"]);
    assert!(out.contains("no changes of its own"), "{out}");
}

#[test]
fn an_unknown_shed_lists_the_candidates_and_exits_1() {
    let (t, _wt, _, _) = repo("unknown");
    let out = yaks(&t, "me", &["changes", "nope"]);
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("no shed matches `nope`; the sheds are:"),
        "{err}"
    );
    assert!(err.contains("/wt"), "{err}");
    assert!(out.stdout.is_empty());
}

#[test]
fn no_argument_means_the_current_shed_and_every_selector_accepts_it() {
    let (t, wt, first, _second) = repo("current");
    ok(&wt, "w-1", &["update", &first, "--note", "worker one"]);

    // From the worker's shed: no argument == its own name == `--shed` it.
    let name = {
        let j: serde_json::Value =
            serde_json::from_str(&ok(&wt, "me", &["sheds", "--json"])).unwrap();
        let here: Vec<_> = j
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["current"] == true)
            .collect();
        assert_eq!(here.len(), 1, "{j}");
        assert_eq!(here[0]["path"], wt.display().to_string().as_str());
        here[0]["name"].as_str().unwrap().to_string()
    };
    let bare = ok(&wt, "me", &["changes"]);
    assert!(bare.contains("  w-1  worker one\n"), "{bare}");
    assert_eq!(bare, ok(&wt, "me", &["changes", &name]));
    assert_eq!(bare, ok(&wt, "me", &["--shed", &name, "changes"]));
    // ...and the same changes as seen from the primary.
    assert_eq!(bare, ok(&t, "me", &["changes", &name]));

    // `main` from the primary is the primary (it used to be an error), and
    // it is the same as no argument there; `--shed main` is a no-op.
    let main = ok(&t, "me", &["changes", "main"]);
    assert!(main.starts_with("main  "), "{main}");
    assert_eq!(main, ok(&t, "me", &["changes"]));
    assert_eq!(main, ok(&t, "me", &["--shed", "main", "changes"]));
}

#[test]
fn answering_names_a_shed_and_the_current_one_is_a_plain_answer() {
    let (_t, wt, first, _second) = repo("answer-current");
    ok(&wt, "w-1", &["ask", &first, "--note", "which way?"]);
    let out = ok(
        &wt,
        "me",
        &["answer", &format!("{first}@main"), "--note", "x"],
    );
    // `main` is another shed from here: written there, not here.
    assert!(out.contains("Wrote the answer into shed main"), "{out}");
    let j: serde_json::Value = serde_json::from_str(&ok(&wt, "me", &["sheds", "--json"])).unwrap();
    let own = j
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["current"] == true)
        .unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string();
    let out = ok(
        &wt,
        "me",
        &["answer", &format!("{first}@{own}"), "--note", "go left"],
    );
    assert!(!out.contains("Wrote the answer into shed"), "{out}");
    assert!(out.starts_with(&format!("Answered {first}")), "{out}");
}

#[test]
fn inbox_and_show_sheds_still_mean_the_other_sheds() {
    let (t, wt, first, _second) = repo("elsewhere");
    ok(&wt, "w-1", &["update", &first, "--note", "worker one"]);
    // From the worker: its own copy is the primary text, never a "Sheds:" block.
    let out = ok(&wt, "me", &["show", &first, "--sheds"]);
    assert!(
        !out.contains("wt"),
        "the viewer is not listed as a shed: {out}"
    );
    // From the primary the worker is listed.
    let out = ok(&t, "me", &["show", &first, "--sheds"]);
    assert!(
        out.contains("Sheds:") && out.contains("worker one"),
        "{out}"
    );
}

/// `--outstanding` / `--status` through the real binary: the landed yak is
/// folded by default, dropped by `--outstanding`; `--status` narrows text and
/// JSON; an unknown status is an error, not "show everything".
#[test]
fn outstanding_and_status_focus_the_list_and_landed_folds() {
    let (t, wt, first, second) = repo("focus");
    ok(&wt, "w-1", &["update", &first, "--note", "still going"]);
    ok(&wt, "w-2", &["shave", &second]);
    // The primary makes the same move, so `second` is landed in the shed.
    ok(&t, "coord", &["shave", &second]);
    git(&t, &["add", "-A"]);
    git(&t, &["commit", "-q", "-m", "primary shaves second"]);

    let all = ok(&t, "me", &["changes", "work"]);
    let i_first = all
        .find(&format!("  {first}  first  [shaving]\n"))
        .expect(&all);
    let i_fold = all.find(&format!("  landed (1): {second}\n")).expect(&all);
    assert!(i_first < i_fold, "outstanding first, landed folded: {all}");
    assert!(
        !all.contains("second  ["),
        "landed yak is not itemized: {all}"
    );

    let out = ok(&t, "me", &["changes", "work", "--outstanding"]);
    assert!(out.contains("  still going\n"), "{out}");
    assert!(!out.contains("landed") && !out.contains(&second), "{out}");

    let out = ok(&t, "me", &["changes", "work", "--status", "shaving"]);
    assert!(
        out.contains(&first) && out.contains(&format!("landed (1): {second}")),
        "{out}"
    );
    let out = ok(&t, "me", &["changes", "work", "--status", "hairy"]);
    assert!(out.ends_with("  nothing in hairy\n"), "{out}");
    let out = ok(
        &t,
        "me",
        &["changes", "work", "--outstanding", "--status", "hairy"],
    );
    assert!(out.ends_with("  nothing outstanding in hairy\n"), "{out}");

    let ids = |args: &[&str]| -> Vec<String> {
        let j: Vec<serde_json::Value> = serde_json::from_str(&ok(&t, "me", args)).unwrap();
        j.iter()
            .map(|y| y["id"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(ids(&["changes", "work", "--json"]).len(), 2);
    assert_eq!(
        ids(&["changes", "work", "--json", "--outstanding"]),
        [first.clone()]
    );
    assert!(ids(&["changes", "work", "--json", "--status", "hairy"]).is_empty());

    let bad = yaks(&t, "me", &["changes", "work", "--status", "nope"]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("unknown status"));
}
