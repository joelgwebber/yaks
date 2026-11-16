package yass

import (
	"fmt"
	"strconv"
	"strings"
)

// Plans are separate when their yass folder isn't in the code repo's git work tree: in another
// repo, or in no repo at all (design §1). A linked worktree of the same repo counts as the same
// repo, so plans in the main checkout are inline from every worktree.

// gitCommonDir is the shared .git folder of the repo dir is in, or "" outside git.
func gitCommonDir(dir string) string {
	out, ok := git(dir, "rev-parse", "--path-format=absolute", "--git-common-dir")
	if !ok {
		return ""
	}
	return canon(strings.TrimSpace(out))
}

// separate says whether a yass folder's plans live outside the code repo. A folder inside this
// repo's work tree is inline without asking git, so this stays cheap with hundreds of team folders;
// only folders elsewhere, or private ones git never sees, are looked up.
func (r *Repo) separate(root *Root) bool {
	if r.codeDir == nil {
		d := gitCommonDir(r.Cwd)
		r.codeDir = &d
	}
	if *r.codeDir == "" {
		return false // no code repo to be separate from
	}
	if within(root.Dir, r.Top) && !root.Included {
		return false
	}
	return gitCommonDir(root.Dir) != *r.codeDir
}

// plansNotes notes, for each separate plans folder in git, uncommitted files there and how far it
// is behind its upstream as of the last fetch. It never fetches.
func (r *Repo) plansNotes() []string {
	var lines []string
	for _, root := range r.Roots {
		if !r.separate(root) {
			continue
		}
		lines = append(lines, fmt.Sprintf("plans: separate (%s)", root.Dir))
		if gitCommonDir(root.Dir) == "" {
			continue
		}
		if out, ok := git(root.Dir, "status", "--porcelain", "--", "."); ok {
			if n := len(strings.Split(strings.TrimSpace(out), "\n")); strings.TrimSpace(out) != "" {
				r.note("%s has %d uncommitted file(s); commit them there, where the plans are kept", root.Dir, n)
			}
		}
		if up, ok := git(root.Dir, "rev-parse", "--abbrev-ref", "--symbolic-full-name", "@{u}"); ok {
			up = strings.TrimSpace(up)
			if out, ok := git(root.Dir, "rev-list", "--count", "HEAD..@{u}"); ok {
				if n, _ := strconv.Atoi(strings.TrimSpace(out)); n > 0 {
					r.note("%s is %d commit(s) behind %s as of its last fetch; pull there (git -C %s pull)", root.Dir, n, up, root.Dir)
				}
			}
		}
	}
	return lines
}

// kind is how yass root -v describes a yass folder.
func (r *Repo) kind(root *Root) string {
	if r.separate(root) {
		return "separate"
	}
	return "inline"
}
