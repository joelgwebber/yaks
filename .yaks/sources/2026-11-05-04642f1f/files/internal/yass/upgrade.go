package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"golang.org/x/mod/semver"
)

// What yass upgrade does with one file, given its stamp and the binary's version (design §1).
const (
	actWrite = "write" // older, unstamped, or the same version from a +dirty binary
	actKeep  = "keep"  // already at the binary's version
	actNewer = "newer" // written by a newer yass: upgrade the binary instead
)

func upgradeAction(stamp, bin string) string {
	b, s := semverOf(bin), semverOf(stamp)
	switch {
	case s == "":
		return actWrite
	case semver.Compare(s, b) < 0:
		return actWrite
	case semver.Compare(s, b) > 0:
		return actNewer
	case strings.HasSuffix(bin, "+dirty"):
		return actWrite
	}
	return actKeep
}

// noVersion explains why a binary without a version can't write stamped files.
func noVersion() error {
	return fmt.Errorf("this yass has no version (%s), so it can't stamp YASS's files; build it from a git clone"+
		" with Go 1.24 or later (go build -o bin/yass ./cmd/yass), or install a release", binVersion)
}

var (
	repoSkillRE = regexp.MustCompile(`(^|/)\.(agents|claude)/skills/yass-[^/]+/SKILL\.md$`)
	repoAgentRE = regexp.MustCompile(`(^|/)AGENTS\.md$`)
)

// upgradeTarget is one file yass upgrade looks after.
type upgradeTarget struct {
	path, kind string // kind: skill, agents or hook
	outside    bool   // not in the repo, so not under version control (user-folder playbooks)
}

// findYassFiles lists every YASS file in the repo, from the root down (tracked or not, minus what
// .gitignore and yass.yaml's ignore: leave out), plus the user folder's playbooks (design §6). A
// skills folder holding any yass-* playbook gets every playbook the binary carries, so an upgrade
// also adds playbooks a newer version introduces.
func findYassFiles(r *Repo) []upgradeTarget {
	out, _ := git(r.Top, "ls-files", "--cached", "--others", "--exclude-standard", "-z")
	skillDirs := map[string]bool{}
	var found []upgradeTarget
	for _, rel := range strings.Split(out, "\x00") {
		if rel == "" {
			continue
		}
		p := filepath.Join(r.Top, filepath.FromSlash(rel))
		if r.ignored(p) || !exists(p) {
			continue
		}
		switch {
		case repoSkillRE.MatchString(rel):
			skillDirs[filepath.Dir(filepath.Dir(p))] = false
		case repoAgentRE.MatchString(rel) && strings.Contains(read(p), "<!-- yass:begin"):
			found = append(found, upgradeTarget{path: p, kind: "agents"})
		case rel == HookPath+"/pre-commit":
			// the folder holds YASS's hooks: upgrade them all, adding any a newer version ships
			for _, name := range hookScripts {
				found = append(found, upgradeTarget{path: filepath.Join(filepath.Dir(p), name), kind: "hook"})
			}
		}
	}
	for _, d := range []string{userSkillsDir(), userClaudeSkillsDir()} {
		if m, _ := filepath.Glob(filepath.Join(d, "yass-*", "SKILL.md")); len(m) > 0 {
			skillDirs[d] = true
		}
	}
	names, _ := kitSkills()
	var dirs []string
	for d := range skillDirs {
		dirs = append(dirs, d)
	}
	sort.Strings(dirs)
	for _, d := range dirs {
		for _, n := range names {
			found = append(found, upgradeTarget{path: filepath.Join(d, n, "SKILL.md"), kind: "skill", outside: skillDirs[d]})
		}
	}
	sort.SliceStable(found, func(i, j int) bool { return found[i].path < found[j].path })
	return found
}

// newText is what yass upgrade writes for a target: the binary's own version of it, stamped.
func newText(t upgradeTarget) string {
	v := stampVersion()
	switch t.kind {
	case "agents":
		block := stampAgents(strings.TrimSpace(render("agents.md", nil)), v)
		return agentsRE.ReplaceAllLiteralString(read(t.path), block)
	case "hook":
		b, _ := kitFile(HookPath + "/" + filepath.Base(t.path))
		return stampHook(b, v)
	}
	_, text := kitSkills()
	return stampSkill(text[filepath.Base(filepath.Dir(t.path))], v)
}

// cmdUpgrade writes YASS's files at this binary's version, then brings each yass folder's
// layout up to date (migrate.go).
func cmdUpgrade(a *args) (int, error) {
	code, err := upgradeFiles()
	if err != nil || code != 0 {
		return code, err
	}
	r := findRepo()
	return 0, migrate(r, migrations)
}

