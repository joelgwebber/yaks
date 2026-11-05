package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// turnOnHook points core.hooksPath at YASS's hook folder, but only when the repo has no hooks setup
// of its own: setting core.hooksPath switches off every hook git would otherwise run (a local or
// global core.hooksPath, as husky uses, and hooks in .git/hooks). When there is one, it leaves it
// alone and prints how to run `yass hook` from it.
func turnOnHook(top string) error {
	effective, _ := git(top, "config", "--get", "core.hooksPath")
	local, _ := git(top, "config", "--local", "--get", "core.hooksPath")
	effective, local = strings.TrimSpace(effective), strings.TrimSpace(local)
	if effective == HookPath {
		fmt.Printf("hook  already on: core.hooksPath = %s\n", HookPath)
		return nil
	}
	switch {
	case local != "":
		fmt.Printf("hook  not turned on: this repo already has a hooks setup (core.hooksPath = %s), and pointing it at\n"+
			"      %s would switch those hooks off.%s\n", local, HookPath, wireIn(top, local))
		return nil
	case effective != "":
		fmt.Printf("hook  not turned on: your global core.hooksPath (%s) runs hooks for every repo, and setting one here\n"+
			"      would override it for this repo.%s\n"+
			"      Or, to use YASS's hook folder in this repo instead:  git config core.hooksPath %s\n", effective, wireIn(top, effective), HookPath)
		return nil
	}
	if found := activeHooks(top); len(found) > 0 {
		fmt.Printf("hook  not turned on: this clone already runs hooks from .git/hooks (%s), and pointing core.hooksPath\n"+
			"      at %s would switch them off.%s\n", strings.Join(found, ", "), HookPath, wireIn(top, ""))
		return nil
	}
	if _, ok := git(top, "config", "core.hooksPath", HookPath); !ok {
		return fmt.Errorf("couldn't set git config core.hooksPath (is this a git repo?)")
	}
	fmt.Printf("hook  core.hooksPath = %s (undo: git config --unset core.hooksPath)\n", HookPath)
	return nil
}

// activeHooks lists the hooks git would run from the clone's own hooks folder (.git/hooks, or the
// common one for a worktree): executable files that aren't git's *.sample examples.
func activeHooks(top string) []string {
	dir, ok := git(top, "rev-parse", "--git-path", "hooks")
	if !ok {
		return nil
	}
	dir = strings.TrimSpace(dir)
	if !filepath.IsAbs(dir) {
		dir = filepath.Join(top, dir)
	}
	entries, _ := os.ReadDir(dir)
	var found []string
	for _, e := range entries {
		info, err := e.Info()
		if err != nil || !info.Mode().IsRegular() || info.Mode()&0o111 == 0 || strings.HasSuffix(e.Name(), ".sample") {
			continue
		}
		found = append(found, e.Name())
	}
	sort.Strings(found)
	return found
}

// wireIn says how to run YASS's check from an existing hooks setup: a hook manager the repo uses,
// or else the pre-commit script in the hooks folder git runs (dir; "" for .git/hooks).
func wireIn(top, dir string) string {
	const indent = "\n      "
	switch {
	case strings.Contains(dir, ".husky") || isDir(filepath.Join(top, ".husky")):
		return indent + "To run YASS's check from husky, add this line to .husky/pre-commit:" + indent + "  yass hook"
	case exists(filepath.Join(top, "lefthook.yml")) || exists(filepath.Join(top, ".lefthook.yml")):
		return indent + "To run YASS's check from lefthook, add this to lefthook.yml:" + indent +
			"  pre-commit:" + indent + "    commands:" + indent + "      yass:" + indent + "        run: yass hook"
	case exists(filepath.Join(top, ".pre-commit-config.yaml")):
		return indent + "To run YASS's check from pre-commit, add this to .pre-commit-config.yaml under repos:" + indent +
			"  - repo: local" + indent + "    hooks:" + indent + "      - id: yass" + indent + "        name: yass hook" + indent +
			"        entry: yass hook" + indent + "        language: system" + indent + "        pass_filenames: false" + indent + "        always_run: true"
	}
	where := ".git/hooks/pre-commit"
	if dir != "" {
		where = filepath.ToSlash(filepath.Join(dir, "pre-commit"))
	}
	return indent + "To run YASS's check too, add this line to " + where + " (create it, executable, if it doesn't exist):" + indent +
		"  " + HookPath + "/pre-commit \"$@\""
}
