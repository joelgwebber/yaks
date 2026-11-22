//! Where a link points (yaks-f505, design `/doks/design/farm-layout-and-links.md`,
//! "The seam"): one resolver from a [`Link`] to a [`Place`], and from a place
//! to a local file, displayable content and a web URL.
//!
//! [`crate::links`] stays the pure grammar. This is the one home of the
//! aliases (`/sources/<id>` names a source: it reads as that source's file,
//! `sources/<id>.md`, [`farm_file`]) and of the pinned-link fallback (a pinned file
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
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A farm path as it reads: `sources/<name>`, which names a source, is that
/// source's file, `sources/<name>.md` (a link such as
/// `/sources/2026-11-03-81364278`, yaks-e83f). Every other path is as
/// written. Lexical, so it holds for a pinned link too; it cannot know a
/// source's slug: [`farm_file_in`] does.
pub fn farm_file(path: &str) -> Cow<'_, str> {
    match path.strip_prefix("sources/") {
        Some(id) if !id.is_empty() && !id.contains(['/', '.']) => {
            Cow::Owned(format!("sources/{id}.md"))
        }
        _ => Cow::Borrowed(path),
    }
}

/// [`farm_file`] against the farm at `root`, where a source may be named
/// `<id>-<slug>` (yaks-7729): a path naming `sources/<id>`, `sources/<id>.md`
/// or `sources/<id>/<file>` (a file of its sibling directory) by the bare
/// `<date>-<sha8>` id finds the one source that carries it, so a link written
/// without the slug keeps resolving. A path that already exists is left as
/// written.
pub fn farm_file_in<'a>(root: &Path, path: &'a str) -> Cow<'a, str> {
    let named = farm_file(path);
    if root.join(&*named).exists() {
        return named;
    }
    let Some(rest) = named.strip_prefix("sources/") else {
        return named;
    };
    let (id, tail) = rest.split_once('/').unwrap_or((rest, ""));
    let id = if tail.is_empty() {
        id.strip_suffix(".md").unwrap_or(id)
    } else {
        id
    };
    match crate::sources::dir_for(root, id) {
        Some(dir) if tail.is_empty() => Cow::Owned(crate::sources::source_file(&dir)),
        Some(dir) => Cow::Owned(format!("{dir}/{tail}")),
        None => named,
    }
}

/// The farm-relative path (no trailing `/`) `link` writes, when it is an
/// unpinned [`Link::Farm`] path or a [`Link::Attachment`] (`<id>/` is the
/// yak's whole attachment directory), a source named by its bare
/// `<date>-<sha8>` id included ([`farm_file_in`]). The path may pass
/// *through* an archive (`<id>/code.zip/src/main.rs`); [`dir_of`] and
/// [`member_of`] say what it names. `None` for a farm path that is a file.
fn rel_of(farm: &Path, link: &Link) -> Option<String> {
    Some(match link {
        Link::Farm {
            path, rev: None, ..
        } => {
            let p = path.trim_end_matches('/');
            // `sources/<id>` names the source's file, not its directory,
            // unless written as a directory.
            if path.ends_with('/') {
                farm_file_in(farm, &format!("{p}/x"))
                    .strip_suffix("/x")?
                    .to_string()
            } else if farm.join(&*farm_file_in(farm, p)).is_file()
                && !crate::entries::is_browsable_name(p)
            {
                return None;
            } else {
                p.to_string()
            }
        }
        Link::Attachment { id, name, .. } => {
            let dir = store::artifact_dir(farm, id);
            let dir = match name.trim_end_matches('/') {
                "" => dir,
                n => crate::farm::attachment_path(farm, id, n)?,
            };
            dir.strip_prefix(farm)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/")
        }
        _ => return None,
    })
}

/// The farm directory `link` names, as a farm-relative path with no trailing
/// `/`, when it names one: a [`rel_of`] path that is a directory on disk, an
/// archive file (`.zip`, `.tar`, `.tar.gz`, `.tgz`: it reads as a read-only
/// directory, yaks-93ee) or a directory inside one. Unpinned only; never a
/// path [`crate::entries::safe_rel`] refuses, nor a symlink.
pub fn dir_of(farm: &Path, link: &Link) -> Option<String> {
    let rel = rel_of(farm, link)?;
    crate::entries::dir_at(farm, &rel).map(|_| rel)
}