func upgradeFiles() (int, error) {
	if semverOf(binVersion) == "" {
		return 0, noVersion()
	}
	r := findRepo()
	if _, ok := git(r.Cwd, "rev-parse", "--show-toplevel"); !ok {
		return 0, fmt.Errorf("yass upgrade works inside a git repo: it finds YASS's files from the repo root down")
	}
	targets := findYassFiles(r)
	if len(targets) == 0 {
		fmt.Println("no YASS files here to upgrade; `yass init` sets a repo up")
		return 0, nil
	}
	var write []upgradeTarget
	from := map[string]bool{}
	for _, t := range targets {
		stamp := ""
		if exists(t.path) {
			stamp = readStamp(t.path)
		}
		switch upgradeAction(stamp, binVersion) {
		case actNewer:
			fmt.Fprintf(os.Stderr, "yass: %s was written by yass %s, newer than this one (%s); `yass update` gets that version, then run yass upgrade\n",
				r.disp(t.path), stamp, binVersion)
			return 1, nil
		case actWrite:
			write = append(write, t)
			if !exists(t.path) {
				from["(new)"] = true
			} else if stamp == "" {
				from["unstamped (v0.2 or earlier)"] = true
			} else {
				from[stamp] = true
			}
		}
	}
	if len(write) == 0 {
		fmt.Printf("already up to date: YASS's files are at %s\n", stampVersion())
		return 0, nil
	}
	for _, t := range write {
		if err := write1(t); err != nil {
			return 0, err
		}
		fmt.Printf("wrote %s%s\n", r.disp(t.path), outsideNote(t.outside))
	}
	var olds []string
	for v := range from {
		if v != "(new)" {
			olds = append(olds, v)
		}
	}
	sort.Strings(olds)
	if len(olds) > 0 {
		fmt.Printf("upgraded YASS's files from %s to %s; review the diff and commit it (chore: upgrade YASS)\n", strings.Join(olds, ", "), stampVersion())
	} else {
		fmt.Printf("added YASS's files at %s; review the diff and commit it (chore: upgrade YASS)\n", stampVersion())
	}
	return 0, nil
}

func write1(t upgradeTarget) error {
	if err := write(t.path, newText(t)); err != nil {
		return err
	}
	if t.kind == "hook" {
		return os.Chmod(t.path, 0o755)
	}
	return nil
}

func outsideNote(outside bool) string {
	if outside {
		return " (outside version control)"
	}
	return ""
}

// stampedFiles are the YASS files at the repo root that carry a version stamp: the AGENTS.md
// section, the playbooks in .agents/ and .claude/, the hook; and, with user, the user folder's
// playbooks. Only the root, so it stays fast in large repos.
func stampedFiles(r *Repo, user bool) []string {
	var paths []string
	if ap := filepath.Join(r.Top, "AGENTS.md"); strings.Contains(read(ap), "<!-- yass:begin") {
		paths = append(paths, ap)
	}
	dirs := []string{filepath.Join(r.Top, ".agents", "skills"), filepath.Join(r.Top, ".claude", "skills")}
	if user {
		dirs = append(dirs, userSkillsDir(), userClaudeSkillsDir())
	}
	for _, d := range dirs {
		m, _ := filepath.Glob(filepath.Join(d, "yass-*", "SKILL.md"))
		paths = append(paths, m...)
	}
	if hp := filepath.Join(r.Top, filepath.FromSlash(HookPath), "pre-commit"); exists(hp) {
		paths = append(paths, hp)
	}
	return paths
}

// repoVersion is the newest version stamped in the repo's own YASS files, or "" outside a repo
// that uses YASS.
func repoVersion(r *Repo) string {
	newest := ""
	for _, p := range stampedFiles(r, false) {
		if v := semverOf(readStamp(p)); v != "" && (newest == "" || semver.Compare(v, newest) > 0) {
			newest = v
		}
	}
	return newest
}

// versionNote is yass status's one line about versions: whether this binary is older than the
// version that wrote the repo's YASS files (upgrade the binary), or newer (yass upgrade would
// upgrade them). It only reads the files at the repo root and the user folder's playbooks, so
// status stays fast in large repos; yass upgrade is what searches the whole repo.
func versionNote(r *Repo) string {
	if semverOf(binVersion) == "" {
		return ""
	}
	paths := stampedFiles(r, true)
	newest, older := "", false
	for _, p := range paths {
		stamp := readStamp(p)
		switch upgradeAction(stamp, binVersion) {
		case actNewer:
			if newest == "" || semver.Compare(semverOf(stamp), semverOf(newest)) > 0 {
				newest = stamp
			}
		case actWrite:
			if !strings.HasSuffix(binVersion, "+dirty") || semver.Compare(semverOf(stamp), semverOf(binVersion)) != 0 {
				older = true
			}
		}
	}
	switch {
	case newest != "":
		return fmt.Sprintf("YASS's files here were written by yass %s, newer than this one (%s); `yass update` gets that version", newest, binVersion)
	case older:
		return fmt.Sprintf("YASS's files here are older than this yass (%s); `yass upgrade` would upgrade them", binVersion)
	}
	return ""
}
