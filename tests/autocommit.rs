//! Per-change farm commits (yaks-a838) through the real binary: each write is
//! one commit with a `Yaks-Actor` trailer, an uncommitted hand edit the write
//! rewrites is committed first and alone, other dirty files are left alone,
//! `yaks undo` reverts an actor's commits, and `YAKS_NO_AUTOCOMMIT` turns it
//! off.

mod common;
use assert_cmd::Command;
use std::path::{Path, PathBuf};

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir =
            std::env::temp_dir().join(format!("yaks-autocommit-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run("me", &["init", "--no-skills"]);
        f
    }

    fn out(&self, actor: &str, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env_remove("YAKS_NO_AUTOCOMMIT")
            .env("YAKS_ACTOR", actor)
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, actor: &str, args: &[&str]) -> String {
        let o = self.out(actor, args);
        assert!(
            o.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8(o.stdout).unwrap()
    }

    fn create(&self, actor: &str, title: &str) -> String {
        let v: serde_json::Value =
            serde_json::from_str(&self.run(actor, &["create", title, "--json"])).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    /// The farm worktree (the farm root's parent).
    fn wt(&self) -> PathBuf {
        PathBuf::from(self.run("me", &["path"]).trim())
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn git(&self, args: &[&str]) -> String {
        let o = std::process::Command::new("git")
            .arg("-C")
            .arg(self.wt())
            .args(args)
            .output()
            .unwrap();
        String::from_utf8(o.stdout).unwrap()
    }

    fn subjects(&self) -> Vec<String> {
        self.git(&["log", "--format=%s"])
            .lines()
            .map(String::from)
            .collect()
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn path_of(f: &Farm, id: &str) -> PathBuf {
    PathBuf::from(f.run("me", &["path", id]).trim())
}

fn append(p: &Path, s: &str) {
    let t = std::fs::read_to_string(p).unwrap();
    std::fs::write(p, format!("{t}{s}")).unwrap();
}

#[test]
fn each_write_is_one_commit_with_its_actor() {
    let f = Farm::new("each");
    let a = f.create("alice", "First");
    f.run("bob", &["shave", &a]);
    f.run("bob", &["dok", "new", "doks/notes.md"]);
    assert_eq!(
        f.subjects(),
        [
            "yaks: doks doks/notes.md".to_string(),
            format!("yaks: shaving {a}"),
            format!("yaks: created {a}"),
            "yaks: new farm".into()
        ]
    );
    let actors = f.git(&["log", "--format=%(trailers:key=Yaks-Actor,valueonly)"]);
    let actors: Vec<&str> = actors.lines().filter(|l| !l.is_empty()).collect();
    assert_eq!(actors, ["bob", "bob", "alice"]);
    assert_eq!(f.git(&["status", "--porcelain"]), "", "nothing left dirty");
    assert_eq!(
        f.run("me", &["status"]).lines().next(),
        Some("farm clean: nothing to commit")
    );
}

#[test]
fn a_hand_edit_is_committed_first_and_other_dirty_files_are_left_alone() {
    let f = Farm::new("hand");
    let a = f.create("me", "Edited by hand");
    let b = f.create("me", "Untouched");
    append(&path_of(&f, &a), "\nhand line\n");
    append(&path_of(&f, &b), "\nwork in progress\n");
    std::fs::write(f.wt().join(".yaks/scratch.txt"), "draft").unwrap();
    f.run("bot", &["update", &a, "--note", "bot note"]);
    let s = f.subjects();
    assert_eq!(s[0], format!("yaks: updated {a}"));
    assert!(
        s[1].starts_with("yaks: hand edits to ") && s[1].contains(&a),
        "{s:?}"
    );
    // The hand edit's commit holds only it; the bot's holds only the note.
    let hand = f.git(&["show", "HEAD~1", "--format="]);
    assert!(
        hand.contains("+hand line") && !hand.contains("bot note"),
        "{hand}"
    );
    let bot = f.git(&["show", "HEAD", "--format="]);
    assert!(
        bot.contains("bot note") && !bot.contains("+hand line"),
        "{bot}"
    );
    // The other yak's edit and the scratch file are still uncommitted.
    let st = f.git(&["status", "--porcelain"]);
    assert!(
        st.contains(&format!("{b}.md")) && st.contains("scratch.txt"),
        "{st}"
    );
    assert!(!st.contains(&format!("{a}.md")), "{st}");
}

#[test]
fn undo_reverts_an_actors_commits_and_keeps_the_rest() {
    let f = Farm::new("undo");
    let keep = f.create("alice", "Keep");
    let gone = f.create("bot", "Bot work");
    f.run("bot", &["update", &keep, "--note", "bot scribble"]);
    f.run("alice", &["update", &keep, "--note", "alice note"]);
    let listed = f.run("me", &["undo", "--by", "bot"]);
    assert!(listed.contains("2 commit(s); add --yes"), "{listed}");
    assert!(
        f.run("me", &["list", "--all"]).contains(&gone),
        "listing changes nothing"
    );
    f.run("me", &["undo", "--by", "bot", "--yes"]);
    assert!(!f.run("me", &["list", "--all"]).contains(&gone));
    let shown = f.run("me", &["show", &keep]);
    assert!(
        shown.contains("alice note") && !shown.contains("bot scribble"),
        "{shown}"
    );
    assert!(f.subjects()[0].starts_with("yaks: undo "));
    f.run("me", &["doctor"]);
    // A dirty farm refuses an undo.
    append(&path_of(&f, &keep), "\nunsaved\n");
    assert!(
        !f.out("me", &["undo", "--by", "alice", "--yes"])
            .status
            .success()
    );
}

#[test]
fn the_opt_out_leaves_writes_uncommitted() {
    let f = Farm::new("optout");
    let o = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&f.dir)
        .env("XDG_DATA_HOME", common::data_home(&f.dir))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env("YAKS_NO_AUTOCOMMIT", "1")
        .args(["create", "Quiet"])
        .output()
        .unwrap();
    assert!(o.status.success());
    assert_eq!(f.subjects(), ["yaks: new farm"]);
    assert!(!f.git(&["status", "--porcelain"]).is_empty());
}

/// A `git` on `PATH` that counts how many times it is run, then runs the real
/// one. Each spawn costs 10-30 ms, so a write's count is its latency (yaks-7058).
#[cfg(unix)]
fn count_git_spawns(f: &Farm, args: &[&str]) -> usize {
    use std::os::unix::fs::PermissionsExt;
    let real = String::from_utf8(
        std::process::Command::new("sh")
            .args(["-c", "command -v git"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();
    let shim = f.dir.with_file_name(format!(
        "{}-shim",
        f.dir.file_name().unwrap().to_string_lossy()
    ));
    let _ = std::fs::remove_dir_all(&shim);
    std::fs::create_dir_all(&shim).unwrap();
    let count = shim.join("count");
    let script = shim.join("git");
    std::fs::write(
        &script,
        format!(
            "#!/bin/sh\necho x >> '{}'\nexec '{}' \"$@\"\n",
            count.display(),
            real.trim()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    let path = format!(
        "{}:{}",
        shim.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let o = Command::cargo_bin("yaks")
        .unwrap()
        .current_dir(&f.dir)
        .env("PATH", path)
        .env("XDG_DATA_HOME", common::data_home(&f.dir))
        .env("YAKS_SKILLS_AUTOSYNC", "0")
        .env_remove("YAKS_NO_AUTOCOMMIT")
        .env("YAKS_ACTOR", "me")
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let n = std::fs::read_to_string(&count).map_or(0, |t| t.lines().count());
    let _ = std::fs::remove_dir_all(&shim);
    n
}

/// A write with its per-change commit is a handful of git calls, not dozens:
/// the farm's top level, git dir and branch are found once, and the commit is
/// built from one `status` (yaks-7058). This was 30 before; a few more is
/// fine, a return to discovery per step is not.
#[cfg(unix)]
#[test]
fn a_write_spawns_few_git_processes() {
    let f = Farm::new("spawns");
    let a = f.create("me", "Count me");
    let n = count_git_spawns(&f, &["update", &a, "--note", "n"]);
    assert!(n <= 24, "an update spawned git {n} times");
    // A command that writes nothing (the yak is already being shaved) pays for
    // discovery and the snapshot only. (Not `update --title <same>`: that
    // rewrites `updated:`, so whether it commits depends on the clock.)
    f.run("me", &["shave", &a]);
    let none = count_git_spawns(&f, &["shave", &a]);
    assert!(none <= 12, "a write-less command spawned git {none} times");
}