/// The file inside an archive `link` names, as the archive's farm-relative
/// path and the file's path inside it (`("<herd>/yaks/<id>/code.zip",
/// "src/main.rs")`), when it names one that is there.
pub fn member_of(farm: &Path, link: &Link) -> Option<(String, String)> {
    use crate::entries::{Node, container_at, stat};
    let rel = rel_of(farm, link)?;
    let (c, inner) = container_at(farm, &rel)?;
    if inner.is_empty() || !matches!(stat(&c, &inner), Some(Node::File(_))) {
        return None;
    }
    let archive = rel[..rel.len() - inner.len()].trim_end_matches('/');
    Some((archive.to_string(), inner))
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
    /// A yak's attachment: a file, or a file inside an attached directory.
    Attachment {
        id: String,
        name: String,
        anchor: Option<Anchor>,
    },
    /// A directory of the farm (yaks-84fe), by its farm-relative path with
    /// no trailing `/`: a yak's attachment directory or a directory in it, a
    /// dok's or source's sibling directory, any other farm directory. What
    /// [`crate::entries`] lists.
    FarmDir {
        rel: String,
    },
    /// A file inside an archive attachment (yaks-93ee), read-only: the
    /// archive's farm-relative path and the file's path inside it. A
    /// directory inside an archive is a [`Place::FarmDir`] whose `rel` passes
    /// through the archive.
    Member {
        archive: String,
        inner: String,
        anchor: Option<Anchor>,
    },
    /// A file of the code repository.
    CodeFile {
        rel: String,
        rev: Option<String>,
        anchor: Option<Anchor>,
    },
    /// A commit of the code repository (yaks-274c).
    Commit {
        sha: String,
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
///
/// A `Space` is also the batch: the git facts [`Space::web_url`] needs that do
/// not depend on the link (a repository's remote and host, whether the farm
/// is private, HEAD, whether a commit or branch is on the remote) are asked
/// once per `Space` and remembered for its life (yaks-bfb2: a changelog of
/// fifty links was 630 git calls). Make a fresh `Space` to see a push or a
/// fetch; whether each file is on the remote is still asked per link.
pub struct Space {
    pub farm: PathBuf,
    pub code: Option<PathBuf>,
    memo: RefCell<Memo>,
}

/// The git facts a [`Space`] has asked, keyed by repository directory.
#[derive(Default)]
struct Memo {
    branch: HashMap<PathBuf, Option<String>>,
    hosting: HashMap<(PathBuf, Option<String>), Hosting>,
    private: HashMap<PathBuf, bool>,
    commit: HashMap<(PathBuf, String), Option<String>>,
    remote_has: HashMap<(PathBuf, String, String), bool>,
    remote_branch: HashMap<(PathBuf, String, String), bool>,
}

/// [`gitrepo::hosting_repo`]'s answer, its error kept as text to be cloned.
type Hosting = Result<(String, hosting::Repo), String>;

/// `map[key]`, computing and remembering it with `ask` the first time.
fn memo<K: std::hash::Hash + Eq, V: Clone>(
    map: &mut HashMap<K, V>,
    key: K,
    ask: impl FnOnce() -> V,
) -> V {
    map.entry(key).or_insert_with(ask).clone()
}

impl Space {
    pub fn new(farm: &Path, code: Option<PathBuf>) -> Space {
        Space {
            farm: farm.to_path_buf(),
            code,
            memo: RefCell::default(),
        }
    }

    // -- the remembered git facts of `web_url` (yaks-bfb2) ------------------

    fn current_branch(&self, dir: &Path) -> Option<String> {
        memo(&mut self.memo.borrow_mut().branch, dir.into(), || {
            gitrepo::current_branch(dir)
        })
    }

    fn hosting_repo(&self, dir: &Path, branch: Option<&str>) -> Result<(String, hosting::Repo)> {
        let key = (dir.to_path_buf(), branch.map(String::from));
        memo(&mut self.memo.borrow_mut().hosting, key, || {
            gitrepo::hosting_repo(dir, branch).map_err(|e| format!("{e:#}"))
        })
        .map_err(anyhow::Error::msg)
    }

    fn is_private(&self, top: &Path) -> bool {
        memo(&mut self.memo.borrow_mut().private, top.into(), || {
            crate::privatefarm::is_private(top)
        })
    }

    fn commit_of(&self, dir: &Path, rev: &str) -> Option<String> {
        let key = (dir.to_path_buf(), rev.to_string());
        memo(&mut self.memo.borrow_mut().commit, key, || {
            gitrepo::commit_of(dir, rev)
        })
    }

    fn remote_has(&self, dir: &Path, remote: &str, sha: &str) -> bool {
        let key = (dir.to_path_buf(), remote.to_string(), sha.to_string());
        memo(&mut self.memo.borrow_mut().remote_has, key, || {
            gitrepo::remote_has(dir, remote, sha)
        })
    }

    fn remote_branch_exists(&self, dir: &Path, remote: &str, branch: &str) -> bool {
        let key = (dir.to_path_buf(), remote.to_string(), branch.to_string());
        memo(&mut self.memo.borrow_mut().remote_branch, key, || {
            gitrepo::remote_branch_exists(dir, remote, branch)
        })
    }

    /// What `link` names. The one place that knows a link can name a
    /// directory: a farm path written with a trailing `/`, or any farm path
    /// or attachment that is a directory on disk or an archive ([`dir_of`]),
    /// resolves to [`Place::FarmDir`], and a file inside an archive
    /// ([`member_of`]) to [`Place::Member`]. Every other link is lexical.
    pub fn resolve(&self, link: &Link) -> Place {
        if let Some(rel) = dir_of(&self.farm, link) {
            return Place::FarmDir { rel };
        }
        if let (
            Some((archive, inner)),
            Link::Farm { anchor, .. } | Link::Attachment { anchor, .. },
        ) = (member_of(&self.farm, link), link)
        {
            return Place::Member {
                archive,
                inner,
                anchor: anchor.clone(),
            };
        }
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
            Link::Commit { sha } => Place::Commit { sha: sha.clone() },
            Link::Upward(s) => Place::Missing(format!(
                "{s}: a link may not climb out with ../; link from the farm root (/doks/...)"
            )),
            Link::Local(_) => Place::Missing("an in-document anchor needs its document".into()),
        }
    }

    /// `link` with a URL on the code repository's own host read for what it
    /// names (yaks-274c, also yaks-390f): `https://<host>/<o>/<r>/commit/<sha>`
    /// is a [`Link::Commit`], and a file pinned to a commit
    /// (`.../blob/<sha>/<path>#L12-L20`, and the GitLab and Gitea forms) is a
    /// pinned [`Link::Code`]. Every other link comes back as it was: another
    /// repository's URL, a file at a branch (it moves, so it stays a URL), or
    /// a farm with no code checkout or no recognised host. Readers that care
    /// what a URL is call this on what [`links::parse`] gave them.
    pub fn read_url(&self, link: Link) -> Link {
        let Link::Url(u) = &link else {
            return link;
        };
        let Some(repo) = self.code_repo() else {
            return link;
        };
        match repo.parse_url(u) {
            Some(hosting::HostRef::Commit(sha)) => Link::Commit { sha },
            Some(hosting::HostRef::File {
                commit,
                path,
                lines,
            }) => Link::Code {
                path,
                rev: Some(commit),
                anchor: lines.map(|(a, b)| Anchor::Lines(a, b)),
            },
            None => link,
        }
    }

    /// The code checkout's host repository, asked of git once per checkout
    /// per process (a render reads many links).
    fn code_repo(&self) -> Option<hosting::Repo> {
        static KNOWN: std::sync::Mutex<
            Option<std::collections::HashMap<PathBuf, Option<hosting::Repo>>>,
        > = std::sync::Mutex::new(None);
        let code = self.code.as_ref()?;
        let mut guard = KNOWN.lock().ok()?;
        let map = guard.get_or_insert_with(Default::default);
        map.entry(code.clone())
            .or_insert_with(|| {
                let branch = gitrepo::current_branch(code);
                gitrepo::hosting_repo(code, branch.as_deref())
                    .ok()
                    .map(|(_, r)| r)
            })
            .clone()
    }

    /// The link a place came from (what [`links::peek`] reads).
    pub fn link(place: &Place) -> Option<Link> {
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
            Place::Commit { sha } => Link::Commit { sha: sha.clone() },
            Place::Url(u) => Link::Url(u.clone()),
            Place::FarmDir { rel } => Link::Farm {
                path: format!("{rel}/"),
                rev: None,
                anchor: None,
            },
            // The path goes through the archive, as the link was written.
            Place::Member {
                archive,
                inner,
                anchor,
            } => Link::Farm {
                path: format!("{archive}/{inner}"),
                rev: None,
                anchor: anchor.clone(),
            },
            Place::Missing(_) => return None,
        })
    }

    /// What the place shows, as `yaks peek` prints it.
    #[allow(dead_code)] // the TUI reads whole files (`code_text`); `peek` goes through `links::peek`
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
            Place::FarmDir { rel } => Some((rel.clone(), None, None)),
            Place::Member {
                archive,
                inner,
                anchor,
            } => Some((format!("{archive}/{inner}"), None, anchor.clone())),
            _ => None,
        }
    }

    /// The whole text of code file `rel` (at commit `rev`, when pinned), for
    /// the TUI viewer: [`Space::content`] cuts to the anchor, this does not.
    /// `Err` is the one line the viewer shows instead: a pin this checkout
    /// does not have ([`missing_commit`]) or a file the commit lacks say so,
    /// rather than naming git's own error.
    pub fn code_text(&self, rel: &str, rev: Option<&str>) -> Result<String, String> {
        let Some(code) = &self.code else {
            return Err(format!(
                "code:{rel}: no code checkout to read it from (run from one)"
            ));
        };
        if let Some(rev) = rev {
            if let Some(why) = missing_commit(code, rev) {
                return Err(why);
            }
            if !gitrepo::has_path(code, rev, &format!("./{rel}")) {
                return Err(format!("{rel} is not in {}", &rev[..rev.len().min(7)]));
            }
        }
        links::read_at(code, rel, rev).map_err(|e| format!("{e:#}"))
    }

    /// The path of the place inside its repository, as written to a person:
    /// a `code:` file's path from the checkout root, a farm file's from the
    /// farm root (a yak's file and an attachment included). `None` for a URL
    /// or a place that names nothing.
    pub fn rel_path(&self, place: &Place) -> Option<String> {
        match place {
            Place::CodeFile { rel, .. } => Some(rel.clone()),
            other => self.farm_rel(other).map(|(rel, ..)| rel),
        }
    }

    /// The path of the place's file on this machine, and whether it is a
    /// read-only copy: a pinned file is the working file when it is unchanged
    /// since the pin, else a read-only copy of the pinned content; a member
    /// of an archive is always a read-only copy ([`extract_member`]).
    pub fn local_path(&self, place: &Place) -> Result<(PathBuf, bool), String> {
        if let Place::Member { archive, inner, .. } = place {
            return extract_member(&self.farm, archive, inner).map(|p| (p, true));
        }
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
        if let Place::Member { archive, inner, .. } = place {
            bail!(
                "{archive}/{inner}: a host cannot show a member of an archive; link the archive file"
            );
        }
        if let Place::Commit { sha } = place {
            let Some(code) = &self.code else {
                bail!("commit:{sha}: no code checkout to link from (run from one)");
            };
            let branch = self.current_branch(code);
            let (remote, repo) = self.hosting_repo(code, branch.as_deref())?;
            let full = self.commit_of(code, sha).unwrap_or_else(|| sha.clone());
            let note = (!self.remote_has(code, &remote, &full)).then(|| {
                format!(
                    "{} is not on {remote} as last fetched: the link leads nowhere until it is pushed",
                    &full[..full.len().min(7)]
                )
            });
            return Ok(Web {
                url: repo.commit_url(&full),
                note,
            });
        }
        if let Place::CodeFile { rel, rev, anchor } = place {
            let Some(code) = &self.code else {
                bail!("code:{rel}: no code checkout to link from (run from one)");
            };
            if rev.is_none() && !code.join(rel).is_file() {
                bail!("code:{rel}: no such file in {}", code.display());
            }
            let branch = self.current_branch(code);
            let (remote, repo) = self.hosting_repo(code, branch.as_deref())?;
            return self.file_web(
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
        if self.is_private(&top) {
            bail!(
                "this farm is private (`git config yaks.private` is true): it never leaves this \
                 machine, so there is no link to give. `yaks private --off` undoes it"
            );
        }
        let branch = store::farm_branch(&self.farm);
        let (remote, repo) = self.hosting_repo(&top, branch.as_deref())?;
        self.file_web(
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

    #[allow(clippy::too_many_arguments)]
    fn file_web(
        &self,
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
            commit = self.commit_of(dir, rev).unwrap_or_else(|| rev.to_string());
            if !self.remote_has(dir, remote, &commit) {
                note = Some(format!(
                    "{} is not on {remote} as last fetched: the link leads nowhere until it is pushed",
                    &commit[..commit.len().min(7)]
                ));
            }
            Ref::Commit(&commit)
        } else {
            // No pin written: the branch, a stable link; `--pin` asks for the
            // commit, which the remote must already have to be worth giving.
            let pushed_head = self
                .commit_of(dir, "HEAD")
                .filter(|h| self.remote_has(dir, remote, h));
            let live_branch = branch.filter(|b| self.remote_branch_exists(dir, remote, b));
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
}

fn git(base: &Path, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(base)
        .args(args)
        .stdin(Stdio::null())
        .output()
}

/// Why `rev` cannot be read in the repository at `base`: `None` when the
/// commit is there, else a one-line message. An agent on another machine can
/// pin a commit this checkout never fetched (or one that was dropped); the
/// reader needs to hear that, not git's error.
pub fn missing_commit(base: &Path, rev: &str) -> Option<String> {
    if gitrepo::commit_of(base, rev).is_some() {
        return None;
    }
    Some(format!(
        "commit {} is not in this checkout; fetch it or open on the host",
        &rev[..rev.len().min(7)]
    ))
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
    if let Some(why) = missing_commit(base, rev) {
        return Err(why);
    }
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
    // The one reader of a file at a rev: an LFS file is its content, not its
    // pointer ([`links::read_bytes_at`]).
    let shown = links::read_bytes_at(base, path, Some(rev)).map_err(|e| format!("{e:#}"))?;
    let tmp = std::env::temp_dir().join("yaks-src").join(rev).join(path);
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(tmp.parent().unwrap())?;
        let _ = std::fs::remove_file(&tmp); // a previous copy is read-only
        std::fs::write(&tmp, &shown)?;
        let mut perm = std::fs::metadata(&tmp)?.permissions();
        perm.set_readonly(true);
        std::fs::set_permissions(&tmp, perm)
    };
    write().map_err(|e| format!("temp copy failed: {e}"))?;
    Ok((tmp, true))
}

/// Most bytes of a member [`extract_member`] writes out.
const EXTRACT_MAX_BYTES: u64 = 64 << 20;

/// A read-only temp copy of member `inner` of the archive at farm path
/// `archive`: `<tmp>/yaks-archive/<key>/<inner>`, `key` naming the archive's
/// path, size and mtime so a changed archive never reuses a stale copy. The
/// path is the one the viewer shows, so `e`/`o` act on what was read. `Err`
/// is a one-line message.
pub fn extract_member(farm: &Path, archive: &str, inner: &str) -> Result<PathBuf, String> {
    use crate::entries::{Node, container_at, read, stat};
    use std::hash::{Hash, Hasher};
    let Some((c, top)) = container_at(farm, archive).filter(|(_, t)| t.is_empty()) else {
        return Err(format!("{archive}: not an archive of this farm"));
    };
    let _ = top;
    if !crate::entries::safe_rel(inner) || !matches!(stat(&c, inner), Some(Node::File(_))) {
        return Err(format!("{archive}/{inner}: no such file in the archive"));
    }
    let bytes = read(&c, inner, EXTRACT_MAX_BYTES).map_err(|e| format!("{e:#}"))?;
    let file = farm.join(archive);
    let meta = std::fs::metadata(&file).map_err(|e| format!("{archive}: {e}"))?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    (&file, meta.len(), meta.modified().ok()).hash(&mut h);
    let tmp = std::env::temp_dir()
        .join("yaks-archive")
        .join(format!("{:016x}", h.finish()))
        .join(inner);
    let write = || -> std::io::Result<()> {
        std::fs::create_dir_all(tmp.parent().unwrap())?;
        let _ = std::fs::remove_file(&tmp); // a previous copy is read-only
        std::fs::write(&tmp, &bytes)?;
        let mut perm = std::fs::metadata(&tmp)?.permissions();
        perm.set_readonly(true);
        std::fs::set_permissions(&tmp, perm)
    };
    write().map_err(|e| format!("temp copy failed: {e}"))?;
    Ok(tmp)
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
        || body.starts_with("commit:")
        || body.starts_with('/') && !Path::new(body).exists()
    {
        return space.read_url(parse(arg));
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
            "sources/2026-11-03-81364278.md"
        );
        for same in [
            "sources/x/shot.png",
            "sources/x.md",
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
                rel: "sources/abc.md".into(),
                rev: Some("1234567".into()),
                anchor: Some(Anchor::Lines(1, 2)),
            }
        );
        assert!(matches!(
            sp.resolve(&Link::Upward("../x".into())),
            Place::Missing(_)
        ));
    }

    #[test]
    fn the_code_repos_own_host_urls_read_as_commits_and_code() {
        let dir = std::env::temp_dir().join(format!("yaks-readurl-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            assert!(
                std::process::Command::new("git")
                    .args(args)
                    .current_dir(&dir)
                    .output()
                    .unwrap()
                    .status
                    .success()
            );
        };
        git(&["init", "-q", "-b", "main"]);
        git(&["remote", "add", "origin", "git@github.com:o/r.git"]);
        let sp = Space::new(Path::new("/nonexistent-farm"), Some(dir.clone()));
        let url = |u: &str| sp.read_url(Link::Url(u.into()));
        assert_eq!(
            url("https://github.com/o/r/commit/abc1234"),
            Link::Commit {
                sha: "abc1234".into()
            }
        );
        assert_eq!(
            url("https://github.com/o/r/blob/abc1234/src/a.rs#L3-L9"),
            Link::Code {
                path: "src/a.rs".into(),
                rev: Some("abc1234".into()),
                anchor: Some(Anchor::Lines(3, 9)),
            }
        );
        let other = "https://github.com/o/other/commit/abc1234";
        assert_eq!(url(other), Link::Url(other.into()));
        // No code checkout: a URL stays a URL.
        let bare = Space::new(Path::new("/nonexistent-farm"), None);
        let u = "https://github.com/o/r/commit/abc1234";
        assert_eq!(bare.read_url(Link::Url(u.into())), Link::Url(u.into()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A farm whose yak `t0` has `a.zip` and `b.tar.gz` attached (and the
    /// fixture's hostile members), for the archive tests.
    fn with_archives() -> (PathBuf, Space) {
        use crate::entries::fixtures;
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        // Links resolve against a directory: no farm files are needed.
        let dir = std::env::temp_dir().join(format!(
            "yaks-locate-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let a = store::artifact_dir(&dir, "t0");
        std::fs::create_dir_all(&a).unwrap();
        let members: &[(&str, &[u8])] = &[
            ("src/x.rs", b"one\ntwo\nthree\n"),
            ("README.md", b"# hi\n"),
            ("../evil.txt", b"evil"),
        ];
        fixtures::zip(&a.join("a.zip"), members);
        fixtures::tgz(&a.join("b.tar.gz"), members, &[]);
        let space = Space::new(&dir, None);
        (dir, space)
    }

    fn parse(dest: &str) -> Link {
        links::parse(dest, links::From::Yak, |t| t == "t0")
    }

    #[test]
    fn a_path_through_an_archive_resolves_to_a_member_or_a_directory() {
        let (dir, space) = with_archives();
        let rel = |n: &str| {
            let a = store::artifact_dir(&space.farm, "t0");
            format!("{}/{n}", a.strip_prefix(&space.farm).unwrap().display())
        };
        for z in ["a.zip", "b.tar.gz"] {
            // A member of the archive, with its anchor.
            let l = parse(&format!("t0/{z}/src/x.rs#L2"));
            assert_eq!(
                space.resolve(&l),
                Place::Member {
                    archive: rel(z),
                    inner: "src/x.rs".into(),
                    anchor: Some(Anchor::Lines(2, 2)),
                },
                "{z}"
            );
            // A directory in it, the archive's top, and the archive itself.
            assert_eq!(
                space.resolve(&parse(&format!("t0/{z}/src/"))),
                Place::FarmDir {
                    rel: rel(&format!("{z}/src"))
                },
                "{z}"
            );
            for top in [format!("t0/{z}/"), format!("t0/{z}")] {
                assert_eq!(
                    space.resolve(&parse(&top)),
                    Place::FarmDir { rel: rel(z) },
                    "{top}"
                );
            }
            // The same through a farm path.
            let l = parse(&format!("/{}", rel(&format!("{z}/src/x.rs"))));
            assert!(matches!(space.resolve(&l), Place::Member { .. }), "{z}");
            // Hostile and missing members are nothing.
            for bad in ["../evil.txt", "evil.txt", "src/nope.rs", "src/x.rs/y"] {
                let l = parse(&format!("t0/{z}/{bad}"));
                assert!(
                    !matches!(
                        space.resolve(&l),
                        Place::Member { .. } | Place::FarmDir { .. }
                    ),
                    "{z}/{bad}"
                );
                assert_eq!(links::exists(&space.farm, &l), Some(false), "{z}/{bad}");
            }
            // exists.
            for good in ["src/x.rs", "src/", "README.md", ""] {
                let l = parse(&format!("t0/{z}/{good}"));
                assert_eq!(links::exists(&space.farm, &l), Some(true), "{z}/{good}");
            }
        }
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn peek_reads_a_member_at_its_lines_and_lists_an_archive_directory() {
        let (dir, space) = with_archives();
        let bases = links::Bases {
            farm: &space.farm,
            code: None,
        };
        let p = links::peek(&parse("t0/a.zip/src/x.rs#L2"), &bases, 0).unwrap();
        assert!(
            p.text.contains("two") && !p.text.contains("three"),
            "{}",
            p.text
        );
        let p = links::peek(&parse("t0/b.tar.gz/"), &bases, 0).unwrap();
        assert_eq!(p.text, "src/\nREADME.md\n");
        let p = links::peek(&parse("t0/b.tar.gz/src/"), &bases, 0).unwrap();
        assert_eq!(p.text, "x.rs\n");
        assert!(links::peek(&parse("t0/a.zip/../evil.txt"), &bases, 0).is_err());
        assert!(links::peek(&parse("t0/a.zip/src/nope.rs"), &bases, 0).is_err());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn a_member_links_back_names_a_read_only_copy_and_has_no_host_url() {
        let (dir, space) = with_archives();
        let l = parse("t0/a.zip/src/x.rs#L2");
        let place = space.resolve(&l);
        // The link it came from goes through the archive, and parses back
        // to the same place.
        let back = Space::link(&place).unwrap();
        assert_eq!(space.resolve(&back), place);
        let (rel, ..) = space.farm_rel(&place).unwrap();
        assert!(rel.ends_with("t0/a.zip/src/x.rs"), "{rel}");
        // A read-only temp copy of the member, not the archive.
        let (path, copy) = space.local_path(&place).unwrap();
        assert!(copy);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "one\ntwo\nthree\n");
        assert!(std::fs::metadata(&path).unwrap().permissions().readonly());
        assert!(path.starts_with(std::env::temp_dir()));
        // A second extraction replaces the read-only copy.
        assert_eq!(space.local_path(&place).unwrap().0, path);
        // A host cannot show a member.
        let why = space.web_url(&place, false).unwrap_err().to_string();
        assert!(
            why.contains("host cannot show a member of an archive"),
            "{why}"
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
