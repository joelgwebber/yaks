//! Web hosts for a repository (yaks-f505, design `/doks/design/farm-layout-and-links.md`,
//! "URL templates"): which forge a remote is, and the URL of a file on it.
//!
//! GitHub (and GitHub Enterprise Server), GitLab and Gitea/Forgejo are
//! shipped, because their templates were checked against their sources.
//! Bitbucket and Azure DevOps are not: nobody has tested them, so a remote
//! there is refused with a message rather than given a URL that may 404.
//!
//! A remote is recognised from its normalized form
//! ([`crate::store::normalize_remote_url`]: `host/owner/repo`, whatever the
//! scheme). A well-known host (`github.com`, `gitlab.com`, `codeberg.org`) or
//! a host with a label naming the forge (`github.corp.example`,
//! `gitlab.example.org`) is detected; any other domain declares its kind with
//! `git config yaks.host github|gitlab|gitea|forgejo` ([`HOST_KEY`]).

use anyhow::{Result, bail};

/// The git config key that names the forge of a custom domain.
pub const HOST_KEY: &str = "yaks.host";

/// The hosts whose URL templates are shipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Forge {
    /// github.com and GitHub Enterprise Server.
    GitHub,
    /// gitlab.com and self-managed GitLab.
    GitLab,
    /// Gitea and Forgejo (Codeberg).
    Gitea,
}

impl Forge {
    /// The kind a `yaks.host` value names.
    pub fn parse(s: &str) -> Option<Forge> {
        match s.trim().to_ascii_lowercase().as_str() {
            "github" | "ghes" => Some(Forge::GitHub),
            "gitlab" => Some(Forge::GitLab),
            "gitea" | "forgejo" | "codeberg" => Some(Forge::Gitea),
            _ => None,
        }
    }

    fn detect(host: &str) -> Option<Forge> {
        match host {
            "github.com" => return Some(Forge::GitHub),
            "gitlab.com" => return Some(Forge::GitLab),
            "codeberg.org" | "gitea.com" => return Some(Forge::Gitea),
            _ => {}
        }
        host.split('.').find_map(|label| match label {
            "github" => Some(Forge::GitHub),
            "gitlab" => Some(Forge::GitLab),
            "gitea" | "forgejo" | "codeberg" => Some(Forge::Gitea),
            _ => None,
        })
    }
}

/// A repository on a host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    pub forge: Forge,
    pub host: String,
    /// `owner/repo`, or `group/subgroup/repo` on GitLab.
    pub path: String,
}

/// What a file URL names: a branch (moves) or a commit (a permalink).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ref<'a> {
    Branch(&'a str),
    Commit(&'a str),
}

/// The part after `#` (or the query) of a file URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Frag {
    None,
    /// 1-based, inclusive.
    Lines(u32, u32),
    /// A heading's slug, on a rendered markdown file.
    Heading(String),
}

impl Repo {
    /// Recognise the host of `remote_url` (any spelling git accepts), with
    /// `declared` the `yaks.host` value, which wins over detection. `Err`
    /// says why nothing was recognised and names `yaks.host`.
    pub fn from_remote(remote_url: &str, declared: Option<&str>) -> Result<Repo> {
        let Some(norm) = crate::store::normalize_remote_url(remote_url) else {
            bail!("the remote URL is empty");
        };
        let (host, path) = match norm.split_once('/') {
            Some((h, p)) if h.contains('.') && h != "." && h != ".." && !p.is_empty() => (h, p),
            _ => bail!(
                "the remote {remote_url} is a local path, not a web host; point it at a \
                 GitHub, GitLab or Gitea/Forgejo repository"
            ),
        };
        let forge = match declared {
            Some(d) => Forge::parse(d).ok_or_else(|| {
                anyhow::anyhow!(
                    "`git config {HOST_KEY}` is {d:?}; use github, gitlab, gitea or forgejo"
                )
            })?,
            None => match Forge::detect(host) {
                Some(f) => f,
                None => {
                    let unsupported = if host.contains("bitbucket") {
                        " (Bitbucket is not supported yet)"
                    } else if host.contains("azure") || host.contains("visualstudio") {
                        " (Azure DevOps is not supported yet)"
                    } else {
                        ""
                    };
                    bail!(
                        "{host} is not a recognised host{unsupported}: yaks knows GitHub, GitLab \
                         and Gitea/Forgejo. If {host} is one of them at a custom domain, say \
                         which with `git config {HOST_KEY} github|gitlab|gitea`"
                    )
                }
            },
        };
        Ok(Repo {
            forge,
            host: host.to_string(),
            path: path.to_string(),
        })
    }

