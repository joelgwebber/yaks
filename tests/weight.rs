//! Checkout weight (yaks-2b68) through the real binary: `yaks weight` counts
//! the farm branch's commits and tree by kind, and `yaks lfs` routes new
//! binary attachments to Git LFS (skipped when git-lfs is not installed).

mod common;
use assert_cmd::Command;
use std::path::{Path, PathBuf};

struct Farm {
    dir: PathBuf,
}

impl Farm {
    fn new(tag: &str) -> Farm {
        let dir = std::env::temp_dir().join(format!("yaks-weight-{tag}-{}", std::process::id()));
        common::cleanup(&dir);
        common::git_init(&dir);
        let f = Farm { dir };
        f.run(&["init", "--no-skills"]);
        f
    }

    fn run(&self, args: &[&str]) -> String {
        let o = Command::cargo_bin("yaks")
            .unwrap()
            .current_dir(&self.dir)
            .env("XDG_DATA_HOME", common::data_home(&self.dir))
            .env("YAKS_SKILLS_AUTOSYNC", "0")
            .env_remove("YAKS_NO_AUTOCOMMIT")
            .env("YAKS_ACTOR", "tester")
            .args(args)
            .output()
            .unwrap();
        assert!(
            o.status.success(),
            "{args:?}: {}",
            String::from_utf8_lossy(&o.stderr)
        );
        String::from_utf8(o.stdout).unwrap()
    }

    fn wt(&self) -> PathBuf {
        PathBuf::from(self.run(&["path"]).trim())
            .parent()
            .unwrap()
            .to_path_buf()
    }

    fn attach(&self, id: &str, name: &str, bytes: usize) {
        let file = self.dir.join(name);
        // Not compressible, so sizes are real.
        let data: Vec<u8> = (0..bytes).map(|i| (i * 7919 % 251) as u8).collect();
        std::fs::write(&file, data).unwrap();
        self.run(&["attach", id, file.to_str().unwrap()]);
    }
}

impl Drop for Farm {
    fn drop(&mut self) {
        common::cleanup(&self.dir);
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let o = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&o.stdout).to_string()
}

#[test]
fn weight_counts_commits_and_the_tree_by_kind() {
    let f = Farm::new("report");
    let v: serde_json::Value =
        serde_json::from_str(&f.run(&["create", "a yak", "--json"])).unwrap();
    let id = v["id"].as_str().unwrap().to_string();
    f.attach(&id, "shot.png", 4000);
    f.run(&["page", "new", "wiki/topics/x.md", "--title", "X"]);
    let r: serde_json::Value = serde_json::from_str(&f.run(&["weight", "--json"])).unwrap();
    assert_eq!(r["branch"], "yaks");
    assert!(r["commits"].as_u64().unwrap() >= 4, "{r}");
    assert_eq!(r["tree"]["yaks"]["files"], 1);
    assert_eq!(r["tree"]["attachments"]["files"], 1);
    assert_eq!(r["tree"]["attachments"]["bytes"], 4000);
    assert_eq!(r["tree"]["pages"]["files"], 1);
    assert_eq!(r["largest"][0]["bytes"], 4000);
    assert!(
        r["largest"][0]["path"]
            .as_str()
            .unwrap()
            .ends_with("/shot.png")
    );
    assert_eq!(r["lfs"], false);
    assert_eq!(r["binary_in_git"], 4000);
    // The farm branch is an orphan: a clone fetches all of it.
    assert_eq!(r["farm_only_bytes"], r["history_bytes"]);
}

#[test]
fn lfs_stores_new_binary_attachments_as_pointers() {
    let have_lfs = std::process::Command::new("git")
        .args(["lfs", "version"])
        .output()
        .is_ok_and(|o| o.status.success());
    if !have_lfs {
        eprintln!("skipped: git-lfs is not installed");
        return;
    }
    let f = Farm::new("lfs");
    let v: serde_json::Value =
        serde_json::from_str(&f.run(&["create", "a yak", "--json"])).unwrap();
    let id = v["id"].as_str().unwrap().to_string();
    f.attach(&id, "before.png", 3000);
    assert!(f.run(&["lfs"]).contains("now go to Git LFS"));
    assert_eq!(f.run(&["lfs"]).trim(), "lfs: already on");
    f.attach(&id, "after.PNG", 3000);
    let wt = f.wt();
    let tracked = git(&wt, &["lfs", "ls-files", "--name-only"]);
    assert!(tracked.contains("after.PNG"), "{tracked}");
    assert!(tracked.contains("before.png"), "converted: {tracked}");
    // The committed blob is a pointer; the checkout holds the bytes.
    let path = format!(".yaks/yak/yaks/{id}/after.PNG");
    let blob = git(&wt, &["show", &format!("HEAD:{path}")]);
    assert!(
        blob.starts_with("version https://git-lfs.github.com/spec/v1"),
        "{blob}"
    );
    assert_eq!(std::fs::read(wt.join(&path)).unwrap().len(), 3000);
    assert!(git(&wt, &["status", "--porcelain"]).trim().is_empty());
    let r: serde_json::Value = serde_json::from_str(&f.run(&["weight", "--json"])).unwrap();
    assert_eq!(r["lfs"], true);
    assert_eq!(r["binary_in_git"], 0);
}
