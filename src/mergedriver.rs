//! The farm's git merge driver (yaks-4a53): merges three versions of one yak
//! file so the result is always a valid yak, never conflict markers.
//!
//! `merge=union` merges two concurrent notes on one yak without a conflict but
//! leaves two `updated:` lines (an invalid file); a text merge of the
//! frontmatter conflicts on any shared line. This driver merges the parsed yak
//! instead, field by field (see [`merge`]), and a scalar both sides changed
//! differently never fails the merge: ours is kept and the other side's value
//! is recorded in an attributed note, so the farm stays valid and a human
//! resolves at leisure.
//!
//! Registered per repository by [`ensure_registered`] (a `.gitattributes` rule
//! plus `merge.yaks.driver` config) and invoked by git as
//! `yaks merge-driver %O %A %B %P`, which is [`run`].

use crate::model::{Status, Task};
use crate::store::{self, NoteEntry};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

/// The actor of the note a merge conflict is recorded under.
const ACTOR: &str = "merge-driver";

/// The `.gitattributes` line routing yak files to the driver.
const ATTR_LINE: &str = ".yaks/**/*.md merge=yaks";

/// The result of merging three versions of one yak file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Merged {
    /// The merged file text: a valid yak, never conflict markers.
    pub text: String,
    /// The fields both sides changed differently (ours kept, the other side
    /// recorded in a `merge-driver` note appended to `text`). Empty for a clean
    /// merge.
    pub conflicts: Vec<String>,
}

/// What the three-way rule did with one value.
fn three<T: PartialEq + Clone>(base: &T, ours: &T, theirs: &T) -> (T, bool) {
    if ours == theirs || theirs == base {
        (ours.clone(), false)
    } else if ours == base {
        (theirs.clone(), false)
    } else {
        (ours.clone(), true)
    }
}

/// Items of `ours` and `theirs`, minus what either side removed from `base`:
/// `base` ∪ each side's additions − each side's removals. Ours' order first.
fn merge_list(base: &[String], ours: &[String], theirs: &[String]) -> Vec<String> {
    let mut out: Vec<String> = ours
        .iter()
        .filter(|x| !(base.contains(x) && !theirs.contains(x)))
        .cloned()
        .collect();
    for x in theirs {
        if !ours.contains(x) && !base.contains(x) && !out.contains(x) {
            out.push(x.clone());
        }
    }
    out
}

/// A yak's unmodelled frontmatter lines grouped per key (`key: value` plus its
/// block-list `- ` items), in file order.
fn extra_groups(extra: &[String]) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for line in extra {
        let starts_key = !line.starts_with([' ', '\t', '-']) && line.contains(':');
        match out.last_mut() {
            Some((_, lines)) if !starts_key => lines.push(line.clone()),
            _ => {
                let key = line.split(':').next().unwrap_or("").trim().to_string();
                out.push((key, vec![line.clone()]));
            }
        }
    }
    out
}

fn show_opt(v: &Option<String>) -> String {
    v.clone().unwrap_or_else(|| "(none)".to_string())
}

/// Merge `ours` and `theirs`, two edits of `base`, into one valid yak file.
///
/// - Notes: the union of both sides' entries, deduped by exact entry, ordered
///   by timestamp (stable: ours first on a tie); an entry in `base` that either
///   side dropped stays dropped.
/// - Description: the side that changed it; both differently: ours kept.
/// - `updated`: the later of the two. `labels`, `depends_on`: `base` plus each
///   side's additions minus each side's removals.
/// - Every other field (title, status, type, priority, parent, source, needs,
///   verify, created, unknown keys): the side that changed it wins; both
///   differently: ours kept.
///
/// Each conflict is one line of a single `merge-driver` note appended to the
/// merged yak, so nothing is lost silently. An empty `base` (both sides added
/// the file) is the empty yak. When any input is not a yak the result is
/// `ours` unchanged (what [`run`] reports as a git conflict instead).
#[allow(dead_code)] // the pure core; `run` is what the command calls
pub fn merge(base: &str, ours: &str, theirs: &str) -> Merged {
    try_merge(base, ours, theirs).unwrap_or_else(|| Merged {
        text: ours.to_string(),
        conflicts: Vec::new(),
    })
}

