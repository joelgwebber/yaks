package yass

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLinkSkipsWhereSymlinksFail(t *testing.T) {
	if os.Geteuid() == 0 {
		t.Skip("root can write anywhere")
	}
	top, plans := t.TempDir(), t.TempDir()
	if err := os.Chmod(top, 0o555); err != nil {
		t.Fatal(err)
	}
	defer os.Chmod(top, 0o755)
	r := &Repo{Top: canon(top), Roots: []*Root{{Dir: canon(plans), Owner: canon(top), Config: filepath.Join(canon(top), ConfigName)}}}
	r.link()
	if _, err := os.Lstat(filepath.Join(top, "yass")); err == nil {
		t.Errorf("made a link in a read-only folder")
	}
	if len(r.Notes) != 1 || len(r.Warnings) != 0 {
		t.Errorf("notes %v, warnings %v; want one note and no warnings", r.Notes, r.Warnings)
	}
}
