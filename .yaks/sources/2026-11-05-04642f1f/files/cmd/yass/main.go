// Command yass is the YASS CLI. See https://github.com/donjaime/yass.
package main

import (
	"os"
	"runtime/debug"

	"github.com/donjaime/yass/internal/yass"
)

// version is set by release builds: -ldflags "-X main.version=1.2.3". channel is "release" in
// release builds (-X main.channel=release), so yass update knows it may replace this binary.
var (
	version = "dev"
	channel = ""
)

func main() {
	if version == "dev" {
		if bi, ok := debug.ReadBuildInfo(); ok && bi.Main.Version != "" && bi.Main.Version != "(devel)" {
			version = bi.Main.Version // go install github.com/donjaime/yass/cmd/yass@v1.2.3
		}
	}
	os.Exit(yass.MainChannel(os.Args[1:], version, channel))
}
