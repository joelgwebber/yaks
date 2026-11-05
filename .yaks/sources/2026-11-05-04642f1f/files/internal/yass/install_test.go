package yass

import (
	"crypto/sha256"
	"encoding/hex"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestSwapBinary(t *testing.T) {
	for _, goos := range []string{"darwin", "windows"} {
		dir := t.TempDir()
		exe, next := filepath.Join(dir, "yass"), filepath.Join(dir, ".yass-new-1")
		os.WriteFile(exe, []byte("old"), 0o755)
		os.WriteFile(next, []byte("new"), 0o755)
		if err := swapBinary(exe, next, goos); err != nil {
			t.Fatalf("%s: %v", goos, err)
		}
		if got, _ := os.ReadFile(exe); string(got) != "new" {
			t.Errorf("%s: binary is %q, want the new one", goos, got)
		}
		if isFile(next) {
			t.Errorf("%s: the new file is still beside it", goos)
		}
		old, _ := os.ReadFile(exe + ".old")
		if goos == "windows" && string(old) != "old" {
			t.Errorf("windows: the old binary should be set aside as yass.old, got %q", old)
		}
		if goos != "windows" && isFile(exe+".old") {
			t.Errorf("%s: nothing should be set aside", goos)
		}
	}
	// A failed swap on Windows puts the old binary back.
	dir := t.TempDir()
	exe := filepath.Join(dir, "yass")
	os.WriteFile(exe, []byte("old"), 0o755)
	if err := swapBinary(exe, filepath.Join(dir, "missing"), "windows"); err == nil {
		t.Fatal("swapping in a missing file should fail")
	}
	if got, _ := os.ReadFile(exe); string(got) != "old" {
		t.Errorf("after a failed swap the binary is %q, want the old one back", got)
	}
}

func TestChecksumOK(t *testing.T) {
	f := filepath.Join(t.TempDir(), "yass_linux_amd64.tar.gz")
	os.WriteFile(f, []byte("archive"), 0o644)
	sum := sha256.Sum256([]byte("archive"))
	good := hex.EncodeToString(sum[:])
	if err := checksumOK(f, good+"  yass_linux_amd64.tar.gz\nabc  other.zip\n", "yass_linux_amd64.tar.gz"); err != nil {
		t.Errorf("matching checksum: %v", err)
	}
	if err := checksumOK(f, strings.Repeat("0", 64)+"  yass_linux_amd64.tar.gz\n", "yass_linux_amd64.tar.gz"); err == nil || !strings.Contains(err.Error(), "doesn't match") {
		t.Errorf("mismatch: %v", err)
	}
	if err := checksumOK(f, good+"  other.tar.gz\n", "yass_linux_amd64.tar.gz"); err == nil || !strings.Contains(err.Error(), "isn't listed") {
		t.Errorf("unlisted: %v", err)
	}
}
