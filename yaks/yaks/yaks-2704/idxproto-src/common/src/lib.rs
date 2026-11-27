//! Shared scanning, git-times and benchmark harness for the two index prototypes.
pub mod gen_util;
pub mod harness;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug)]
pub struct Doc {
    pub path: String, // farm-relative, '/'-separated
    pub kind: String, // top-level dir: yaks|doks|sources|sessions|other
    pub title: String,
    pub tags: String, // space-joined
    pub body: String,
    pub links: Vec<(String, String)>, // (normalised target path or url, anchor)
    pub mtime_fs: i64,
}

pub fn walk(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for e in rd.flatten() {
            let p = e.path();
            let name = e.file_name();
            if name == ".git" {
                continue;
            }
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                stack.push(p);
            } else if p.extension().map_or(false, |x| x == "md") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

pub fn rel(root: &Path, p: &Path) -> String {
    p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/")
}

fn normalise(dir: &str, target: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    let t = if let Some(t) = target.strip_prefix('/') {
        t
    } else {
        for s in dir.split('/').filter(|s| !s.is_empty()) {
            parts.push(s);
        }
        target
    };
    for s in t.split('/') {
        match s {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

/// Parse one markdown file: frontmatter (title, tags), body, link edges.
pub fn parse(root: &Path, path: &Path) -> Option<Doc> {
    let text = std::fs::read_to_string(path).ok()?;
    let meta = std::fs::metadata(path).ok()?;
    let mtime_fs = meta
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs() as i64;
    let rel = rel(root, path);
    let kind = rel.split('/').next().filter(|_| rel.contains('/')).unwrap_or("other").to_string();
    let (mut title, mut tags, mut body) = (String::new(), Vec::new(), text.as_str());
    if let Some(rest) = text.strip_prefix("---\n") {
        if let Some(end) = rest.find("\n---\n").map(|i| (i, 5)).or_else(|| rest.strip_suffix("\n---").map(|r| (r.len(), 4))) {
            let fm = &rest[..end.0];
            body = rest.get(end.0 + end.1..).unwrap_or("");
            let mut in_tags = false;
            for line in fm.lines() {
                if let Some(v) = line.strip_prefix("title:") {
                    title = v.trim().trim_matches(|c| c == '\'' || c == '"').to_string();
                    in_tags = false;
                } else if line.starts_with("tags:") {
                    in_tags = true;
                    let v = line["tags:".len()..].trim();
                    if v.starts_with('[') {
                        for t in v.trim_matches(|c| c == '[' || c == ']').split(',') {
                            tags.push(t.trim().to_string());
                        }
                        in_tags = false;
                    }
                } else if in_tags {
                    if let Some(t) = line.strip_prefix("- ") {
                        tags.push(t.trim().to_string());
                    } else {
                        in_tags = false;
                    }
                }
            }
        }
    }
    if title.is_empty() {
        title = body
            .lines()
            .find_map(|l| l.strip_prefix("# "))
            .unwrap_or_else(|| rel.rsplit('/').next().unwrap())
            .to_string();
    }
    // links: ](target)
    let dir = rel.rsplit_once('/').map_or("", |x| x.0);
    let mut links = Vec::new();
    let b = body.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        if b[i] == b']' && b[i + 1] == b'(' {
            if let Some(end) = body[i + 2..].find(')') {
                let raw = body[i + 2..i + 2 + end].trim().trim_matches(|c| c == '<' || c == '>');
                if !raw.is_empty() && !raw.contains(' ') && !raw.contains('\n') {
                    let (t, a) = raw.split_once('#').unwrap_or((raw, ""));
                    let t = if t.contains("://") || t.starts_with("code:") || t.starts_with("mailto:") {
                        t.to_string()
                    } else if t.is_empty() {
                        rel.clone()
                    } else {
                        normalise(dir, t)
                    };
                    links.push((t, a.to_string()));
                }
                i += 2 + end;
            }
        }
        i += 1;
    }
    Some(Doc { path: rel, kind, title, tags: tags.join(" "), body: body.to_string(), links, mtime_fs })
}

pub fn scan(root: &Path) -> Vec<Doc> {
    walk(root).iter().filter_map(|p| parse(root, p)).collect()
}

/// Modified time per path from one `git log --name-only` pass; commits carrying a
/// `Mechanical:` trailer are skipped; files not in the log, or dirty, fall back to fs mtime.
pub fn git_times(root: &Path, docs: &[Doc]) -> HashMap<String, i64> {
    let out = Command::new("git")
        .current_dir(root)
        .args(["-c", "core.quotepath=off", "log", "--name-only", "--no-renames", "--format=%x01%ct%x02%(trailers:key=Mechanical,valueonly,separator=%x20)%x02"])
        .output()
        .expect("git log");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut times: HashMap<String, i64> = HashMap::new();
    for chunk in text.split('\x01').skip(1) {
        let mut lines = chunk.lines();
        let head = lines.next().unwrap_or("");
        let mut hp = head.split('\x02');
        let ct: i64 = hp.next().and_then(|s| s.parse().ok()).unwrap_or(0);
        let mech = hp.next().unwrap_or("").trim();
        if !mech.is_empty() {
            continue;
        }
        for l in lines {
            if !l.is_empty() {
                times.entry(l.to_string()).or_insert(ct); // log is newest first
            }
        }
    }
    let st = Command::new("git")
        .current_dir(root)
        .args(["-c", "core.quotepath=off", "status", "--porcelain", "--untracked-files=all"])
        .output()
        .expect("git status");
    let dirty: std::collections::HashSet<String> = String::from_utf8_lossy(&st.stdout)
        .lines()
        .filter(|l| l.len() > 3)
        .map(|l| l[3..].to_string())
        .collect();
    let mut result = HashMap::new();
    for d in docs {
        let t = match times.get(&d.path) {
            Some(t) if !dirty.contains(&d.path) => *t,
            _ => d.mtime_fs,
        };
        result.insert(d.path.clone(), t);
    }
    result
}

pub fn dir_size(p: &Path) -> u64 {
    let mut total = 0;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let m = e.metadata().unwrap();
            if m.is_dir() {
                total += dir_size(&e.path());
            } else {
                total += m.len();
            }
        }
    } else if let Ok(m) = std::fs::metadata(p) {
        total = m.len();
    }
    total
}
