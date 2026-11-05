package yass

import (
	"reflect"
	"strings"
	"testing"
)

func TestParseQueue(t *testing.T) {
	got := parseQueue("# Queue\nTop first.\n\n1. 2026-10-02-a — a note\n- `2026-09-14-b`\n* [2026-09-01-c](changes/2026-09-01-c/) - linked\n" +
		"10) 2026-08-01-d/\n<!-- - 2026-01-01-hidden -->\nprose with 2026-07-01-e in it\n  + 2026-10-02-a\n")
	want := []string{"2026-10-02-a", "2026-09-14-b", "2026-09-01-c", "2026-08-01-d", "2026-10-02-a"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("parseQueue = %q, want %q", got, want)
	}
}

func TestBoxes(t *testing.T) {
	got := boxes("- [ ] a\n* [x] b\n+ [X] c\n- [/] d\n- [-] e\n- [ ] \n<!-- - [ ] hidden -->\n- [?] f\n")
	want := []box{{" ", "a"}, {"x", "b"}, {"x", "c"}, {"/", "d"}, {"-", "e"}}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("boxes = %v, want %v", got, want)
	}
}

func TestFrontmatter(t *testing.T) {
	meta, body := frontmatter("---\nplatforms: [ios, android]\nsource: gh#41\nblocked:    # set when stuck\n---\n# Title\n")
	want := map[string]string{"platforms": "ios, android", "source": "gh#41", "blocked": ""}
	if !reflect.DeepEqual(meta, want) || body != "# Title\n" {
		t.Fatalf("frontmatter = %v %q", meta, body)
	}
	if meta, _ := frontmatter("---\nsource: gh issue #41\nfollows: x   # a note\nblocked: #\n---\n"); meta["source"] != "gh issue #41" || meta["follows"] != "x" || meta["blocked"] != "" {
		t.Fatalf("comments = %v", meta)
	}
	if meta, body := frontmatter("---\n---\n# t\n"); len(meta) != 0 || body != "---\n---\n# t\n" {
		t.Fatalf("empty frontmatter = %v %q", meta, body)
	}
}

func TestSection(t *testing.T) {
	text := "# T\n## Goal\nship it\n\n## Acceptance\n- [ ] a\n## Log\n### 2026-10-02 (c)\n- Next: go\n"
	if got := section(text, "Goal"); got != "ship it\n\n" {
		t.Fatalf("Goal = %q", got)
	}
	if got := section(text, "Log"); got != "### 2026-10-02 (c)\n- Next: go\n" {
		t.Fatalf("Log = %q", got)
	}
	if got := section(text, "Steps"); got != "" {
		t.Fatalf("Steps = %q", got)
	}
}

func TestSlug(t *testing.T) {
	for in, want := range map[string]string{
		"Fix double-tap save":     "fix-double-tap-save",
		"Offline sync, web badge": "offline-sync-web-badge",
		"!!!":                     "change",
		"A very long title that keeps going past forty characters": "a-very-long-title-that-keeps-going-past",
		"Paths that don't exist yet compare through symlinks":      "paths-that-don-t-exist-yet-compare",
		"Large monorepos first, separate plans repo done right":    "large-monorepos-first-separate-plans",
		"Exactly forty characters long title here x":               "exactly-forty-characters-long-title-here",
		strings.Repeat("a", 45):                                    strings.Repeat("a", 40),
	} {
		if got := slug(in, 40); got != want {
			t.Errorf("slug(%q) = %q, want %q", in, got, want)
		}
	}
}

func TestHookWhere(t *testing.T) {
	h := &hook{prefixes: []string{"planning"}}
	for p, want := range map[string]place{
		"yass/changes/2026-01-01-x/prd.md":        {"changes", "prd.md", "yass/changes/2026-01-01-x"},
		"svc/yass/archive/2026-01-01-x/change.md": {"archive", "change.md", "svc/yass/archive/2026-01-01-x"},
		"planning/changes/2026-01-01-x/plan.md":   {"changes", "plan.md", "planning/changes/2026-01-01-x"},
		"planning/README.md":                      {kind: "meta"},
		"yass/README.md":                          {kind: "meta"},
		"yass.yaml":                               {kind: "meta"},
		"src/app.go":                              {},
	} {
		if got := h.where(p); got != want {
			t.Errorf("where(%q) = %+v, want %+v", p, got, want)
		}
	}
}

func TestNormIgnoresProgressMarks(t *testing.T) {
	a := norm("- [ ] AC1 x\n- [/] AC2 y\n")
	b := norm("- [x] AC1 x\n- [X] AC2 y\n")
	if !reflect.DeepEqual(a, b) {
		t.Fatalf("marking boxes changed intent: %v vs %v", a, b)
	}
	if reflect.DeepEqual(norm("- [ ] AC1 x\n"), norm("- [-] AC1 x\n")) {
		t.Fatal("dropping a box should count as an intent change")
	}
}

func TestCodeRefs(t *testing.T) {
	for in, want := range map[string][]string{
		"Disable Save — code: a1b2c3d":                          {"a1b2c3d"},
		"Queue table - code: A1B2C3D, e4f5a6b7c":                {"a1b2c3d", "e4f5a6b7c"},
		"Given x, then y — verify: unit code: 1234567":          {"1234567"},
		"Explain the code: it's fine":                           nil,
		"No refs here":                                          nil,
		"A repo-qualified token isn't read — code: web@a1b2c3d": nil,
		"Mixed — code: a1b2c3d, web@1234567":                    {"a1b2c3d"},
	} {
		if got := codeRefs(in); !reflect.DeepEqual(got, want) {
			t.Errorf("codeRefs(%q) = %v, want %v", in, got, want)
		}
	}
}

func TestSetMeta(t *testing.T) {
	for _, tc := range []struct{ in, want string }{
		{"---\nsource: x\narchived: old\nblocked:\n---\n# T\n", "---\nsource: x\narchived: NOW\nblocked:\n---\n# T\n"},
		{"---\nsource: x\n---\n# T\n- [x] archived: no\n", "---\nsource: x\narchived: NOW\n---\n# T\n- [x] archived: no\n"},
		{"---\r\nsource: x\r\n---\r\n# T\r\n", "---\r\nsource: x\r\narchived: NOW\r\n---\r\n# T\r\n"},
		{"---\n---\n# T\n", "---\narchived: NOW\n---\n# T\n"},
		{"# T\n", "---\narchived: NOW\n---\n# T\n"},
	} {
		if got := setMeta(tc.in, "archived", "NOW"); got != tc.want {
			t.Errorf("setMeta(%q) = %q, want %q", tc.in, got, tc.want)
		}
	}
}
