//! `yaks sheds` / `changes` / `show --sheds` / `inbox --sheds` / `answer
//! <id>@<shed>` next to a checkout whose farm is still on the old layout
//! (status directories, schema 3), which this yaks no longer reads
//! (phase 4 of the flat farm, yaks-8b7a). The old-layout trees are made by hand
//! and committed in git (nothing writes them any more); the flat side is
//! written by the binary itself.
//!
//! The rules under test: an old-layout shed's farm is reported as such, never
//! read as an empty farm (which would look like it deleted every yak); and a
//! base commit from before a farm migrated has no flat files, so it is a
//! fallback to the next baseline, never a "deleted every yak" delta.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as Proc;

fn git(dir: &Path, args: &[&str]) {
    let out = Proc::new("git")
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

fn run(dir: &Path, actor: &str, args: &[&str]) -> std::process::Output {
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

fn yaks(dir: &Path, actor: &str, args: &[&str]) -> String {
    let out = run(dir, actor, args);
    assert!(
        out.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn json(dir: &Path, args: &[&str]) -> Value {
    serde_json::from_str(&yaks(dir, "viewer", args)).unwrap()
}

const A: &str = "yaks-aaaa";
const B: &str = "yaks-bbbb";
const C: &str = "yaks-cccc";

/// An old-layout (schema 3) yak file: status is the directory, no `status:`.
fn old_yak(root: &Path, status: &str, id: &str, body: &str) {
    let dir = root.join(".yaks").join(status);
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntitle: {id} title\ntype: task\npriority: 3\n\
             created: '2026-01-01T00:00:00Z'\nupdated: '2026-01-01T00:00:00Z'\n---\n{body}\n"
        ),
    )
    .unwrap();
}

fn note(ts: &str, actor: &str, text: &str) -> String {
    format!("\n\n---\n\u{25b8} {ts} [{actor}]\n{text}")
}

/// The old-layout yak `id` as it is in `root`: (status dir, path).
fn old_file(root: &Path, id: &str) -> (String, PathBuf) {
    for st in ["hairy", "shaving", "shorn", "dead"] {
        let p = root.join(".yaks").join(st).join(format!("{id}.md"));
        if p.is_file() {
            return (st.to_string(), p);
        }
    }
    panic!("no old-layout {id} in {}", root.display());
}

/// Append a note to an old-layout yak by hand.
fn old_note(root: &Path, id: &str, ts: &str, actor: &str, text: &str) {
    let (_, p) = old_file(root, id);
    let mut s = fs::read_to_string(&p).unwrap();
    s = s.trim_end().to_string();
    s.push_str(&note(ts, actor, text));
    s.push('\n');
    fs::write(p, s).unwrap();
}

/// What `yaks migrate` (0.0.16) did, by hand (the layout change and nothing else):
/// each `<status>/<id>.md` becomes `<id>.md` with `status:` after `title:`,
/// `artifacts/<id>/` becomes `<id>/` and its links follow, schema 4.
fn migrate_by_hand(root: &Path) {
    let farm = root.join(".yaks");
    for st in ["hairy", "shaving", "shorn", "dead"] {
        let dir = farm.join(st);
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let text = fs::read_to_string(&p).unwrap();
            let text = text
                .replacen(" title\n", &format!(" title\nstatus: {st}\n"), 1)
                .replace("](artifacts/", "](");
            fs::write(farm.join(p.file_name().unwrap()), text).unwrap();
            fs::remove_file(p).unwrap();
        }
        fs::remove_dir(&dir).unwrap();
    }
    if let Ok(rd) = fs::read_dir(farm.join("artifacts")) {
        for e in rd.flatten() {
            fs::rename(e.path(), farm.join(e.file_name())).unwrap();
        }
        fs::remove_dir(farm.join("artifacts")).unwrap();
    }
    fs::write(farm.join("schema"), "4\n").unwrap();
}

struct Repo {
    base: PathBuf,
    /// The primary checkout.
    dir: PathBuf,
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

impl Repo {
    /// A repo whose first commit is an OLD-layout farm: A hairy, B shaving
    /// (with an attachment linked from its body), C shorn.
    fn old(tag: &str) -> Repo {
        let base = std::env::temp_dir().join(format!("yaks-layouts-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        let base = base.canonicalize().unwrap();
        let dir = base.join("repo").canonicalize_or_create();
        git(&dir, &["init", "-q"]);
        let farm = dir.join(".yaks");
        fs::create_dir_all(&farm).unwrap();
        fs::write(farm.join("schema"), "3\n").unwrap();
        old_yak(&dir, "hairy", A, "first");
        old_yak(
            &dir,
            "shaving",
            B,
            &format!("second\n\n![shot](artifacts/{B}/shot.png)"),
        );
        fs::create_dir_all(farm.join("artifacts").join(B)).unwrap();
        fs::write(farm.join("artifacts").join(B).join("shot.png"), "png").unwrap();
        old_yak(&dir, "shorn", C, "third");
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "old layout"]);
        Repo { base, dir }
    }

    /// A worktree shed forked from the current HEAD.
    fn shed(&self, name: &str) -> PathBuf {
        let p = self.base.join(name);
        git(
            &self.dir,
            &["worktree", "add", "-q", "-b", name, p.to_str().unwrap()],
        );
        p
    }
}

trait CanonicalOrCreate {
    fn canonicalize_or_create(self) -> PathBuf;
}
impl CanonicalOrCreate for PathBuf {
    fn canonicalize_or_create(self) -> PathBuf {
        fs::create_dir_all(&self).unwrap();
        self.canonicalize().unwrap()
    }
}

/// The farm part of the `yaks sheds` row for the checkout at `path`: its
/// `farm:` line and the itemized lines under it (rows are told apart by the
/// path ending their first line).
fn farm_text(sheds_out: &str, path: &Path) -> String {
    let is_row = |l: &str| l.starts_with("  worktree") || l.starts_with("* worktree");
    let lines: Vec<&str> = sheds_out.lines().collect();
    let start = lines
        .iter()
        .position(|l| is_row(l) && l.ends_with(path.to_str().unwrap()))
        .unwrap_or_else(|| panic!("no row for {} in\n{sheds_out}", path.display()));
    let block: Vec<&str> = lines[start + 1..]
        .iter()
        .take_while(|l| !is_row(l))
        .copied()
        .collect();
    let at = block
        .iter()
        .position(|l| l.trim_start().starts_with("farm:"))
        .unwrap_or_else(|| panic!("no farm line:\n{sheds_out}"));
    block[at..].join("\n")
}

fn shed_json<'a>(j: &'a Value, path: &Path) -> &'a Value {
    j.as_array()
        .unwrap()
        .iter()
        .find(|s| s["path"] == path.to_str().unwrap())
        .unwrap_or_else(|| panic!("{} not in {j}", path.display()))
}

