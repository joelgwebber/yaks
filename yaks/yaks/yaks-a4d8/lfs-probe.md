# Probe: reading a Git LFS image at a commit, locally and on GitHub (2026-11-21)

Run for yaks-a4d8 against this farm (`origin` = github.com/joelgwebber/yaks, branch `yaks`), on the LFS-tracked file `sources/2026-11-05-0272f64e-proton-engineering-manifesto-page-2-of-3/manifesto-2.png` at farm commit `33f5db7f3e26ce5fba47e99218ed8438812e3916`.

## Locally

```
$ git cat-file -p 33f5db7f:sources/…/manifesto-2.png
version https://git-lfs.github.com/spec/v1
oid sha256:0272f64e919d44ea476bc6d9c4f50d03a3ba332f2bf5740519a3beb96ce32fb4
size 170083
$ git cat-file -p 33f5db7f:sources/…/manifesto-2.png | git lfs smudge -- sources/…/manifesto-2.png > x.png
$ file x.png
x.png: PNG image data, 1210 x 1566, 8-bit colormap, non-interlaced
```

`git show <rev>:<path>` (what `links::read_at` runs) returns the 131-byte pointer, not the image; piping the pointer through `git lfs smudge` gives the image (from the local LFS cache, or fetched from the remote).

## On GitHub, by URL at the full commit sha (curl -L)

| URL form | HTTP | Content-Type | Bytes | Result |
|---|---|---|---|---|
| `https://media.githubusercontent.com/media/joelgwebber/yaks/<sha>/<path>` | 200 | image/png | 170083 | the PNG |
| `https://github.com/joelgwebber/yaks/raw/<sha>/<path>` | 200 | image/png | 170083 | the PNG (redirects to media.githubusercontent.com) |
| `https://raw.githubusercontent.com/joelgwebber/yaks/<sha>/<path>` | 200 | text/plain | 131 | the LFS pointer |

So a markdown image pinned to a commit renders on GitHub when written as a `github.com/<owner>/<repo>/raw/<sha>/<path>` URL; `raw.githubusercontent.com` must not be used for LFS files. Not checked: whether a *relative* image link inside a file viewed at `blob/<sha>/…` renders the LFS image (GitHub's blob view normally does for the current tree), and other hosts (GitLab, Gitea).
