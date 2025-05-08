package config

import (
	"log"
	"path/filepath"
	"sync"
	"time"

	"github.com/adrg/xdg"
	"github.com/fsnotify/fsnotify"

	"github.com/khing/hyde-ipc/internal/utils"
)

// ConfigWatcher monitors config file changes using fsnotify
type ConfigWatcher struct {
	watcher    *fsnotify.Watcher
	configPath string
	onChange   func() error // Changed to return error
	mutex      sync.Mutex
	lastEvent  time.Time
	cooldown   time.Duration // Added cooldown period
	verbose    bool
}

// NewConfigWatcher creates a new watcher for the config file
func NewConfigWatcher(onChange func() error, verbose bool) (*ConfigWatcher, error) {
	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		return nil, err
	}

	configPath := filepath.Join(xdg.ConfigHome, "hyde")

	cw := &ConfigWatcher{
		watcher:    watcher,
		configPath: configPath,
		onChange:   onChange,
		lastEvent:  time.Now(),
		cooldown:   500 * time.Millisecond, // Add cooldown to wait for complete writes
		verbose:    verbose,
	}

	return cw, nil
}

// Start begins watching for config file changes
func (cw *ConfigWatcher) Start() error {
	// Watch the directory containing the config file
	if err := cw.watcher.Add(cw.configPath); err != nil {
		return err
	}

	go cw.watchLoop()
	return nil
}

// watchLoop handles file system events
func (cw *ConfigWatcher) watchLoop() {
	pending := false
	var timer *time.Timer

	for {
		select {
		case event, ok := <-cw.watcher.Events:
			if !ok {
				return
			}

			// Check if this is our config.toml file
			if filepath.Base(event.Name) == "config.toml" {
				// Only reload on write or create events
				if event.Op&(fsnotify.Write|fsnotify.Create) != 0 {
					cw.mutex.Lock()

					// Set pending flag and create/reset timer
					if !pending {
						pending = true
						if timer != nil {
							timer.Stop()
						}
						timer = time.AfterFunc(cw.cooldown, func() {
							cw.handleConfigChange()
							cw.mutex.Lock()
							pending = false
							cw.mutex.Unlock()
						})
					}

					cw.mutex.Unlock()
				}
			}

		case err, ok := <-cw.watcher.Errors:
			if !ok {
				return
			}
			log.Printf("Config watcher error: %v", err)
		}
	}
}

// handleConfigChange processes config changes after cooldown
func (cw *ConfigWatcher) handleConfigChange() {
	utils.LogInfo("Config file changed, reloading...")

	// Try to reload, but keep existing config on failure
	err := cw.onChange()
	if err != nil {
		log.Printf("Failed to reload config: %v", err)
		log.Printf("Continuing with previous configuration")
	}
}

// Close stops the watcher
func (cw *ConfigWatcher) Close() error {
	return cw.watcher.Close()
}