// -- an old-layout shed next to a flat viewer ----------------------------------

/// The primary migrates and commits; the shed forked before that and stays on
/// the old layout with real work in it.
fn flat_primary_and_old_shed(tag: &str) -> (Repo, PathBuf) {
    let r = Repo::old(tag);
    let wt = r.shed("old-shed");
    migrate_by_hand(&r.dir);
    git(&r.dir, &["add", "-A"]);
    git(&r.dir, &["commit", "-q", "-m", "migrate the farm"]);
    old_note(&wt, A, "2026-02-01T00:00:00Z", "w-1", "found it");
    old_note(&wt, C, "2026-02-03T00:00:00Z", "w-1", "asked: which db?");
    (r, wt)
}

#[test]
fn an_old_layout_shed_is_named_as_such_never_read_as_an_empty_farm() {
    let (r, wt) = flat_primary_and_old_shed("sheds-old");

    let text = yaks(&r.dir, "viewer", &["sheds"]);
    let row = farm_text(&text, &wt);
    assert!(
        row.contains("old layout (pre-0.0.16 clone)"),
        "names the clone: {text}"
    );
    assert!(row.contains("not read"), "{text}");
    // Never an empty farm: no "no changes", no "deleted"/"removed" claim.
    assert!(!row.contains("no changes of its own"), "{text}");
    assert!(!row.contains("no differences"), "{text}");
    // The flat primary is unaffected.
    assert!(
        farm_text(&text, &r.dir).contains("no changes of its own"),
        "{text}"
    );

    let j = json(&r.dir, &["sheds", "--json"]);
    let f = &shed_json(&j, &wt)["farm"];
    assert_eq!(f["state"], "unreadable", "{j}");
    assert!(
        f["message"].as_str().unwrap().contains("pre-0.0.16 clone"),
        "{j}"
    );
    // No `layout` key survives anywhere.
    assert!(f.get("layout").is_none(), "{j}");
    assert!(shed_json(&j, &r.dir)["farm"].get("layout").is_none(), "{j}");
}

#[test]
fn changes_show_and_inbox_name_an_old_layout_shed_and_skip_it() {
    let (r, wt) = flat_primary_and_old_shed("cmds-old");

    let ch = yaks(&r.dir, "viewer", &["changes", "old-shed"]);
    assert!(ch.contains("nothing to show"), "{ch}");
    assert!(ch.contains("old layout (pre-0.0.16 clone)"), "{ch}");
    assert!(!ch.contains("no changes of its own"), "{ch}");

    // `show --sheds`: the old shed is counted as having no readable farm, its
    // copy is not shown (nor shown as absent).
    let out = yaks(&r.dir, "viewer", &["show", B, "--sheds"]);
    assert!(out.contains("no farm of its own in 1 shed"), "{out}");
    assert!(!out.contains("absent in 1"), "{out}");

    // `inbox --sheds`: the ask in the old shed's C is not read.
    let out = yaks(&r.dir, "viewer", &["inbox", "--sheds"]);
    assert!(!out.contains("Shed w-1"), "{out}");
    let _ = wt;
}

