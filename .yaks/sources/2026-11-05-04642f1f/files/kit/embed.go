// Package kit carries the files YASS installs into a repo: the playbooks and the hook script.
// Every yass binary embeds them, so `yass init` can set up a repo from the binary alone.
package kit

import "embed"

// FS holds .agents/skills/yass-*/SKILL.md and tools/yass/githooks/pre-commit, unstamped;
// `yass init` adds the version as it writes them.
//
//go:embed all:.agents tools
var FS embed.FS
