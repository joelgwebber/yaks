//! `yaks commits` and the landed-sha record (yaks-7a11, yaks-735a), through the
//! real binary in temp git repos whose farm is a branch, with `HOME` and
//! `XDG_DATA_HOME` in the temp dir so nothing outside it is touched.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn temp(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("yaks-commits-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d.canonicalize().unwrap()
}

/// `HOME` and the data dir, outside every checkout (a farm worktree inside the
/// repo would show up in `git status`); farm homes are keyed by repository.
fn home() -> PathBuf {
    let h = std::env::temp_dir().join(format!("yaks-commits-home-{}", std::process::id()));
    std::fs::create_dir_all(&h).unwrap();
    h.canonicalize().unwrap()
}

fn yaks(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_yaks"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", home())
        .env("XDG_DATA_HOME", home().join("xdg"))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_ACTOR", "tester")
        .env_remove("YAKS_DIR")
        .env_remove("YAKS_NO_BOOTSTRAP")
        .output()
        .unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

fn ok(cwd: &Path, args: &[&str]) -> String {
    let o = yaks(cwd, args);
    assert!(o.status.success(), "yaks {args:?} failed:\n{}", text(&o));
    text(&o)
}

fn git(cwd: &Path, args: &[&str]) -> String {
    let o = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@t"])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .current_dir(cwd)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .output()
        .unwrap();
    assert!(o.status.success(), "git {args:?}: {}", text(&o));
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

/// A code repo on `main` with a branch farm holding one yak; returns
/// (repo, farm worktree, yak id).
fn branch_farm(tag: &str) -> (PathBuf, PathBuf, String) {
    let r = temp(tag);
    git(&r, &["init", "-q", "-b", "main", "."]);
    std::fs::write(r.join("README.md"), "x\n").unwrap();
    git(&r, &["add", "README.md"]);
    git(&r, &["commit", "-qm", "init"]);
    ok(
        &r,
        &[
            "init", "--mode", "branch", "--skills", "none", "--herd", "t",
        ],
    );
    let farm = PathBuf::from(ok(&r, &["path"]).trim());
    let created = ok(&r, &["create", "a yak"]);
    let id = created
        .split(|c: char| c == ' ' || c == ':')
        .find(|w| w.starts_with("t-"))
        .unwrap_or_else(|| panic!("no id in {created}"))
        .to_string();
    ok(&r, &["commit"]);
    (r, farm, id)
}

fn code_commit(r: &Path, file: &str, msg: &str) -> String {
    std::fs::write(r.join(file), "y\n").unwrap();
    git(r, &["add", file]);
    git(r, &["commit", "-qm", msg]);
    git(r, &["rev-parse", "HEAD"])
}

#[test]
fn commits_greps_the_code_repo_not_the_farm_branch() {
    let (r, farm, id) = branch_farm("grep");
    let on_main = code_commit(&r, "a.txt", &format!("{id}: first change"));
    git(&r, &["checkout", "-q", "-b", "side"]);
    let on_side = code_commit(&r, "b.txt", &format!("{id}: side change"));
    git(&r, &["checkout", "-q", "main"]);
    code_commit(&r, "c.txt", "unrelated");

    let out = ok(&r, &["commits", &id]);
    let (naming, farm_part) = out.split_once("Farm history").expect(&out);
    assert!(naming.contains(&on_main[..7]), "{out}");
    assert!(
        naming.contains(&on_side[..7]),
        "all branches searched:\n{out}"
    );
    assert!(
        !naming.contains("yaks: created"),
        "farm commits are not code:\n{out}"
    );
    assert!(!naming.contains("unrelated"), "{out}");
    assert!(
        farm_part.contains("yaks: created"),
        "the yak file's own history, labelled as farm history:\n{out}"
    );
    // Newest first: the side-branch commit came after the main one.
    assert!(
        naming.find(&on_side[..7]).unwrap() < naming.find(&on_main[..7]).unwrap(),
        "{out}"
    );

    // Run from a subdirectory, and with -C from elsewhere: the same code repo.
    std::fs::create_dir_all(r.join("sub")).unwrap();
    assert!(ok(&r.join("sub"), &["commits", &id]).contains(&on_main[..7]));
    let elsewhere = temp("grep-elsewhere");
    let via_c = ok(&elsewhere, &["-C", r.to_str().unwrap(), "commits", &id]);
    assert!(via_c.contains(&on_main[..7]), "{via_c}");

    // From the farm's own worktree there is no code to search: say so.
    let from_farm = ok(&farm, &["commits", &id]);
    assert!(
        from_farm.contains("not run from a code checkout"),
        "{from_farm}"
    );
    assert!(!from_farm.contains(&on_main[..7]), "{from_farm}");
}

/// A worker branch off `main` with one commit (`msg`) adding `file`; back on
/// `main`. Returns the worker's sha.
fn worker_commit(r: &Path, file: &str, msg: &str) -> String {
    git(r, &["checkout", "-q", "-b", "worker"]);
    let sha = code_commit(r, file, msg);
    git(r, &["checkout", "-q", "main"]);
    sha
}

fn notes_of(farm: &Path, id: &str) -> String {
    std::fs::read_to_string(farm.join(format!("t/yaks/{id}.md"))).unwrap()
}

#[test]
fn land_records_the_landed_and_worker_shas_and_commits_reads_them() {
    let (r, farm, id) = branch_farm("land");
    let w = worker_commit(&r, "w.txt", &format!("{id}: the work"));
    // Something else lands first, so the squash is a new commit, not the worker's.
    code_commit(&r, "other.txt", "other");

    let out = ok(&r, &["land", &w]);
    assert!(
        out.contains(&format!("recorded the landing on {id}")),
        "{out}"
    );
    let landed = git(&r, &["rev-parse", "HEAD"]);
    assert_ne!(landed, w);

    let body = notes_of(&farm, &id);
    assert!(body.contains("kind: landed"), "{body}");
    assert!(body.contains(&format!("commit: {landed}")), "{body}");
    assert!(body.contains(&format!("source: {w}")), "{body}");
    assert!(body.contains("branch: main"), "{body}");
    assert!(
        body.contains(&format!("landed: {} {id}: the work", &landed[..7])),
        "{body}"
    );

    // `yaks commits` reads them first, with the subject from the code repo.
    let c = ok(&r, &["commits", &id]);
    let (recorded, rest) = c.split_once("Commits naming").expect(&c);
    assert!(recorded.contains("Recorded"), "{c}");
    assert!(
        recorded.contains(&format!("landed {} on main", &landed[..7])),
        "{c}"
    );
    assert!(recorded.contains(&format!("{id}: the work")), "{c}");
    assert!(recorded.contains("[tester]"), "{c}");
    assert!(
        rest.contains(&landed[..7]),
        "the message grep still lists it: {c}"
    );

    // JSON carries the same three sections.
    let j: serde_json::Value = serde_json::from_str(&ok(&r, &["commits", &id, "--json"])).unwrap();
    assert_eq!(j["recorded"][0]["kind"], "landed");
    assert_eq!(j["recorded"][0]["commit"], landed.as_str());
    assert_eq!(j["recorded"][0]["source"], w.as_str());
    assert_eq!(j["recorded"][0]["found"]["short"], &landed[..7]);
    assert!(j["named"].as_array().unwrap().len() >= 1, "{j}");
    assert!(j["farm_history"].as_array().unwrap().len() >= 1, "{j}");

    // Landing the same worker again adds no commit and no second note.
    let again = ok(&r, &["land", &w]);
    assert!(again.contains("nothing to commit"), "{again}");
    assert_eq!(notes_of(&farm, &id).matches("kind: landed").count(), 1);
}

#[test]
fn land_yak_flag_names_yaks_a_private_farm_keeps_out_of_messages() {
    let (r, farm, id) = branch_farm("land-yak");
    let w = worker_commit(&r, "w.txt", "the work, with no id in the message");
    let out = ok(&r, &["land", &w, "--yak", &id, "--yak", "t-nope"]);
    assert!(
        out.contains(&format!("recorded the landing on {id}")),
        "{out}"
    );
    assert!(
        out.contains("warning: no such yak t-nope"),
        "unknown ids warn, never fail:\n{out}"
    );
    let body = notes_of(&farm, &id);
    assert!(body.contains("kind: landed"), "{body}");
    assert!(body.contains(&format!("commit: {}", git(&r, &["rev-parse", "HEAD"]))));
}

#[test]
fn land_dry_run_says_what_it_would_record_and_writes_nothing() {
    let (r, farm, id) = branch_farm("land-dry");
    let w = worker_commit(&r, "w.txt", &format!("{id}: the work"));
    let before = notes_of(&farm, &id);
    let head = git(&r, &["rev-parse", "HEAD"]);
    let out = ok(&r, &["land", &w, "--dry-run"]);
    assert!(
        out.contains(&format!(
            "dry run: would record a `kind: landed` note on {id}"
        )),
        "{out}"
    );
    assert_eq!(notes_of(&farm, &id), before, "nothing written");
    assert_eq!(git(&r, &["rev-parse", "HEAD"]), head);
}

#[test]
fn commits_marks_a_recorded_sha_the_code_repo_lacks() {
    let (r, farm, id) = branch_farm("absent");
    ok(
        &r,
        &[
            "update",
            &id,
            "--note",
            "landed elsewhere\n\nnot a note with metadata",
        ],
    );
    // A note with a commit this repository has never seen (a clone, a squash).
    let path = farm.join(format!("t/yaks/{id}.md"));
    let mut body = std::fs::read_to_string(&path).unwrap();
    body.push_str(&format!(
        "\n\n---\n\u{25b8} 2026-01-01T00:00:00Z [x]\nkind: landed\ncommit: {}\nbranch: other\n---\nlanded: dead gone\n",
        "deadbeef".repeat(5)
    ));
    std::fs::write(&path, body).unwrap();
    let c = ok(&r, &["commits", &id]);
    assert!(c.contains("landed deadbee on other"), "{c}");
    assert!(c.contains("(not in this repository)"), "{c}");
}
