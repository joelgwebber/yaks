package yass

import (
	"fmt"
	"net/http"
	"os"
	"path"
	"runtime"
	"strings"
	"time"

	"golang.org/x/mod/module"
	"golang.org/x/mod/semver"
)

// yass update: get a newer yass binary. It decides what to install from the binary's version, the
// version the repo's YASS files are stamped with, and the latest release (design §1–2).

// releasesURL is where releases come from. YASS_RELEASES_URL is a seam for tests (design §5), not
// a supported way to use mirrors.
func releasesURL() string {
	if u := os.Getenv("YASS_RELEASES_URL"); u != "" {
		return strings.TrimRight(u, "/")
	}
	return "https://github.com/donjaime/yass/releases"
}

// assetName is the release archive for a platform, as .goreleaser.yaml names it.
func assetName(goos, goarch string) string {
	if goos == "windows" {
		return fmt.Sprintf("yass_%s_%s.zip", goos, goarch)
	}
	return fmt.Sprintf("yass_%s_%s.tar.gz", goos, goarch)
}

var httpClient = &http.Client{
	Timeout:       15 * time.Second,
	CheckRedirect: func(*http.Request, []*http.Request) error { return http.ErrUseLastResponse },
}

// latestRelease reads the latest release's version from where <releases>/latest redirects to
// (…/releases/tag/vX.Y.Z), so it needs no API and no token (design §2).
func latestRelease() (string, error) {
	resp, err := httpClient.Head(releasesURL() + "/latest")
	if err != nil {
		return "", fmt.Errorf("couldn't reach %s: %v", releasesURL(), err)
	}
	resp.Body.Close()
	loc := resp.Header.Get("Location")
	if resp.StatusCode/100 != 3 || loc == "" {
		return "", fmt.Errorf("couldn't find the latest release at %s/latest (HTTP %d)", releasesURL(), resp.StatusCode)
	}
	v := semverOf(path.Base(loc))
	if v == "" {
		return "", fmt.Errorf("the latest release at %s isn't a version", loc)
	}
	return v, nil
}

// updatePlan is what yass update would do, and why.
type updatePlan struct {
	bin, repo, latest string // semver with a v; repo is "" outside a repo that uses YASS
	target            string // what it would install
	why               string // "to match this repo", "the latest release", …
}

func planUpdate(a *args, bin, repo, latest string) (updatePlan, error) {
	p := updatePlan{bin: bin, repo: repo, latest: latest}
	switch {
	case a.v["version"] != "":
		p.target, p.why = semverOf(a.v["version"]), "as asked"
		if p.target == "" {
			return p, fmt.Errorf("--version takes a release like v0.4.0")
		}
	case a.b["latest"]:
		p.target, p.why = latest, "the latest release"
	case repo != "" && (bin == "" || semver.Compare(repo, bin) > 0):
		p.target, p.why = repo, "to match this repo's YASS files"
	default:
		p.target, p.why = latest, "the latest release"
	}
	if bin != "" && semver.Compare(p.target, bin) < 0 {
		return p, fmt.Errorf("%s is older than this yass (%s); yass update never downgrades", p.target, bin)
	}
	return p, nil
}

func plain(v string) string { return strings.TrimPrefix(v, "v") }

// sourceBuild says how to update a binary that wasn't built by the release workflow: a version
// that's a release tag came from go install; anything else (a pseudo-version, +dirty, dev) from a clone.
func sourceBuild(target string) string {
	if v := semverOf(binVersion); v != "" && !module.IsPseudoVersion(v) && semver.Build(v) == "" {
		return fmt.Sprintf("it was built with go install; update it the same way: go install github.com/donjaime/yass/cmd/yass@%s", target)
	}
	return fmt.Sprintf("it was built from a clone; update it there: git pull (or git checkout %s), then go build -o bin/yass ./cmd/yass", target)
}

func cmdUpdate(a *args) (int, error) {
	bin := semverOf(binVersion)
	repo := repoVersion(findRepo())
	latest, err := latestRelease()
	if err != nil {
		return 0, err
	}
	p, err := planUpdate(a, bin, repo, latest)
	if err != nil {
		return 0, err
	}
	kind := "release build"
	if binChannel != "release" {
		kind = "built from source"
	}
	fmt.Printf("yass %s (%s, %s/%s)\n", binVersion, kind, runtime.GOOS, runtime.GOARCH)
	if repo != "" {
		fmt.Printf("this repo's YASS files: %s\n", plain(repo))
	}
	fmt.Printf("latest release: %s\n", plain(latest))
	upToDate := bin != "" && semver.Compare(p.target, bin) == 0
	installed := false
	switch {
	case upToDate:
		fmt.Printf("yass is up to date (%s)\n", plain(bin))
	case binChannel != "release":
		fmt.Printf("yass update won't replace this binary: %s\n", sourceBuild(p.target))
	case a.b["check"]:
		fmt.Printf("yass update would install %s (%s)\n", plain(p.target), p.why)
	default:
		fmt.Printf("installing %s (%s) …\n", plain(p.target), p.why)
		how, err := install(p.target, a.b["require-provenance"])
		if err != nil {
			return 0, err
		}
		fmt.Printf("installed yass %s (%s)\n", plain(p.target), how)
		installed = true
	}
	if semver.Compare(latest, p.target) > 0 && !a.b["latest"] && a.v["version"] == "" {
		fmt.Printf("%s is newer: `yass update --latest` gets it; then run `yass upgrade` in each repo and commit the diff (chore: upgrade YASS)\n", plain(latest))
	}
	if installed && repo != "" && semver.Compare(p.target, repo) > 0 {
		fmt.Printf("next: this repo's YASS files are older (%s); run `yass upgrade` here and commit the diff (chore: upgrade YASS), and teammates then need yass %s too (`yass update`)\n",
			plain(repo), plain(p.target))
	}
	return 0, nil
}
