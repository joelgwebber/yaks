//! Session notes (yaks-17f5, design yaks-bca6): a short account of one
//! working session, left in the farm so a later review (yaks-e58f) and the
//! weekly reflection can learn from it. The same shape as Hearth's
//! `session-review` files, so the ones `import-hearth` brought in read alike.
//!
//! A note lives at `sessions/YYYY/MM/<actor>-<day>-<hhmm>-<slug>.md`:
//! frontmatter (`type: session-review`, `title`, `source: session-note`,
//! who, where, when, and a count of each verdict), then four sections:
//! `## Summary`, `## Turns` (one `- <verdict>: <reason>` per request),
//! `## Saved` and `## Lessons`, plus an optional `## Discussion`. A session
//! whose work is in yak notes cites those notes ([`touched`]) rather than
//! restating them; yaks writes `## Saved` from them when the note has none.
//!
//! yaks decides only what the files say: the path, the frontmatter, the
//! verdict counts, what the actor wrote and which notes are unreviewed. What a
//! session was worth, and what to keep from it, is the agent's (the
//! `yaks-upkeep` skill).

use crate::doks::Dok;
use crate::fields::{Fields, Value};
use crate::store;
use anyhow::{Context, Result, bail};
use chrono::{DateTime, Local, Utc};
use std::path::Path;

/// The farm directory session notes live in.
pub const DIR: &str = "sessions";
/// A session note's `type:`.
pub const KIND: &str = "session-review";
/// The `source:` of a note an agent wrote about its own session (Hearth's
/// server reviews of its conversations carry another, or none).
pub const SOURCE: &str = "session-note";
/// The field a review stamps on a note once its facts are kept.
pub const PROMOTED: &str = "promoted";
/// How a turn went, in the order counts are written.
pub const VERDICTS: [&str; 4] = ["good", "partial", "failed", "unclear"];
/// A note is a summary, not a transcript (Hearth's cap).
pub const MAX_CHARS: usize = 12_000;

/// Every markdown file under `sessions/`, farm-relative, sorted (so oldest
/// first: the path starts with the year and month).
pub fn paths(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![(DIR.to_string(), root.join(DIR))];
    while let Some((rel, dir)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let Some(name) = e.file_name().to_str().map(String::from) else {
                continue;
            };
            if name.starts_with('.') {
                continue;
            }
            let child = format!("{rel}/{name}");
            if e.path().is_dir() {
                stack.push((child, e.path()));
            } else if name.ends_with(".md") {
                out.push(child);
            }
        }
    }
    out.sort();
    out
}

/// Every session file, parsed.
pub fn load_all(root: &Path) -> Result<Vec<Dok>> {
    paths(root)
        .into_iter()
        .map(|rel| load(root, &rel))
        .collect()
}

fn load(root: &Path, rel: &str) -> Result<Dok> {
    let path = root.join(rel);
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    Ok(Dok::parse(rel, &text))
}

/// Whether a session file is a note an agent wrote about its own session.
pub fn is_note(d: &Dok) -> bool {
    d.str_field("type").as_deref() == Some(KIND) && d.str_field("source").as_deref() == Some(SOURCE)
}

/// Whether a note still waits for review: no `promoted:` yet.
pub fn is_unreviewed(d: &Dok) -> bool {
    is_note(d) && !d.fields.contains(PROMOTED)
}

// ------------------------------------------------------------ the body

/// What [`check_body`] read from a note's body.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BodyInfo {
    /// Turns per verdict, in [`VERDICTS`] order.
    pub counts: [usize; 4],
    pub has_saved: bool,
}

/// The `##` sections of `body` by slug, with their text (heading excluded).
fn level2(body: &str) -> Vec<(String, String)> {
    crate::links::sections(body)
        .into_iter()
        .filter(|(level, _, _)| *level == 2)
        .map(|(_, slug, r)| {
            let text = &body[r];
            let text = text.split_once('\n').map_or("", |(_, rest)| rest);
            (slug, text.to_string())
        })
        .collect()
}

