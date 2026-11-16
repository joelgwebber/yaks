package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"

	"golang.org/x/mod/semver"
)

// The branch view (design §7): which unmerged local and remote-tracking branches with recent
// commits touch each change or piece, who touched it last, and how far that branch has it. It reads
// only what git already has (no fetch, no partial-clone download), in a fixed number of git
// processes: for-each-ref, one log over every branch, one cat-file for the files at their tips.

type branchRow struct {
	branch, author string
	when           time.Time
	done, total    int
	next           string
	unread         bool // the files at the branch tip aren't local (a partial clone): no progress
}

type branchView struct {
	rows  map[string][]branchRow // by change or piece folder, repo-relative
	fresh string                 // how old the remote-tracking refs are
}

var sinceDaysRE = regexp.MustCompile(`^(\d+)d$`)

// parseSince reads --since: Nd, or a date like 2026-10-01.
func parseSince(v string, now time.Time) (time.Time, error) {
	if v == "" {
		return now.AddDate(0, 0, -30), nil
	}
	if m := sinceDaysRE.FindStringSubmatch(v); m != nil {
		n, _ := strconv.Atoi(m[1])
		return now.AddDate(0, 0, -n), nil
	}
	if t, err := time.Parse("2006-01-02", v); err == nil {
		return t, nil
	}
	return time.Time{}, fmt.Errorf("--since takes days (14d) or a date (2026-10-01)")
}

func gitAtLeast(v string) bool {
	out, ok := git("", "version")
	if !ok {
		return false
	}
	f := strings.Fields(out) // git version 2.54.0 (Apple Git-157)
	if len(f) < 3 {
		return false
	}
	return semver.Compare(semverOf(f[2]), v) >= 0
}

// branches builds the view for the yass folders inside this repo. It returns nil when there's
// nothing unmerged to show.
func (r *Repo) branches(since time.Time) *branchView {
	var paths []string
	var root *Root
	for _, rt := range r.Roots {
		if within(rt.Dir, r.Top) {
			rel, _ := filepath.Rel(r.Top, filepath.Join(rt.Dir, "changes"))
			paths = append(paths, filepath.ToSlash(rel))
			if root == nil {
				root = rt
			}
		}
	}
	if root == nil {
		return nil
	}
	merged := r.mergedBranch(root)
	if merged == "" {
		return nil
	}
	local := strings.TrimPrefix(merged, "origin/") // the local copy of the merged branch is noise
	out, ok := git(r.Top, "for-each-ref", "--no-merged="+merged, "--format=%(refname:short)%09%(committerdate:unix)", "refs/heads", "refs/remotes")
	if !ok {
		return nil
	}
	var refs []string
	for _, l := range strings.Split(strings.TrimSpace(out), "\n") {
		name, ts, _ := strings.Cut(l, "\t")
		sec, _ := strconv.ParseInt(ts, 10, 64)
		if name == "" || name == merged || name == local || strings.HasSuffix(name, "/HEAD") || time.Unix(sec, 0).Before(since) {
			continue
		}
		refs = append(refs, name)
	}
	if len(refs) == 0 {
		return nil
	}
	// One log over every branch, limited to the yass folders' changes/: which branch reached each
	// commit (--source), who made it and when, and the files it touched.
	args := append([]string{"log", "--source", "--no-renames", "--format=%x01%S%x09%an%x09%ct", "--name-only",
		fmt.Sprintf("--since=@%d", since.Unix())}, refs...)
	args = append(append(append(args, "--not", merged), "--"), paths...)
	log, _ := git(r.Top, args...)
	type key struct{ branch, folder string }
	seen := map[key]*branchRow{}
	var order []key
	var ref, author string
	var when time.Time
	for _, l := range strings.Split(log, "\n") {
		if strings.HasPrefix(l, "\x01") {
			f := strings.SplitN(l[1:], "\t", 3)
			if len(f) == 3 {
				ref, author = f[0], f[1]
				sec, _ := strconv.ParseInt(f[2], 10, 64)
				when = time.Unix(sec, 0)
			}
			continue
		}
		if l == "" || ref == "" {
			continue
		}
		for _, folder := range changeFolders(l, paths) {
			k := key{ref, folder}
			if seen[k] == nil { // newest first: the first commit seen is the latest
				seen[k] = &branchRow{branch: ref, author: author, when: when}
				order = append(order, k)
			}
		}
	}
	if len(order) == 0 {
		return nil
	}
	// The files at each branch's tip, in one cat-file, never downloading what a partial clone lacks.
	partial, _ := git(r.Top, "config", "--get", "extensions.partialclone")
	canRead := strings.TrimSpace(partial) == "" || gitAtLeast("v2.44.0")
	var specs []string
	for _, k := range order {
		specs = append(specs, k.branch+":"+k.folder+"/change.md", k.branch+":"+k.folder+"/plan.md")
	}
	var blobs map[string]string
	if canRead {
		blobs = catFilesNoFetch(r.Top, specs)
	}
	v := &branchView{rows: map[string][]branchRow{}}
	for _, k := range order {
		row := seen[k]
		head, ok := blobs[k.branch+":"+k.folder+"/change.md"]
		if !ok {
			row.unread = true
		} else {
			for _, text := range []string{head, blobs[k.branch+":"+k.folder+"/plan.md"]} {
				for _, b := range boxes(normalize(text)) {
					if b.mark == markDropped {
						continue
					}
					row.total++
					if b.mark == markDone {
						row.done++
					}
				}
			}
			if m := nextRE.FindAllStringSubmatch(commentRE.ReplaceAllString(section(normalize(head), "Log"), ""), -1); len(m) > 0 {
				row.next = strings.TrimSpace(m[len(m)-1][1])
			}
		}
		v.rows[k.folder] = append(v.rows[k.folder], *row)
	}
	for f, rows := range v.rows {
		sort.SliceStable(rows, func(i, j int) bool { return rows[i].when.After(rows[j].when) })
		v.rows[f] = rows
	}
	v.fresh = r.fetchFreshness()
	return v
}

