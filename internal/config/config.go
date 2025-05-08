package config

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/adrg/xdg"
	"github.com/pelletier/go-toml/v2"

	"github.com/khing/hyde-ipc/internal/utils"
)

// Config stores the application configuration
type Config struct {
	HydeIPC struct {
		MaxConcurrent int `toml:"max_concurrent"`
		Timeout       int `toml:"timeout"`
		DebounceTime  int `toml:"debounce_time"`
	} `toml:"hyde-ipc"`
	HyprlandIPC map[string]string `toml:"hyprland-ipc"`
}

// ConfigHandler manages all configuration operations
type ConfigHandler struct {
	path      string
	rawData   map[string]interface{}
	lastValid *Config
	fileSize  int64
	verbose   bool
}

// NewConfigHandler creates a new config handler
func NewConfigHandler(verbose bool) (*ConfigHandler, error) {
	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")
	return &ConfigHandler{
		path:      configPath,
		rawData:   make(map[string]interface{}),
		lastValid: nil,
		verbose:   verbose,
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

	// Create a new config with defaults
	cfg := &Config{
		HyprlandIPC: make(map[string]string),
	}

	// Set default settings with 60-second timeout
	cfg.HydeIPC.MaxConcurrent = 2
	cfg.HydeIPC.Timeout = 60
	cfg.HydeIPC.DebounceTime = 100

	// Parse the TOML directly into our config struct using pelletier/go-toml/v2
	if err := toml.Unmarshal(content, cfg); err != nil {
		return nil, fmt.Errorf("failed to parse toml: %w", err)
	}

	// Also parse into a raw map for debugging
	if err := toml.Unmarshal(content, &h.rawData); err != nil {
		return nil, fmt.Errorf("failed to parse raw toml: %w", err)
	}

	// Validate that we have extracted necessary data
	if len(h.rawData) == 0 {
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

// FindSection searches for a section by name (case-insensitive)
func (h *ConfigHandler) FindSection(sectionName string) (interface{}, bool) {
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
	if !h.verbose {
		return
	}

	utils.LogInfo("Raw config sections:")
	if len(h.rawData) == 0 {
		utils.LogInfo("  <empty config>")
		return
	}

	for key, val := range h.rawData {
		utils.LogInfo("  [%s] -> %T", key, val)

		// Print nested maps
		if mapVal, ok := val.(map[string]interface{}); ok {
			for k, v := range mapVal {
				utils.LogInfo("    %s = %v (%T)", k, v, v)
			}
		}
	}
}