/// The list items of a section: lines starting `- ` or `* ` (not nested).
fn items(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|l| l.strip_prefix("- ").or_else(|| l.strip_prefix("* ")))
        .map(str::trim)
        .collect()
}

/// Check a note's body and count its verdicts. It needs `## Summary` (not
/// empty), `## Turns` with every item `<verdict>: <reason>`, and `## Lessons`
/// (`- none` is fine); `## Saved` may be left for yaks to write.
pub fn check_body(body: &str) -> Result<BodyInfo> {
    if body.len() > MAX_CHARS {
        bail!(
            "the note is {} characters; keep it under {MAX_CHARS}: it is a summary, not a \
             transcript (cite yak notes for the detail)",
            body.len()
        );
    }
    let secs = level2(body);
    let get = |slug: &str| {
        secs.iter()
            .find(|(s, _)| s == slug)
            .map(|(_, t)| t.as_str())
    };
    match get("summary") {
        Some(t) if !t.trim().is_empty() => {}
        _ => bail!(
            "the note needs a `## Summary`: one to three sentences on what the session was about"
        ),
    }
    let Some(turns) = get("turns") else {
        bail!(
            "the note needs a `## Turns` section: one `- <verdict>: <reason>` per request, \
             verdict good, partial, failed or unclear"
        );
    };
    let mut counts = [0; 4];
    for item in items(turns) {
        let verdict = item.split_once(':').map(|(v, _)| v.trim().to_lowercase());
        match verdict
            .as_deref()
            .and_then(|v| VERDICTS.iter().position(|x| *x == v))
        {
            Some(i) => counts[i] += 1,
            None => bail!(
                "turn `{item}` does not start with a verdict: write it as `good: ...`, \
                 `partial: ...`, `failed: ...` or `unclear: ...`"
            ),
        }
    }
    if counts.iter().sum::<usize>() == 0 {
        bail!("`## Turns` lists no turn: one `- <verdict>: <reason>` per request the person made");
    }
    if get("lessons").is_none() {
        bail!("the note needs a `## Lessons` section (`- none` when there are none)");
    }
    Ok(BodyInfo {
        counts,
        has_saved: get("saved").is_some(),
    })
}

/// `body` with a `## Saved` section holding `saved`, put before `## Lessons`.
pub fn with_saved(body: &str, saved: &str) -> String {
    let section = format!("## Saved\n\n{}\n\n", saved.trim_end());
    let lessons = crate::links::sections(body)
        .into_iter()
        .find(|(level, slug, _)| *level == 2 && slug == "lessons");
    match lessons {
        Some((_, _, r)) => format!("{}{section}{}", &body[..r.start], &body[r.start..]),
        None => format!("{}\n\n{section}", body.trim_end()),
    }
}

// ------------------------------------------------------------ writing

/// Who and where a note is from.
#[derive(Debug, Clone, Default)]
pub struct Origin {
    /// The actor the note is stamped with (the agent's name).
    pub actor: String,
    /// The person the session was with (git `user.name`), when known.
    pub person: Option<String>,
    pub host: Option<String>,
    /// The agent app (`delta`, `claude-code`, ...).
    pub harness: Option<String>,
    /// The harness's id for the conversation, when it has one: the way back
    /// to the full transcript.
    pub thread: Option<String>,
}

/// Lowercase-kebab, at most `max` characters.
fn slug(s: &str, max: usize) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let mut out = out.trim_matches('-').to_string();
    if out.len() > max {
        out.truncate(max);
        out = out.trim_end_matches('-').to_string();
    }
    out
}

