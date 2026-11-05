package yass

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Migrations bring an older yass folder up to how this version lays it out. Each step tells from
// the files whether it's needed and does nothing when it isn't, so there's no format version to
// track, and a step can be deleted in a later release once nobody needs it (design §5).
type migration struct {
	name   string
	needed func(root *Root) bool
	// run changes the working tree and stages what it can, and returns what it did in a line.
	run func(r *Repo, root *Root) (string, error)
}

var migrations = []migration{
	{"archive-into-months", flatArchived, archiveIntoMonths},
}

// migrate runs every needed step for every yass folder and says how to commit each one on its own.
func migrate(r *Repo, steps []migration) error {
	for _, root := range r.Roots {
		for _, m := range steps {
			if !m.needed(root) {
				continue
			}
			did, err := m.run(r, root)
			if err != nil {
				return fmt.Errorf("%s in %s: %v", m.name, r.disp(root.Dir), err)
			}
			fmt.Printf("migrated %s (%s): %s\n", r.disp(root.Dir), m.name, did)
			if top, ok := git(root.Dir, "rev-parse", "--show-toplevel"); ok {
				rel := r.disp(root.Dir)
				cmd := fmt.Sprintf("git commit -m \"yass: %s\" -- %s", m.name, rel)
				if gitTop := canon(filepath.FromSlash(strings.TrimSpace(top))); gitTop != r.Top {
					cmd = fmt.Sprintf("git -C %s commit -m \"yass: %s\" -- %s", gitTop, m.name, root.Dir)
				}
				fmt.Printf("  commit it on its own: %s\n", cmd)
			}
		}
	}
	return nil
}

// flatArchived lists the changes sitting directly in archive/, from before archives went by month.
func flatArchived(root *Root) bool { return len(flatArchive(root)) > 0 }

func flatArchive(root *Root) []string {
	var out []string
	for _, name := range subdirs(filepath.Join(root.Dir, "archive")) {
		if !yearRE.MatchString(name) {
			out = append(out, name)
		}
	}
	return out
}

// archiveIntoMonths moves each flat archived change to archive/<YYYY>/<MM>/ and stamps archived:.
// The date is its archived: stamp if it has one, else the date of the latest commit that added
// files to it where it is, else the date in its name.
func archiveIntoMonths(r *Repo, root *Root) (string, error) {
	base := filepath.Join(root.Dir, "archive")
	added := archiveDates(base)
	n := 0
	for _, name := range flatArchive(root) {
		src := filepath.Join(base, name)
		head := filepath.Join(src, "change.md")
		meta, _ := frontmatter(readText(head))
		when, err := time.Parse(time.RFC3339, meta["archived"])
		if err != nil {
			if t, ok := added[name]; ok {
				when = t
			} else if t, err := time.Parse("2006-01-02", name[:min(len(name), 10)]); err == nil {
				when = t
			} else {
				return "", fmt.Errorf("%s: no archived: stamp, no commit that added it, and no date in its name; give it an archived: stamp", r.disp(src))
			}
		}
		dest := archiveDir(root, name, when)
		if exists(dest) {
			return "", fmt.Errorf("%s already exists", r.disp(dest))
		}
		if err := os.MkdirAll(filepath.Dir(dest), 0o755); err != nil {
			return "", err
		}
		if _, ok := git(base, "mv", src, dest); !ok {
			if err := os.Rename(src, dest); err != nil {
				return "", err
			}
			git(base, "add", "-A", "--", src)
		}
		if meta["archived"] == "" && isFile(filepath.Join(dest, "change.md")) {
			h := filepath.Join(dest, "change.md")
			if err := write(h, setMeta(read(h), "archived", when.UTC().Format(time.RFC3339))); err != nil {
				return "", err
			}
		}
		git(base, "add", "-A", "--", dest)
		n++
	}
	return fmt.Sprintf("moved %d archived change(s) into archive/<YYYY>/<MM>/", n), nil
}

// archiveDates is when git last added files to each flat archived change, by change name, from
// one git log over the archive. Outside git, or for changes git never saw, there's no entry.
func archiveDates(base string) map[string]time.Time {
	out := map[string]time.Time{}
	log, ok := git(base, "log", "--diff-filter=A", "--relative", "--name-only", "--format=@%cI", "--", ".")
	if !ok {
		return out
	}
	var when time.Time
	for _, l := range strings.Split(log, "\n") {
		if strings.HasPrefix(l, "@") {
			when, _ = time.Parse(time.RFC3339, l[1:])
			continue
		}
		parts := strings.Split(l, "/")
		if len(parts) < 2 || when.IsZero() {
			continue
		}
		if _, seen := out[parts[0]]; !seen { // newest first: the latest add wins
			out[parts[0]] = when
		}
	}
	return out
}
