package yass

import (
	"reflect"
	"strings"
	"testing"
)

func TestParseTrailer(t *testing.T) {
	for _, tc := range []struct {
		in, target  string
		ac, unknown []string
		bad         bool
	}{
		{in: "2026-02-01-login", target: "2026-02-01-login"},
		{in: "  2026-02-01-LOGIN/2026-02-02-Form ", target: "2026-02-01-login/2026-02-02-form"},
		{in: "none", target: "none"},
		{in: "login ac:AC1,ac3", target: "login", ac: []string{"ac1", "ac3"}},
		{in: "login ac:ac1 ac:ac2", target: "login", ac: []string{"ac1", "ac2"}},
		{in: "login foo:bar", target: "login", unknown: []string{"foo"}},
		{in: "", bad: true},
		{in: "ac:ac1", bad: true},
		{in: "login ac", bad: true},
		{in: "login ac:", bad: true},
		{in: "login ac:ac1,", bad: true},
		{in: "login ac:ac1,,ac2", bad: true},
	} {
		got, err := parseTrailer(tc.in)
		if (err != nil) != tc.bad {
			t.Errorf("parseTrailer(%q): err = %v, want bad=%v", tc.in, err, tc.bad)
			continue
		}
		if tc.bad {
			continue
		}
		if got.target != tc.target || !reflect.DeepEqual(got.ac, tc.ac) || !reflect.DeepEqual(got.unknown, tc.unknown) {
			t.Errorf("parseTrailer(%q) = %+v", tc.in, got)
		}
	}
}

func TestCheckTarget(t *testing.T) {
	named := map[string]string{"2026-02-01-login": "2026-02-01-login", "2026-02-01-login/2026-02-03-api": "2026-02-01-login/2026-02-03-api", "2026-02-03-api": "2026-02-01-login/2026-02-03-api"}
	amb := map[string]bool{"2026-02-02-form": true}
	for target, want := range map[string]string{
		"2026-02-01-login": "", "2026-02-03-api": "", "none": "",
		"2026-02-02-form": "more than one change",
		"2026-02-01-logn": "did you mean 2026-02-01-login",
	} {
		got := checkTarget(trailer{target: target}, named, amb)
		if want == "" && got != "" || want != "" && !strings.Contains(got, want) {
			t.Errorf("checkTarget(%q) = %q, want %q", target, got, want)
		}
	}
	if got := checkTarget(trailer{target: "anything"}, nil, nil); got != "" {
		t.Errorf("without the plans, names aren't checked; got %q", got)
	}
}