fn parse(text: &str) -> Option<Task> {
    store::parse_task(text, Status::Hairy)
}

fn try_merge(base: &str, ours: &str, theirs: &str) -> Option<Merged> {
    let o = parse(ours)?;
    let t = parse(theirs)?;
    // Both sides adding the file hands us an empty base: nothing in common.
    let b = if base.trim().is_empty() {
        Task {
            id: o.id.clone(),
            title: String::new(),
            kind: String::new(),
            priority: 0,
            status: Status::Hairy,
            created: None,
            updated: None,
            parent: None,
            labels: Vec::new(),
            depends_on: Vec::new(),
            source: None,
            needs: None,
            verify: None,
            extra: Vec::new(),
            body: String::new(),
        }
    } else {
        parse(base)?
    };

    // (field, ours, theirs) for every scalar both sides changed differently.
    let mut clashes: Vec<(String, String, String)> = Vec::new();
    macro_rules! scalar {
        ($name:literal, $f:ident, $show:expr) => {{
            let (v, c) = three(&b.$f, &o.$f, &t.$f);
            if c {
                clashes.push(($name.to_string(), $show(&o.$f), $show(&t.$f)));
            }
            v
        }};
    }
    let id = scalar!("id", id, |v: &String| v.clone());
    let title = scalar!("title", title, |v: &String| v.clone());
    let kind = scalar!("type", kind, |v: &String| v.clone());
    let priority = scalar!("priority", priority, |v: &u8| v.to_string());
    let status = scalar!("status", status, |v: &Status| v.as_str().to_string());
    let created = scalar!("created", created, show_opt);
    let parent = scalar!("parent", parent, show_opt);
    let source = scalar!("source", source, show_opt);
    let needs = scalar!("needs", needs, show_opt);
    let verify = scalar!("verify", verify, show_opt);

    // Unknown frontmatter keys, one scalar per key (a block list is one value).
    let (bg, og, tg) = (
        extra_groups(&b.extra),
        extra_groups(&o.extra),
        extra_groups(&t.extra),
    );
    let find = |g: &[(String, Vec<String>)], k: &str| -> Option<Vec<String>> {
        g.iter().find(|(key, _)| key == k).map(|(_, l)| l.clone())
    };
    let mut keys: Vec<&String> = og.iter().map(|(k, _)| k).collect();
    for (k, _) in &tg {
        if !keys.contains(&k) {
            keys.push(k);
        }
    }
    let mut extra: Vec<String> = Vec::new();
    for k in keys {
        let (ov, tv) = (find(&og, k), find(&tg, k));
        let (v, c) = three(&find(&bg, k), &ov, &tv);
        if c {
            let show = |v: &Option<Vec<String>>| match v {
                Some(l) => l
                    .iter()
                    .map(|s| s.trim())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .splitn(2, ':')
                    .nth(1)
                    .unwrap_or("")
                    .trim()
                    .to_string(),
                None => "(none)".to_string(),
            };
            clashes.push((k.clone(), show(&ov), show(&tv)));
        }
        extra.extend(v.into_iter().flatten());
    }

    let updated = std::cmp::max(o.updated.clone(), t.updated.clone());

    // Body: description by the same three-way rule, then the notes.
    let (bd, od, td) = (
        store::description(&b.body),
        store::description(&o.body),
        store::description(&t.body),
    );
    let (desc, desc_clash) = three(&bd, &od, &td);

    let (bn, on, tn) = (
        store::parse_notes(&b.body),
        store::parse_notes(&o.body),
        store::parse_notes(&t.body),
    );
    let mut notes: Vec<NoteEntry> = on.clone();
    for n in &tn {
        if !on.contains(n) {
            notes.push(n.clone());
        }
    }
    notes.retain(|n| !(bn.contains(n) && !(on.contains(n) && tn.contains(n))));
    notes.sort_by(|a, b| a.ts.cmp(&b.ts)); // stable: ours first on a tie

    let mut conflicts: Vec<String> = clashes.iter().map(|(f, _, _)| f.clone()).collect();
    if desc_clash {
        conflicts.push("description".to_string());
    }
    let mut updated = updated;
    if !conflicts.is_empty() {
        let mut lines: Vec<String> = clashes
            .iter()
            .map(|(f, ov, tv)| {
                format!("merge conflict: {f} was {ov} here, {tv} on the other side; kept {ov}")
            })
            .collect();
        if desc_clash {
            let quoted: Vec<String> = td.lines().map(|l| format!("> {l}")).collect();
            lines.push(format!(
                "merge conflict: description was changed here and on the other side; \
                 kept this one. The other side's:\n\n{}",
                quoted.join("\n")
            ));
        }
        let ts = store::now_iso();
        updated = std::cmp::max(updated, Some(ts.clone()));
        notes.push(NoteEntry {
            ts,
            actor: Some(ACTOR.to_string()),
            text: lines.join("\n"),
        });
    }

    let mut body = desc;
    for n in &notes {
        body = store::append_note(&body, &n.ts, n.actor.as_deref(), &n.text);
    }
    let merged = Task {
        id,
        title,
        kind,
        priority,
        status,
        created,
        updated,
        parent,
        labels: merge_list(&b.labels, &o.labels, &t.labels),
        depends_on: merge_list(&b.depends_on, &o.depends_on, &t.depends_on),
        source,
        needs,
        verify,
        extra,
        body,
    };
    Some(Merged {
        text: store::write::render(&merged),
        conflicts,
    })
}

