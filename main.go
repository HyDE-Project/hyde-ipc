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

var (
	config             *Config
	configLock         sync.RWMutex
	verbose            bool
	executor           *CommandExecutor
	configWatcher      *ConfigWatcher
	dispatcher         *EventDispatcher
	lastEvents         = make(map[string]time.Time)
	lastEventsLock     sync.Mutex
	configHandler      *ConfigHandler
	commandLineTimeout int
)

func main() {
	flag.BoolVar(&verbose, "verbose", false, "Enable verbose logging")
	memLimit := flag.Int("memlimit", 8, "Memory limit in MB")
	noWatch := flag.Bool("nowatch", false, "Disable config file watching")
	cmdTimeout := flag.Int("timeout", 0, "Override script execution timeout in seconds (0 = use config value)")
	flag.Parse()

	setMemoryLimit(*memLimit)

	// Store command line timeout for later use
	commandLineTimeout = *cmdTimeout

	log.SetPrefix("hyde-ipc: ")
	if verbose {
		log.SetFlags(log.Ldate | log.Ltime)
	} else {
		log.SetFlags(0)
	}

	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		logInfo("Creating default config at %s", configPath)
		if err := createDefaultConfig(); err != nil {
			log.Fatal("Config error: ", err)
		}
	}

	if err := reloadConfig(); err != nil {
		log.Fatal("Config error: ", err)
	}

	// Initialize event dispatcher (queue size = 100)
	dispatcher = NewEventDispatcher(config.HydeIPC.MaxConcurrent, 100, executor)

	if !*noWatch {
		setupConfigWatcher()
	}

	socketPath := getHyprlandSocketPath()
	conn, err := net.Dial("unix", socketPath)
	if err != nil {
		log.Fatal("Socket error: ", err)
	}
	defer conn.Close()

	logInfo("Connected to Hyprland socket, listening for events...")

	scanner := bufio.NewScanner(conn)

	buf := make([]byte, 0, 64*1024)
	scanner.Buffer(buf, 64*1024)

	for scanner.Scan() {
		handleEvent(scanner.Text())
	}

	if err := scanner.Err(); err != nil {
		log.Fatal("Socket error: ", err)
	}
}

func setMemoryLimit(limitMB int) {

}

func logInfo(format string, v ...interface{}) {
	if verbose {
		log.Printf(format, v...)
	}
}

// reloadConfig reloads the configuration from file
func reloadConfig() error {
	// Initialize handler if needed
	if configHandler == nil {
		var err error
		configHandler, err = NewConfigHandler()
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

	// Dump raw config structure for debugging
	configHandler.DumpConfig()

	// Update configuration with write lock
	configLock.Lock()
	defer configLock.Unlock()

	// Store the old configuration for comparison
	oldConfig := config
	config = newConfig

	// Initialize or update executor based on max_concurrent setting
	maxConcurrent := config.HydeIPC.MaxConcurrent
	if maxConcurrent <= 0 {
		maxConcurrent = 2
	}

	if executor == nil {
		executor = NewCommandExecutor(maxConcurrent)
	} else if oldConfig != nil && oldConfig.HydeIPC.MaxConcurrent != maxConcurrent {
		// Only create a new executor if the concurrency setting changed
		executor = NewCommandExecutor(maxConcurrent)
	}

	// Create or update dispatcher if needed
	if dispatcher == nil && executor != nil {
		dispatcher = NewEventDispatcher(maxConcurrent, 100, executor)
	}

	// Clear event debounce cache on config reload
	lastEventsLock.Lock()
	for k := range lastEvents {
		delete(lastEvents, k)
	}
	lastEventsLock.Unlock()

	// Debug config contents
	logInfo("Configuration reloaded successfully")
	logInfo("Settings: max_concurrent=%d, timeout=%d, debounce_time=%d",
		config.HydeIPC.MaxConcurrent,
		config.HydeIPC.Timeout,
		config.HydeIPC.DebounceTime)

	logInfo("Configured events (%d):", len(config.HyprlandIPC))
	for event, script := range config.HyprlandIPC {
		if script != "" {
			logInfo("  %s -> %s", event, limitString(script, 50))
		}
	}

	return nil
}

func setupConfigWatcher() {
	watcher, err := NewConfigWatcher(reloadConfig)

	if err != nil {
		log.Printf("Failed to initialize config watcher: %v", err)
		return
	}

	if err := watcher.Start(); err != nil {
		log.Printf("Failed to start config watcher: %v", err)
		return
	}

	configWatcher = watcher
	logInfo("Config file watcher started")
}

func loadConfig() (*Config, error) {
	if err := reloadConfig(); err != nil {
		return nil, err
	}
	return config, nil
}

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
	script, exists := config.HyprlandIPC[eventName]
	configLock.RUnlock()

	if !exists || script == "" {
		return
	}

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
		logInfo("Command not found: %s", cmdName)
		return
	}

	// IMPORTANT: Launch immediately in a completely separate goroutine
	// This is the key fix to prevent blocking
	go func(eventName, script, eventData string) {
		logInfo("Executing script for event: %s", eventName)

		// Create command
		cmd := exec.Command("sh", "-c", script)
		cmd.Env = append(os.Environ(), "HYDE_EVENT_DATA="+eventData)

		// Set timeout context - use command line value if specified
		var timeout time.Duration
		if commandLineTimeout > 0 {
			timeout = time.Duration(commandLineTimeout) * time.Second
			logInfo("Using command-line timeout of %d seconds for event %s", commandLineTimeout, eventName)
		} else {
			timeout = time.Duration(config.HydeIPC.Timeout) * time.Second
		}
		ctx, cancel := context.WithTimeout(context.Background(), timeout)
		defer cancel()

		// Run with timeout
		done := make(chan error, 1)
		var output []byte

		go func() {
			var err error
			output, err = cmd.CombinedOutput()
			done <- err
		}()

		// Wait for completion or timeout
		select {
		case err := <-done:
			if err != nil && verbose {
				logInfo("Script error [%s]: %v", eventName, err)
			} else if verbose && len(output) > 0 {
				logInfo("Script output [%s]: %s", eventName, limitString(string(output), 100))
			}
		case <-ctx.Done():
			// Try to kill the process if it times out
			if cmd.Process != nil {
				cmd.Process.Kill()
			}
			logInfo("Script timeout [%s] after %d seconds", eventName, timeout/time.Second)
		}
	}(eventName, script, eventData)
}

func shouldDebounce(eventName string) bool {

	if eventName == "urgent" || eventName == "closewindow" {
		return false
	}

	lastEventsLock.Lock()
	defer lastEventsLock.Unlock()

	now := time.Now()
	lastTime, exists := lastEvents[eventName]

	if exists && now.Sub(lastTime) < time.Duration(config.HydeIPC.DebounceTime)*time.Millisecond {
		return true
	}

	lastEvents[eventName] = now

	if now.Nanosecond()%100 == 0 {
		for k, v := range lastEvents {
			if now.Sub(v) > 10*time.Second {
				delete(lastEvents, k)
			}
		}
	}

	return false
}

func limitString(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}

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
