//! Where a link points (yaks-f505, design `/doks/design/farm-layout-and-links.md`,
//! "The seam"): one resolver from a [`Link`] to a [`Place`], and from a place
//! to a local file, displayable content and a web URL.
//!
//! [`crate::links`] stays the pure grammar. This is the one home of the
//! aliases (`/sources/<id>` names a directory: it reads as that source's
//! `source.md`, [`farm_file`]) and of the pinned-link fallback (a pinned file
//! that differs from the working tree gets a read-only copy,
//! [`pinned_file`]), so `peek`, `doctor`, the TUI viewer and `yaks url` agree.
//!
//! The farm's position in its git repository is read from
//! [`crate::store::farm_git`] and the path between the farm root and the
//! worktree top, never from a literal directory name, so a layout change
//! moves nothing here.

use crate::hosting::{Frag, Ref};
use crate::links::{Anchor, Link, Peek};
use crate::{gitrepo, hosting, links, store};
use anyhow::{Result, bail};
use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A farm path as it reads: `sources/<id>`, which names a directory, is that
/// source's `source.md` (a link such as `/sources/2026-11-03-81364278`). Every
/// other path is as written. Lexical, so it holds for a pinned link too.
pub fn farm_file(path: &str) -> Cow<'_, str> {
    match path.strip_prefix("sources/") {
        Some(id) if !id.is_empty() && !id.contains(['/', '.']) => {
            Cow::Owned(format!("sources/{id}/source.md"))
        }
        _ => Cow::Borrowed(path),
    }
}

/// What a link resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// A yak's file; `note` names one of its notes by timestamp.
    Yak {
        id: String,
        note: Option<String>,
    },
    /// A file of the farm (aliases applied), by its farm-relative path,
    /// optionally as it was at farm commit `rev`.
    FarmFile {
        rel: String,
        rev: Option<String>,
        anchor: Option<Anchor>,
    },
    /// A yak's attachment.
    Attachment {
        id: String,
        name: String,
        anchor: Option<Anchor>,
    },
    /// A file of the code repository.
    CodeFile {
        rel: String,
        rev: Option<String>,
        anchor: Option<Anchor>,
    },
    Url(String),
    /// Nothing to point at (an in-document anchor, a `../` link).
    Missing(String),
}

/// A web URL, and what the reader should be told about how it was chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Web {
    pub url: String,
    pub note: Option<String>,
}

/// What a link resolves against: the farm root and the code checkout.
pub struct Space {
    pub farm: PathBuf,
    pub code: Option<PathBuf>,
}

impl Space {
    pub fn new(farm: &Path, code: Option<PathBuf>) -> Space {
        Space {
            farm: farm.to_path_buf(),
            code,
        }
    }

    pub fn resolve(&self, link: &Link) -> Place {
        match link {
            Link::Url(u) => Place::Url(u.clone()),
            Link::Yak { id, note } => Place::Yak {
                id: id.clone(),
                note: note.clone(),
            },
            Link::Farm { path, rev, anchor } => Place::FarmFile {
                rel: farm_file(path).into_owned(),
                rev: rev.clone(),
                anchor: anchor.clone(),
            },
            Link::Attachment { id, name, anchor } => Place::Attachment {
                id: id.clone(),
                name: name.clone(),
                anchor: anchor.clone(),
            },
            Link::Code { path, rev, anchor } => Place::CodeFile {
                rel: path.clone(),
                rev: rev.clone(),
                anchor: anchor.clone(),
            },
            Link::Upward(s) => Place::Missing(format!(
                "{s}: a link may not climb out with ../; link from the farm root (/doks/...)"
            )),
            Link::Local(_) => Place::Missing("an in-document anchor needs its document".into()),
        }
    }

    /// The link a place came from (what [`links::peek`] reads).
    #[allow(dead_code)] // with `content` and `local_path`: the TUI's one `follow(Place)` (yaks-c3d0) is their caller
    fn link(place: &Place) -> Option<Link> {
        Some(match place {
            Place::Yak { id, note } => Link::Yak {
                id: id.clone(),
                note: note.clone(),
            },
            Place::FarmFile { rel, rev, anchor } => Link::Farm {
                path: rel.clone(),
                rev: rev.clone(),
                anchor: anchor.clone(),
            },
            Place::Attachment { id, name, anchor } => Link::Attachment {
                id: id.clone(),
                name: name.clone(),
                anchor: anchor.clone(),
            },
            Place::CodeFile { rel, rev, anchor } => Link::Code {
                path: rel.clone(),
                rev: rev.clone(),
                anchor: anchor.clone(),
            },
            Place::Url(u) => Link::Url(u.clone()),
            Place::Missing(_) => return None,
        })
    }

