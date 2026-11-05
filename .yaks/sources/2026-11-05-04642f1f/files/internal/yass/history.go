package yass

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strconv"
	"strings"
)

// Evicted months are read back from git only when a query reaches them, using the commit and path
// in each month's .evicted file (design §3): one git ls-tree per month, then one git cat-file
// --batch for every file, so the number of git processes doesn't grow with the number of changes.

// evictedStubs groups the name-only evicted changes by yass folder and month (<YYYY>/<MM>).
func (r *Repo) evictedStubs() map[*Root]map[string][]*Change {
	out := map[*Root]map[string][]*Change{}
	for _, c := range r.Archived {
		if !c.Evicted {
			continue
		}
		m := filepath.ToSlash(filepath.Dir(c.Path))
		m = m[len(m)-7:]
		if out[c.Root] == nil {
			out[c.Root] = map[string][]*Change{}
		}
		out[c.Root][m] = append(out[c.Root][m], c)
	}
	return out
}

// readEvicted fills in the evicted changes of the months named (by yass folder) from git: their
// change.md, their other .md files, and their pieces. It returns the months it couldn't read
// because their commit isn't in this clone.
func (r *Repo) readEvicted(want map[*Root][]string) []string {
	stubs := r.evictedStubs()
	var missing []string
	for root, months := range want {
		manifests := map[string]evictedMonth{}
		for _, m := range evictedMonths(root) {
			manifests[m.Month] = m
		}
		top, ok := git(root.Dir, "rev-parse", "--show-toplevel")
		if !ok {
			for _, m := range months {
				missing = append(missing, r.disp(filepath.Join(root.Dir, "archive", filepath.FromSlash(m))))
			}
			continue
		}
		top = strings.TrimSpace(top)
		var specs []string // <commit>:<path>
		for _, m := range months {
			man, ok := manifests[m]
			if !ok {
				continue
			}
			if _, ok := git(top, "cat-file", "-e", man.Commit+"^{commit}"); !ok {
				missing = append(missing, r.disp(filepath.Join(root.Dir, "archive", filepath.FromSlash(m))))
				continue
			}
			out, _ := git(top, "ls-tree", "-r", "--name-only", man.Commit, "--", man.Path)
			for _, f := range strings.Split(strings.TrimSpace(out), "\n") {
				if strings.HasSuffix(f, ".md") {
					specs = append(specs, man.Commit+":"+f)
				}
			}
		}
		blobs := catFiles(top, specs)
		for _, m := range months {
			man, ok := manifests[m]
			if !ok {
				continue
			}
			byName := map[string]*Change{}
			for _, c := range stubs[root][m] {
				byName[c.Name] = c
			}
			pieces := map[string]*Change{}
			for _, spec := range specs {
				f := strings.TrimPrefix(spec, man.Commit+":")
				if !strings.HasPrefix(spec, man.Commit+":") || !strings.HasPrefix(f, man.Path+"/") {
					continue
				}
				parts := strings.Split(strings.TrimPrefix(f, man.Path+"/"), "/")
				c := byName[parts[0]]
				if c == nil || len(parts) < 2 || len(parts) > 3 {
					continue
				}
				if len(parts) == 3 { // a piece's file
					key := parts[0] + "/" + parts[1]
					if pieces[key] == nil {
						p := nameOnly(filepath.Join(c.Path, parts[1]), root, c)
						p.Archived, p.Evicted, p.files = true, true, map[string]string{}
						pieces[key] = p
						c.Pieces = append(c.Pieces, p)
					}
					c = pieces[key]
				}
				if c.files == nil {
					c.files = map[string]string{}
				}
				name := parts[len(parts)-1]
				c.files[name] = normalize(blobs[spec])
				if name == "change.md" {
					c.setHead(c.files[name])
				}
				if name == "prd.md" || name == "plan.md" {
					c.Large = true
				}
			}
			for _, c := range stubs[root][m] {
				byCreated(c.Pieces)
			}
		}
	}
	return missing
}

// catFiles reads many files from git in one process: `git cat-file --batch`, by <commit>:<path>.
func catFiles(dir string, specs []string, env ...string) map[string]string {
	out := map[string]string{}
	if len(specs) == 0 {
		return out
	}
	cmd := exec.Command("git", "cat-file", "--batch")
	cmd.Dir = dir
	if len(env) > 0 {
		cmd.Env = append(os.Environ(), env...)
	}
	cmd.Stdin = strings.NewReader(strings.Join(specs, "\n") + "\n")
	pipe, err := cmd.StdoutPipe()
	if err != nil || cmd.Start() != nil {
		return out
	}
	rd := bufio.NewReader(pipe)
	for _, spec := range specs {
		header, err := rd.ReadString('\n')
		if err != nil {
			break
		}
		fields := strings.Fields(header)
		if len(fields) != 3 { // "<spec> missing"
			continue
		}
		size, _ := strconv.Atoi(fields[2])
		buf := make([]byte, size+1) // the content, then a newline
		if _, err := io.ReadFull(rd, buf); err != nil {
			break
		}
		out[spec] = string(buf[:size])
	}
	cmd.Wait()
	return out
}

// monthEnd is the last day a month (<YYYY>/<MM>) could hold, as a date that sorts with others.
func monthEnd(m string) string { return strings.Replace(m, "/", "-", 1) + "-31" }

// missingNote says which evicted months a query couldn't read, and how to get them.
func missingNote(missing []string) string {
	return fmt.Sprintf("couldn't read %s: evicted, and their commit isn't in this clone (`git fetch --unshallow` gets it)",
		strings.Join(sortedCopy(missing), ", "))
}
