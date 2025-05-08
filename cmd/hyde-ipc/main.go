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
	"runtime/debug"
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

	verbose := flag.Bool("verbose", false, "Enable verbose logging")
	memLimit := flag.Int("memlimit", 8, "Memory limit in MB")
	noWatch := flag.Bool("nowatch", false, "Disable config file watching")
	cmdTimeout := flag.Int("timeout", 0, "Override script execution timeout in seconds (0 = use config value)")
	flag.Parse()

	utils.SetupLogging(*verbose)

	commandLineTimeout = *cmdTimeout

	optimizeMemoryUsage(*memLimit)

	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		utils.LogInfo("Creating default config at %s", configPath)
		if err := createDefaultConfig(); err != nil {
			log.Fatal("Config error: ", err)
		}
	}

	if err := reloadConfig(*verbose); err != nil {
		log.Fatal("Config error: ", err)
	}

	if !*noWatch {
		setupConfigWatcher(*verbose)
	}

	socketPath := getHyprlandSocketPath()
	conn, err := net.Dial("unix", socketPath)
	if err != nil {
		log.Fatal("Socket error: ", err)
	}
	defer conn.Close()

	utils.LogInfo("Connected to Hyprland socket, listening for events...")

	scanner := bufio.NewScanner(conn)
	buf := make([]byte, 0, 16*1024)
	scanner.Buffer(buf, 64*1024)

	for scanner.Scan() {
		handleEvent(scanner.Text())
	}

	if err := scanner.Err(); err != nil {
		log.Fatal("Socket error: ", err)
	}
}

func optimizeMemoryUsage(memLimitMB int) {

	debug.SetGCPercent(20)

	go func() {
		for {
			debug.FreeOSMemory()
			time.Sleep(30 * time.Second)
		}
	}()

	if memLimitMB > 0 {
		setMemoryLimit(memLimitMB)
	}
}

func setMemoryLimit(limitMB int) {

}

func reloadConfig(verbose bool) error {

	if configHandler == nil {
		var err error
		configHandler, err = config.NewConfigHandler(verbose)
		if err != nil {
			return fmt.Errorf("failed to create config handler: %w", err)
		}
	}

	newConfig, err := configHandler.Load()
	if err != nil {

		lastValid := configHandler.GetLastValidConfig()
		if lastValid != nil {
			log.Printf("Warning: config load error: %v", err)
			log.Printf("Continuing with previous valid configuration")
			return nil
		}
		return fmt.Errorf("failed to load config and no previous valid config available: %w", err)
	}

	configLock.Lock()
	defer configLock.Unlock()

	cfg = newConfig

	lastEventsLock.Lock()
	for k := range lastEvents {
		delete(lastEvents, k)
	}
	lastEventsLock.Unlock()

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

	if strings.Contains(script, "{") {

		if strings.Contains(script, "{0}") {
			script = strings.Replace(script, "{0}", eventData, -1)
		}

		dataArgs := strings.Split(eventData, ",")
		for i, arg := range dataArgs {

			placeholder := fmt.Sprintf("{%d}", i+1)
			script = strings.Replace(script, placeholder, arg, -1)
		}
	}

	cmdName := script
	if idx := strings.Index(script, " "); idx > 0 {
		cmdName = script[:idx]
	}

	_, err := exec.LookPath(cmdName)
	if err != nil {
		utils.LogInfo("Command not found: %s", cmdName)
		return
	}

	go executeScript(eventName, script, eventData)
}

func executeScript(eventName, script, eventData string) {
	utils.LogInfo("Executing script for event: %s", eventName)

	cmd := exec.Command("sh", "-c", script)

	cmd.Env = []string{
		"PATH=" + os.Getenv("PATH"),
		"HOME=" + os.Getenv("HOME"),
		"DISPLAY=" + os.Getenv("DISPLAY"),
		"WAYLAND_DISPLAY=" + os.Getenv("WAYLAND_DISPLAY"),
		"XDG_RUNTIME_DIR=" + os.Getenv("XDG_RUNTIME_DIR"),
		"DBUS_SESSION_BUS_ADDRESS=" + os.Getenv("DBUS_SESSION_BUS_ADDRESS"),
		"HYPRLAND_EVENT=" + eventData,
	}

	var timeout time.Duration
	if commandLineTimeout > 0 {
		timeout = time.Duration(commandLineTimeout) * time.Second
		utils.LogInfo("Using command-line timeout of %d seconds for event %s", commandLineTimeout, eventName)
	} else {
		timeout = time.Duration(cfg.HydeIPC.Timeout) * time.Second
	}
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

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

		}
	}()

	select {
	case result := <-done:
		if result.err != nil && utils.Verbose {
			utils.LogInfo("Script error [%s]: %v", eventName, result.err)
		} else if utils.Verbose && len(result.output) > 0 {
			utils.LogInfo("Script output [%s]: %s", eventName, utils.LimitString(string(result.output), 100))
		}
	case <-ctx.Done():

		if cmd.Process != nil {
			cmd.Process.Kill()
		}
		utils.LogInfo("Script timeout [%s] after %d seconds", eventName, timeout/time.Second)
	}

	done = nil
	cmd = nil
}

func shouldDebounce(eventName string) bool {

	if eventName == "urgent" || eventName == "closewindow" {
		return false
	}

	lastEventsLock.Lock()
	defer lastEventsLock.Unlock()

	now := time.Now()
	lastTime, exists := lastEvents[eventName]

	if exists && now.Sub(lastTime) < time.Duration(cfg.HydeIPC.DebounceTime)*time.Millisecond {
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