    /// The URL of file `path` (repo-relative) at `r`, at `frag`.
    pub fn file_url(&self, r: Ref, path: &str, frag: &Frag) -> String {
        let path = encode(path, false);
        let base = format!("https://{}/{}", self.host, self.path);
        let mut url = match (self.forge, r) {
            (Forge::GitHub, r) => format!("{base}/blob/{}/{path}", ref_name(r)),
            (Forge::GitLab, r) => format!("{base}/-/blob/{}/{path}", ref_name(r)),
            (Forge::Gitea, Ref::Branch(b)) => {
                format!("{base}/src/branch/{}/{path}", encode(b, false))
            }
            (Forge::Gitea, Ref::Commit(c)) => format!("{base}/src/commit/{c}/{path}"),
        };
        match frag {
            Frag::None => {}
            Frag::Heading(h) => {
                url.push('#');
                url.push_str(&encode(h, true));
            }
            Frag::Lines(a, b) => {
                // GitHub renders a markdown file as prose unless asked for
                // its source, which is the only view with line anchors.
                if self.forge == Forge::GitHub && path.ends_with(".md") {
                    url.push_str("?plain=1");
                }
                url.push_str(&match (self.forge, a == b) {
                    (_, true) => format!("#L{a}"),
                    (Forge::GitLab, false) => format!("#L{a}-{b}"),
                    (_, false) => format!("#L{a}-L{b}"),
                });
            }
        }
        url
    }
}

fn ref_name(r: Ref) -> String {
    match r {
        Ref::Branch(b) => encode(b, false),
        Ref::Commit(c) => c.to_string(),
    }
}