    /// What the place shows, as `yaks peek` prints it.
    #[allow(dead_code)]
    pub fn content(&self, place: &Place, context: u32) -> Result<Peek> {
        let Some(link) = Self::link(place) else {
            let Place::Missing(why) = place else {
                unreachable!()
            };
            bail!("{why}");
        };
        links::peek(
            &link,
            &links::Bases {
                farm: &self.farm,
                code: self.code.as_deref(),
            },
            context,
        )
    }

    /// The farm-relative path of the place when it is a file of the farm
    /// (a yak's file and an attachment included).
    fn farm_rel(&self, place: &Place) -> Option<(String, Option<String>, Option<Anchor>)> {
        let rel = |p: PathBuf| {
            p.strip_prefix(&self.farm)
                .ok()
                .map(|r| r.to_string_lossy().replace('\\', "/"))
        };
        match place {
            Place::Yak { id, .. } => Some((rel(store::task_path(&self.farm, id))?, None, None)),
            Place::Attachment { id, name, anchor } => Some((
                rel(store::artifact_dir(&self.farm, id).join(name))?,
                None,
                anchor.clone(),
            )),
            Place::FarmFile { rel, rev, anchor } => {
                Some((rel.clone(), rev.clone(), anchor.clone()))
            }
            _ => None,
        }
    }

    /// The path of the place's file on this machine, and whether it is a
    /// read-only copy: a pinned file is the working file when it is unchanged
    /// since the pin, else a read-only copy of the pinned content.
    #[allow(dead_code)]
    pub fn local_path(&self, place: &Place) -> Result<(PathBuf, bool), String> {
        if let Place::CodeFile { rel, rev, .. } = place {
            let Some(code) = &self.code else {
                return Err(format!("code:{rel}: no code checkout to read it from"));
            };
            return pinned_file(code, rel, rev.as_deref());
        }
        let Some((rel, rev, _)) = self.farm_rel(place) else {
            return Err("this link names no file".into());
        };
        match rev {
            None => pinned_file(&self.farm, &rel, None),
            Some(rev) => {
                // The farm worktree's own root is the git base.
                let (top, prefix) = self.farm_top().map_err(|e| e.to_string())?;
                pinned_file(&top, &format!("{prefix}{rel}"), Some(&rev))
            }
        }
    }

    /// The farm worktree's top and the path from it to the farm root
    /// (`""`, or `<dir>/` when the farm root is below the top).
    fn farm_top(&self) -> Result<(PathBuf, String)> {
        let Some(g) = store::farm_git(&self.farm) else {
            bail!(
                "{} is not on its own farm branch (reached through YAKS_DIR), so it has no git \
                 repository to link into",
                self.farm.display()
            );
        };
        let root = gitrepo::canonical(&self.farm);
        let rel = root
            .strip_prefix(&g.top)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_default();
        let prefix = if rel.is_empty() {
            String::new()
        } else {
            format!("{rel}/")
        };
        Ok((g.top, prefix))
    }

    /// The host URL of the place. A file of the farm is at the farm branch
    /// (`pin`: at the commit the farm is at, when the remote has it, else
    /// the branch, saying so); a `code:` file is at its own pin, else the
    /// code branch (`pin`: the commit). Refuses, saying why, when the
    /// repository has no remote or no recognised host, and for a private farm.
    pub fn web_url(&self, place: &Place, pin: bool) -> Result<Web> {
        if let Place::Url(u) = place {
            return Ok(Web {
                url: u.clone(),
                note: None,
            });
        }
        if let Place::Missing(why) = place {
            bail!("{why}");
        }
        if let Place::CodeFile { rel, rev, anchor } = place {
            let Some(code) = &self.code else {
                bail!("code:{rel}: no code checkout to link from (run from one)");
            };
            if rev.is_none() && !code.join(rel).is_file() {
                bail!("code:{rel}: no such file in {}", code.display());
            }
            let branch = gitrepo::current_branch(code);
            let (remote, repo) = gitrepo::hosting_repo(code, branch.as_deref())?;
            return file_web(
                code,
                &remote,
                &repo,
                branch.as_deref(),
                rev.as_deref(),
                pin,
                rel,
                anchor,
            );
        }
        let (rel, rev, anchor) = self.farm_rel(place).expect("a farm place");
        if rev.is_none() && !self.farm.join(&rel).exists() {
            match place {
                Place::Yak { id, .. } => bail!("{id}: no such yak"),
                Place::Attachment { id, name, .. } => bail!("{id}/{name}: no such attachment"),
                _ => bail!("{rel}: no such file in the farm"),
            }
        }
        let (top, prefix) = self.farm_top()?;
        if crate::privatefarm::is_private(&top) {
            bail!(
                "this farm is private (`git config yaks.private` is true): it never leaves this \
                 machine, so there is no link to give. `yaks private --off` undoes it"
            );
        }
        let branch = store::farm_branch(&self.farm);
        let (remote, repo) = gitrepo::hosting_repo(&top, branch.as_deref())?;
        file_web(
            &top,
            &remote,
            &repo,
            branch.as_deref(),
            rev.as_deref(),
            pin,
            &format!("{prefix}{rel}"),
            &anchor,
        )
    }
}

