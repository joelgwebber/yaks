//! `yaks attach <dir>` through the real binary (yaks-a57c): a directory is
//! stored with its layout under the attachment directory, in one write and one
//! commit, with the directory source's skip rules and limits.

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-attdir-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f
    }

    fn cmd(&self, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env_remove("YAKS_NO_AUTOCOMMIT")
            .env("YAKS_ACTOR", "tester")
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        let o = self.cmd(args);
        assert!(
            o.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8(o.stdout).unwrap()
    }

    fn fails(&self, args: &[&str]) -> String {
        let o = self.cmd(args);
        assert!(!o.status.success(), "{args:?} should fail");
        String::from_utf8(o.stderr).unwrap()
    }

    fn yak(&self) -> String {
        let v: Value = serde_json::from_str(&self.run(&["create", "a yak", "--json"])).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    fn root(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
    }

    fn commits(&self) -> usize {
        let o = std::process::Command::new("git")
            .arg("-C")
            .arg(self.root())
            .args(["rev-list", "--count", "HEAD"])
            .output()
            .unwrap();
        String::from_utf8(o.stdout).unwrap().trim().parse().unwrap()
    }

    /// A project directory `proj/` with a nested layout and things to skip.
    fn project(&self) -> PathBuf {
        let p = self.dir.join("proj");
        for (rel, text) in [
            ("README.md", "# proj"),
            (".github/ci.yml", "on: push"),
            ("src/main.rs", "fn main() {}"),
            ("src/util/mod.rs", "pub fn f() {}"),
            (".DS_Store", "x"),
            (".git/config", "x"),
            ("node_modules/dep/index.js", "x"),
            ("target/debug/out", "x"),
        ] {
            write(&p.join(rel), text);
        }
        p
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// Every file under `dir`, relative and sorted.
fn tree(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            if e.path().is_dir() {
                stack.push(e.path());
            } else {
                let rel = e.path().strip_prefix(dir).unwrap().to_path_buf();
                out.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    out.sort();
    out
}

#[test]
fn a_directory_keeps_its_layout_on_a_yak_and_is_linked_once() {
    let f = Farm::new("yak");
    let id = f.yak();
    let proj = f.project();
    let before = f.commits();
    let out = f.run(&["attach", &id, proj.to_str().unwrap()]);
    assert!(out.contains("proj/ (4 files)"), "{out}");
    assert_eq!(f.commits(), before + 1, "one write, one commit");

    let yak_file = PathBuf::from(f.run(&["path", &id]).trim());
    let dir = yak_file.with_extension("").join("proj");
    assert_eq!(
        tree(&dir),
        [
            ".github/ci.yml",
            "README.md",
            "src/main.rs",
            "src/util/mod.rs"
        ],
        "layout kept, dot dirs kept, skip names skipped"
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("src/util/mod.rs")).unwrap(),
        "pub fn f() {}"
    );
    let body = std::fs::read_to_string(&yak_file).unwrap();
    assert_eq!(
        body.matches(&format!("[proj/]({id}/proj/)")).count(),
        1,
        "{body}"
    );
    assert!(!body.contains("main.rs"), "{body}");
}

#[test]
fn a_directory_on_a_dok_goes_in_the_doks_directory() {
    let f = Farm::new("dok");
    f.run(&["dok", "new", "doks/topics/x.md", "--body", "Notes."]);
    let proj = f.project();
    // A `.md` file under a dok's directory would read as another dok: refused
    // with nothing written, as for a single `.md` file.
    let err = f.fails(&["attach", "doks/topics/x.md", proj.to_str().unwrap()]);
    assert!(err.contains("proj/README.md"), "{err}");
    assert!(!f.root().join("doks/topics/x").exists(), "nothing written");

    std::fs::remove_file(proj.join("README.md")).unwrap();
    f.run(&["attach", "doks/topics/x.md", proj.to_str().unwrap()]);
    let root = f.root();
    assert_eq!(
        tree(&root.join("doks/topics/x/proj")),
        [".github/ci.yml", "src/main.rs", "src/util/mod.rs"]
    );
    let text = std::fs::read_to_string(root.join("doks/topics/x.md")).unwrap();
    assert!(text.contains("[proj/](x/proj/)"), "{text}");
}

#[test]
fn no_link_stores_without_touching_the_body_and_name_renames_the_directory() {
    let f = Farm::new("name");
    let id = f.yak();
    let proj = f.project();
    let yak_file = PathBuf::from(f.run(&["path", &id]).trim());
    let body = std::fs::read_to_string(&yak_file).unwrap();
    f.run(&[
        "attach",
        &id,
        proj.to_str().unwrap(),
        "--no-link",
        "--name",
        "code",
    ]);
    assert_eq!(std::fs::read_to_string(&yak_file).unwrap(), body);
    assert!(
        yak_file
            .with_extension("")
            .join("code/src/main.rs")
            .is_file()
    );
}

#[test]
fn over_the_limit_refuses_and_writes_nothing() {
    let f = Farm::new("limit");
    let id = f.yak();
    let big = f.dir.join("big");
    for i in 0..201 {
        write(&big.join(format!("f{i}.txt")), "x");
    }
    let before = f.commits();
    let err = f.fails(&["attach", &id, big.to_str().unwrap()]);
    assert!(err.contains("more than 200 files"), "{err}");

    let fat = f.dir.join("fat");
    write(&fat.join("a.bin"), &"x".repeat(6 * 1024 * 1024));
    let err = f.fails(&["attach", &id, fat.to_str().unwrap()]);
    assert!(err.contains("byte limit"), "{err}");

    let yak_file = PathBuf::from(f.run(&["path", &id]).trim());
    assert!(!yak_file.with_extension("").exists(), "nothing written");
    assert_eq!(f.commits(), before);
}

#[test]
fn a_directory_with_nothing_to_store_and_a_caption_are_refused() {
    let f = Farm::new("empty");
    let id = f.yak();
    let skipped = f.dir.join("skipped");
    write(&skipped.join("node_modules/a.js"), "x");
    let err = f.fails(&["attach", &id, skipped.to_str().unwrap()]);
    assert!(err.contains("no files to attach"), "{err}");
    let proj = f.project();
    let err = f.fails(&["attach", &id, proj.to_str().unwrap(), "--caption", "c"]);
    assert!(err.contains("--caption"), "{err}");
}

/// A symlink in the directory (yaks-8218) is never followed or copied, to a
/// file or a directory, and `attach` says it left each one out.
#[cfg(unix)]
#[test]
fn a_symlink_in_a_directory_is_skipped_and_reported_not_followed() {
    use std::os::unix::fs::symlink;
    let f = Farm::new("links");
    let id = f.yak();
    let outside = f.dir.join("outside");
    write(&outside.join("secret.txt"), "s");
    write(&outside.join("sub/deep.txt"), "d");
    let proj = f.dir.join("linked");
    write(&proj.join("real.txt"), "r");
    symlink(outside.join("secret.txt"), proj.join("file-link")).unwrap();
    symlink(&outside, proj.join("dir-link")).unwrap();
    symlink("/etc", proj.join("etc-link")).unwrap();

    let out = f.run(&["attach", &id, proj.to_str().unwrap()]);
    assert!(out.contains("attached linked/ (1 file) to"), "{out}");
    assert!(
        out.contains("skipped 3: linked/dir-link, linked/etc-link, linked/file-link"),
        "{out}"
    );
    let yak_file = PathBuf::from(f.run(&["path", &id]).trim());
    let dir = yak_file.with_extension("").join("linked");
    assert_eq!(tree(&dir), ["real.txt"]);
}
