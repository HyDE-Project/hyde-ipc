package main

import (
	"bufio"
	"context"
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/adrg/xdg"

	"github.com/khing/hyde-ipc/internal/config"
	"github.com/khing/hyde-ipc/internal/utils"
)

var (
	cfg                *config.Config
	configLock         sync.RWMutex
	configHandler      *config.ConfigHandler
	configWatcher      *config.ConfigWatcher
	lastEvents         = make(map[string]time.Time)
	lastEventsLock     sync.Mutex
	commandLineTimeout int
)

func main() {
	// Parse command line flags
	verbose := flag.Bool("verbose", false, "Enable verbose logging")
	memLimit := flag.Int("memlimit", 8, "Memory limit in MB")
	noWatch := flag.Bool("nowatch", false, "Disable config file watching")
	cmdTimeout := flag.Int("timeout", 0, "Override script execution timeout in seconds (0 = use config value)")
	flag.Parse()

	// Setup logging
	utils.SetupLogging(*verbose)

	// Store command line timeout
	commandLineTimeout = *cmdTimeout

	// Set memory limit if needed
	if *memLimit > 0 {
		setMemoryLimit(*memLimit)
	}

	// Check and create default config if needed
	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		utils.LogInfo("Creating default config at %s", configPath)
		if err := createDefaultConfig(); err != nil {
			log.Fatal("Config error: ", err)
		}
	}

	// Load config
	if err := reloadConfig(*verbose); err != nil {
		log.Fatal("Config error: ", err)
	}

	// Setup config watcher unless disabled
	if !*noWatch {
		setupConfigWatcher(*verbose)
	}

	// Connect to Hyprland socket
	socketPath := getHyprlandSocketPath()
	conn, err := net.Dial("unix", socketPath)
	if err != nil {
		log.Fatal("Socket error: ", err)
	}
	defer conn.Close()

	utils.LogInfo("Connected to Hyprland socket, listening for events...")

	// Read events from socket
	scanner := bufio.NewScanner(conn)
	buf := make([]byte, 0, 64*1024)
	scanner.Buffer(buf, 64*1024)

	// Process events
	for scanner.Scan() {
		handleEvent(scanner.Text())
	}

	// Check for scanner errors
	if err := scanner.Err(); err != nil {
		log.Fatal("Socket error: ", err)
	}
}

// setMemoryLimit sets a soft limit on memory usage
func setMemoryLimit(limitMB int) {
	// Platform-dependent implementation would go here
	// Currently a no-op
}

// reloadConfig reloads the configuration
func reloadConfig(verbose bool) error {
	// Initialize handler if needed
	if configHandler == nil {
		var err error
		configHandler, err = config.NewConfigHandler(verbose)
		if err != nil {
			return fmt.Errorf("failed to create config handler: %w", err)
		}
	}

	// Load config using the handler
	newConfig, err := configHandler.Load()
	if err != nil {
		// If we have a previous valid config, keep using it
		lastValid := configHandler.GetLastValidConfig()
		if lastValid != nil {
			log.Printf("Warning: config load error: %v", err)
			log.Printf("Continuing with previous valid configuration")
			return nil // Not a fatal error if we can use previous config
		}
		return fmt.Errorf("failed to load config and no previous valid config available: %w", err)
	}

	// Update configuration with write lock
	configLock.Lock()
	defer configLock.Unlock()

	// Store the new configuration
	cfg = newConfig

	// Clear event debounce cache on config reload
	lastEventsLock.Lock()
	for k := range lastEvents {
		delete(lastEvents, k)
	}
	lastEventsLock.Unlock()

	// Debug config contents
	utils.LogInfo("Configuration loaded successfully")
	utils.LogInfo("Settings: max_concurrent=%d, timeout=%d, debounce_time=%d",
		cfg.HydeIPC.MaxConcurrent,
		cfg.HydeIPC.Timeout,
		cfg.HydeIPC.DebounceTime)

	utils.LogInfo("Configured events (%d):", len(cfg.HyprlandIPC))
	for event, script := range cfg.HyprlandIPC {
		if script != "" {
			utils.LogInfo("  %s -> %s", event, utils.LimitString(script, 50))
		}
	}

	return nil
}

// setupConfigWatcher initializes the config file watcher
func setupConfigWatcher(verbose bool) {
	watcher, err := config.NewConfigWatcher(func() error {
		return reloadConfig(verbose)
	}, verbose)

	if err != nil {
		log.Printf("Failed to initialize config watcher: %v", err)
		return
	}

	if err := watcher.Start(); err != nil {
		log.Printf("Failed to start config watcher: %v", err)
		return
	}

	configWatcher = watcher
	utils.LogInfo("Config file watcher started")
}

// getHyprlandSocketPath returns the path to the Hyprland IPC socket
func getHyprlandSocketPath() string {
	his := os.Getenv("HYPRLAND_INSTANCE_SIGNATURE")
	if his == "" {
		log.Fatal("HYPRLAND_INSTANCE_SIGNATURE not set")
	}

	runtimeDir := os.Getenv("XDG_RUNTIME_DIR")
	if runtimeDir == "" {
		log.Fatal("XDG_RUNTIME_DIR not set")
	}

	return filepath.Join(runtimeDir, "hypr", his, ".socket2.sock")
}

