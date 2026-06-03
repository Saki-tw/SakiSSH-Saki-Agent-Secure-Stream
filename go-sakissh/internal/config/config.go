package config

import (
	"encoding/json"
	"os"
	"path/filepath"
)

type TlsConfig struct {
	Enabled           bool   `json:"enabled"`
	CertPath          string `json:"cert_path"`
	KeyPath           string `json:"key_path"`
	CaCertPath        string `json:"ca_cert_path"`
	RequireClientCert bool   `json:"require_client_cert"`
	AutoGenerate      bool   `json:"auto_generate"`
}

type DaemonConfig struct {
	BindAddress  string    `json:"bind_address"`
	Tls          TlsConfig `json:"tls"`
	// Additional fields like audit, filetransfer omitted for brevity in Go dual-impl init
}

func DefaultPath() string {
	home, err := os.UserHomeDir()
	if err == nil {
		configDir := filepath.Join(home, ".sakissh")
		os.MkdirAll(configDir, 0755)
		return filepath.Join(configDir, "config.json")
	}
	exe, _ := os.Executable()
	return filepath.Join(filepath.Dir(exe), "config.json")
}

func LoadOrCreate(path string) (*DaemonConfig, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			cfg := &DaemonConfig{
				BindAddress: "0.0.0.0:19284",
				Tls: TlsConfig{
					Enabled:      true,
					AutoGenerate: true,
				},
			}
			data, _ = json.MarshalIndent(cfg, "", "  ")
			os.WriteFile(path, data, 0644)
			return cfg, nil
		}
		return nil, err
	}
	var cfg DaemonConfig
	if err := json.Unmarshal(data, &cfg); err != nil {
		return nil, err
	}
	return &cfg, nil
}