/// What `yaks merge-driver %O %A %B %P` does: merge the three files git hands
/// over and write the result to `ours` (`%A`). Returns the process exit code:
/// 0 for a merged yak, conflicts included (they are recorded in the file); 1
/// when any of the three is not a yak, leaving `ours` untouched so git reports
/// a conflict for that path. An I/O error is an `Err`.
pub fn run(base: &Path, ours: &Path, theirs: &Path, path: &str) -> Result<i32> {
    let read = |p: &Path| -> Result<String> {
        std::fs::read_to_string(p).with_context(|| format!("reading {}", p.display()))
    };
    let (b, o, t) = (read(base)?, read(ours)?, read(theirs)?);
    let Some(m) = try_merge(&b, &o, &t) else {
        eprintln!("yaks merge-driver: {path}: not a yak file, left for a manual merge");
        return Ok(1);
    };
    std::fs::write(ours, &m.text).with_context(|| format!("writing {}", ours.display()))?;
    if !m.conflicts.is_empty() {
        eprintln!(
            "yaks merge-driver: {path}: both sides changed {}; kept this side, \
             the other recorded in a note",
            m.conflicts.join(", ")
        );
    }
    Ok(0)
}

fn git_config(worktree: &Path, key: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .args(["config", "--get", key])
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn set_git_config(worktree: &Path, key: &str, value: &str) -> Result<()> {
    if git_config(worktree, key).as_deref() == Some(value) {
        return Ok(());
    }
    let out = Command::new("git")
        .arg("-C")
        .arg(worktree)
        .args(["config", key, value])
        .output()
        .context("running git config")?;
    if !out.status.success() {
        bail!(
            "git config {key}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(())
}

/// The driver command line: plain `yaks`, resolved on `PATH` when git runs
/// it, when the `yaks` there is this very binary (the normal case: an
/// installed yaks). Pinning an absolute path would break every later merge
/// once that binary moves (a reinstall elsewhere, a deleted build tree), and
/// git then reports each yak file as a conflict. Only a binary run from
/// outside `PATH` (a dev build, a test) pins its own path, shell-quoted when
/// needed.
fn driver_command() -> Result<String> {
    let exe = std::env::current_exe().context("locating the yaks executable")?;
    if on_path_as_yaks(&exe) {
        return Ok("yaks merge-driver %O %A %B %P".to_string());
    }
    let exe = exe.to_string_lossy();
    let plain = exe
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || "/._-+:\\".contains(c));
    let exe = if plain {
        exe.to_string()
    } else {
        format!("'{}'", exe.replace('\'', r"'\''"))
    };
    Ok(format!("{exe} merge-driver %O %A %B %P"))
}

/// Make sure the farm worktree at `worktree` (the git top-level holding
/// `.yaks/`) merges yak files with the yaks driver: the `.gitattributes` rule
/// on the farm branch and the repository's `merge.yaks.driver` config.
/// Idempotent; changes nothing that is already in place.
pub fn ensure_registered(worktree: &Path) -> Result<()> {
    let attrs = worktree.join(".gitattributes");
    let text = match std::fs::read_to_string(&attrs) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e).with_context(|| format!("reading {}", attrs.display())),
    };
    if !text.lines().any(|l| l.trim() == ATTR_LINE) {
        let sep = if text.is_empty() || text.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        store::write::atomic_write(&attrs, &format!("{text}{sep}{ATTR_LINE}\n"))?;
    }
    set_git_config(worktree, "merge.yaks.name", "yaks yak-file merge")?;
    set_git_config(worktree, "merge.yaks.driver", &driver_command()?)?;
    Ok(())
}

