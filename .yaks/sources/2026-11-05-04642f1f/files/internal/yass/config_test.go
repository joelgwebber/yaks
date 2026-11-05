package yass

import (
	"os"
	"path/filepath"
	"testing"
)

func TestResolvePath(t *testing.T) {
	base := t.TempDir()
	t.Setenv("YASS_TEST_HOME", "/srv/plans")
	t.Setenv("YASS_TEST_EMPTY", "")
	home, _ := os.UserHomeDir()
	for in, want := range map[string]string{
		"../elsewhere":               filepath.Join(filepath.Dir(base), "elsewhere"),
		"planning":                   filepath.Join(base, "planning"),
		"/abs/yass":                  "/abs/yass",
		"$YASS_TEST_HOME":            "/srv/plans",
		"${YASS_TEST_HOME}/my-repo":  "/srv/plans/my-repo",
		"~/yass/my-repo":             filepath.Join(home, "yass", "my-repo"),
		"${YASS_TEST_HOME:-..}/web":  "/srv/plans/web",
		"${YASS_TEST_UNSET:-..}/web": filepath.Join(filepath.Dir(base), "web"),
		"${YASS_TEST_EMPTY:-~/p}":    filepath.Join(home, "p"),
		"${YASS_TEST_UNSET:-}":       base,
	} {
		got, err := resolvePath(in, base)
		if err != nil || got != canon(want) {
			t.Errorf("resolvePath(%q) = %q, %v; want %q", in, got, err, canon(want))
		}
	}
	if _, err := resolvePath("${YASS_TEST_UNSET}/x", base); err == nil || err.Error() != "$YASS_TEST_UNSET isn't set" {
		t.Errorf("unset variable: %v", err)
	}
}

func TestConfigTextRoundTrips(t *testing.T) {
	f := filepath.Join(t.TempDir(), ConfigName)
	for _, p := range []string{"../private", "${YASS_HOME}/it's mine", `C:\plans`} {
		os.WriteFile(f, []byte(configText(p)), 0o644)
		c, unknown, err := loadConfig(f)
		if err != nil || c.Path != p || len(unknown) != 0 {
			t.Errorf("round trip %q: got %q %v %v", p, c.Path, unknown, err)
		}
	}
	os.WriteFile(f, []byte("path: x\ncolor: blue\n"), 0o644)
	if _, unknown, _ := loadConfig(f); len(unknown) != 1 || unknown[0] != "color" {
		t.Errorf("unknown = %v", unknown)
	}
}

func TestCanonResolvesMissingPathsThroughSymlinks(t *testing.T) {
	real := t.TempDir()
	link := filepath.Join(t.TempDir(), "link")
	if err := os.Symlink(real, link); err != nil {
		t.Skip("no symlinks here")
	}
	want := filepath.Join(canon(real), "not", "there")
	if got := canon(filepath.Join(link, "not", "there")); got != want {
		t.Errorf("canon = %q, want %q", got, want)
	}
}

func TestArchiveKeep(t *testing.T) {
	dir := t.TempDir()
	f := filepath.Join(dir, ConfigName)
	os.WriteFile(f, []byte("archive:\n  keep: 3\n  kepp: 4\n"), 0o644)
	c, unknown, err := loadConfig(f)
	if err != nil || c.Archive.Keep != 3 || len(unknown) != 1 || unknown[0] != "archive.kepp" {
		t.Fatalf("loadConfig = %+v, %v, %v", c, unknown, err)
	}
	if k := (&Root{Config: f}).keep(); k != 3 {
		t.Fatalf("keep = %d, want 3", k)
	}
	if k := (&Root{}).keep(); k != DefaultKeep {
		t.Fatalf("keep without a yass.yaml = %d, want %d", k, DefaultKeep)
	}
}
