package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// yassPaths are the repo-relative globs that are YASS's: each yass folder inside the repo, and each
// yass.yaml. A plans folder outside the repo has none, and folders a yass.yaml ignores aren't YASS's.
// The hook scripts in tools/yass/ aren't either: changing them is changing code. Nor are team
// folders included through git config: git never sees them, so no commit contains them.
func yassPaths(r *Repo) []string {
	var out []string
	for _, root := range r.Roots {
		if !within(root.Dir, r.Top) || root.Included { // git never sees an included folder
			continue
		}
		if root.Dir == r.Top { // a plans repo (path: .): everything in it is YASS's
			return []string{"**"}
		}
		if rel, err := filepath.Rel(r.Top, root.Dir); err == nil {
			out = append(out, filepath.ToSlash(rel)+"/**")
		}
	}
	for _, f := range r.Configs {
		if rel, err := filepath.Rel(r.Top, f); err == nil && within(f, r.Top) {
			out = append(out, filepath.ToSlash(rel))
		}
	}
	sort.Strings(out)
	return out
}

// isYassPath says whether a repo-relative path is under one of the globs yassPaths prints.
func isYassPath(p string, globs []string) bool {
	for _, g := range globs {
		if g == "**" {
			return true
		}
		if dir, ok := strings.CutSuffix(g, "/**"); ok {
			if strings.HasPrefix(p, dir+"/") {
				return true
			}
		} else if p == g {
			return true
		}
	}
	return false
}

// cmdPaths prints YASS's paths, or with --only, says whether a commit range changed nothing else:
// exit 0 when it's plans only (CI can skip builds and tests), 1 when anything else changed (or
// nothing did: when in doubt, run the checks), 2 when the range can't be read.
func cmdPaths(a *args) int {
	r := findRepo()
	globs := yassPaths(r)
	rng, only := a.v["only"]
	if !only {
		for _, g := range globs {
			fmt.Println(g)
		}
		return 0
	}
	out, ok := git(r.Top, "diff", "--name-only", "--no-renames", "-z", rng)
	if rng == "" || !ok {
		fmt.Fprintf(os.Stderr, "yass: can't read the range '%s'. In CI, fetch history first (e.g. actions/checkout with fetch-depth: 0).\n", rng)
		return 2
	}
	var other []string
	n := 0
	for _, f := range strings.Split(out, "\x00") {
		if f == "" {
			continue
		}
		n++
		if !isYassPath(f, globs) {
			other = append(other, f)
		}
	}
	switch {
	case n == 0:
		fmt.Println("nothing changed in the range")
		return 1
	case len(other) == 0:
		fmt.Printf("plans only: %d file(s), all YASS's\n", n)
		return 0
	}
	fmt.Printf("%d of %d changed file(s) aren't YASS's, e.g. %s\n", len(other), n, other[0])
	return 1
}
