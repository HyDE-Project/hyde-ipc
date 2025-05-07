package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/BurntSushi/toml"
	"github.com/adrg/xdg"
)

// ConfigHandler manages all configuration operations
type ConfigHandler struct {
	path      string
	rawData   map[string]interface{}
	lastValid *Config // Keep track of last valid config
	fileSize  int64   // Track file size to detect empty files
}

// NewConfigHandler creates a new config handler
func NewConfigHandler() (*ConfigHandler, error) {
	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")
	return &ConfigHandler{
		path:      configPath,
		rawData:   make(map[string]interface{}),
		lastValid: nil,
	}, nil
}

// Load reads and parses the config file
func (h *ConfigHandler) Load() (*Config, error) {
	// Check if config file exists
	stat, err := os.Stat(h.path)
	if err != nil {
		if os.IsNotExist(err) {
			return nil, fmt.Errorf("config file not found at %s", h.path)
		}
		return nil, err
	}

	// Check if file is empty or too small to be valid
	if stat.Size() < 10 {
		return nil, fmt.Errorf("config file is empty or too small: %d bytes", stat.Size())
	}

	h.fileSize = stat.Size()

	// Read the file content
	content, err := os.ReadFile(h.path)
	if err != nil {
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	// Verify that content doesn't have common editor temp patterns
	if strings.Contains(string(content), "<<<<<<< HEAD") ||
		strings.Contains(string(content), ">>>>>>> ") {
		return nil, fmt.Errorf("config appears to contain merge conflict markers")
	}

	// Parse toml data into a generic map
	rawData := make(map[string]interface{})
	if err := toml.Unmarshal(content, &rawData); err != nil {
		return nil, fmt.Errorf("failed to parse toml: %w", err)
	}
	h.rawData = rawData

	// Create a new config with defaults
	cfg := &Config{
		HyprlandIPC: make(map[string]string),
	}

	// Set default settings
	cfg.Settings.MaxConcurrent = 2
	cfg.Settings.Timeout = 5 * time.Second
	cfg.Settings.DebounceTime = 100 * time.Millisecond

	// Extract settings
	if settings, ok := h.findSection("settings"); ok {
		if settingsMap, ok := settings.(map[string]interface{}); ok {
			if val, ok := settingsMap["max_concurrent"].(int64); ok {
				cfg.Settings.MaxConcurrent = int(val)
			}
			if val, ok := settingsMap["timeout"].(int64); ok {
				cfg.Settings.Timeout = time.Duration(val) * time.Second
			}
			if val, ok := settingsMap["debounce_time"].(int64); ok {
				cfg.Settings.DebounceTime = time.Duration(val) * time.Millisecond
			}
		}
	}

	// Extract hyprland-ipc events
	if events, ok := h.findSection("hyprland-ipc"); ok {
		if eventsMap, ok := events.(map[string]interface{}); ok {
			for key, val := range eventsMap {
				if strVal, ok := val.(string); ok {
					cfg.HyprlandIPC[key] = strVal
				}
			}
		}
	}

	// Validate that we have extracted necessary data
	if len(rawData) == 0 {
		return nil, fmt.Errorf("config file parsing resulted in empty data")
	}

	// Store as last valid config
	h.lastValid = cfg

	return cfg, nil
}

// GetLastValidConfig returns the last successfully loaded config
func (h *ConfigHandler) GetLastValidConfig() *Config {
	return h.lastValid
}

// findSection searches for a section by name (case-insensitive)
func (h *ConfigHandler) findSection(sectionName string) (interface{}, bool) {
	// Try direct lookup first (for performance)
	if val, ok := h.rawData[sectionName]; ok {
		return val, true
	}

	// Try case-insensitive lookup
	lowerName := strings.ToLower(sectionName)
	for key, val := range h.rawData {
		if strings.ToLower(key) == lowerName {
			return val, true
		}
	}

	return nil, false
}

// DumpConfig prints the parsed config for debugging
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

		// Print nested maps
		if mapVal, ok := val.(map[string]interface{}); ok {
			for k, v := range mapVal {
				logInfo("    %s = %v (%T)", k, v, v)
			}
		}
	}
}
