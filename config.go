package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/adrg/xdg"
)

type ConfigHandler struct {
	path      string
	rawData   map[string]interface{}
	lastValid *Config
	fileSize  int64
}

func NewConfigHandler() (*ConfigHandler, error) {
	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")
	return &ConfigHandler{
		path:      configPath,
		rawData:   make(map[string]interface{}),
		lastValid: nil,
	}, nil
}

func (h *ConfigHandler) Load() (*Config, error) {

	stat, err := os.Stat(h.path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("config file not found at %s", h.path)
		}
		return nil, err
	}

	if stat.Size() < 10 {
		return nil, fmt.Errorf("config file is empty or too small: %d bytes", stat.Size())
	}

	h.fileSize = stat.Size()

	content, err := os.ReadFile(h.path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	if strings.Contains(string(content), "<<<<<<< HEAD") ||
		strings.Contains(string(content), ">>>>>>> ") {
		return nil, fmt.Errorf("config appears to contain merge conflict markers")
	}

	cfg := &Config{
		HyprlandIPC: make(map[string]string),
	}

	cfg.HydeIPC.MaxConcurrent = 2
	cfg.HydeIPC.Timeout = 999999
	cfg.HydeIPC.DebounceTime = 100

	_, err = toml.Decode(string(content), cfg)
	if err != nil {
		return nil, fmt.Errorf("failed to parse toml: %w", err)
	}

	if err := toml.Unmarshal(content, &h.rawData); err != nil {
		return nil, fmt.Errorf("failed to parse raw toml: %w", err)
	}

	if len(h.rawData) == 0 {
		return nil, fmt.Errorf("config file parsing resulted in empty data")
	}

	h.lastValid = cfg

	return cfg, nil
}

func (h *ConfigHandler) GetLastValidConfig() *Config {
	return h.lastValid
}

func (h *ConfigHandler) findSection(sectionName string) (interface{}, bool) {

	if val, ok := h.rawData[sectionName]; ok {
		return val, true
	}

	lowerName := strings.ToLower(sectionName)
	for key, val := range h.rawData {
		if strings.ToLower(key) == lowerName {
			return val, true
		}
	}

	return nil, false
}

func (h *ConfigHandler) DumpConfig() {
	if !verbose {
		return
	}

	logInfo("Raw config sections:")
	if len(h.rawData) == 0 {
		logInfo("  <empty config>")
		return
	}

	for key, val := range h.rawData {
		logInfo("  [%s] -> %T", key, val)

		if mapVal, ok := val.(map[string]interface{}); ok {
			for k, v := range mapVal {
				logInfo("    %s = %v (%T)", k, v, v)
			}
		}
	}
}
