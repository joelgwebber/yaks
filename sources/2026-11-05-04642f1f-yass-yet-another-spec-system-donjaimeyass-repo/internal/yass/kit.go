package yass

import (
	"fmt"
	"io/fs"
	"os"
	"path"
	"path/filepath"
	"regexp"
	"sort"
	"strings"

	"golang.org/x/mod/semver"

	"github.com/donjaime/yass/kit"
)

// binVersion is this binary's version, as Main receives it; the stamps written into a repo use it.
var binVersion = "dev"

// binChannel is "release" for binaries built by the release workflow, which yass update may replace.
var binChannel = ""

// HookPath is where the hook script lives in a repo, and what --hooks points core.hooksPath at.
const HookPath = "tools/yass/githooks"

var metadataRE = regexp.MustCompile(`(?m)^metadata:[ \t]*$`)

// stampVersion is the version as it's written into files: without a leading "v".
func stampVersion() string { return strings.TrimPrefix(binVersion, "v") }

// stampSkill adds metadata.yass-version to a playbook's frontmatter (the Agent Skills spec's
// extension point), replacing any yass-version already there.
func stampSkill(text, version string) string {
	m := fmRE.FindStringSubmatchIndex(text)
	if m == nil {
		return text
	}
	front := text[m[2]:m[3]]
	line := fmt.Sprintf("  yass-version: %q", version)
	var out []string
	for _, l := range lines(front) {
		if !strings.HasPrefix(strings.TrimSpace(l), "yass-version:") {
			out = append(out, l)
		}
	}
	front = strings.Join(out, "\n")
	if loc := metadataRE.FindStringIndex(front); loc != nil {
		front = front[:loc[1]] + "\n" + line + front[loc[1]:]
	} else {
		front += "\nmetadata:\n" + line
	}
	return text[:m[2]] + front + text[m[3]:]
}

// stampHook adds a "# yass-version:" line after the hook script's first line.
func stampHook(text, version string) string {
	first, rest, _ := strings.Cut(text, "\n")
	return first + "\n# yass-version: " + version + "\n" + rest
}

// stampAgents adds version=V to the AGENTS.md section's begin marker.
func stampAgents(block, version string) string {
	return strings.Replace(block, "<!-- yass:begin ", "<!-- yass:begin version="+version+" ", 1)
}

// kitSkills lists the playbooks the binary carries: name and unstamped text, sorted by name.
func kitSkills() (names []string, text map[string]string) {
	text = map[string]string{}
	matches, _ := fs.Glob(kit.FS, ".agents/skills/*/SKILL.md")
	for _, m := range matches {
		b, _ := kit.FS.ReadFile(m)
		name := path.Base(path.Dir(m))
		names, text[name] = append(names, name), string(b)
	}
	sort.Strings(names)
	return names, text
}

// kitFile reads one file the binary carries.
func kitFile(p string) (string, error) {
	b, err := kit.FS.ReadFile(p)
	return string(b), err
}

// userSkillsDir is where --global puts the playbooks: $YASS_SKILLS_DIR, or ~/.agents/skills.
func userSkillsDir() string {
	if d := os.Getenv("YASS_SKILLS_DIR"); d != "" {
		return d
	}
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".agents", "skills")
}

func userClaudeSkillsDir() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".claude", "skills")
}

// kitWriter writes stamped kit files, keeping any that already exist; it remembers which it
// wrote and which it kept, so init can list the first and check the second's versions.
type kitWriter struct{ made, kept []string }

func (w *kitWriter) place(p, text string, mode os.FileMode) error {
	if exists(p) {
		w.kept = append(w.kept, p)
		return nil
	}
	if err := write(p, text); err != nil {
		return err
	}
	if mode != 0 {
		if err := os.Chmod(p, mode); err != nil {
			return err
		}
	}
	w.made = append(w.made, p)
	return nil
}

// agentFiles writes the playbooks into dir's .agents/skills (or the user folder, with global) and,
// with claude, Claude Code's copies and the CLAUDE.md import in dir. dir is the repo root, or a
// monorepo folder that keeps its own agent files (yass init <folder> --agents).
func (w *kitWriter) agentFiles(dir string, claude, global bool) error {
	v := stampVersion()
	skills, claudeSkills := filepath.Join(dir, ".agents", "skills"), filepath.Join(dir, ".claude", "skills")
	if global {
		skills, claudeSkills = userSkillsDir(), userClaudeSkillsDir()
	}
	names, text := kitSkills()
	for _, n := range names {
		stamped := stampSkill(text[n], v)
		if err := w.place(filepath.Join(skills, n, "SKILL.md"), stamped, 0); err != nil {
			return err
		}
		if claude {
			if err := w.place(filepath.Join(claudeSkills, n, "SKILL.md"), stamped, 0); err != nil {
				return err
			}
		}
	}
	if claude {
		cm := filepath.Join(dir, "CLAUDE.md")
		if cur := read(cm); !strings.Contains(cur, "@AGENTS.md") {
			next := "@AGENTS.md\n"
			if cur != "" {
				next += "\n" + cur
			}
			if err := write(cm, next); err != nil {
				return err
			}
			w.made = append(w.made, cm)
		}
	}
	return nil
}

// hookScripts are the git hooks YASS ships: the intent check, and the trailer hooks that only act
// when plans are kept apart from the code.
var hookScripts = []string{"pre-commit", "prepare-commit-msg", "commit-msg"}

// hook writes the hook scripts, which belong to the repo root: git hooks are per repo.
func (w *kitWriter) hook(top string) error {
	for _, name := range hookScripts {
		b, _ := kit.FS.ReadFile(HookPath + "/" + name)
		if err := w.place(filepath.Join(top, filepath.FromSlash(HookPath), name), stampHook(string(b), stampVersion()), 0o755); err != nil {
			return err
		}
	}
	return nil
}

var (
	skillStampRE  = regexp.MustCompile(`(?m)^[ \t]+yass-version:[ \t]*"?([^"\s]+)"?[ \t]*$`)
	agentsStampRE = regexp.MustCompile(`<!-- yass:begin version=(\S+)`)
	hookStampRE   = regexp.MustCompile(`(?m)^# yass-version:[ \t]*(\S+)[ \t]*$`)
)

// readStamp returns the version stamped in a YASS file (a playbook, an AGENTS.md or the hook), or ""
// for a file with none, as v0.1 and v0.2 wrote them.
func readStamp(p string) string {
	text := read(p)
	var m []string
	switch {
	case filepath.Base(p) == "SKILL.md":
		if f := fmRE.FindStringSubmatch(text); f != nil {
			m = skillStampRE.FindStringSubmatch(f[1])
		}
	case filepath.Base(p) == "AGENTS.md":
		m = agentsStampRE.FindStringSubmatch(text)
	default:
		m = hookStampRE.FindStringSubmatch(text)
	}
	if m == nil {
		return ""
	}
	return m[1]
}

// semverOf turns a stamp or the binary's version into SemVer ("0.3.0" -> "v0.3.0"); "" if it isn't one.
func semverOf(v string) string {
	if v == "" {
		return ""
	}
	if !strings.HasPrefix(v, "v") {
		v = "v" + v
	}
	if !semver.IsValid(v) {
		return ""
	}
	return v
}

// olderThanBinary says whether a stamp is older than this binary's version. An unstamped file is
// older than any stamped one (design §3); a binary without a version compares with nothing.
func olderThanBinary(stamp string) bool {
	bin := semverOf(binVersion)
	if bin == "" {
		return false
	}
	s := semverOf(stamp)
	return s == "" || semver.Compare(s, bin) < 0
}
