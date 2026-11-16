package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// cmdDeinit takes YASS back out of a repo: what yass init and yass upgrade write (the playbooks,
// AGENTS.md sections, the CLAUDE.md import and the hook), from the repo root down. The yass folders
// and yass.yaml stay, since they're the repo's plans and history; it says how to remove them. With
// global, the user folder's playbooks go too. It never commits: everything it removes in the repo is
// in git, so the diff shows it and git restore undoes it.
func cmdDeinit(a *args) error {
	r := findRepo()
	var removed []string
	gone := func(p, what string) { removed = append(removed, what+r.disp(p)+outsideNote(!within(p, r.Top))) }

	out, _ := git(r.Top, "ls-files", "--cached", "--others", "--exclude-standard", "-z")
	skillDirs, agents, hookDir := map[string]bool{}, []string{}, ""
	for _, rel := range strings.Split(out, "\x00") {
		p := filepath.Join(r.Top, filepath.FromSlash(rel))
		if rel == "" || r.ignored(p) || !exists(p) {
			continue
		}
		switch {
		case repoSkillRE.MatchString(rel):
			skillDirs[filepath.Dir(p)] = true
		case repoAgentRE.MatchString(rel) && strings.Contains(read(p), "<!-- yass:begin"):
			agents = append(agents, p)
		case rel == HookPath+"/pre-commit":
			hookDir = filepath.Dir(p)
		}
	}
	userDirs := yassSkillDirs(userSkillsDir(), userClaudeSkillsDir())
	if a.b["global"] {
		for _, d := range userDirs {
			skillDirs[d] = true
		}
	}
	var dirs []string
	for d := range skillDirs {
		if yassPlaybook(filepath.Join(d, "SKILL.md")) {
			dirs = append(dirs, d)
		}
	}
	sort.Strings(dirs)
	for _, d := range dirs {
		if err := os.RemoveAll(d); err != nil {
			return err
		}
		pruneEmpty(filepath.Dir(d), r.Top)
		gone(d, "removed ")
	}

	sort.Strings(agents)
	for _, p := range agents {
		rest := strings.TrimSpace(agentsRE.ReplaceAllLiteralString(read(p), ""))
		if rest != "" {
			if err := write(p, rest+"\n"); err != nil {
				return err
			}
			gone(p, "removed the YASS section of ")
			continue
		}
		if err := os.Remove(p); err != nil {
			return err
		}
		gone(p, "removed ")
		// CLAUDE.md's @AGENTS.md import, which yass init --claude adds, now points at nothing.
		cm := filepath.Join(filepath.Dir(p), "CLAUDE.md")
		if cur := read(cm); strings.Contains(cur, "@AGENTS.md") {
			var keep []string
			for _, l := range lines(cur) {
				if strings.TrimSpace(l) != "@AGENTS.md" {
					keep = append(keep, l)
				}
			}
			if next := strings.TrimSpace(strings.Join(keep, "\n")); next == "" {
				if err := os.Remove(cm); err != nil {
					return err
				}
				gone(cm, "removed ")
			} else {
				if err := write(cm, next+"\n"); err != nil {
					return err
				}
				gone(cm, "removed the @AGENTS.md import from ")
			}
		}
	}

	if hookDir != "" {
		for _, name := range hookScripts {
			if p := filepath.Join(hookDir, name); exists(p) {
				if err := os.Remove(p); err != nil {
					return err
				}
				gone(p, "removed ")
			}
		}
		pruneEmpty(hookDir, r.Top)
	}
	if local, _ := git(r.Top, "config", "--local", "--get", "core.hooksPath"); strings.TrimSpace(local) == HookPath {
		git(r.Top, "config", "--local", "--unset", "core.hooksPath")
		removed = append(removed, "unset core.hooksPath (it pointed at "+HookPath+")")
	}
	if f := useFile(); f != "" {
		os.Remove(f) // yass use's choice for this worktree: never committed, nothing to list
	}

	for _, l := range removed {
		fmt.Println(l)
	}
	if len(removed) == 0 {
		fmt.Println("no YASS playbooks, AGENTS.md section or hook here to remove")
	}
	if !a.b["global"] && len(userDirs) > 0 {
		fmt.Printf("note: your user folder's playbooks stay (%s), since they serve every repo; `yass deinit --global` removes them too\n",
			strings.Join(userDirs, ", "))
	}
	for _, f := range hookCalls(r.Top) {
		fmt.Printf("note: %s still runs `yass hook`; remove that line by hand\n", f)
	}

	var kept, rm []string
	for _, root := range r.Roots {
		switch {
		case root.Config != "" && within(root.Dir, r.Top) && !root.Included:
			kept = append(kept, r.disp(root.Dir)+"/ and "+r.disp(root.Config))
			rm = append(rm, rmCmd(r, root.Dir, root.Config))
		case root.Config != "":
			kept = append(kept, r.disp(root.Config)+" (pointing to "+r.disp(root.Dir)+")")
			rm = append(rm, rmCmd(r, root.Config))
			if ln := filepath.Join(root.Owner, "yass"); isSymlinkTo(ln, root.Dir) {
				rm = append(rm, "rm "+r.disp(ln))
			}
		case within(root.Dir, r.Top) && !root.Included:
			kept = append(kept, r.disp(root.Dir)+"/")
			rm = append(rm, rmCmd(r, root.Dir))
		default:
			kept = append(kept, r.disp(root.Dir)+"/")
			rm = append(rm, "rm -r "+r.disp(root.Dir))
		}
	}
	if len(kept) > 0 {
		fmt.Printf("\nKept your plans: %s. They're the record of what was changed and why: keep them, or remove them:\n  %s\n",
			strings.Join(kept, ", "), strings.Join(rm, " && "))
	}
	if len(removed) > 0 {
		fmt.Println("\nReview it with `git status`, then commit it:  git add -A && git commit -m \"chore: remove YASS\"")
	}
	bin, _ := os.Executable()
	fmt.Printf("The yass binary stays (%s): delete it once you're done with YASS everywhere. `yass init` sets this repo up again.\n", bin)
	return nil
}

