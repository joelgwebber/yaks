//! The farm's git merge driver (yaks-4a53, yaks-4c8c): merges three versions
//! of one farm document so its fields always stay well formed.
//!
//! - A yak (a file under `<herd>/yaks/`) merges field by field ([`merge`]).
//! - Any other farm markdown file (a dok, a source's text) merges its
//!   frontmatter by the same field rule and its body as text
//!   ([`merge_document`]).
//! - Prose (a yak's description, a dok body) merges with `git merge-file`;
//!   when both sides changed the same lines, the conflict markers stay in the
//!   prose and the merge still succeeds, so the farm stays readable and a
//!   human or agent resolves the text afterwards (`yaks conflicts` lists such
//!   files, and doctor reports them).
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

use crate::fields::{Fields, Value};
use crate::model::{Status, Task};
use crate::store::{self, NoteEntry};
use anyhow::{Context, Result, bail};
use std::path::Path;
use std::process::Command;

/// The actor of the note a merge conflict is recorded under.
const ACTOR: &str = "merge-driver";

/// The `.gitattributes` line routing farm markdown files to the driver.
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
fn merge_list<T: PartialEq + Clone>(base: &[T], ours: &[T], theirs: &[T]) -> Vec<T> {
    let mut out: Vec<T> = ours
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

fn show_opt(v: &Option<String>) -> String {
    v.clone().unwrap_or_else(|| "(none)".to_string())
}

/// One frontmatter key both sides changed differently: ours was kept.
#[derive(Debug, Clone)]
struct FieldClash {
    key: String,
    /// Each side's raw lines for the key (`None`: that side removed it).
    ours: Option<Vec<String>>,
    theirs: Option<Vec<String>>,
}

/// A key's raw lines shown as its value on one line, for a message.
fn show_raw(v: &Option<Vec<String>>) -> String {
    match v {
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
    }
}

/// The list a key's raw lines hold, when they hold one.
fn as_list(key: &str, raw: &Option<Vec<String>>) -> Option<Vec<Value>> {
    match Fields::from_lines(raw.as_ref()?).get(key)? {
        Value::List(items) => Some(items),
        _ => None,
    }
}

/// Merge frontmatter fields, the one rule for a yak's extra fields and a
/// dok's frontmatter. Starts from ours (so its comments and order stay) and,
/// key by key: the side that changed a key wins (a removal included); a list
/// both sides changed merges as a set (`base` plus each side's additions minus
/// each side's removals); any other value both sides changed differently keeps
/// ours and is returned as a clash. Keys only theirs added go at the end.
/// `skip` keys are left as ours has them (the caller merges them).
fn merge_fields(b: &Fields, o: &Fields, t: &Fields, skip: &[&str]) -> (Fields, Vec<FieldClash>) {
    let raw = |f: &Fields, k: &str| f.raw(k).map(<[String]>::to_vec);
    let mut keys: Vec<String> = o.keys().map(String::from).collect();
    for k in t.keys() {
        if !keys.iter().any(|x| x == k) {
            keys.push(k.to_string());
        }
    }
    // A key both sides removed is gone from both; nothing to visit.
    let mut out = o.clone();
    let mut clashes = Vec::new();
    for k in keys.iter().filter(|k| !skip.contains(&k.as_str())) {
        let (bv, ov, tv) = (raw(b, k), raw(o, k), raw(t, k));
        let (mut v, clash) = three(&bv, &ov, &tv);
        if clash {
            let (bl, ol, tl) = (as_list(k, &bv), as_list(k, &ov), as_list(k, &tv));
            match (ol, tl) {
                (Some(ol), Some(tl)) if bv.is_none() || bl.is_some() => {
                    let merged = merge_list(&bl.unwrap_or_default(), &ol, &tl);
                    let flow = ov.as_ref().is_some_and(|l| l.len() == 1);
                    let mut f = Fields::default();
                    if flow {
                        f.set_flow(k, &Value::List(merged));
                    } else {
                        f.set(k, &Value::List(merged));
                    }
                    v = f.raw(k).map(<[String]>::to_vec);
                }
                _ => clashes.push(FieldClash {
                    key: k.clone(),
                    ours: ov.clone(),
                    theirs: tv.clone(),
                }),
            }
        }
        match v {
            Some(lines) => out.set_raw(k, lines),
            None => {
                out.remove(k);
            }
        }
    }
    (out, clashes)
}

/// Labels on the conflict markers [`merge_text`] leaves.
const HERE: &str = "here";
const THERE: &str = "other side";

/// Three-way merge of prose with `git merge-file`. Returns the merged text and
/// whether it holds conflict markers (both sides changed the same lines; each
/// side's version is kept between `<<<<<<< here` and `>>>>>>> other side`).
/// `None` when git cannot merge them (binary content, git missing).
pub fn merge_text(base: &str, ours: &str, theirs: &str) -> Option<(String, bool)> {
    if ours == theirs || theirs == base {
        return Some((ours.to_string(), false));
    }
    if ours == base {
        return Some((theirs.to_string(), false));
    }
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("yaks-merge-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let write = |name: &str, text: &str| -> Option<std::path::PathBuf> {
        let p = dir.join(name);
        std::fs::write(&p, text).ok()?;
        Some(p)
    };
    let result = (|| {
        let (o, b, t) = (
            write("ours", ours)?,
            write("base", base)?,
            write("theirs", theirs)?,
        );
        let out = Command::new("git")
            .args(["merge-file", "-p", "-L", HERE, "-L", "base", "-L", THERE])
            .args([&o, &b, &t])
            .output()
            .ok()?;
        // Exit status: 0 clean, the number of conflicts (capped at 127), or
        // negative (shown as 128 and up) on an error.
        let code = out.status.code()?;
        if !(0..128).contains(&code) {
            return None;
        }
        Some((String::from_utf8(out.stdout).ok()?, code > 0))
    })();
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Merge two edits of a description (text with no trailing newline).
fn merge_description(base: &str, ours: &str, theirs: &str) -> Option<(String, bool)> {
    let nl = |s: &str| {
        if s.is_empty() {
            String::new()
        } else {
            format!("{s}\n")
        }
    };
    let (text, conflict) = merge_text(&nl(base), &nl(ours), &nl(theirs))?;
    Some((text.trim_end().to_string(), conflict))
}

/// Merge `ours` and `theirs`, two edits of `base`, into one valid yak file.
///
/// - Notes: the union of both sides' entries, deduped by exact entry, ordered
///   by timestamp (stable: ours first on a tie); an entry in `base` that either
///   side dropped stays dropped.
/// - Description: merged as text ([`merge_text`]); lines both sides changed
///   are kept from each between conflict markers.
/// - `updated`: the later of the two. `tags`, `depends_on`, `prs`: `base` plus each
///   side's additions minus each side's removals.
/// - Every other field (title, status, type, priority, parent, source, needs,
///   verify, created, other keys): the side that changed it wins; both
///   differently: ours kept. A list in another key merges like `tags`.
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
            tags: Vec::new(),
            depends_on: Vec::new(),
            source: None,
            prs: Vec::new(),
            needs: None,
            verify: None,
            extra: Default::default(),
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

    // The other frontmatter keys, by the shared field rule.
    let (extra, extra_clashes) = merge_fields(&b.extra, &o.extra, &t.extra, &[]);
    for c in &extra_clashes {
        clashes.push((c.key.clone(), show_raw(&c.ours), show_raw(&c.theirs)));
    }

    let updated = std::cmp::max(o.updated.clone(), t.updated.clone());

    // Body: description by the same three-way rule, then the notes.
    let (bd, od, td) = (
        store::description(&b.body),
        store::description(&o.body),
        store::description(&t.body),
    );
    // Merged as text; if git cannot, ours is kept and theirs quoted in the note.
    let (desc, desc_markers, desc_quote) = match merge_description(&bd, &od, &td) {
        Some((d, markers)) => (d, markers, false),
        None => {
            let (d, clash) = three(&bd, &od, &td);
            (d, false, clash)
        }
    };
    let desc_clash = desc_markers || desc_quote;

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
        if desc_markers {
            lines.push(
                "merge conflict: description was changed here and on the other side; \
                 both versions are kept between conflict markers: edit it to resolve"
                    .to_string(),
            );
        }
        if desc_quote {
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
            meta: Vec::new(),
            text: lines.join("\n"),
        });
    }

    let mut body = desc;
    for n in &notes {
        body = store::append_entry(&body, n);
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
        tags: merge_list(&b.tags, &o.tags, &t.tags),
        depends_on: merge_list(&b.depends_on, &o.depends_on, &t.depends_on),
        source,
        prs: merge_list(&b.prs, &o.prs, &t.prs),
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

/// The frontmatter key a dok records field clashes under: a list of
/// `{field, theirs}`, the other side's value of a field both sides set
/// differently (ours was kept). Doctor and `yaks conflicts` report it;
/// remove an entry once settled.
pub const DOK_CONFLICTS_KEY: &str = "merge_conflicts";

/// Merge three versions of a farm document that is not a yak: a dok, a
/// source's text, any markdown file. With frontmatter on both sides, fields
/// merge by the shared rule ([`merge_fields`]); a value both sides set
/// differently keeps ours and adds `{field, theirs}` to `merge_conflicts:`.
/// The body (or the whole file, without frontmatter) merges as text, keeping
/// conflict markers where both sides changed the same lines. `conflicts`
/// names the clashing fields, and `body` when markers were left. `None` when
/// the text cannot be merged (binary content).
pub fn merge_document(base: &str, ours: &str, theirs: &str) -> Option<Merged> {
    use crate::doks::Dok;
    let (o, t) = (Dok::parse("", ours), Dok::parse("", theirs));
    if !(o.has_frontmatter && t.has_frontmatter) {
        let (text, markers) = merge_text(base, ours, theirs)?;
        let conflicts = if markers {
            vec!["body".to_string()]
        } else {
            Vec::new()
        };
        return Some(Merged { text, conflicts });
    }
    let b = if base.trim().is_empty() {
        Dok::parse("", "---\n---\n")
    } else {
        Dok::parse("", base)
    };
    let (mut fields, clashes) = merge_fields(&b.fields, &o.fields, &t.fields, &[DOK_CONFLICTS_KEY]);

    // Clash records: both sides' existing ones, then the new ones.
    let records = |f: &Fields| -> Vec<Value> {
        f.get(DOK_CONFLICTS_KEY)
            .map(|v| v.as_list().into_iter().cloned().collect())
            .unwrap_or_default()
    };
    let mut list = merge_list(
        &records(&b.fields),
        &records(&o.fields),
        &records(&t.fields),
    );
    for c in &clashes {
        let theirs = match &c.theirs {
            Some(lines) => Fields::from_lines(lines)
                .get(&c.key)
                .unwrap_or_else(|| Value::Str(show_raw(&c.theirs))),
            None => Value::Plain("(none)".to_string()),
        };
        let rec = Value::Map(vec![
            ("field".to_string(), Value::Plain(c.key.clone())),
            ("theirs".to_string(), theirs),
        ]);
        if !list.contains(&rec) {
            list.push(rec);
        }
    }
    let same_as = |f: &Fields| records(f) == list;
    if same_as(&o.fields) {
        // Ours' lines, untouched.
    } else if same_as(&t.fields) && !list.is_empty() {
        fields.set_raw(
            DOK_CONFLICTS_KEY,
            t.fields.raw(DOK_CONFLICTS_KEY).unwrap_or_default().to_vec(),
        );
    } else if list.is_empty() {
        fields.remove(DOK_CONFLICTS_KEY);
    } else {
        fields.set(DOK_CONFLICTS_KEY, &Value::List(list));
    }

    let (body, markers) = merge_text(&b.body, &o.body, &t.body)?;
    let dok = Dok {
        path: String::new(),
        fields,
        body,
        has_frontmatter: true,
    };
    let mut conflicts: Vec<String> = clashes.into_iter().map(|c| c.key).collect();
    if markers {
        conflicts.push("body".to_string());
    }
    Some(Merged {
        text: dok.render(),
        conflicts,
    })
}

/// Whether repository path `path` is where a yak lives (`…/<herd>/yaks/<id>.md`,
/// not inside doks/): such a file must merge as a yak or not at all.
fn is_yak_path(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').collect();
    parts.len() >= 2 && parts[parts.len() - 2] == "yaks" && !parts.contains(&crate::doks::DOKS_DIR)
}

/// What `yaks merge-driver %O %A %B %P` does: merge the three files git hands
/// over and write the result to `ours` (`%A`). Returns the process exit code:
/// 0 for a merged file, conflicts included (recorded in the file: a note, a
/// `merge_conflicts:` entry, or markers in the prose); 1 when it cannot be
/// merged (a file in a yaks directory that is not a yak, binary content),
/// leaving `ours` untouched so git reports a conflict for that path. An I/O
/// error is an `Err`.
pub fn run(base: &Path, ours: &Path, theirs: &Path, path: &str) -> Result<i32> {
    let read = |p: &Path| -> Result<Option<String>> {
        let bytes = std::fs::read(p).with_context(|| format!("reading {}", p.display()))?;
        Ok(String::from_utf8(bytes).ok())
    };
    let (Some(b), Some(o), Some(t)) = (read(base)?, read(ours)?, read(theirs)?) else {
        eprintln!("yaks merge-driver: {path}: not text, left for a manual merge");
        return Ok(1);
    };
    let yak = is_yak_path(path);
    let merged = if yak {
        try_merge(&b, &o, &t)
    } else {
        merge_document(&b, &o, &t)
    };
    let Some(m) = merged else {
        let what = if yak {
            "not a yak file"
        } else {
            "cannot be merged as text"
        };
        eprintln!("yaks merge-driver: {path}: {what}, left for a manual merge");
        return Ok(1);
    };
    std::fs::write(ours, &m.text).with_context(|| format!("writing {}", ours.display()))?;
    if !m.conflicts.is_empty() {
        let prose = ["description", "body"];
        let fields: Vec<&str> = m
            .conflicts
            .iter()
            .map(String::as_str)
            .filter(|c| !prose.contains(c))
            .collect();
        if !fields.is_empty() {
            let where_ = if yak { "a note" } else { "merge_conflicts:" };
            eprintln!(
                "yaks merge-driver: {path}: both sides changed {}; kept this side, \
                 the other recorded in {where_}",
                fields.join(", ")
            );
        }
        if m.conflicts.iter().any(|c| prose.contains(&c.as_str())) {
            eprintln!(
                "yaks merge-driver: {path}: both sides changed the same lines; \
                 conflict markers left in the text (`yaks conflicts` lists them)"
            );
        }
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
        let base = yak("hairy", "tags: [x, y]\n", "Desc");
        let a = yak("hairy", "tags: [x, y, z]\n", "Desc"); // adds z
        let b = yak("hairy", "tags: [y]\ndepends_on: [t-2222]\n", "Desc"); // drops x
        let t = parsed(&merge(&base, &a, &b));
        assert_eq!(t.tags, ["y", "z"]);
        assert_eq!(t.depends_on, ["t-2222"]);
    }

    #[test]
    fn prs_added_on_both_sides_both_survive() {
        let a_url = "https://github.com/o/r/pull/1";
        let b_url = "https://github.com/o/r/pull/2";
        let base = yak("hairy", "", "Desc");
        let a = yak("hairy", &format!("prs:\n- {a_url}\n"), "Desc");
        let b = yak("hairy", &format!("prs: [{b_url}]\n"), "Desc");
        let m = merge(&base, &a, &b);
        assert!(m.conflicts.is_empty());
        assert_eq!(parsed(&m).prs, [a_url, b_url]);

        // A removal on one side and an unrelated add on the other combine.
        let base = yak("hairy", &format!("prs: [{a_url}]\n"), "Desc");
        let a = yak("hairy", "", "Desc"); // drops pull/1
        let b = yak("hairy", &format!("prs: [{a_url}, {b_url}]\n"), "Desc");
        assert_eq!(parsed(&merge(&base, &a, &b)).prs, [b_url]);
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
    fn a_description_changed_both_ways_keeps_both_between_markers() {
        let base = yak("hairy", "", "Old");
        let m = merge(&base, &yak("hairy", "", "Mine"), &yak("hairy", "", "Yours"));
        assert_eq!(m.conflicts, ["description"]);
        let t = parse(&m.text).expect("still a yak");
        assert_eq!(
            store::description(&t.body),
            "<<<<<<< here\nMine\n=======\nYours\n>>>>>>> other side"
        );
        let notes = store::parse_notes(&t.body);
        assert_eq!(notes.len(), 1);
        assert!(
            notes[0].text.contains("conflict markers"),
            "{}",
            notes[0].text
        );
    }

    #[test]
    fn description_edits_to_different_lines_both_land() {
        let base = yak("hairy", "", "one\n\ntwo\n\nthree");
        let a = yak("hairy", "", "ONE\n\ntwo\n\nthree");
        let b = yak("hairy", "", "one\n\ntwo\n\nTHREE");
        let m = merge(&base, &a, &b);
        assert!(m.conflicts.is_empty(), "{:?}", m.conflicts);
        assert_eq!(store::description(&parsed(&m).body), "ONE\n\ntwo\n\nTHREE");
    }

    #[test]
    fn an_extra_list_field_both_sides_changed_merges_as_a_set() {
        let base = yak("hairy", "aliases: [a]\n", "Desc");
        let a = yak("hairy", "aliases: [a, b]\n", "Desc");
        let b = yak("hairy", "aliases: [c]\n", "Desc");
        let m = merge(&base, &a, &b);
        assert!(m.conflicts.is_empty(), "{:?}", m.conflicts);
        let t = parsed(&m);
        assert_eq!(t.extra.lines().collect::<Vec<_>>(), ["aliases: [b, c]"]);
    }

    fn dok(front: &str, body: &str) -> String {
        format!("---\ntype: topic\ntitle: Dok\n{front}---\n\n{body}\n")
    }

    #[test]
    fn a_dok_merges_fields_by_rule_and_body_as_text() {
        let base = dok("tags: [x]\nstatus: draft\n", "# P\n\none\n\ntwo\n");
        let a = dok("tags: [x, y]\nstatus: draft\n", "# P\n\nONE\n\ntwo\n");
        let b = dok("tags: [x, z]\nstatus: final\n", "# P\n\none\n\nTWO\n");
        let m = merge_document(&base, &a, &b).unwrap();
        assert!(m.conflicts.is_empty(), "{:?}", m.conflicts);
        assert_eq!(
            m.text,
            "---\ntype: topic\ntitle: Dok\ntags: [x, y, z]\nstatus: final\n---\n\n# P\n\nONE\n\nTWO\n\n"
        );
    }

    #[test]
    fn a_dok_field_both_sides_set_keeps_ours_and_records_theirs() {
        let base = dok("status: draft\n", "Body");
        let a = dok("status: review\n", "Body");
        let b = dok("status: final\n", "Body");
        let m = merge_document(&base, &a, &b).unwrap();
        assert_eq!(m.conflicts, ["status"]);
        assert_eq!(
            m.text,
            "---\ntype: topic\ntitle: Dok\nstatus: review\nmerge_conflicts:\n  - {field: status, theirs: final}\n---\n\nBody\n"
        );
        // Merging the same again (a rebase replaying it) adds nothing.
        let again = merge_document(&a, &m.text, &b).unwrap();
        assert_eq!(
            again.text.matches("field: status").count(),
            1,
            "{}",
            again.text
        );
    }

    #[test]
    fn a_dok_body_both_sides_changed_keeps_markers_and_valid_frontmatter() {
        let base = dok("", "line");
        let m = merge_document(&base, &dok("", "mine"), &dok("", "yours")).unwrap();
        assert_eq!(m.conflicts, ["body"]);
        let p = crate::doks::Dok::parse("doks/p.md", &m.text);
        assert_eq!(p.title(), "Dok");
        assert_eq!(
            p.body,
            "<<<<<<< here\nmine\n=======\nyours\n>>>>>>> other side\n"
        );
    }

    #[test]
    fn a_file_without_frontmatter_merges_as_text() {
        let m = merge_document("a\nb\nc\n", "A\nb\nc\n", "a\nb\nC\n").unwrap();
        assert_eq!(m.text, "A\nb\nC\n");
        assert!(m.conflicts.is_empty());
    }

    #[test]
    fn run_merges_a_dok_by_its_path() {
        let dir = std::env::temp_dir().join(format!("yaks-md-dok-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (b, o, t) = (dir.join("b"), dir.join("o"), dir.join("t"));
        std::fs::write(&b, dok("", "x")).unwrap();
        std::fs::write(&o, dok("", "mine")).unwrap();
        std::fs::write(&t, dok("", "yours")).unwrap();
        assert_eq!(run(&b, &o, &t, ".yaks/doks/topics/p.md").unwrap(), 0);
        assert!(
            std::fs::read_to_string(&o)
                .unwrap()
                .contains("<<<<<<< here")
        );
    }

    #[test]
    fn unknown_frontmatter_keys_merge_per_key() {
        let base = yak("hairy", "foo: 1\n", "Desc");
        let a = yak("hairy", "foo: 2\nbar: 5\n", "Desc");
        let b = yak("hairy", "foo: 1\nbaz: 7\n", "Desc");
        let m = merge(&base, &a, &b);
        assert!(m.conflicts.is_empty());
        let t = parsed(&m);
        assert_eq!(
            t.extra.lines().collect::<Vec<_>>(),
            ["foo: 2", "bar: 5", "baz: 7"]
        );
    }

    #[test]
    fn an_empty_base_merges_as_both_sides_adding_the_file() {
        let a = yak("hairy", "tags: [a]\n", "Desc");
        let b = yak("hairy", "tags: [b]\n", "Desc");
        let m = merge("", &a, &b);
        assert!(m.conflicts.is_empty(), "{:?}", m.conflicts);
        assert_eq!(parsed(&m).tags, ["a", "b"]);
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
        assert_eq!(run(&b, &o, &t, ".yaks/h/yaks/notes.md").unwrap(), 1);
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
