//! `yaks sheds` / `changes` / `show --sheds` / `inbox --sheds` / `answer
//! <id>@<shed>` across the two farm layouts (yaks-ca28, phase 2 of the flat
//! farm, yaks-8b7a): a checkout whose farm migrated after it forked, a flat
//! viewer next to an unmigrated shed, and the reverse. The old-layout trees
//! are made by hand and committed in git (nothing writes them any more); the
//! flat side is written by the binary itself.
//!
//! The rule under test: a yak whose only difference is the layout is not a
//! change, so a migration alone reads "no changes of its own".

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

/// What `yaks migrate` does, by hand (the layout change and nothing else):
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

// -- a shed that migrated after it forked -------------------------------------

fn migrated_shed_reads_as_no_changes_of_its_own(committed: bool) {
    let r = Repo::old(if committed {
        "mig-committed"
    } else {
        "mig-dirty"
    });
    let wt = r.shed("migrated");
    migrate_by_hand(&wt);
    if committed {
        git(&wt, &["add", "-A"]);
        git(&wt, &["commit", "-q", "-m", "migrate the farm"]);
    }

    // The fork point is an old-layout commit and the working tree is flat:
    // the migration alone is not a change, from either viewer.
    let text = yaks(&r.dir, "viewer", &["sheds"]);
    let row = farm_text(&text, &wt);
    assert!(row.contains("farm: no changes of its own"), "{text}");
    assert!(!row.contains("old layout"), "{text}");
    let j = json(&r.dir, &["sheds", "--json"]);
    let f = &shed_json(&j, &wt)["farm"];
    assert_eq!(f["layout"], "flat", "{j}");
    assert_eq!(f["vs"], "fork", "{j}");
    for k in ["added", "moved", "notes", "needs", "edited"] {
        assert_eq!(f[k].as_array().unwrap().len(), 0, "{k}: {j}");
    }
    // The viewer is unmigrated, and says so on its own row.
    let here = farm_text(&text, &r.dir);
    assert!(here.contains("(old layout)"), "{text}");

    let ch = yaks(&r.dir, "viewer", &["changes", "migrated"]);
    assert!(ch.contains("no changes of its own"), "{ch}");
    assert_eq!(
        json(&r.dir, &["changes", "migrated", "--json"]),
        serde_json::json!([])
    );

    // From the migrated shed itself (a flat viewer of an old-layout primary).
    let text = yaks(&wt, "viewer", &["sheds"]);
    assert!(
        farm_text(&text, &wt).contains("no changes of its own"),
        "{text}"
    );
    assert!(farm_text(&text, &r.dir).contains("(old layout)"), "{text}");

    // Then real work in the shed (the binary's own flat writer): exactly it
    // is reported, none of the migration.
    yaks(&wt, "shed-worker", &["update", A, "--note", "found it"]);
    yaks(&wt, "shed-worker", &["shave", A]);
    let j = json(&r.dir, &["changes", "migrated", "--json"]);
    let rows = j.as_array().unwrap();
    assert_eq!(rows.len(), 1, "{j}");
    assert_eq!(rows[0]["id"], A);
    assert_eq!(
        rows[0]["moved"],
        serde_json::json!({"from": "hairy", "to": "shaving"})
    );
    assert_eq!(rows[0]["edited"], false);
    assert_eq!(rows[0]["layout"], "flat");
    let text = yaks(&r.dir, "viewer", &["sheds"]);
    let row = farm_text(&text, &wt);
    assert!(
        row.contains("1 moved, 1 new notes") || row.contains("1 moved"),
        "{text}"
    );
    assert!(
        !row.contains(B) && !row.contains(C),
        "only the real change: {text}"
    );
}

#[test]
fn a_shed_that_migrated_uncommitted_shows_no_changes_of_its_own() {
    migrated_shed_reads_as_no_changes_of_its_own(false);
}

#[test]
fn a_shed_that_committed_its_migration_shows_no_changes_of_its_own() {
    migrated_shed_reads_as_no_changes_of_its_own(true);
}

// -- a flat viewer and an unmigrated shed, both ways --------------------------