/// Build the note (not written): its farm-relative path and the dok. `now`
/// is when it is saved; the day and the file name use its local date.
pub fn build(
    root: &Path,
    title: &str,
    body: &str,
    origin: &Origin,
    now: DateTime<Utc>,
) -> Result<Dok> {
    let title = title.split_whitespace().collect::<Vec<_>>().join(" ");
    if title.is_empty() {
        bail!("a session note needs a --title: a few words naming the session");
    }
    let info = check_body(body)?;
    let local = now.with_timezone(&Local);
    let day = local.format("%Y-%m-%d").to_string();
    let who = slug(&origin.actor, 40);
    let who = if who.is_empty() { "agent".into() } else { who };
    let stem = format!(
        "{DIR}/{}/{who}-{day}-{}-{}",
        local.format("%Y/%m"),
        local.format("%H%M"),
        slug(&title, 40)
    );
    let mut rel = format!("{stem}.md");
    let mut n = 2;
    while root.join(&rel).exists() {
        rel = format!("{stem}-{n}.md");
        n += 1;
    }
    let mut f = Fields::default();
    f.set("type", &Value::Plain(KIND.into()));
    f.set("title", &Value::Str(title));
    f.set("source", &Value::Plain(SOURCE.into()));
    f.set("actor", &Value::Str(origin.actor.clone()));
    let opt = |f: &mut Fields, k: &str, v: &Option<String>| {
        if let Some(v) = v.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            f.set(k, &Value::Str(v.to_string()));
        }
    };
    opt(&mut f, "person", &origin.person);
    opt(&mut f, "host", &origin.host);
    opt(&mut f, "harness", &origin.harness);
    opt(&mut f, "thread", &origin.thread);
    f.set("day", &Value::Plain(day));
    f.set(
        "reviewed",
        &Value::Str(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
    );
    for (i, v) in VERDICTS.iter().enumerate() {
        if info.counts[i] > 0 {
            f.set(v, &Value::Plain(info.counts[i].to_string()));
        }
    }
    Ok(Dok {
        path: rel,
        fields: f,
        body: body.trim().to_string() + "\n",
        has_frontmatter: true,
    })
}

/// Stamp `promoted: <day>` on the note at `rel`. `Ok(false)`: it already had
/// one. Refuses anything that is not a session note.
pub fn promote(root: &Path, rel: &str, day: &str) -> Result<bool> {
    let rel = crate::doks::normalize(rel);
    if !rel.starts_with("sessions/") || !root.join(&rel).is_file() {
        bail!("{rel}: no such session note (`yaks session list` shows them)");
    }
    let mut d = load(root, &rel)?;
    if d.str_field("type").as_deref() != Some(KIND) {
        bail!("{rel} is not a session note (its type is not {KIND})");
    }
    if d.fields.contains(PROMOTED) {
        return Ok(false);
    }
    d.fields.set(PROMOTED, &Value::Plain(day.to_string()));
    store::write::atomic_write(&root.join(&rel), &d.render())?;
    Ok(true)
}

// ------------------------------------------------------------ touched

/// One note an actor wrote on a yak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchedNote {
    /// The note's link anchor (see [`crate::links::note_anchor`]).
    pub anchor: String,
    /// Its `kind:` metadata, if any (`answer`, `landed`, ...).
    pub kind: Option<String>,
    /// Its first line, shortened.
    pub gist: String,
}

/// A yak an actor worked on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchedYak {
    pub id: String,
    pub title: String,
    /// State moves, oldest first: `(from, to)`.
    pub moves: Vec<(String, String)>,
    pub notes: Vec<TouchedNote>,
    /// Created by the actor in the window.
    pub created: bool,
}

/// A farm document (dok, source, schema) an actor wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TouchedFile {
    /// Farm-relative.
    pub path: String,
    /// `created`, `changed`, `deleted` or `renamed`.
    pub verb: String,
    pub title: Option<String>,
}

/// What an actor wrote in a window: the yaks (with their notes) and the
/// documents.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Touched {
    pub yaks: Vec<TouchedYak>,
    pub files: Vec<TouchedFile>,
}

