package yass

import (
	"strings"
	"testing"
)

func TestAssetName(t *testing.T) {
	for _, tc := range []struct{ goos, goarch, want string }{
		{"darwin", "arm64", "yass_darwin_arm64.tar.gz"},
		{"darwin", "amd64", "yass_darwin_amd64.tar.gz"},
		{"linux", "amd64", "yass_linux_amd64.tar.gz"},
		{"linux", "arm64", "yass_linux_arm64.tar.gz"},
		{"windows", "amd64", "yass_windows_amd64.zip"},
		{"windows", "arm64", "yass_windows_arm64.zip"},
	} {
		if got := assetName(tc.goos, tc.goarch); got != tc.want {
			t.Errorf("assetName(%s, %s) = %s, want %s", tc.goos, tc.goarch, got, tc.want)
		}
	}
}

func TestPlanUpdate(t *testing.T) {
	mk := func(latest bool, version string) *args {
		return &args{b: map[string]bool{"latest": latest}, v: map[string]string{"version": version}}
	}
	for _, tc := range []struct {
		name, bin, repo, latest string
		a                       *args
		want, err               string
	}{
		{"repo ahead: match it", "v0.4.0", "v0.5.0", "v0.6.0", mk(false, ""), "v0.5.0", ""},
		{"repo behind: latest", "v0.5.0", "v0.4.0", "v0.6.0", mk(false, ""), "v0.6.0", ""},
		{"no repo: latest", "v0.4.0", "", "v0.6.0", mk(false, ""), "v0.6.0", ""},
		{"--latest over the repo", "v0.4.0", "v0.5.0", "v0.6.0", mk(true, ""), "v0.6.0", ""},
		{"--version", "v0.4.0", "v0.5.0", "v0.6.0", mk(false, "0.5.0"), "v0.5.0", ""},
		{"never downgrades", "v0.5.0", "", "v0.6.0", mk(false, "v0.4.0"), "", "never downgrades"},
		{"--version must be a version", "v0.4.0", "", "v0.6.0", mk(false, "newest"), "", "takes a release"},
	} {
		p, err := planUpdate(tc.a, tc.bin, tc.repo, tc.latest)
		switch {
		case tc.err != "" && (err == nil || !strings.Contains(err.Error(), tc.err)):
			t.Errorf("%s: err = %v, want %q", tc.name, err, tc.err)
		case tc.err == "" && (err != nil || p.target != tc.want):
			t.Errorf("%s: target = %s, %v; want %s", tc.name, p.target, err, tc.want)
		}
	}
}

func TestSourceBuild(t *testing.T) {
	defer func(v string) { binVersion = v }(binVersion)
	for v, want := range map[string]string{
		"v0.4.0":                               "go install github.com/donjaime/yass/cmd/yass@v0.5.0",
		"v0.3.1-0.20261008201306-a4857ed4ea9a": "built from a clone",
		"v0.3.1-0.20261008201306-a4857ed4ea9a+dirty": "built from a clone",
		"dev": "built from a clone",
	} {
		binVersion = v
		if got := sourceBuild("v0.5.0"); !strings.Contains(got, want) {
			t.Errorf("sourceBuild with %s = %q, want it to say %q", v, got, want)
		}
	}
}
