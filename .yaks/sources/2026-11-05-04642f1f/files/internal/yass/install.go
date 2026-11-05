package yass

import (
	"archive/tar"
	"archive/zip"
	"compress/gzip"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
	"time"
)

// Installing a release: fetch the platform's archive and checksums.txt into a temporary folder,
// verify them (design §3), check the new binary runs and is the version asked for, then swap it in
// with a rename in the binary's own folder (design §4). Nothing outside the temporary folder
// changes until every check has passed.

var downloadClient = &http.Client{Timeout: 5 * time.Minute} // follows redirects, as GitHub's downloads need

// exePath is the running binary, symlinks resolved, so the swap happens where the file really is.
func exePath() (string, error) {
	p, err := os.Executable()
	if err != nil {
		return "", err
	}
	return filepath.EvalSymlinks(p)
}

// writable says whether yass can create a file in dir, by creating one.
func writable(dir string) bool {
	f, err := os.CreateTemp(dir, ".yass-update-*")
	if err != nil {
		return false
	}
	f.Close()
	os.Remove(f.Name())
	return true
}

func fetch(url, to string) error {
	resp, err := downloadClient.Get(url)
	if err != nil {
		return fmt.Errorf("couldn't reach %s: %v", url, err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("%s: HTTP %d", url, resp.StatusCode)
	}
	f, err := os.Create(to)
	if err != nil {
		return err
	}
	_, err = io.Copy(f, resp.Body)
	if cerr := f.Close(); err == nil {
		err = cerr
	}
	return err
}

// releaseExists checks a release by its checksums.txt, before anything is downloaded.
func releaseExists(version string) error {
	url := fmt.Sprintf("%s/download/%s/checksums.txt", releasesURL(), version)
	resp, err := httpClient.Head(url)
	if err != nil {
		return fmt.Errorf("couldn't reach %s: %v", releasesURL(), err)
	}
	resp.Body.Close()
	if resp.StatusCode == http.StatusNotFound {
		return fmt.Errorf("there's no release %s (see %s)", version, releasesURL())
	}
	if resp.StatusCode/100 != 2 && resp.StatusCode/100 != 3 {
		return fmt.Errorf("couldn't check release %s: HTTP %d", version, resp.StatusCode)
	}
	return nil
}

// checksumOK checks a file against its line in checksums.txt ("<sha256>  <name>").
func checksumOK(file, sums, name string) error {
	want := ""
	for _, l := range strings.Split(sums, "\n") {
		if f := strings.Fields(l); len(f) == 2 && f[1] == name {
			want = strings.ToLower(f[0])
		}
	}
	if want == "" {
		return fmt.Errorf("%s isn't listed in the release's checksums.txt", name)
	}
	data, err := os.ReadFile(file)
	if err != nil {
		return err
	}
	sum := sha256.Sum256(data)
	if got := hex.EncodeToString(sum[:]); got != want {
		return fmt.Errorf("%s doesn't match the release's checksums.txt (expected %s, got %s)", name, want, got)
	}
	return nil
}

// provenance verifies the archive's signed build provenance with the GitHub CLI. checked says
// whether it ran; a failed verification is an error, and why it didn't run is in note.
func provenance(archive string) (checked bool, note string, err error) {
	gh, lerr := exec.LookPath("gh")
	if lerr != nil {
		return false, "the GitHub CLI isn't installed", nil
	}
	if exec.Command(gh, "auth", "status").Run() != nil {
		return false, "the GitHub CLI isn't signed in (gh auth login)", nil
	}
	out, verr := exec.Command(gh, "attestation", "verify", archive, "--repo", "donjaime/yass").CombinedOutput()
	if verr != nil {
		return true, "", fmt.Errorf("its build provenance didn't verify (gh attestation verify):\n%s", strings.TrimSpace(string(out)))
	}
	return true, "", nil
}

// unpack finds the yass binary in a release archive and returns its contents.
func unpack(archive string) ([]byte, error) {
	name := "yass"
	if runtime.GOOS == "windows" {
		name = "yass.exe"
	}
	if strings.HasSuffix(archive, ".zip") {
		zr, err := zip.OpenReader(archive)
		if err != nil {
			return nil, err
		}
		defer zr.Close()
		for _, f := range zr.File {
			if filepath.Base(f.Name) == name && !f.FileInfo().IsDir() {
				rc, err := f.Open()
				if err != nil {
					return nil, err
				}
				defer rc.Close()
				return io.ReadAll(rc)
			}
		}
		return nil, fmt.Errorf("%s has no %s", filepath.Base(archive), name)
	}
	f, err := os.Open(archive)
	if err != nil {
		return nil, err
	}
	defer f.Close()
	gz, err := gzip.NewReader(f)
	if err != nil {
		return nil, err
	}
	tr := tar.NewReader(gz)
	for {
		h, err := tr.Next()
		if err == io.EOF {
			return nil, fmt.Errorf("%s has no %s", filepath.Base(archive), name)
		}
		if err != nil {
			return nil, err
		}
		if filepath.Base(h.Name) == name && h.Typeflag == tar.TypeReg {
			return io.ReadAll(tr)
		}
	}
}

// swapBinary replaces exe with next, a file in the same folder. On Unix a rename is atomic, so the
// binary is never missing or half written. Windows can't overwrite a running .exe, so the old one
// is renamed aside to <exe>.old first, and removed on a later run (cleanOldBinary).
func swapBinary(exe, next, goos string) error {
	if goos == "windows" {
		old := exe + ".old"
		os.Remove(old)
		if err := os.Rename(exe, old); err != nil {
			return err
		}
		if err := os.Rename(next, exe); err != nil {
			os.Rename(old, exe) // put it back
			return err
		}
		return nil
	}
	return os.Rename(next, exe)
}

// cleanOldBinary removes what a Windows update set aside, once it's no longer running.
func cleanOldBinary() {
	if runtime.GOOS != "windows" {
		return
	}
	if exe, err := exePath(); err == nil {
		os.Remove(exe + ".old")
	}
}

// install fetches, verifies and swaps in version. It returns how it verified the download.
func install(version string, requireProvenance bool) (string, error) {
	exe, err := exePath()
	if err != nil {
		return "", fmt.Errorf("couldn't find this yass binary: %v", err)
	}
	dir := filepath.Dir(exe)
	if !writable(dir) {
		return "", fmt.Errorf("can't write to %s, where this yass is; rerun with permission to write there, or install yass into a folder of your own (see docs/install.md)", dir)
	}
	if err := releaseExists(version); err != nil {
		return "", err
	}
	tmp, err := os.MkdirTemp("", "yass-update-")
	if err != nil {
		return "", err
	}
	defer os.RemoveAll(tmp)
	asset := assetName(runtime.GOOS, runtime.GOARCH)
	base := fmt.Sprintf("%s/download/%s", releasesURL(), version)
	archive, sums := filepath.Join(tmp, asset), filepath.Join(tmp, "checksums.txt")
	if err := fetch(base+"/"+asset, archive); err != nil {
		return "", err
	}
	if err := fetch(base+"/checksums.txt", sums); err != nil {
		return "", err
	}
	if err := checksumOK(archive, read(sums), asset); err != nil {
		return "", fmt.Errorf("not installing %s: %v", plain(version), err)
	}
	checked, why, err := provenance(archive)
	if err != nil {
		return "", fmt.Errorf("not installing %s: %v", plain(version), err)
	}
	if !checked && requireProvenance {
		return "", fmt.Errorf("not installing %s: --require-provenance, and provenance can't be checked: %s", plain(version), why)
	}
	bin, err := unpack(archive)
	if err != nil {
		return "", fmt.Errorf("not installing %s: %v", plain(version), err)
	}
	next, err := os.CreateTemp(dir, ".yass-new-*")
	if err != nil {
		return "", err
	}
	nextPath := next.Name()
	defer os.Remove(nextPath) // gone after a successful swap; cleaned up otherwise
	_, err = next.Write(bin)
	if cerr := next.Close(); err == nil {
		err = cerr
	}
	if err == nil {
		err = os.Chmod(nextPath, 0o755)
	}
	if err != nil {
		return "", err
	}
	// The new binary has to run here and be the version asked for: a wrong platform or a broken
	// download stops before anything is replaced.
	out, rerr := exec.Command(nextPath, "version").Output()
	if got := strings.TrimSpace(string(out)); rerr != nil || got != "yass "+plain(version) {
		if rerr != nil {
			got = rerr.Error()
		}
		return "", fmt.Errorf("not installing %s: the downloaded yass doesn't run as %s here (it said: %s)", plain(version), plain(version), firstLine(got))
	}
	if err := swapBinary(exe, nextPath, runtime.GOOS); err != nil {
		return "", fmt.Errorf("couldn't replace %s: %v", exe, err)
	}
	how := "checksum verified"
	if checked {
		how += ", build provenance verified"
	} else {
		how += "; build provenance not checked: " + why
	}
	return how, nil
}

func firstLine(s string) string {
	line, _, _ := strings.Cut(s, "\n")
	return line
}
