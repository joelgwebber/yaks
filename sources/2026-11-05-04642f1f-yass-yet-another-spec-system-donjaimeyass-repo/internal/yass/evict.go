package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	"gopkg.in/yaml.v3"
)

// Eviction moves a yass folder's oldest archived months out of the working tree into git history
// when the folder is past its keep: setting. Each evicted month leaves archive/<YYYY>/<MM>.evicted,
// written once, naming its changes and the commit that still has them (design §3–4).

const evictedExt = ".evicted"

type evictedMonth struct {
	Month   string   `yaml:"-"`       // 2026/03
	Commit  string   `yaml:"commit"`  // the last commit that touched the month's folder
	Path    string   `yaml:"path"`    // the month's folder at that commit, from the top of its git repo
	Changes []string `yaml:"changes"` // top-level changes, sorted
}

// manifestText is a .evicted file. It's the same for the same month from the same history, so two
// branches that evict a month write identical files and merge cleanly.
func manifestText(m evictedMonth) string {
	var b strings.Builder
	b.WriteString("# Evicted by yass evict. Read a change back: git show <commit>:<path>/<name>/change.md\n")
	fmt.Fprintf(&b, "commit: %s\npath: %s\nchanges:\n", m.Commit, m.Path)
	for _, c := range m.Changes {
		fmt.Fprintf(&b, "  - %s\n", c)
	}
	return b.String()
}

// evictedMonths reads a yass folder's .evicted files, oldest month first.
func evictedMonths(root *Root) []evictedMonth {
	base := filepath.Join(root.Dir, "archive")
	var out []evictedMonth
	for _, y := range subdirs(base) {
		if !yearRE.MatchString(y) {
			continue
		}
		files, _ := filepath.Glob(filepath.Join(base, y, "*"+evictedExt))
		sort.Strings(files)
		for _, f := range files {
			mm := strings.TrimSuffix(filepath.Base(f), evictedExt)
			if !monthRE.MatchString(mm) {
				continue
			}
			var m evictedMonth
			if yaml.Unmarshal([]byte(read(f)), &m) != nil {
				continue
			}
			m.Month = y + "/" + mm
			out = append(out, m)
		}
	}
	return out
}

// archiveMonths lists the month folders in a yass folder's archive that hold changes, oldest
// first, with how many top-level changes each holds.
func archiveMonths(root *Root) ([]string, map[string]int) {
	base := filepath.Join(root.Dir, "archive")
	var months []string
	count := map[string]int{}
	for _, y := range subdirs(base) {
		if !yearRE.MatchString(y) {
			continue
		}
		for _, m := range subdirs(filepath.Join(base, y)) {
			if !monthRE.MatchString(m) {
				continue
			}
			if n := len(subdirs(filepath.Join(base, y, m))); n > 0 {
				months = append(months, y+"/"+m)
				count[y+"/"+m] = n
			}
		}
	}
	return months, count
}

// kept is how many archived changes a yass folder holds in its working tree.
func (r *Repo) kept(root *Root) int {
	n := 0
	for _, c := range r.Archived {
		if c.Root == root && !c.Evicted {
			n++
		}
	}
	return n
}

// pastKeep notes each yass folder holding more archived changes than its keep: setting.
func (r *Repo) pastKeep() {
	for _, root := range r.Roots {
		if n, k := r.kept(root), root.keep(); n > k {
			r.note("%s holds %d archived changes, %d past its keep of %d; `yass evict` moves its oldest months to git history",
				r.disp(filepath.Join(root.Dir, "archive")), n, n-k, k)
		}
	}
}

func cmdEvict(a *args) error {
	r := loadRepo()
	if len(r.Roots) == 0 {
		return fmt.Errorf("%s", r.noRoot("no yass folder here; `yass init` sets one up"))
	}
	current := time.Now().UTC().Format("2006/01")
	over := false
	for _, root := range r.Roots {
		n, k := r.kept(root), root.keep()
		where := r.disp(filepath.Join(root.Dir, "archive"))
		if n <= k {
			continue
		}
		over = true
		top, ok := git(root.Dir, "rev-parse", "--show-toplevel")
		if !ok {
			return fmt.Errorf("%s isn't in git, so evicting would delete changes for good; it holds %d, %d past its keep of %d", where, n, n-k, k)
		}
		gitTop := canon(filepath.FromSlash(strings.TrimSpace(top)))
		// Decide every month first, and check each, so a refusal changes nothing.
		months, count := archiveMonths(root)
		var plan []evictedMonth
		left := n
		for _, m := range months {
			if left <= k || m == current {
				break
			}
			dir := filepath.Join(root.Dir, "archive", filepath.FromSlash(m))
			if out, _ := git(root.Dir, "status", "--porcelain", "--", dir); strings.TrimSpace(out) != "" {
				return fmt.Errorf("%s has changes that aren't committed; commit them first, then evict", r.disp(dir))
			}
			sha, _ := git(root.Dir, "log", "-1", "--format=%H", "--", dir)
			if strings.TrimSpace(sha) == "" {
				return fmt.Errorf("%s has never been committed, so evicting it would delete it for good", r.disp(dir))
			}
			rel, _ := filepath.Rel(gitTop, canon(dir))
			plan = append(plan, evictedMonth{Month: m, Commit: strings.TrimSpace(sha), Path: filepath.ToSlash(rel), Changes: subdirs(dir)})
			left -= count[m]
		}
		if len(plan) == 0 {
			hint := "only this month's"
			if flatArchived(root) {
				hint = "the rest sit directly in archive/ (`yass upgrade` moves them into months)"
			}
			fmt.Printf("%s holds %d archived changes, %d past its keep of %d, but has no month to evict: %s\n", where, n, n-k, k, hint)
			continue
		}
		for _, m := range plan {
			dir := filepath.Join(root.Dir, "archive", filepath.FromSlash(m.Month))
			if _, ok := git(root.Dir, "rm", "-r", "-q", "--", dir); !ok {
				return fmt.Errorf("couldn't remove %s with git rm", r.disp(dir))
			}
			os.RemoveAll(dir)
			f := dir + evictedExt
			if err := write(f, manifestText(m)); err != nil {
				return err
			}
			git(root.Dir, "add", "--", f)
			fmt.Printf("evicted %s: %d archived change(s), still in git at %s\n", r.disp(dir), len(m.Changes), m.Commit[:12])
		}
		fmt.Printf("%s now holds %d archived changes (keep: %d)\n", where, left, k)
		cmd := fmt.Sprintf("git commit -m \"yass: evict archived months\" -- %s", r.disp(filepath.Join(root.Dir, "archive")))
		if gitTop != r.Top {
			cmd = fmt.Sprintf("git -C %s commit -m \"yass: evict archived months\" -- %s", gitTop, filepath.Join(root.Dir, "archive"))
		}
		fmt.Printf("Commit it on its own: %s\n", cmd)
	}
	if !over {
		fmt.Println("nothing to evict: every yass folder is within its keep")
	}
	return nil
}
