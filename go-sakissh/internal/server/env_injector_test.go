// =============================================================================
// Package server — env_injector_test.go
// SASS Plugin #7: Volatile Cache Redirection (EnvInjector) 單元測試
//
// 對照 C# PluginTests.cs: EnvInjectorTests
// Copyright (c) 2026 Saki Studio. All rights reserved.
// =============================================================================

package server

import (
	"strings"
	"testing"
)

// --- npm / yarn / pnpm ---

func TestInjectVolumeReductionEnv_NpmCommand_InjectsNpmAndYarnVars(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("npm install express", env)

	if _, ok := result["npm_config_cache"]; !ok {
		t.Error("npm 指令應注入 npm_config_cache")
	}
	if _, ok := result["YARN_CACHE_FOLDER"]; !ok {
		t.Error("npm 指令應注入 YARN_CACHE_FOLDER")
	}
	if !strings.Contains(result["npm_config_cache"], "sass_vol") {
		t.Errorf("npm_config_cache 應包含 sass_vol，實際為 %q", result["npm_config_cache"])
	}
}

func TestInjectVolumeReductionEnv_YarnCommand_MatchesNodeDetection(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("yarn add lodash", env)

	if _, ok := result["npm_config_cache"]; !ok {
		t.Error("yarn 指令應注入 npm_config_cache")
	}
	if _, ok := result["YARN_CACHE_FOLDER"]; !ok {
		t.Error("yarn 指令應注入 YARN_CACHE_FOLDER")
	}
}

func TestInjectVolumeReductionEnv_PnpmCommand_InjectsNodeVars(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("pnpm install", env)

	if _, ok := result["npm_config_cache"]; !ok {
		t.Error("pnpm 指令應注入 npm_config_cache")
	}
}

// --- cargo / rustc ---

func TestInjectVolumeReductionEnv_CargoCommand_InjectsCargoVars(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("cargo build --release", env)

	if _, ok := result["CARGO_TARGET_DIR"]; !ok {
		t.Error("cargo 指令應注入 CARGO_TARGET_DIR")
	}
	if _, ok := result["CARGO_HOME"]; !ok {
		t.Error("cargo 指令應注入 CARGO_HOME")
	}
}

func TestInjectVolumeReductionEnv_RustcCommand_InjectsCargoVars(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("rustc main.rs", env)

	if _, ok := result["CARGO_TARGET_DIR"]; !ok {
		t.Error("rustc 指令應注入 CARGO_TARGET_DIR")
	}
	if _, ok := result["CARGO_HOME"]; !ok {
		t.Error("rustc 指令應注入 CARGO_HOME")
	}
}

// --- pip ---

func TestInjectVolumeReductionEnv_PipCommand_InjectsPipVar(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("pip install requests", env)

	if _, ok := result["PIP_CACHE_DIR"]; !ok {
		t.Error("pip 指令應注入 PIP_CACHE_DIR")
	}
}

// --- TMPDIR 一律注入 ---

func TestInjectVolumeReductionEnv_AnyCommand_AlwaysInjectsTmpdir(t *testing.T) {
	t.Parallel()
	commands := []string{
		"ls -la",
		"npm install",
		"cargo build",
		"pip install",
		"echo hello",
	}
	for _, cmd := range commands {
		env := make(map[string]string)
		result := InjectVolumeReductionEnv(cmd, env)
		if _, ok := result["TMPDIR"]; !ok {
			t.Errorf("指令 %q 應注入 TMPDIR", cmd)
		}
	}
}

func TestInjectVolumeReductionEnv_Tmpdir_PointsToVolatileCache(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("echo test", env)

	tmpdir := result["TMPDIR"]
	if !strings.Contains(tmpdir, "sass_vol") {
		t.Errorf("TMPDIR 應指向 sass_vol，實際為 %q", tmpdir)
	}
}

// --- 大小寫不敏感 ---

func TestInjectVolumeReductionEnv_CaseInsensitive(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("NPM INSTALL", env)

	if _, ok := result["npm_config_cache"]; !ok {
		t.Error("大寫 NPM 指令應被偵測到（大小寫不敏感）")
	}
}

// --- 不污染不相關的變數 ---

func TestInjectVolumeReductionEnv_NpmDoesNotInjectCargoVars(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("npm install", env)

	if _, ok := result["CARGO_TARGET_DIR"]; ok {
		t.Error("npm 指令不應注入 CARGO_TARGET_DIR")
	}
	if _, ok := result["CARGO_HOME"]; ok {
		t.Error("npm 指令不應注入 CARGO_HOME")
	}
	if _, ok := result["PIP_CACHE_DIR"]; ok {
		t.Error("npm 指令不應注入 PIP_CACHE_DIR")
	}
}

func TestInjectVolumeReductionEnv_CargoDoesNotInjectNpmVars(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("cargo build", env)

	if _, ok := result["npm_config_cache"]; ok {
		t.Error("cargo 指令不應注入 npm_config_cache")
	}
	if _, ok := result["YARN_CACHE_FOLDER"]; ok {
		t.Error("cargo 指令不應注入 YARN_CACHE_FOLDER")
	}
}

// --- 既有環境變數保留 ---

func TestInjectVolumeReductionEnv_PreservesExistingEnvVars(t *testing.T) {
	t.Parallel()
	env := map[string]string{
		"HOME":  "/home/user",
		"PATH":  "/usr/bin",
		"SHELL": "/bin/zsh",
	}
	result := InjectVolumeReductionEnv("ls", env)

	if result["HOME"] != "/home/user" {
		t.Error("既有的 HOME 變數不應被覆蓋")
	}
	if result["PATH"] != "/usr/bin" {
		t.Error("既有的 PATH 變數不應被覆蓋")
	}
}

// --- 回傳值是同一個 map ---

func TestInjectVolumeReductionEnv_ReturnsSameMap(t *testing.T) {
	t.Parallel()
	env := make(map[string]string)
	result := InjectVolumeReductionEnv("ls", env)

	// Go 中 map 是參考型別，回傳的應該是同一個 map
	env["TEST_KEY"] = "test_value"
	if result["TEST_KEY"] != "test_value" {
		t.Error("回傳值應為同一個 map（參考型別）")
	}
}
