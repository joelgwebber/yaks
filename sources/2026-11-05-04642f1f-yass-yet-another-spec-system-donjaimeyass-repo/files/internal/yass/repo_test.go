package yass

import (
	"reflect"
	"strings"
	"testing"
)

func TestByCreated(t *testing.T) {
	mk := func(name, created string) *Change {
		return &Change{Name: name, Meta: map[string]string{"created": created}}
	}
	cs := []*Change{ // as loaded: by name
		mk("2026-10-04-zebra", ""),
		mk("2026-10-05-apple", "2026-10-05T15:00:00Z"),
		mk("2026-10-05-bad-stamp", "yesterday"),
		mk("2026-10-05-mango", ""),
		mk("2026-10-05-zebra", "2026-10-05T10:00:00-04:00"),
		mk("no-date", ""),
	}
	byCreated(cs)
	var got []string
	for _, c := range cs {
		got = append(got, c.Name)
	}
	want := []string{"no-date", "2026-10-04-zebra", "2026-10-05-bad-stamp", "2026-10-05-mango", "2026-10-05-zebra", "2026-10-05-apple"}
	if !reflect.DeepEqual(got, want) {
		t.Fatalf("byCreated = %q, want %q", got, want)
	}
}

func TestChangeFolder(t *testing.T) {
	for _, tc := range []struct{ path, want string }{
		{"archive/2026-10-01-a/change.md", "archive/2026-10-01-a"},
		{"archive/2026/10/2026-10-01-a/change.md", "archive/2026/10/2026-10-01-a"},
		{"archive/2026/10/2026-10-01-a/2026-10-02-p/change.md", "archive/2026/10/2026-10-01-a"},
		{"changes/2026/10/x/change.md", "changes/2026"},
	} {
		if got := changeFolder(strings.Split(tc.path, "/")); got != tc.want {
			t.Errorf("changeFolder(%q) = %q, want %q", tc.path, got, tc.want)
		}
	}
}

func TestDeliveredIDs(t *testing.T) {
	for text, want := range map[string][]int{
		"Delivers AC1, AC3 in [plan.md](../plan.md)": {1, 3},
		"Delivers AC2–AC5":                           {2, 3, 4, 5},
		"Delivers AC2-AC4, AC9":                      {2, 3, 4, 9},
		"Delivers AC1, AC10, and AC19's preview card, in [plan.md](../plan.md)":      {1, 10, 19},
		"Delivers AC1, AC2 and AC3 of the plan.":                                     {1, 2, 3},
		"Delivers AC9–AC19, AC34 and AC35 in plan.md; the live checks are elsewhere": {9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 34, 35},
		"Builds AC1 but doesn't say delivers":                                        nil,
		"Fix AC2 (no delivers here)":                                                 nil,
	} {
		if got := deliveredIDs(text); !reflect.DeepEqual(got, want) {
			t.Errorf("deliveredIDs(%q) = %v, want %v", text, got, want)
		}
	}
}

func TestMarkDelivered(t *testing.T) {
	in := "## Acceptance\n<!-- e.g.\n- [ ] AC1 (R1) Given … -->\n- [ ] AC1 (R1) real\n- [/] AC2 (R1) going\n- [ ] AC3 (R2) not delivered\n- [-] AC4 dropped\r\n- [ ] AC5 crlf\r\nprose AC1\n"
	want := "## Acceptance\n<!-- e.g.\n- [ ] AC1 (R1) Given … -->\n- [x] AC1 (R1) real\n- [x] AC2 (R1) going\n- [ ] AC3 (R2) not delivered\n- [-] AC4 dropped\r\n- [x] AC5 crlf\r\nprose AC1\n"
	if got := markDelivered(in, map[int]bool{1: true, 2: true, 3: false, 4: true, 5: true}); got != want {
		t.Errorf("markDelivered =\n%q\nwant\n%q", got, want)
	}
}