impl Touched {
    pub fn is_empty(&self) -> bool {
        self.yaks.is_empty() && self.files.is_empty()
    }
}

fn ts_of(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

/// A note's first line of its own: past the status line the inbox commands
/// write (`asked: needs human`, `picked up`, ...).
fn gist(text: &str) -> String {
    const STAMPS: [&str; 4] = ["asked:", "answered:", "requested:", "picked up"];
    let mut lines = text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .peekable();
    if lines
        .peek()
        .is_some_and(|l| STAMPS.iter().any(|s| l.starts_with(s)))
    {
        let stamp = lines.next().unwrap_or("");
        if lines.peek().is_none() {
            return stamp.to_string();
        }
    }
    let line = link_text(lines.next().unwrap_or(""));
    let line: String = line.chars().filter(|c| !matches!(c, '[' | ']')).collect();
    let line = line.trim_start_matches(['#', ' ']).to_string();
    if line.chars().count() > 80 {
        let cut: String = line.chars().take(77).collect();
        format!("{}...", cut.trim_end())
    } else {
        line
    }
}

/// Markdown links reduced to their text: `[x](dest)` -> `[x]` (the brackets
/// go later), so a gist reads as prose.
fn link_text(line: &str) -> String {
    let mut out = String::new();
    let mut rest = line;
    while let Some(i) = rest.find("](") {
        out.push_str(&rest[..=i]);
        let after = &rest[i + 2..];
        match after.find(')') {
            Some(j) => rest = &after[j + 1..],
            None => {
                rest = after;
                break;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The start of a session for `actor`, when none is given: its latest
/// session note, else `fallback` (and never before `fallback`).
pub fn default_since(root: &Path, actor: &str, fallback: DateTime<Utc>) -> DateTime<Utc> {
    load_all(root)
        .unwrap_or_default()
        .iter()
        .filter(|d| is_note(d) && d.str_field("actor").as_deref() == Some(actor))
        .filter_map(|d| d.str_field("reviewed").and_then(|s| ts_of(&s)))
        .max()
        // Times are whole seconds: start just after the note, so what the
        // last session wrote in its final second stays in that session.
        .map_or(fallback, |t| {
            (t + chrono::Duration::seconds(1)).max(fallback)
        })
}

/// What `actor` wrote in the farm at `root` since `since`: its notes on yaks
/// (read from the yak files) and the doks, sources and schemas its farm
/// commits touched.
pub fn touched(root: &Path, actor: &str, since: DateTime<Utc>) -> Result<Touched> {
    let tasks = store::load(root, &crate::farm::EVERY)?;
    let mut out = Touched::default();
    for t in &tasks {
        let notes = store::parse_notes(&t.body);
        let mut y = TouchedYak {
            id: t.id.clone(),
            title: t.title.clone(),
            moves: Vec::new(),
            notes: Vec::new(),
            created: false,
        };
        let mine = |n: &store::NoteEntry| {
            n.actor.as_deref() == Some(actor) && ts_of(&n.ts).is_some_and(|ts| ts >= since)
        };
        for (i, n) in notes.iter().enumerate().filter(|(_, n)| mine(n)) {
            match n.kind() {
                Some("move") => {
                    if let (Some(a), Some(b)) = (n.get("from"), n.get("to")) {
                        y.moves.push((a.to_string(), b.to_string()));
                    }
                }
                kind => y.notes.push(TouchedNote {
                    anchor: crate::links::note_anchor(&notes, i),
                    kind: kind.map(String::from),
                    gist: gist(&n.text),
                }),
            }
        }
        out.yaks.push(y);
    }

    let commits = crate::autocommit::farm_commits(root, Some(actor), Some(since.timestamp()))
        .unwrap_or_default();
    let mut files: Vec<TouchedFile> = Vec::new();
    let mut created_yaks = std::collections::HashSet::new();
    // Oldest first, so a document created then edited reads `created`.
    for c in commits.iter().rev() {
        for (status, path) in changed_paths(root, &c.sha) {
            if let Some(id) = yak_file_id(&path) {
                if status == 'A' {
                    created_yaks.insert(id);
                }
                continue;
            }
            let kept = crate::doks::is_dok_path(&path)
                || (path.starts_with("sources/") && path.ends_with("/source.md"))
                || path.starts_with("schemas/");
            if !kept || files.iter().any(|f| f.path == path) {
                continue;
            }
            let verb = match status {
                'A' => "created",
                'D' => "deleted",
                'R' => "renamed",
                _ => "changed",
            };
            files.push(TouchedFile {
                title: (status != 'D')
                    .then(|| load(root, &path).ok().map(|d| d.title()))
                    .flatten(),
                path,
                verb: verb.into(),
            });
        }
    }
    for y in &mut out.yaks {
        y.created = created_yaks.contains(&y.id);
    }
    out.yaks
        .retain(|y| y.created || !y.moves.is_empty() || !y.notes.is_empty());
    out.files = files;
    Ok(out)
}

/// `<herd>/yaks/<id>.md` -> `<id>`.
fn yak_file_id(path: &str) -> Option<String> {
    let mut parts = path.split('/');
    let (_herd, dir, file) = (parts.next()?, parts.next()?, parts.next()?);
    (dir == "yaks" && parts.next().is_none())
        .then(|| file.strip_suffix(".md").map(String::from))
        .flatten()
}

/// The farm-relative paths a farm commit changed, with git's status letter
/// (a rename gives its new path).
fn changed_paths(root: &Path, sha: &str) -> Vec<(char, String)> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", "--relative", "--name-status", "--format=", sha])
        .stdin(std::process::Stdio::null())
        .output();
    let Ok(out) = out else { return Vec::new() };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let mut cols = l.split('\t');
            let status = cols.next()?.chars().next()?;
            let path = cols.next_back()?.to_string();
            Some((status, path))
        })
        .collect()
}

/// The markdown list for a note's `## Saved`: each yak with links to the
/// notes written on it, then each document. `- nothing` when empty.
pub fn render_saved(t: &Touched) -> String {
    if t.is_empty() {
        return "- nothing\n".into();
    }
    let mut s = String::new();
    for y in &t.yaks {
        let mut head = format!("- {}: {}", y.id, y.title);
        let mut what = Vec::new();
        if y.created {
            what.push("created".to_string());
        }
        if let (Some(first), Some(last)) = (y.moves.first(), y.moves.last()) {
            what.push(format!("{} -> {}", first.0, last.1));
        }
        if !what.is_empty() {
            head.push_str(&format!(" ({})", what.join(", ")));
        }
        s.push_str(&head);
        s.push('\n');
        for n in &y.notes {
            let label = match &n.kind {
                // A verify note already reads `verify: ...`.
                Some(k) if n.gist.starts_with(&format!("{k}:")) => n.gist.clone(),
                Some(k) if !n.gist.is_empty() => format!("{k}: {}", n.gist),
                Some(k) => k.clone(),
                None if n.gist.is_empty() => "note".into(),
                None => n.gist.clone(),
            };
            s.push_str(&format!("  - [{label}]({}#{})\n", y.id, n.anchor));
        }
    }
    for f in &t.files {
        let title = f
            .title
            .as_deref()
            .map(|t| format!(": {t}"))
            .unwrap_or_default();
        if f.verb == "deleted" {
            s.push_str(&format!("- deleted `{}`\n", f.path));
        } else {
            s.push_str(&format!("- {} [/{p}](/{p}){title}\n", f.verb, p = f.path));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    const BODY: &str = "## Summary\n\nTried the parser.\n\n## Turns\n\n- good: built it\n- partial: docs late\n\n## Lessons\n\n- none\n";

    #[test]
    fn the_body_needs_its_sections_and_verdicts() {
        let info = check_body(BODY).unwrap();
        assert_eq!(info.counts, [1, 1, 0, 0]);
        assert!(!info.has_saved);
        let bad = BODY.replace("- partial: docs late", "- docs late");
        assert!(
            check_body(&bad)
                .unwrap_err()
                .to_string()
                .contains("does not start with a verdict")
        );
        assert!(check_body(&BODY.replace("## Lessons\n\n- none\n", "")).is_err());
        assert!(check_body(&BODY.replace("Tried the parser.", "")).is_err());
        assert!(check_body(&"x".repeat(MAX_CHARS + 1)).is_err());
    }

    #[test]
    fn a_gist_reads_as_prose() {
        assert_eq!(
            gist("Done: the [skill](code:a/b.md@1#L2) and [x](y).\nmore"),
            "Done: the skill and x."
        );
        assert_eq!(gist("asked: needs human\n\nWhich one?"), "Which one?");
        assert_eq!(gist("picked up"), "picked up");
    }

    #[test]
    fn saved_goes_before_lessons() {
        let b = with_saved(BODY, "- nothing\n");
        assert!(
            b.contains("- partial: docs late\n\n## Saved\n\n- nothing\n\n## Lessons"),
            "{b}"
        );
        assert!(check_body(&b).unwrap().has_saved);
    }

    #[test]
    fn build_names_and_counts() {
        let root = std::env::temp_dir().join(format!("yaks-sessions-build-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let now = ts_of("2026-11-06T12:34:56Z").unwrap();
        let o = Origin {
            actor: "delta:Herd Upkeep".into(),
            person: Some("Joel".into()),
            harness: Some("delta".into()),
            ..Default::default()
        };
        let d = build(&root, "Parser: first pass!", BODY, &o, now).unwrap();
        let local = now.with_timezone(&Local);
        assert_eq!(
            d.path,
            format!(
                "sessions/{}/delta-herd-upkeep-{}-{}-parser-first-pass.md",
                local.format("%Y/%m"),
                local.format("%Y-%m-%d"),
                local.format("%H%M")
            )
        );
        assert_eq!(d.str_field("type").as_deref(), Some(KIND));
        assert_eq!(d.str_field("good").as_deref(), Some("1"));
        assert_eq!(d.str_field("failed"), None);
        assert!(is_unreviewed(&d));
        crate::doks::save(&root, &d).unwrap();
        let again = build(&root, "Parser: first pass!", BODY, &o, now).unwrap();
        assert!(
            again.path.ends_with("-parser-first-pass-2.md"),
            "{}",
            again.path
        );
        assert!(promote(&root, &d.path, "2026-11-07").unwrap());
        assert!(!promote(&root, &d.path, "2026-11-08").unwrap());
        assert!(!is_unreviewed(&load(&root, &d.path).unwrap()));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn saved_lists_yak_notes_by_link() {
        let t = Touched {
            yaks: vec![TouchedYak {
                id: "yaks-1a2b".into(),
                title: "Parser".into(),
                moves: vec![
                    ("hairy".into(), "shaving".into()),
                    ("shaving".into(), "shorn".into()),
                ],
                notes: vec![TouchedNote {
                    anchor: "2026-11-06T12:00:00Z".into(),
                    kind: None,
                    gist: "Built the parser".into(),
                }],
                created: false,
            }],
            files: vec![TouchedFile {
                path: "doks/topics/parsing.md".into(),
                verb: "created".into(),
                title: Some("Parsing".into()),
            }],
        };
        assert_eq!(
            render_saved(&t),
            "- yaks-1a2b: Parser (hairy -> shorn)\n  - [Built the parser](yaks-1a2b#2026-11-06T12:00:00Z)\n- created [/doks/topics/parsing.md](/doks/topics/parsing.md): Parsing\n"
        );
        assert_eq!(render_saved(&Touched::default()), "- nothing\n");
    }
}