#[allow(clippy::too_many_arguments)]
fn file_web(
    dir: &Path,
    remote: &str,
    repo: &hosting::Repo,
    branch: Option<&str>,
    rev: Option<&str>,
    pin: bool,
    repo_path: &str,
    anchor: &Option<Anchor>,
) -> Result<Web> {
    let frag = match anchor {
        None => Frag::None,
        Some(Anchor::Lines(a, b)) => Frag::Lines(*a, *b),
        Some(Anchor::Heading(h)) => Frag::Heading(h.clone()),
    };
    let mut note = None;
    let (commit, branch_name);
    let r = if let Some(rev) = rev {
        // An explicit pin: the commit as written, expanded when git knows it.
        commit = gitrepo::commit_of(dir, rev).unwrap_or_else(|| rev.to_string());
        if !gitrepo::remote_has(dir, remote, &commit) {
            note = Some(format!(
                "{} is not on {remote} as last fetched: the link leads nowhere until it is pushed",
                &commit[..commit.len().min(7)]
            ));
        }
        Ref::Commit(&commit)
    } else {
        // No pin written: the branch, a stable link; `--pin` asks for the
        // commit, which the remote must already have to be worth giving.
        let pushed_head =
            gitrepo::commit_of(dir, "HEAD").filter(|h| gitrepo::remote_has(dir, remote, h));
        let live_branch = branch.filter(|b| gitrepo::remote_branch_exists(dir, remote, b));
        let (want_commit, fallback_note) = match (pin, pushed_head, live_branch) {
            (true, Some(h), _) => (Some((h, None)), None),
            (true, None, Some(b)) => (
                None,
                Some(format!(
                    "HEAD is not on {remote} as last fetched (this checkout is ahead), so there \
                     is no commit to link; linking the branch {b} instead"
                )),
            ),
            (false, _, Some(_)) => (None, None),
            (false, Some(h), None) => (
                Some((
                    h.clone(),
                    Some(format!(
                        "{remote} has no branch to name; linking the commit {}",
                        &h[..7]
                    )),
                )),
                None,
            ),
            (_, _, None) => bail!(
                "neither the branch nor HEAD is on {remote} as last fetched, so there is \
                 nothing to link to: push it first (`yaks sync` for a farm, `git push` for code)"
            ),
        };
        match want_commit {
            Some((h, n)) => {
                commit = h;
                note = n;
                Ref::Commit(&commit)
            }
            None => {
                note = fallback_note;
                branch_name = live_branch.expect("a live branch").to_string();
                Ref::Branch(&branch_name)
            }
        }
    };
    // The file may be new since the last push: say so rather than hand out a 404.
    let at = match r {
        Ref::Commit(c) => c.to_string(),
        Ref::Branch(b) => format!("{remote}/{b}"),
    };
    if note.is_none() && !gitrepo::has_path(dir, &at, repo_path) {
        note = Some(format!(
            "{repo_path} is not in {at} (not pushed yet?): the link leads nowhere until it is"
        ));
    }
    Ok(Web {
        url: repo.file_url(r, repo_path, &frag),
        note,
    })
}

fn git(base: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(base)
        .args(args)
        .stdin(Stdio::null())
        .output()
}