#[test]
fn answering_into_an_old_layout_shed_refuses_and_writes_nothing() {
    let (r, wt) = flat_primary_and_old_shed("answer-old");
    let before = fs::read_to_string(old_file(&wt, A).1).unwrap();

    let out = run(
        &r.dir,
        "joel",
        &["answer", &format!("{A}@old-shed"), "--note", "use sqlite"],
    );
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("old-shed"), "names the shed: {err}");
    assert!(err.contains("old layout"), "{err}");
    assert_eq!(before, fs::read_to_string(old_file(&wt, A).1).unwrap());
    assert!(
        !wt.join(".yaks").join(format!("{A}.md")).exists(),
        "no flat file beside the old layout"
    );
}

// -- a base commit from before the farm migrated --------------------------------

#[test]
fn a_flat_shed_forked_before_the_migration_falls_back_to_the_next_base_not_to_nothing() {
    let r = Repo::old("old-base");
    let wt = r.shed("migrated");
    // Both migrate after the fork: the shed's fork point is an old-layout
    // commit with no flat file in it.
    migrate_by_hand(&r.dir);
    git(&r.dir, &["add", "-A"]);
    git(&r.dir, &["commit", "-q", "-m", "migrate the farm"]);
    migrate_by_hand(&wt);
    git(&wt, &["add", "-A"]);
    git(&wt, &["commit", "-q", "-m", "migrate the farm"]);
    yaks(&wt, "shed-worker", &["update", A, "--note", "found it"]);

    let j = json(&r.dir, &["sheds", "--json"]);
    let f = &shed_json(&j, &wt)["farm"];
    assert_eq!(f["state"], "own", "{j}");
    // The fork commit has no flat file: the existing fallback applies, to the
    // next base (the merge-base, a flat commit here), not an empty baseline.
    assert_eq!(f["vs"], "merge-base", "{j}");
    // Not "every yak is new" (nor gone): only the real note shows.
    assert_eq!(f["added"].as_array().unwrap().len(), 0, "{j}");
    assert_eq!(f["moved"].as_array().unwrap().len(), 0, "{j}");
    assert_eq!(f["edited"].as_array().unwrap().len(), 0, "{j}");
    assert_eq!(f["notes"].as_array().unwrap().len(), 1, "{j}");
    assert_eq!(f["notes"][0]["id"], A, "{j}");

    let ch = json(&r.dir, &["changes", "migrated", "--json"]);
    let rows = ch.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{ch}");
    assert_eq!(rows[0]["id"], A);
    assert_eq!(rows[0]["added"], false);
}

// -- an old-layout viewer --------------------------------------------------------

#[test]
fn an_old_layout_checkout_refuses_every_command_naming_the_release_that_migrates_it() {
    let r = Repo::old("viewer-old");
    for args in [&["list"][..], &["sheds"], &["doctor"], &["create", "x"]] {
        let out = run(&r.dir, "viewer", args);
        assert!(!out.status.success(), "{args:?} should refuse");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(
            err.contains("old yak layout") && err.contains("0.0.16"),
            "{args:?}: {err}"
        );
    }
}

#[test]
fn when_every_base_predates_the_migration_the_shed_is_compared_to_the_checkout() {
    let r = Repo::old("all-old-bases");
    let wt = r.shed("migrated");
    // The migration is never committed: every base commit is old-layout, so
    // there is no flat file to read at any of them.
    migrate_by_hand(&r.dir);
    migrate_by_hand(&wt);
    yaks(&wt, "shed-worker", &["update", A, "--note", "found it"]);

    let j = json(&r.dir, &["sheds", "--json"]);
    let f = &shed_json(&j, &wt)["farm"];
    assert_eq!(f["state"], "own", "{j}");
    assert_eq!(f["vs"], "checkout", "{j}");
    // Not "deleted every yak", not "every yak new": the note alone.
    assert_eq!(f["added"].as_array().unwrap().len(), 0, "{j}");
    assert_eq!(f["moved"].as_array().unwrap().len(), 0, "{j}");
    assert_eq!(f["notes"].as_array().unwrap().len(), 1, "{j}");
    let text = yaks(&r.dir, "viewer", &["sheds"]);
    assert!(
        farm_text(&text, &wt).contains("(vs this checkout)"),
        "{text}"
    );
}
