package yass

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestMigrateRunsOnlyNeededSteps(t *testing.T) {
	root := &Root{Dir: t.TempDir()}
	r := &Repo{Top: root.Dir, Roots: []*Root{root}}
	ran := map[string]int{}
	step := func(name string, needed bool) migration {
		return migration{name, func(*Root) bool { return needed },
			func(*Repo, *Root) (string, error) { ran[name]++; return "did it", nil }}
	}
	if err := migrate(r, []migration{step("a", true), step("b", false)}); err != nil {
		t.Fatal(err)
	}
	if ran["a"] != 1 || ran["b"] != 0 {
		t.Fatalf("ran = %v, want a once and not b", ran)
	}
	// A list without the archive step (as after it's deleted) still runs clean.
	if err := migrate(r, nil); err != nil {
		t.Fatal(err)
	}
	for _, m := range migrations {
		if m.name == "" || m.needed == nil || m.run == nil {
			t.Fatalf("incomplete step %+v", m)
		}
	}
}

func TestArchiveIntoMonthsOutsideGit(t *testing.T) {
	root := &Root{Dir: t.TempDir()}
	r := &Repo{Top: root.Dir, Roots: []*Root{root}}
	mk := func(name, text string) {
		p := filepath.Join(root.Dir, "archive", name)
		os.MkdirAll(p, 0o755)
		os.WriteFile(filepath.Join(p, "change.md"), []byte(text), 0o644)
	}
	mk("2026-03-04-plain", "# Plain\n")
	mk("2026-03-05-stamped", "---\narchived: 2026-05-01T10:00:00Z\n---\n# Stamped\n")
	os.MkdirAll(filepath.Join(root.Dir, "archive", "2026", "02", "2026-02-01-already"), 0o755)
	if !flatArchived(root) {
		t.Fatal("flat archive not seen")
	}
	if _, err := archiveIntoMonths(r, root); err != nil {
		t.Fatal(err)
	}
	got := readText(filepath.Join(root.Dir, "archive", "2026", "03", "2026-03-04-plain", "change.md"))
	if !strings.HasPrefix(got, "---\narchived: 2026-03-04T00:00:00Z\n---\n# Plain") {
		t.Fatalf("plain = %q", got)
	}
	if !isFile(filepath.Join(root.Dir, "archive", "2026", "05", "2026-03-05-stamped", "change.md")) {
		t.Fatal("a change with archived: goes by its stamp")
	}
	if flatArchived(root) {
		t.Fatal("still flat after migrating")
	}
}