/// Percent-encode everything but unreserved characters (and `/` in a path).
fn encode(s: &str, fragment: bool) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let keep = b.is_ascii_alphanumeric()
            || matches!(b, b'-' | b'_' | b'.' | b'~')
            || (b == b'/' && !fragment);
        if keep {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(url: &str) -> Repo {
        Repo::from_remote(url, None).unwrap()
    }

    #[test]
    fn remote_spellings_name_one_repository() {
        let want = Repo {
            forge: Forge::GitHub,
            host: "github.com".into(),
            path: "o/r".into(),
        };
        for url in [
            "git@github.com:o/r.git",
            "ssh://git@github.com/o/r.git",
            "ssh://git@github.com:22/o/r",
            "https://github.com/o/r",
            "https://user:tok@GitHub.com/o/r.git/",
            "git://github.com/o/r.git",
        ] {
            assert_eq!(repo(url), want, "{url}");
        }
    }

    #[test]
    fn hosts_are_detected_by_name_and_declared_for_custom_domains() {
        assert_eq!(repo("https://gitlab.com/g/r").forge, Forge::GitLab);
        assert_eq!(repo("https://codeberg.org/o/r").forge, Forge::Gitea);
        assert_eq!(repo("git@github.corp.example:o/r.git").forge, Forge::GitHub);
        assert_eq!(
            repo("https://gitlab.example.org/a/b/r").forge,
            Forge::GitLab
        );
        let custom = "https://git.example.com/o/r.git";
        let e = Repo::from_remote(custom, None).unwrap_err().to_string();
        assert!(e.contains("yaks.host"), "{e}");
        for (kind, forge) in [
            ("github", Forge::GitHub),
            ("GitLab", Forge::GitLab),
            ("gitea", Forge::Gitea),
            ("forgejo", Forge::Gitea),
        ] {
            assert_eq!(Repo::from_remote(custom, Some(kind)).unwrap().forge, forge);
        }
        let e = Repo::from_remote(custom, Some("svn"))
            .unwrap_err()
            .to_string();
        assert!(e.contains("github, gitlab, gitea or forgejo"), "{e}");
        // A declaration wins over detection.
        assert_eq!(
            Repo::from_remote("https://github.com/o/r", Some("gitea"))
                .unwrap()
                .forge,
            Forge::Gitea
        );
    }

    #[test]
    fn gitlab_subgroups_keep_their_whole_path() {
        let r = repo("git@gitlab.com:group/sub/deep/repo.git");
        assert_eq!(r.path, "group/sub/deep/repo");
        assert_eq!(
            r.file_url(Ref::Branch("main"), "a/b.rs", &Frag::None),
            "https://gitlab.com/group/sub/deep/repo/-/blob/main/a/b.rs"
        );
    }

    #[test]
    fn unsupported_and_local_remotes_are_refused_with_a_reason() {
        // Azure DevOps over ssh, https, and Bitbucket.
        for (url, word) in [
            ("git@ssh.dev.azure.com:v3/org/proj/repo", "Azure DevOps"),
            ("https://dev.azure.com/org/proj/_git/repo", "Azure DevOps"),
            ("git@bitbucket.org:ws/r.git", "Bitbucket"),
        ] {
            let e = Repo::from_remote(url, None).unwrap_err().to_string();
            assert!(e.contains(word) && e.contains("yaks.host"), "{url}: {e}");
        }
        for url in ["/src/repo.git", "../repo", "file:///src/repo", ""] {
            assert!(Repo::from_remote(url, None).is_err(), "{url}");
        }
        let e = Repo::from_remote("/src/repo.git", None)
            .unwrap_err()
            .to_string();
        assert!(e.contains("local path"), "{e}");
    }

    #[test]
    fn github_templates() {
        let r = repo("https://github.com/o/r");
        assert_eq!(
            r.file_url(Ref::Branch("main"), "src/a.rs", &Frag::None),
            "https://github.com/o/r/blob/main/src/a.rs"
        );
        assert_eq!(
            r.file_url(Ref::Commit("abc1234"), "src/a.rs", &Frag::Lines(100, 120)),
            "https://github.com/o/r/blob/abc1234/src/a.rs#L100-L120"
        );
        assert_eq!(
            r.file_url(Ref::Branch("yaks"), "a/b.md", &Frag::Lines(3, 3)),
            "https://github.com/o/r/blob/yaks/a/b.md?plain=1#L3"
        );
        assert_eq!(
            r.file_url(
                Ref::Branch("feat/x"),
                "a b.md",
                &Frag::Heading("setup".into())
            ),
            "https://github.com/o/r/blob/feat/x/a%20b.md#setup"
        );
        // GHES: the same shape on its own host.
        assert_eq!(
            repo("git@github.corp.example:o/r.git").file_url(
                Ref::Branch("main"),
                "x.rs",
                &Frag::None
            ),
            "https://github.corp.example/o/r/blob/main/x.rs"
        );
    }

    #[test]
    fn gitlab_templates() {
        let r = repo("https://gitlab.com/g/r");
        assert_eq!(
            r.file_url(Ref::Commit("abc1234"), "src/a.rs", &Frag::Lines(100, 120)),
            "https://gitlab.com/g/r/-/blob/abc1234/src/a.rs#L100-120"
        );
        assert_eq!(
            r.file_url(Ref::Branch("main"), "a.md", &Frag::Lines(7, 7)),
            "https://gitlab.com/g/r/-/blob/main/a.md#L7"
        );
    }

    #[test]
    fn gitea_and_forgejo_templates() {
        let r = repo("https://codeberg.org/o/r");
        assert_eq!(
            r.file_url(Ref::Branch("main"), "src/a.rs", &Frag::None),
            "https://codeberg.org/o/r/src/branch/main/src/a.rs"
        );
        assert_eq!(
            r.file_url(Ref::Commit("abc1234"), "src/a.rs", &Frag::Lines(100, 120)),
            "https://codeberg.org/o/r/src/commit/abc1234/src/a.rs#L100-L120"
        );
    }
}