#[test]
fn a_flat_viewer_and_an_unmigrated_shed_compare_correctly_both_ways() {
    let r = Repo::old("both-ways");
    let wt = r.shed("old-shed");
    // The primary migrates and commits; the shed stays on the old layout.
    migrate_by_hand(&r.dir);
    git(&r.dir, &["add", "-A"]);
    git(&r.dir, &["commit", "-q", "-m", "migrate the farm"]);

    // The unmigrated shed: a note on A, B shorn (a move), a question on C.
    old_note(&wt, A, "2026-02-01T00:00:00Z", "w-1", "found it");
    let (_, bp) = old_file(&wt, B);
    old_note(
        &wt,
        B,
        "2026-02-02T00:00:00Z",
        "w-1",
        "moved: shaving -> shorn",
    );
    fs::create_dir_all(wt.join(".yaks/shorn")).unwrap();
    fs::rename(bp, wt.join(".yaks/shorn").join(format!("{B}.md"))).unwrap();
    old_note(&wt, C, "2026-02-03T00:00:00Z", "w-1", "asked: which db?");
    {
        let (_, p) = old_file(&wt, C);
        let s = fs::read_to_string(&p).unwrap().replacen(
            "priority: 3\n",
            "priority: 3\nneeds: human\n",
            1,
        );
        fs::write(p, s).unwrap();
    }

    // Flat viewer, old shed: its real changes only, and it says old layout.
    let text = yaks(&r.dir, "viewer", &["sheds"]);
    let row = farm_text(&text, &wt);
    assert!(
        row.contains("1 moved") && row.contains("1 needs") && row.contains("(old layout)"),
        "{text}"
    );
    assert!(row.contains(&format!("{B} shaving -> shorn")), "{text}");
    assert!(row.contains(&format!("{A} +1 notes")), "{text}");
    assert!(
        !row.contains(" edited"),
        "layout alone is not an edit: {text}"
    );
    let j = json(&r.dir, &["sheds", "--json"]);
    let f = &shed_json(&j, &wt)["farm"];
    assert_eq!(f["layout"], "old", "{j}");
    assert_eq!(f["vs"], "fork", "{j}");
    assert_eq!(f["moved"][0]["id"], B);
    assert_eq!(f["moved"].as_array().unwrap().len(), 1, "{j}");
    assert_eq!(f["edited"].as_array().unwrap().len(), 0, "{j}");
    assert_eq!(f["added"].as_array().unwrap().len(), 0, "{j}");
    // The flat primary has no changes of its own (its migration is not one).
    let primary = shed_json(&j, &r.dir);
    assert_eq!(primary["farm"]["layout"], "flat");
    assert!(
        farm_text(&text, &r.dir).contains("no changes of its own"),
        "{text}"
    );

    let ch = yaks(&r.dir, "viewer", &["changes", "old-shed"]);
    assert!(ch.contains("(old layout)"), "{ch}");
    assert!(
        ch.contains(&format!("{B}  {B} title  [shaving -> shorn]")),
        "{ch}"
    );
    assert!(ch.contains(&format!("{A}  {A} title  [hairy]")), "{ch}");
    assert!(ch.contains("needs: human"), "{ch}");

    // Old-layout viewer (the shed), flat primary.
    let text = yaks(&wt, "viewer", &["sheds"]);
    let main = farm_text(&text, &r.dir);
    assert!(main.contains("no changes of its own"), "{text}");
    assert!(!main.contains("old layout"), "{text}");
    assert!(farm_text(&text, &wt).contains("(old layout)"), "{text}");
    assert!(
        farm_text(&text, &wt).contains(&format!("{B} shaving -> shorn")),
        "{text}"
    );
    let ch = yaks(&wt, "viewer", &["changes", "main"]);
    assert!(ch.contains("no changes of its own"), "{ch}");

    // show --sheds: the shed's old-layout copy is read, notes and all.
    let out = yaks(&r.dir, "viewer", &["show", B, "--sheds"]);
    assert!(out.contains("status: shorn"), "{out}");
    assert!(out.contains("moved: shaving -> shorn"), "{out}");
    // inbox --sheds: the shed's ask is news to the flat viewer.
    let out = yaks(&r.dir, "viewer", &["inbox", "--sheds"]);
    assert!(out.contains("Shed w-1") && out.contains(C), "{out}");
    // The other way: the old-layout viewer sees the flat primary's farm.
    let out = yaks(&wt, "viewer", &["show", A, "--sheds"]);
    assert!(out.contains("Sheds:"), "{out}");
}

#[test]
fn landed_reads_across_layouts() {
    let r = Repo::old("landed");
    let wt = r.shed("old-shed");
    // The shed (old layout) adds a note; the same entry reaches the (flat,
    // migrated) primary, which is the reference every shed is measured to.
    old_note(&wt, A, "2026-02-01T00:00:00Z", "w-1", "found it");
    migrate_by_hand(&r.dir);
    {
        let p = r.dir.join(".yaks").join(format!("{A}.md"));
        let mut s = fs::read_to_string(&p).unwrap().trim_end().to_string();
        s.push_str(&note("2026-02-01T00:00:00Z", "w-1", "found it"));
        s.push('\n');
        fs::write(p, s).unwrap();
    }
    let text = yaks(&r.dir, "viewer", &["sheds"]);
    let row = farm_text(&text, &wt);
    assert!(
        row.contains("landed (1 yak)") && row.contains("(old layout)"),
        "{text}"
    );
    let ch = yaks(&r.dir, "viewer", &["changes", "old-shed"]);
    assert!(ch.contains(", landed]") || ch.contains("landed"), "{ch}");
    let j = json(&r.dir, &["sheds", "--json"]);
    assert_eq!(
        shed_json(&j, &wt)["farm"]["landed"],
        serde_json::json!([A]),
        "{j}"
    );
}

// -- answering into an unmigrated shed -----------------------------------------

#[test]
fn answering_into_an_unmigrated_shed_refuses_naming_its_farm_and_writes_nothing() {
    let r = Repo::old("answer");
    let wt = r.shed("old-shed");
    migrate_by_hand(&r.dir);
    git(&r.dir, &["add", "-A"]);
    git(&r.dir, &["commit", "-q", "-m", "migrate the farm"]);
    let before = fs::read_to_string(old_file(&wt, A).1).unwrap();

    let out = run(
        &r.dir,
        "joel",
        &["answer", &format!("{A}@old-shed"), "--note", "use sqlite"],
    );
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("schema v3") && err.contains("yaks migrate"),
        "{err}"
    );
    // The message is `store::lock`'s: it names the shed's farm by path (the
    // shed's own name is not in it; see the yaks-ca28 notes).
    assert!(err.contains(wt.to_str().unwrap()), "names the farm: {err}");
    assert_eq!(before, fs::read_to_string(old_file(&wt, A).1).unwrap());
    assert!(
        !wt.join(".yaks").join(format!("{A}.md")).exists(),
        "no flat file beside the old layout"
    );
}
