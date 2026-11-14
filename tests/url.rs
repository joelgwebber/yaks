//! `yaks url` and the `/sources/<id>` link (yaks-f505), through the real
//! binary: a scratch code repo with a GitHub-looking `origin`, its farm
//! branch in a worktree, and refs under `refs/remotes/origin/` standing in for
//! a push (nothing touches the network).

mod common;
use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-url-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        std::fs::create_dir_all(f.dir.join("src")).unwrap();
        std::fs::write(f.dir.join("src/a.rs"), "one\ntwo\n").unwrap();
        git(&f.dir, &["add", "."]);
        git(&f.dir, &["commit", "-q", "-m", "code"]);
        f
    }

    fn cmd(&self, args: &[&str]) -> std::process::Output {
        Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env("YAKS_ACTOR", "tester")
            .args(args)
            .output()
            .unwrap()
    }

    fn run(&self, args: &[&str]) -> String {
        let out = self.cmd(args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    }

    /// (stdout, stderr) of a command that must succeed.
    fn both(&self, args: &[&str]) -> (String, String) {
        let out = self.cmd(args);
        assert!(
            out.status.success(),
            "{args:?} failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        (
            String::from_utf8(out.stdout).unwrap().trim().to_string(),
            String::from_utf8(out.stderr).unwrap(),
        )
    }

    fn fails(&self, args: &[&str]) -> String {
        let out = self.cmd(args);
        assert!(!out.status.success(), "{args:?} should fail");
        String::from_utf8(out.stderr).unwrap()
    }

    fn root(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
    }

    /// The farm worktree's top (the parent of a `.yaks` root, else the root).
    fn top(&self) -> PathBuf {
        let r = self.root();
        if r.file_name().is_some_and(|n| n == ".yaks") {
            r.parent().unwrap().to_path_buf()
        } else {
            r
        }
    }

    fn create(&self) -> String {
        let v: Value = serde_json::from_str(&self.run(&["create", "A yak", "--json"])).unwrap();
        v["id"].as_str().unwrap().to_string()
    }

    /// Pretend a push: `origin` is `url`, and has `branch` at the checkout's HEAD.
    fn publish(&self, repo: &Path, url: &str, branch: &str) {
        let _ = std::process::Command::new("git")
            .args(["remote", "remove", "origin"])
            .current_dir(repo)
            .output();
        git(repo, &["remote", "add", "origin", url]);
        let head = git(repo, &["rev-parse", "HEAD"]);
        git(
            repo,
            &[
                "update-ref",
                &format!("refs/remotes/origin/{branch}"),
                &head,
            ],
        );
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = std::process::Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "-c",
            "commit.gpgsign=false",
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
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn farm_files_link_on_the_farm_branch_and_code_on_the_code_branch() {
    let f = Farm::new("flow");
    let id = f.create();
    f.run(&["dok", "new", "doks/a.md", "--body", "## Setup\n\nText.\n"]);
    std::fs::write(f.dir.join("shot.txt"), "x").unwrap();
    f.run(&["attach", &id, f.dir.join("shot.txt").to_str().unwrap()]);
    // The farm worktree and the code checkout share the repository's remote.
    f.run(&["commit"]);
    let top = f.top();
    let prefix = f.root().strip_prefix(&top).unwrap().to_path_buf();
    let prefix = if prefix.as_os_str().is_empty() {
        String::new()
    } else {
        format!("{}/", prefix.display())
    };
    f.publish(&f.dir, "git@github.com:o/r.git", "main");
    git(
        &f.dir,
        &[
            "update-ref",
            "refs/remotes/origin/yaks",
            &git(&top, &["rev-parse", "HEAD"]),
        ],
    );
    let farm_head = git(&top, &["rev-parse", "HEAD"]);
    let code_head = git(&f.dir, &["rev-parse", "main"]);
    let herd = id.split('-').next().unwrap();

    // A yak, a dok with a heading, an attachment.
    let (url, err) = f.both(&["url", &id]);
    assert_eq!(
        url,
        format!("https://github.com/o/r/blob/yaks/{prefix}{herd}/yaks/{id}.md")
    );
    assert_eq!(err, "");
    let (url, _) = f.both(&["url", "/doks/a.md#setup"]);
    assert_eq!(
        url,
        format!("https://github.com/o/r/blob/yaks/{prefix}doks/a.md#setup")
    );
    let (url, _) = f.both(&["url", &format!("{id}/shot.txt")]);
    assert_eq!(
        url,
        format!("https://github.com/o/r/blob/yaks/{prefix}{herd}/yaks/{id}/shot.txt")
    );
    // A path on disk is the same file.
    let path = f.root().join("doks/a.md");
    let (url, _) = f.both(&["url", path.to_str().unwrap()]);
    assert_eq!(
        url,
        format!("https://github.com/o/r/blob/yaks/{prefix}doks/a.md")
    );

    // `--pin`: the farm commit the remote has.
    let (url, err) = f.both(&["url", "--pin", "/doks/a.md#L1-L2"]);
    assert_eq!(
        url,
        format!("https://github.com/o/r/blob/{farm_head}/{prefix}doks/a.md?plain=1#L1-L2")
    );
    assert_eq!(err, "");

    // A pinned `code:` path is its own commit, lines kept; plain `code:` and
    // a path in the checkout are the code branch.
    let (url, _) = f.both(&[
        "url",
        &format!("code:src/a.rs@{}#L100-L120", &code_head[..10]),
    ]);
    assert_eq!(
        url,
        format!("https://github.com/o/r/blob/{code_head}/src/a.rs#L100-L120")
    );
    let (url, _) = f.both(&["url", "code:src/a.rs"]);
    assert_eq!(url, "https://github.com/o/r/blob/main/src/a.rs");
    let (url, _) = f.both(&["url", f.dir.join("src/a.rs").to_str().unwrap()]);
    assert_eq!(url, "https://github.com/o/r/blob/main/src/a.rs");

    // The farm is ahead of `origin/yaks`: `--pin` falls back to the branch.
    f.run(&["dok", "new", "doks/b.md", "--body", "B."]);
    f.run(&["commit"]);
    let (url, err) = f.both(&["url", "--pin", "/doks/b.md"]);
    assert_eq!(
        url,
        format!("https://github.com/o/r/blob/yaks/{prefix}doks/b.md")
    );
    assert!(
        err.contains("ahead") && err.contains("branch yaks"),
        "{err}"
    );
    // A file the remote does not have yet is said so.
    let (_, err) = f.both(&["url", "/doks/b.md"]);
    assert!(err.contains("not in origin/yaks"), "{err}");
}

#[test]
fn url_refuses_with_a_reason() {
    let f = Farm::new("refuse");
    f.run(&["dok", "new", "doks/a.md", "--body", "A."]);
    f.run(&["commit"]);
    // No remote at all.
    let err = f.fails(&["url", "/doks/a.md"]);
    assert!(err.contains("no remote"), "{err}");
    // A host yaks does not know names `yaks.host`; declaring it works.
    f.publish(&f.dir, "https://git.example.com/o/r.git", "yaks");
    let err = f.fails(&["url", "/doks/a.md"]);
    assert!(err.contains("yaks.host"), "{err}");
    git(&f.dir, &["config", "yaks.host", "forgejo"]);
    let (url, _) = f.both(&["url", "code:src/a.rs"]);
    assert!(url.starts_with("https://git.example.com/o/r/src/"), "{url}");
    // Missing things, and a private farm.
    assert!(f.fails(&["url", "/doks/none.md"]).contains("no such file"));
    assert!(f.fails(&["url", "code:nope.rs"]).contains("no such file"));
    f.run(&["private"]);
    assert!(f.fails(&["url", "/doks/a.md"]).contains("private"));
}

#[test]
fn a_source_directory_link_peeks_and_doctor_flags_a_broken_one() {
    let f = Farm::new("sources");
    let src = f.root().join("sources/2026-11-03-81364278");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::write(
        src.join("source.md"),
        "---\ntitle: A source\n---\n\nThe body.\n",
    )
    .unwrap();
    f.run(&[
        "dok",
        "new",
        "doks/a.md",
        "--body",
        "[ok](/sources/2026-11-03-81364278) [bad](/sources/nope) [old](/doks/sessions/x/y.md)",
    ]);
    let peek = f.run(&["peek", "/sources/2026-11-03-81364278"]);
    assert!(
        peek.starts_with("── sources/2026-11-03-81364278\n"),
        "{peek}"
    );
    assert!(peek.contains("title: A source"), "{peek}");

    let out = f.cmd(&["doctor", "--json"]);
    assert!(!out.status.success());
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    let msgs: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .filter(|i| i["kind"] == "broken-link")
        .map(|i| i["message"].as_str().unwrap())
        .collect();
    assert_eq!(msgs.len(), 2, "{msgs:?}");
    assert!(msgs[0].contains("/sources/nope"), "{msgs:?}");
    assert!(msgs[1].contains("/doks/sessions/x/y.md"), "{msgs:?}");
}