// handleEvent processes Hyprland events directly without queuing
func handleEvent(eventLine string) {
	parts := strings.SplitN(eventLine, ">>", 2)
	if len(parts) != 2 {
		return
	}

	eventName := parts[0]
	eventData := parts[1]

	if shouldDebounce(eventName) {
		return
	}

	configLock.RLock()
	script, exists := cfg.HyprlandIPC[eventName]
	configLock.RUnlock()

	if !exists || script == "" {
		return
	}

	// Process placeholders in the script
	if strings.Contains(script, "{") {
		// {0} always represents ALL data
		if strings.Contains(script, "{0}") {
			script = strings.Replace(script, "{0}", eventData, -1)
		}

		// Handle individual positional arguments starting at {1}
		dataArgs := strings.Split(eventData, ",")
		for i, arg := range dataArgs {
			// Use {1} for first item, {2} for second, etc.
			placeholder := fmt.Sprintf("{%d}", i+1)
			script = strings.Replace(script, placeholder, arg, -1)
		}
	}

	// Extract the command from the script
	cmdName := script
	if idx := strings.Index(script, " "); idx > 0 {
		cmdName = script[:idx]
	}

	// Check if command exists
	_, err := exec.LookPath(cmdName)
	if err != nil {
		utils.LogInfo("Command not found: %s", cmdName)
		return
	}

	// Launch immediately in a separate goroutine
	go executeScript(eventName, script, eventData)
}

// executeScript runs a script for an event with timeout handling
func executeScript(eventName, script, eventData string) {
	utils.LogInfo("Executing script for event: %s", eventName)

	// Create command
	cmd := exec.Command("sh", "-c", script)
	cmd.Env = append(os.Environ(), "HYDE_EVENT_DATA="+eventData)

	// Set timeout context - use command line value if specified
	var timeout time.Duration
	if commandLineTimeout > 0 {
		timeout = time.Duration(commandLineTimeout) * time.Second
		utils.LogInfo("Using command-line timeout of %d seconds for event %s", commandLineTimeout, eventName)
	} else {
		timeout = time.Duration(cfg.HydeIPC.Timeout) * time.Second
	}
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	// Run with timeout
	done := make(chan struct {
		output []byte
		err    error
	}, 1)

	go func() {
		output, err := cmd.CombinedOutput()
		select {
		case done <- struct {
			output []byte
			err    error
		}{output, err}:
		case <-ctx.Done():
			// Context already done, don't send results
		}
	}()

	// Wait for completion or timeout
	select {
	case result := <-done:
		if result.err != nil && utils.Verbose {
			utils.LogInfo("Script error [%s]: %v", eventName, result.err)
		} else if utils.Verbose && len(result.output) > 0 {
			utils.LogInfo("Script output [%s]: %s", eventName, utils.LimitString(string(result.output), 100))
		}
	case <-ctx.Done():
		// Try to kill the process if it times out
		if cmd.Process != nil {
			cmd.Process.Kill()
		}
		utils.LogInfo("Script timeout [%s] after %d seconds", eventName, timeout/time.Second)
	}
}

// shouldDebounce checks if an event should be debounced (ignored)
func shouldDebounce(eventName string) bool {
	// Never debounce certain critical events
	if eventName == "urgent" || eventName == "closewindow" {
		return false
	}

	lastEventsLock.Lock()
	defer lastEventsLock.Unlock()

	now := time.Now()
	lastTime, exists := lastEvents[eventName]

	// If event was seen recently, debounce it
	if exists && now.Sub(lastTime) < time.Duration(cfg.HydeIPC.DebounceTime)*time.Millisecond {
		return true
	}

	// Update last seen time
	lastEvents[eventName] = now

	// Occasionally clean up old entries
	if now.Nanosecond()%100 == 0 {
		for k, v := range lastEvents {
			if now.Sub(v) > 10*time.Second {
				delete(lastEvents, k)
			}
		}
	}

	return false
}

// createDefaultConfig creates a default config file if none exists
func createDefaultConfig() error {
	configDir := filepath.Join(xdg.ConfigHome, "hyde")

	if err := os.MkdirAll(configDir, 0755); err != nil {
		return err
	}

	configPath := filepath.Join(configDir, "config.toml")

	if _, err := os.Stat(configPath); err == nil {
		return nil
	}

	defaultConfig := `# Hyde IPC Configuration for Hyprland

[hyde-ipc]
# Maximum number of concurrent script executions
max_concurrent = 2
# Timeout for script execution in seconds
timeout = 60
# Debounce time for frequent events in milliseconds
debounce_time = 100

[hyprland-ipc]
# Most common events are configured here
windowtitle = "notify-send \"Window Title Changed\" \"$HYDE_EVENT_DATA\""
workspace = ""
fullscreen = ""
screencast = ""
activewindow = ""
`
	return os.WriteFile(configPath, []byte(defaultConfig), 0644)
}