/// The file to show for `path` of the repository at `base`: the working
/// copy, or for a pinned link whose file differs from the working tree, a
/// read-only copy of `git show <rev>:<path>` in the temp directory. The bool
/// is `true` for that copy. `Err` is a one-line message.
pub fn pinned_file(base: &Path, path: &str, rev: Option<&str>) -> Result<(PathBuf, bool), String> {
    let work = base.join(path);
    let Some(rev) = rev else {
        return if work.is_file() {
            Ok((work, false))
        } else {
            Err(format!("{path}: no such file"))
        };
    };
    let rel = format!("./{path}");
    // `git diff <rev> -- <file>` is clean for an untracked file too, so first
    // ask whether the commit has the file at all.
    let has =
        git(base, &["cat-file", "-e", &format!("{rev}:{rel}")]).map_err(|e| format!("git: {e}"))?;
    if !has.status.success() {
        let short = &rev[..rev.len().min(7)];
        let err = String::from_utf8_lossy(&has.stderr);
        return Err(
            if err.contains("exists on disk, but not in") || err.contains("does not exist in") {
                format!("{path} not in {short}")
            } else {
                format!("{short}: {}", err.lines().next().unwrap_or("not a commit"))
            },
        );
    }
    let diff = git(base, &["diff", "--quiet", rev, "--", &rel]).map_err(|e| format!("git: {e}"))?;
    match diff.status.code() {
        Some(0) if work.is_file() => return Ok((work, false)),
        Some(0 | 1) => {}
        _ => {
            let err = String::from_utf8_lossy(&diff.stderr);
            let line = err.lines().next().unwrap_or("git diff failed");
            return Err(format!("{rev}: {line}"));
        }
    }
    let shown = git(base, &["show", &format!("{rev}:{rel}")]).map_err(|e| format!("git: {e}"))?;
    if !shown.status.success() {
        return Err(format!("{path} not in {}", &rev[..rev.len().min(7)]));
    }
    let tmp = std::env::temp_dir().join("yaks-src").join(rev).join(path);
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(tmp.parent().unwrap())?;
        let _ = std::fs::remove_file(&tmp); // a previous copy is read-only
        std::fs::write(&tmp, &shown.stdout)?;
        let mut perm = std::fs::metadata(&tmp)?.permissions();
        perm.set_readonly(true);
        std::fs::set_permissions(&tmp, perm)
    };
    write().map_err(|e| format!("temp copy failed: {e}"))?;
    Ok((tmp, true))
}

/// What `yaks url <arg>` links: a link as written in a yak (`yaks-1a2b`,
/// `/doks/x.md#setup`, `code:src/a.rs@3f2a9c1#L12-L20`), or a file path. A
/// path inside the farm is a farm file, one inside the code checkout a
/// `code:` file; a `#L..` fragment is kept.
pub fn parse_target(space: &Space, cwd: &Path, arg: &str) -> Link {
    let ids = store::all_ids(&space.farm);
    let parse = |s: &str| links::parse(s, links::From::Cli, |t| ids.contains(t));
    let (body, frag) = match arg.split_once('#') {
        Some((b, f)) => (b, Some(f)),
        None => (arg, None),
    };
    let with_frag = |s: String| match frag {
        Some(f) => format!("{s}#{f}"),
        None => s,
    };
    if body.contains("://")
        || body.starts_with("code:")
        || body.starts_with('/') && !Path::new(body).exists()
    {
        return parse(arg);
    }
    let abs = if Path::new(body).is_absolute() {
        PathBuf::from(body)
    } else {
        cwd.join(body)
    };
    if abs.is_file()
        && let Ok(abs) = abs.canonicalize()
    {
        let rel_in = |base: &Path| {
            abs.strip_prefix(gitrepo::canonical(base))
                .ok()
                .map(|r| r.to_string_lossy().replace('\\', "/"))
        };
        if let Some(rel) = rel_in(&space.farm) {
            return parse(&with_frag(format!("/{rel}")));
        }
        if let Some(rel) = space.code.as_deref().and_then(rel_in) {
            return parse(&with_frag(format!("code:{rel}")));
        }
    }
    parse(arg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_source_directory_reads_as_its_source_file() {
        assert_eq!(
            farm_file("sources/2026-11-03-81364278"),
            "sources/2026-11-03-81364278/source.md"
        );
        for same in [
            "sources/x/source.md",
            "sources/x/extracted.md",
            "sources/",
            "sources",
            "doks/sources/x",
            "doks/a.md",
            "sources/a.md",
        ] {
            assert_eq!(farm_file(same), same);
        }
    }

    #[test]
    fn resolve_applies_the_alias_and_keeps_the_rest() {
        let sp = Space::new(Path::new("/farm"), None);
        let l = Link::Farm {
            path: "sources/abc".into(),
            rev: Some("1234567".into()),
            anchor: Some(Anchor::Lines(1, 2)),
        };
        assert_eq!(
            sp.resolve(&l),
            Place::FarmFile {
                rel: "sources/abc/source.md".into(),
                rev: Some("1234567".into()),
                anchor: Some(Anchor::Lines(1, 2)),
            }
        );
        assert!(matches!(
            sp.resolve(&Link::Upward("../x".into())),
            Place::Missing(_)
        ));
    }
}
