package server

import (
	"strings"
)

func InjectVolumeReductionEnv(command string, env map[string]string) map[string]string {
	cmdLower := strings.ToLower(command)

	if strings.Contains(cmdLower, "npm") || strings.Contains(cmdLower, "yarn") || strings.Contains(cmdLower, "pnpm") {
		env["npm_config_cache"] = "/tmp/sass_volatile_cache/npm"
		env["YARN_CACHE_FOLDER"] = "/tmp/sass_volatile_cache/yarn"
	} else if strings.Contains(cmdLower, "cargo") || strings.Contains(cmdLower, "rustc") {
		env["CARGO_TARGET_DIR"] = "/tmp/sass_volatile_cache/cargo_target"
		env["CARGO_HOME"] = "/tmp/sass_volatile_cache/cargo_home"
	} else if strings.Contains(cmdLower, "pip") {
		env["PIP_CACHE_DIR"] = "/tmp/sass_volatile_cache/pip"
	}

	env["TMPDIR"] = "/tmp/sass_volatile_cache/tmp"
	return env
}
