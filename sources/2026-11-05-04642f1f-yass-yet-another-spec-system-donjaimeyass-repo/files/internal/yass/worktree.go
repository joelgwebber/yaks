package yass

import (
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// worktrees lists the clone's checkouts, the main one first (a bare repo has none of its own), and
// says whether dir is in a linked worktree rather than the main checkout.
func worktrees(dir string) (all []string, linked bool) {
	out, ok := git(dir, "worktree", "list", "--porcelain")
	if !ok {
		return nil, false
	}
	for _, rec := range strings.Split(strings.TrimSpace(out), "\n\n") {
		ls := strings.Split(rec, "\n")
		p, ok := strings.CutPrefix(ls[0], "worktree ")
		if ok && !contains(ls[1:], "bare") {
			all = append(all, canon(filepath.FromSlash(p)))
		}
	}
	gd, ok1 := git(dir, "rev-parse", "--path-format=absolute", "--git-dir")
	cd, ok2 := git(dir, "rev-parse", "--path-format=absolute", "--git-common-dir")
	linked = ok1 && ok2 && canon(strings.TrimSpace(gd)) != canon(strings.TrimSpace(cd))
	return all, linked
}

// fromOtherCheckouts resolves a yass.yaml's path the way the clone's other checkouts see it. A
// relative path like ../plans means "next to the checkout", and a linked worktree usually lives
// somewhere else (~/worktrees/x, .claude/worktrees/x), so from there it points at nothing. It
// returns the first folder that exists, every place it looked, and whether this is a linked worktree.
func fromOtherCheckouts(file, path string) (found string, looked []string, linked bool) {
	owner := filepath.Dir(file)
	out, ok := git(owner, "rev-parse", "--show-toplevel")
	if !ok {
		return "", nil, false
	}
	top := canon(filepath.FromSlash(strings.TrimSpace(out)))
	rel, err := filepath.Rel(top, owner)
	if err != nil {
		return "", nil, false
	}
	all, linked := worktrees(owner)
	for _, t := range all {
		if t == top {
			continue
		}
		d, err := resolvePath(path, filepath.Join(t, rel))
		if err != nil || contains(looked, d) {
			continue
		}
		looked = append(looked, d)
		if isDir(d) {
			return d, looked, linked
		}
	}
	return "", looked, linked
}

// borrowConfig covers a yass.yaml kept out of git (a private pointer): a linked worktree doesn't
// get one, so it uses the one the clone's other checkouts have at the same place, at the top or
// above the current folder.
func (r *Repo) borrowConfig() {
	all, linked := worktrees(r.Top)
	if !linked {
		return
	}
	for d := r.Cwd; within(d, r.Top); d = filepath.Dir(d) {
		rel, _ := filepath.Rel(r.Top, d)
		for _, t := range all {
			f := filepath.Join(t, rel, ConfigName)
			if t == r.Top || !isFile(f) {
				continue
			}
			dir, cfg, _, err := configDir(f)
			if err != nil || !isDir(dir) {
				continue
			}
			r.Roots = append(r.Roots, &Root{Dir: canon(dir), Owner: d, Config: f, Branch: strings.TrimSpace(cfg.Branch)})
			return
		}
		if d == r.Top {
			break
		}
	}
}

// includes adds the yass.yaml of each team folder named in the clone's git config (yass.include):
// folders git never sees, like a private/ kept out of the repo, so git's file list can't find them.
// Git config is shared by every worktree of the clone; a worktree without the folder reads the
// yass.yaml from another checkout, and gets its own copy of the folder (for the link) later.
func (r *Repo) includes(configs map[string]bool) (included map[string]bool, borrowed []*Root) {
	included = map[string]bool{}
	out, ok := git(r.Top, "config", "--get-all", "yass.include")
	if !ok {
		return included, nil
	}
	var others []string
	for _, rel := range lines(strings.TrimSpace(out)) {
		rel = strings.Trim(filepath.Clean(filepath.FromSlash(strings.TrimSpace(rel))), string(filepath.Separator))
		if rel == "" || rel == "." {
			continue
		}
		owner := filepath.Join(r.Top, rel)
		if f := filepath.Join(owner, ConfigName); isFile(f) {
			configs[f], included[f] = true, true
			continue
		}
		if others == nil {
			others, _ = worktrees(r.Top)
		}
		found := false
		for _, t := range others {
			f := filepath.Join(t, rel, ConfigName)
			if t == r.Top || !isFile(f) {
				continue
			}
			dir, cfg, _, err := configDir(f)
			if err != nil || !isDir(dir) {
				continue
			}
			borrowed = append(borrowed, &Root{Dir: canon(dir), Owner: owner, Config: f, Branch: strings.TrimSpace(cfg.Branch), Included: true})
			found = true
			break
		}
		if !found {
			r.warn("git config yass.include: '%s' has no %s in any checkout of this clone (to stop including it: git config --unset yass.include '^%s$')",
				filepath.ToSlash(rel), ConfigName, regexp.QuoteMeta(filepath.ToSlash(rel)))
		}
	}
	return included, borrowed
}

// link keeps a yass/ symlink next to each yass.yaml whose folder is outside the repo, so people,
// editors and agents find the plans at yass/ as they would anywhere else. It's ignored through the
// clone's info/exclude, which every worktree shares and nobody commits. Nothing depends on it:
// yass.yaml stays the source of truth, and a real yass/ already there is left alone.
func (r *Repo) link() {
	for _, root := range r.Roots {
		ln := filepath.Join(root.Owner, "yass")
		if root.Config == "" || within(root.Dir, r.Top) {
			// Plans moved into the repo: a link left from before points somewhere else.
			if t, err := os.Readlink(ln); err == nil && root.Config != "" && t != root.Dir && !within(t, r.Top) {
				r.warn("%s/ is a link to %s, but %s points to %s; remove the link", r.disp(ln), t, r.disp(root.Config), r.disp(root.Dir))
			}
			continue
		}
		if fi, err := os.Lstat(ln); err == nil {
			if fi.Mode()&os.ModeSymlink == 0 {
				// A yass/ with changes in it is already reported as ignored; say it once.
				if !isRealYassDir(ln) {
					r.warn("%s/: already there, so it isn't a link to %s", r.disp(ln), root.Dir)
				}
				continue
			}
			if isSymlinkTo(ln, root.Dir) {
				r.exclude(ln)
				continue
			}
			if err := os.Remove(ln); err != nil {
				r.note("couldn't repoint %s/ to %s: %v", r.disp(ln), root.Dir, err)
				continue
			}
		}
		if root.Included { // a worktree's own copy of the team folder, which git never creates
			os.MkdirAll(root.Owner, 0o755)
		}
		if err := os.Symlink(root.Dir, ln); err != nil {
			r.note("couldn't link %s/ to %s (%v); `yass root` prints where the plans are", r.disp(ln), root.Dir, err)
			continue
		}
		r.exclude(ln)
	}
}

// isRealYassDir is a yass folder on disk that isn't a link.
func isRealYassDir(p string) bool {
	fi, err := os.Lstat(p)
	return err == nil && fi.IsDir() && (isDir(filepath.Join(p, "changes")) || isDir(filepath.Join(p, "archive")))
}

func isSymlinkTo(p, target string) bool {
	t, err := os.Readlink(p)
	return err == nil && t == target
}

// exclude adds a path to the clone's info/exclude, once.
func (r *Repo) exclude(p string) {
	out, ok := git(r.Top, "rev-parse", "--path-format=absolute", "--git-path", "info/exclude")
	if !ok {
		return
	}
	f := filepath.FromSlash(strings.TrimSpace(out))
	rel, err := filepath.Rel(r.Top, p)
	if err != nil {
		return
	}
	entry := "/" + filepath.ToSlash(rel)
	cur := readText(f)
	if contains(lines(cur), entry) {
		return
	}
	if cur != "" && !strings.HasSuffix(cur, "\n") {
		cur += "\n"
	}
	if err := write(f, cur+entry+"\n"); err != nil {
		r.note("couldn't add %s to %s: %v", entry, f, err)
	}
}
