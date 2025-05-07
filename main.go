package main

import (
	"bufio"
	"flag"
	"fmt"
	"log"
	"net"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"github.com/BurntSushi/toml"
	"github.com/adrg/xdg"
)

type Config struct {
	HyprlandIPC map[string]string `toml:"hyprland-ipc"`
	Settings    struct {
		MaxConcurrent int           `toml:"max_concurrent"`
		Timeout       time.Duration `toml:"timeout"`
		DebounceTime  time.Duration `toml:"debounce_time"`
	} `toml:"settings"`
}

var (
	config     *Config
	configLock sync.RWMutex
	verbose    bool
	executor   *CommandExecutor

	lastEvents     = make(map[string]time.Time)
	lastEventsLock sync.Mutex
)

func main() {

	flag.BoolVar(&verbose, "verbose", false, "Enable verbose logging")
	memLimit := flag.Int("memlimit", 8, "Memory limit in MB")
	flag.Parse()

	setMemoryLimit(*memLimit)

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

	var err error
	config, err = loadConfig()
	if err != nil {
		log.Fatal("Config error: ", err)
	}

	maxConcurrent := config.Settings.MaxConcurrent
	if maxConcurrent <= 0 {
		maxConcurrent = 2
	}
	executor = NewCommandExecutor(maxConcurrent)

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

func loadConfig() (*Config, error) {
	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")

	var cfg Config
	if _, err := toml.DecodeFile(configPath, &cfg); err != nil {
		return nil, err
	}

	if cfg.Settings.Timeout <= 0 {
		cfg.Settings.Timeout = 5 * time.Second
	}
	if cfg.Settings.DebounceTime <= 0 {
		cfg.Settings.DebounceTime = 100 * time.Millisecond
	}

	return &cfg, nil
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
	script, exists := config.HyprlandIPC[eventName]
	configLock.RUnlock()

	if !exists || script == "" {
		return
	}

	if strings.Contains(script, "{") {
		dataArgs := strings.Split(eventData, ",")
		for i, arg := range dataArgs {
			placeholder := fmt.Sprintf("{%d}", i)
			script = strings.Replace(script, placeholder, arg, -1)
		}
	}

	go func() {
		output, err := executor.ExecuteWithTimeout(script, eventData, config.Settings.Timeout)
		if err != nil {
			logInfo("Script error [%s]: %v", eventName, err)
			return
		}

		if verbose && len(output) > 0 {
			logInfo("Script output [%s]: %s", eventName, limitString(string(output), 100))
		}
	}()
}

func shouldDebounce(eventName string) bool {

	if eventName == "urgent" || eventName == "closewindow" {
		return false
	}

	lastEventsLock.Lock()
	defer lastEventsLock.Unlock()

	now := time.Now()
	lastTime, exists := lastEvents[eventName]

	if exists && now.Sub(lastTime) < config.Settings.DebounceTime {
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

[settings]
# Maximum number of concurrent script executions
max_concurrent = 2
# Timeout for script execution in seconds
timeout = 5
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