// changeFolders maps a file under a yass folder's changes/ to the change it's in and, when it's in
// a subfolder, that subfolder too (a piece, if it holds a change.md at the branch tip).
func changeFolders(file string, paths []string) []string {
	for _, p := range paths {
		if !strings.HasPrefix(file, p+"/") {
			continue
		}
		parts := strings.Split(strings.TrimPrefix(file, p+"/"), "/")
		if len(parts) < 2 {
			return nil
		}
		change := p + "/" + parts[0]
		if len(parts) >= 3 {
			return []string{change + "/" + parts[1]}
		}
		return []string{change}
	}
	return nil
}

// catFilesNoFetch is catFiles that never makes a partial clone download what it doesn't have.
func catFilesNoFetch(dir string, specs []string) map[string]string {
	return catFiles(dir, specs, "GIT_NO_LAZY_FETCH=1") // git 2.44+; older git is only asked outside partial clones
}

// fetchFreshness says how old the remote-tracking refs are: when FETCH_HEAD was last written.
func (r *Repo) fetchFreshness() string {
	out, _ := git(r.Top, "for-each-ref", "--count=1", "--format=%(refname)", "refs/remotes")
	if strings.TrimSpace(out) == "" {
		return "no remote branches here (nothing fetched); local branches only"
	}
	dir, _ := git(r.Top, "rev-parse", "--git-common-dir")
	d := strings.TrimSpace(dir)
	if !filepath.IsAbs(d) {
		d = filepath.Join(r.Top, d)
	}
	st, err := os.Stat(filepath.Join(d, "FETCH_HEAD"))
	if err != nil {
		return "remote branches as of the clone (never fetched since)"
	}
	return "remote branches as of the last fetch, " + ago(st.ModTime())
}

func ago(t time.Time) string {
	d := time.Since(t)
	switch {
	case d < time.Minute:
		return "just now"
	case d < time.Hour:
		return fmt.Sprintf("%d min ago", int(d.Minutes()))
	case d < 48*time.Hour:
		return fmt.Sprintf("%d h ago", int(d.Hours()))
	}
	return fmt.Sprintf("%d days ago", int(d.Hours()/24))
}

// branchRowMark marks a status row that's a branch line, printed as is rather than in columns.
const branchRowMark = "\x00branch"

// linesFor are a change's or piece's branch lines in yass status.
func (v *branchView) linesFor(r *Repo, c *Change, indent string) [][2]string {
	if v == nil {
		return nil
	}
	rel, err := filepath.Rel(r.Top, c.Path)
	if err != nil {
		return nil
	}
	var out [][2]string
	for _, l := range branchLines(v.rows[filepath.ToSlash(rel)], indent) {
		out = append(out, [2]string{branchRowMark, l})
	}
	return out
}

// branchLines are the lines for one change's or piece's branches.
func branchLines(rows []branchRow, indent string) []string {
	var out []string
	for _, b := range rows {
		line := fmt.Sprintf("%s↳ %s  %s, %s", indent, b.branch, b.author, ago(b.when))
		if b.unread {
			line += "  (its files aren't local)"
		} else if b.total > 0 {
			line += fmt.Sprintf("  %d/%d", b.done, b.total)
		}
		if b.next != "" {
			line += "  next: " + trunc(b.next, 60)
		}
		out = append(out, line)
	}
	return out
}
