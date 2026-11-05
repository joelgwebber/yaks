package yass

import (
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
)

// Reading evicted months takes a fixed number of git processes, however many changes they hold.
func TestReadEvictedGitProcesses(t *testing.T) {
	if _, err := exec.LookPath("git"); err != nil {
		t.Skip("no git")
	}
	dir := t.TempDir()
	run := func(args ...string) string {
		cmd := exec.Command("git", args...)
		cmd.Dir = dir
		cmd.Env = append(os.Environ(), "GIT_AUTHOR_NAME=t", "GIT_AUTHOR_EMAIL=t@t", "GIT_COMMITTER_NAME=t", "GIT_COMMITTER_EMAIL=t@t")
		out, err := cmd.CombinedOutput()
		if err != nil {
			t.Fatalf("git %v: %v\n%s", args, err, out)
		}
		return strings.TrimSpace(string(out))
	}
	run("init", "-q", "-b", "main")
	month := filepath.Join(dir, "yass", "archive", "2025", "01")
	const n = 1000
	var names []string
	for i := 0; i < n; i++ {
		name := fmt.Sprintf("2025-01-01-c%04d", i)
		names = append(names, name)
		os.MkdirAll(filepath.Join(month, name), 0o755)
		os.WriteFile(filepath.Join(month, name, "change.md"), []byte(fmt.Sprintf("# C%d\n\n## Decisions\n- Call %d (t, 2025-01-02)\n", i, i)), 0o644)
	}
	run("add", "-A")
	run("commit", "-q", "-m", "archive")
	sha := run("rev-parse", "HEAD")
	run("rm", "-r", "-q", "yass/archive/2025/01")
	os.MkdirAll(filepath.Dir(month), 0o755) // git rm takes the emptied folders with it
	os.WriteFile(month+evictedExt, []byte(manifestText(evictedMonth{Commit: sha, Path: "yass/archive/2025/01", Changes: names})), 0o644)

	root := &Root{Dir: canon(filepath.Join(dir, "yass"))}
	r := &Repo{Top: canon(dir), Roots: []*Root{root}}
	r.Archived = r.load(root, "archive", false)
	trace := filepath.Join(t.TempDir(), "trace")
	t.Setenv("GIT_TRACE", trace)
	if missing := r.readEvicted(map[*Root][]string{root: {"2025/01"}}); len(missing) > 0 {
		t.Fatalf("missing %v", missing)
	}
	t.Setenv("GIT_TRACE", "")
	text, _ := os.ReadFile(trace)
	procs := strings.Count(string(text), "trace: built-in: git ")
	if procs > 4 {
		t.Errorf("read %d evicted changes with %d git processes, want at most 4 (rev-parse, cat-file -e, ls-tree, cat-file --batch)", n, procs)
	}
	read := 0
	for _, c := range r.Archived {
		if len(decisionsOf(c)) == 1 {
			read++
		}
	}
	if read != n {
		t.Errorf("read decisions from %d of %d evicted changes", read, n)
	}
}