/// Whether `yaks` on `PATH` is `exe` (same file, after resolving links).
fn on_path_as_yaks(exe: &Path) -> bool {
    let Ok(me) = std::fs::canonicalize(exe) else {
        return false;
    };
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path)
        .map(|d| d.join(if cfg!(windows) { "yaks.exe" } else { "yaks" }))
        .find(|p| p.is_file())
        .and_then(|p| std::fs::canonicalize(p).ok())
        .is_some_and(|p| p == me)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn yak(status: &str, extra_fm: &str, body: &str) -> String {
        format!(
            "---\nid: t-1111\ntitle: A yak\nstatus: {status}\ntype: task\npriority: 3\n\
             created: 2026-01-01T00:00:00Z\nupdated: 2026-01-01T00:00:00Z\n{extra_fm}---\n\n{body}\n"
        )
    }

    fn with_updated(text: &str, ts: &str) -> String {
        text.replace("updated: 2026-01-01T00:00:00Z", &format!("updated: {ts}"))
    }

    fn note(ts: &str, who: &str, text: &str) -> String {
        format!("\n\n---\n\u{25b8} {ts} [{who}]\n\n{text}")
    }

    fn parsed(m: &Merged) -> Task {
        assert!(!m.text.contains("<<<<<<<") && !m.text.contains(">>>>>>>"));
        assert_eq!(m.text.matches("\nupdated:").count(), 1, "{}", m.text);
        parse(&m.text).expect("merged text parses as a yak")
    }

    #[test]
    fn two_different_notes_are_both_kept_in_time_order() {
        let base = yak("hairy", "", "Desc");
        let a = with_updated(
            &yak(
                "hairy",
                "",
                &format!("Desc{}", note("2026-01-02T00:00:00Z", "a", "from a")),
            ),
            "2026-01-02T00:00:00Z",
        );
        let b = with_updated(
            &yak(
                "hairy",
                "",
                &format!("Desc{}", note("2026-01-01T12:00:00Z", "b", "from b")),
            ),
            "2026-01-01T12:00:00Z",
        );
        let m = merge(&base, &a, &b);
        assert!(m.conflicts.is_empty());
        let t = parsed(&m);
        assert_eq!(t.updated.as_deref(), Some("2026-01-02T00:00:00Z"));
        let notes = store::parse_notes(&t.body);
        assert_eq!(
            notes.iter().map(|n| n.text.as_str()).collect::<Vec<_>>(),
            ["from b", "from a"]
        );
        assert_eq!(store::description(&t.body), "Desc");
    }

    #[test]
    fn a_note_both_sides_hold_is_kept_once_and_base_notes_stay() {
        let shared = note("2026-01-01T01:00:00Z", "x", "shared");
        let base = yak("hairy", "", &format!("Desc{shared}"));
        let a = yak(
            "hairy",
            "",
            &format!("Desc{shared}{}", note("2026-01-02T00:00:00Z", "a", "n")),
        );
        let b = yak(
            "hairy",
            "",
            &format!("Desc{shared}{}", note("2026-01-02T00:00:00Z", "a", "n")),
        );
        let t = parsed(&merge(&base, &a, &b));
        assert_eq!(store::parse_notes(&t.body).len(), 2);
    }

    #[test]
    fn one_side_changing_a_scalar_wins_without_a_conflict() {
        let base = yak("hairy", "", "Desc");
        let a = yak("shaving", "", "Desc");
        let m = merge(&base, &a, &base);
        assert!(m.conflicts.is_empty());
        assert_eq!(parsed(&m).status, Status::Shaving);
        let m = merge(&base, &base, &a);
        assert_eq!(parsed(&m).status, Status::Shaving);
    }

    #[test]
    fn a_scalar_changed_both_ways_keeps_ours_and_records_a_note() {
        let base = yak("hairy", "", "Desc");
        let a = yak("shaving", "", "Desc");
        let b = yak("shorn", "", "Desc");
        let m = merge(&base, &a, &b);
        assert_eq!(m.conflicts, ["status"]);
        let t = parsed(&m);
        assert_eq!(t.status, Status::Shaving);
        let notes = store::parse_notes(&t.body);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].actor.as_deref(), Some("merge-driver"));
        assert_eq!(
            notes[0].text,
            "merge conflict: status was shaving here, shorn on the other side; kept shaving"
        );
    }

    #[test]
    fn several_conflicting_fields_are_one_note_one_line_each() {
        let base = yak("hairy", "", "Desc");
        let a = yak("shaving", "", "Desc").replace("title: A yak", "title: Ours");
        let b = yak("shorn", "", "Desc").replace("title: A yak", "title: Theirs");
        let m = merge(&base, &a, &b);
        assert_eq!(m.conflicts, ["title", "status"]);
        let t = parsed(&m);
        let notes = store::parse_notes(&t.body);
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].text.lines().count(), 2, "{}", notes[0].text);
    }

    #[test]
    fn labels_add_on_one_side_and_remove_on_the_other() {
        let base = yak("hairy", "labels: [x, y]\n", "Desc");
        let a = yak("hairy", "labels: [x, y, z]\n", "Desc"); // adds z
        let b = yak("hairy", "labels: [y]\ndepends_on: [t-2222]\n", "Desc"); // drops x
        let t = parsed(&merge(&base, &a, &b));
        assert_eq!(t.labels, ["y", "z"]);
        assert_eq!(t.depends_on, ["t-2222"]);
    }

    #[test]
    fn a_description_changed_on_one_side_wins() {
        let base = yak("hairy", "", "Old");
        let a = yak("hairy", "", "New");
        let m = merge(&base, &base, &a);
        assert!(m.conflicts.is_empty());
        assert_eq!(store::description(&parsed(&m).body), "New");
        let m = merge(&base, &a, &base);
        assert_eq!(store::description(&parsed(&m).body), "New");
    }

    #[test]
    fn a_description_changed_both_ways_keeps_ours_and_quotes_theirs() {
        let base = yak("hairy", "", "Old");
        let m = merge(&base, &yak("hairy", "", "Mine"), &yak("hairy", "", "Yours"));
        assert_eq!(m.conflicts, ["description"]);
        let t = parsed(&m);
        assert_eq!(store::description(&t.body), "Mine");
        assert!(store::parse_notes(&t.body)[0].text.contains("> Yours"));
    }

    #[test]
    fn unknown_frontmatter_keys_merge_per_key() {
        let base = yak("hairy", "foo: 1\n", "Desc");
        let a = yak("hairy", "foo: 2\nbar: 5\n", "Desc");
        let b = yak("hairy", "foo: 1\nbaz: 7\n", "Desc");
        let m = merge(&base, &a, &b);
        assert!(m.conflicts.is_empty());
        let t = parsed(&m);
        assert_eq!(t.extra, ["foo: 2", "bar: 5", "baz: 7"]);
    }

    #[test]
    fn an_empty_base_merges_as_both_sides_adding_the_file() {
        let a = yak("hairy", "labels: [a]\n", "Desc");
        let b = yak("hairy", "labels: [b]\n", "Desc");
        let m = merge("", &a, &b);
        assert!(m.conflicts.is_empty(), "{:?}", m.conflicts);
        assert_eq!(parsed(&m).labels, ["a", "b"]);
    }

    #[test]
    fn a_non_yak_input_falls_back_to_ours_and_reports_none_merged() {
        let base = "prefix: t\n";
        assert!(try_merge(base, "prefix: t\n", "prefix: u\n").is_none());
        let yakk = yak("hairy", "", "Desc");
        assert!(try_merge(&yakk, &yakk, "prefix: u\n").is_none());
        assert_eq!(merge(base, "ours\n", "theirs\n").text, "ours\n");
    }

    #[test]
    fn run_writes_ours_and_exits_one_for_a_non_yak() {
        let dir = std::env::temp_dir().join(format!("yaks-md-run-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (b, o, t) = (dir.join("b"), dir.join("o"), dir.join("t"));
        std::fs::write(&b, "x: 1\n").unwrap();
        std::fs::write(&o, "x: 2\n").unwrap();
        std::fs::write(&t, "x: 3\n").unwrap();
        assert_eq!(run(&b, &o, &t, ".yaks/config.yaml").unwrap(), 1);
        assert_eq!(std::fs::read_to_string(&o).unwrap(), "x: 2\n");
        let base = yak("hairy", "", "Desc");
        std::fs::write(&b, &base).unwrap();
        std::fs::write(&o, yak("shaving", "", "Desc")).unwrap();
        std::fs::write(&t, yak("shorn", "", "Desc")).unwrap();
        assert_eq!(run(&b, &o, &t, ".yaks/y/yaks/t-1111.md").unwrap(), 0);
        assert!(
            std::fs::read_to_string(&o)
                .unwrap()
                .contains("merge conflict")
        );
    }

    #[test]
    fn ensure_registered_writes_the_rule_and_config_once() {
        let dir = std::env::temp_dir().join(format!("yaks-md-reg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .arg("-C")
                .arg(&dir)
                .args(args)
                .output()
                .unwrap();
            assert!(out.status.success(), "{args:?}");
            String::from_utf8(out.stdout).unwrap()
        };
        git(&["init", "-q"]);
        std::fs::write(dir.join(".gitattributes"), "*.png binary").unwrap(); // no newline
        ensure_registered(&dir).unwrap();
        ensure_registered(&dir).unwrap();
        let attrs = std::fs::read_to_string(dir.join(".gitattributes")).unwrap();
        assert_eq!(attrs, "*.png binary\n.yaks/**/*.md merge=yaks\n");
        assert_eq!(
            git(&["config", "merge.yaks.name"]).trim(),
            "yaks yak-file merge"
        );
        let driver = git(&["config", "merge.yaks.driver"]);
        let exe = std::env::current_exe().unwrap();
        assert_eq!(
            driver.trim(),
            format!("{} merge-driver %O %A %B %P", exe.display())
        );
        assert_eq!(
            git(&["config", "--get-all", "merge.yaks.driver"])
                .lines()
                .count(),
            1
        );
    }
}