// rmCmd is the command that removes paths in the repo: git rm when git tracks them, else rm.
func rmCmd(r *Repo, paths ...string) string {
	args := []string{"ls-files", "--"}
	var shown []string
	for _, p := range paths {
		args, shown = append(args, p), append(shown, r.disp(p))
	}
	if out, _ := git(r.Top, args...); strings.TrimSpace(out) != "" {
		return "git rm -r -q " + strings.Join(shown, " ")
	}
	return "rm -r " + strings.Join(shown, " ")
}

// yassSkillDirs lists the YASS playbook folders (yass-*) in skills folders.
func yassSkillDirs(roots ...string) []string {
	var out []string
	for _, d := range roots {
		m, _ := filepath.Glob(filepath.Join(d, "yass-*", "SKILL.md"))
		for _, p := range m {
			if yassPlaybook(p) {
				out = append(out, filepath.Dir(p))
			}
		}
	}
	return out
}

// yassPlaybook says whether a SKILL.md is one of YASS's: a playbook this binary carries, or one
// stamped with a yass-version (a playbook an older or newer version carried).
func yassPlaybook(p string) bool {
	names, _ := kitSkills()
	return contains(names, filepath.Base(filepath.Dir(p))) || readStamp(p) != ""
}

// pruneEmpty removes dir and its parents while they're empty, stopping at top.
func pruneEmpty(dir, top string) {
	for within(dir, top) && dir != top {
		if os.Remove(dir) != nil {
			return
		}
		dir = filepath.Dir(dir)
	}
}

// hookCalls lists the hook-manager files at the repo root that still run `yass hook`.
func hookCalls(top string) []string {
	files, _ := filepath.Glob(filepath.Join(top, ".husky", "*"))
	files = append(files, filepath.Join(top, "lefthook.yml"), filepath.Join(top, ".lefthook.yml"), filepath.Join(top, ".pre-commit-config.yaml"))
	var out []string
	for _, f := range files {
		if isFile(f) && strings.Contains(read(f), "yass hook") {
			rel, _ := filepath.Rel(top, f)
			out = append(out, filepath.ToSlash(rel))
		}
	}
	return out
}
