package defense

import (
	"os"
	"path/filepath"

	"gopkg.in/yaml.v3"
)

type Policy13 struct {
	DangerousCommands []string `yaml:"dangerous_commands"`
	TarpitSizeMB      int      `yaml:"tarpit_size_mb"`
}

func DefaultPolicy() *Policy13 {
	return &Policy13{
		DangerousCommands: []string{
			"rm -rf /",
			"mkfs",
			"dd if=/dev/zero",
			":(){ :|:& };:",
		},
		TarpitSizeMB: 40,
	}
}

func LoadOrCreatePolicy() *Policy13 {
	home, err := os.UserHomeDir()
	if err != nil {
		return DefaultPolicy()
	}
	path := filepath.Join(home, ".sakissh", "13policy.yaml")
	data, err := os.ReadFile(path)
	if err != nil {
		p := DefaultPolicy()
		data, _ = yaml.Marshal(p)
		os.WriteFile(path, data, 0644)
		return p
	}
	var p Policy13
	if err := yaml.Unmarshal(data, &p); err != nil {
		return DefaultPolicy()
	}
	return &p
}
