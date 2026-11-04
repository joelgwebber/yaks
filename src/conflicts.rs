//! Merge conflicts left for a person or agent to settle (yaks-4c8c): the merge
//! driver ([`crate::mergedriver`]) never fails a merge on prose, so where both
//! sides changed the same lines it leaves conflict markers in the text, and a
//! dok field both sides set differently is kept from this side with the other
//! recorded under `merge_conflicts:`. This finds both, for `yaks conflicts`
//! and `yaks doctor`.

use crate::mergedriver::DOK_CONFLICTS_KEY;
use std::path::Path;

/// One file holding something to settle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// From the farm root, `/`-separated.
    pub path: String,
    /// 1-based lines where a `<<<<<<<` marker opens (outside code fences).
    pub marker_lines: Vec<usize>,
    /// Fields listed under `merge_conflicts:` (the other side's value kept
    /// there), in order.
    pub fields: Vec<String>,
}

/// The 1-based lines of `text` that open a conflict region: a `<<<<<<<` line
/// later closed by a `>>>>>>>` line, neither inside a fenced code block (so a
/// dok can show an example of markers).
pub fn marker_lines(text: &str) -> Vec<usize> {
    if !text.contains("<<<<<<<") {
        return Vec::new();
    }
    let lines: Vec<&str> = text.lines().collect();
    let fenced = crate::store::fenced_lines(&lines);
    let opens = |l: &str| l == "<<<<<<<" || l.starts_with("<<<<<<< ");
    let closes = |l: &str| l == ">>>>>>>" || l.starts_with(">>>>>>> ");
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    for (i, l) in lines.iter().enumerate() {
        if fenced[i] {
            continue;
        }
        if opens(l) {
            open = Some(i);
        } else if closes(l)
            && let Some(o) = open.take()
        {
            out.push(o + 1);
        }
    }
    out
}

/// The fields a document's `merge_conflicts:` lists.
fn recorded_fields(text: &str) -> Vec<String> {
    let Some((front, _)) = crate::store::split_frontmatter(text) else {
        return Vec::new();
    };
    if !front.contains(DOK_CONFLICTS_KEY) {
        return Vec::new();
    }
    crate::fields::Fields::parse(front)
        .get(DOK_CONFLICTS_KEY)
        .map(|v| {
            v.as_list()
                .into_iter()
                .filter_map(|r| r.get("field").and_then(|f| f.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Every markdown file of the farm at `root` holding conflict markers or a
/// `merge_conflicts:` list, sorted by path. Dot files and directories are
/// skipped.
pub fn scan(root: &Path) -> Vec<Conflict> {
    let mut out = Vec::new();
    let mut stack = vec![(String::new(), root.to_path_buf())];
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
            let child = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let path = e.path();
            if path.is_dir() {
                stack.push((child, path));
            } else if name.ends_with(".md")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                let marker_lines = marker_lines(&text);
                let fields = recorded_fields(&text);
                if !marker_lines.is_empty() || !fields.is_empty() {
                    out.push(Conflict {
                        path: child,
                        marker_lines,
                        fields,
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markers_outside_fences_are_found() {
        let t = "a\n<<<<<<< here\nx\n=======\ny\n>>>>>>> other side\n\n```\n<<<<<<< here\n>>>>>>> b\n```\n";
        assert_eq!(marker_lines(t), [2]);
        assert!(marker_lines("<<<<<<< alone\n").is_empty());
    }

    #[test]
    fn recorded_fields_are_read() {
        let t =
            "---\ntitle: x\nmerge_conflicts:\n  - field: status\n    theirs: final\n---\nbody\n";
        assert_eq!(recorded_fields(t), ["status"]);
    }
}
