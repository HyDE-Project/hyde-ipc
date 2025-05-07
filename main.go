package main

import (
	"bufio"
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"github.com/BurntSushi/toml"
	"github.com/adrg/xdg"
)

// Configuration structure
type Config struct {
	HyprlandIPC map[string]string `toml:"hyprland-ipc"`
}

func main() {
	// Setup logging
	log.SetPrefix("hyde-ipc: ")
	log.SetFlags(log.Ldate | log.Ltime)

	log.Println("Starting Hyde IPC for Hyprland")

	// Create default config if it doesn't exist
	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		log.Println("No configuration found, creating default config at", configPath)
		if err := createDefaultConfig(); err != nil {
			log.Fatalf("Failed to create default config: %v", err)
		}
	}

	// Load configuration
	config, err := loadConfig()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	// Get Hyprland socket path
	socketPath, err := getHyprlandSocketPath()
	if err != nil {
		log.Fatalf("Failed to get Hyprland socket path: %v", err)
	}

	// Connect to socket2 for events
	conn, err := net.Dial("unix", socketPath)
	if err != nil {
		log.Fatalf("Failed to connect to Hyprland socket: %v", err)
	}
	defer conn.Close()

	log.Printf("Connected to Hyprland socket at %s", socketPath)
	log.Println("Listening for events...")

	// Read events from socket
	scanner := bufio.NewScanner(conn)
	for scanner.Scan() {
		line := scanner.Text()
		handleEvent(line, config.HyprlandIPC)
	}

	if err := scanner.Err(); err != nil {
		log.Fatalf("Error reading from socket: %v", err)
	}
}

// loadConfig reads the configuration file
func loadConfig() (*Config, error) {
	// Use xdg library to get the proper config path
	configPath := filepath.Join(xdg.ConfigHome, "hyde", "config.toml")

	// Check if config exists
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("config file not found at %s", configPath)
	}

	var config Config
	if _, err := toml.DecodeFile(configPath, &config); err != nil {
		return nil, fmt.Errorf("failed to decode config file: %w", err)
	}

	return &config, nil
}

// getHyprlandSocketPath returns the path to the Hyprland socket2 for events
func getHyprlandSocketPath() (string, error) {
	his := os.Getenv("HYPRLAND_INSTANCE_SIGNATURE")
	if his == "" {
		return "", fmt.Errorf("HYPRLAND_INSTANCE_SIGNATURE environment variable not set")
	}

	runtimeDir := os.Getenv("XDG_RUNTIME_DIR")
	if runtimeDir == "" {
		return "", fmt.Errorf("XDG_RUNTIME_DIR environment variable not set")
	}

	return filepath.Join(runtimeDir, "hypr", his, ".socket2.sock"), nil
}

// handleEvent processes Hyprland events and executes configured scripts
func handleEvent(eventLine string, config map[string]string) {
	parts := strings.SplitN(eventLine, ">>", 2)
	if len(parts) != 2 {
		log.Printf("Ignoring malformed event: %s", eventLine)
		return
	}

	eventName := parts[0]
	eventData := parts[1]

	log.Printf("Received event: %s with data: %s", eventName, eventData)

	// Check if we have a script configured for this event
	script, exists := config[eventName]
	if !exists || script == "" {
		return // No script configured for this event
	}

	// Replace argument placeholders like {0}, {1}, etc. with actual values
	dataArgs := strings.Split(eventData, ",")
	for i, arg := range dataArgs {
		placeholder := fmt.Sprintf("{%d}", i)
		script = strings.ReplaceAll(script, placeholder, arg)
	}

	// Execute the script
	cmd := exec.Command("sh", "-c", script)
	cmd.Env = append(os.Environ(), fmt.Sprintf("HYDE_EVENT_DATA=%s", eventData))

	go func() {
		output, err := cmd.CombinedOutput()
		if err != nil {
			log.Printf("Error executing script for event %s: %v", eventName, err)
			if len(output) > 0 {
				log.Printf("Script output: %s", output)
			}
		} else if len(output) > 0 {
			log.Printf("Script for event %s output: %s", eventName, output)
		}
	}()
}

// createDefaultConfig generates a default configuration file if none exists
func createDefaultConfig() error {
	configDir := filepath.Join(xdg.ConfigHome, "hyde")

	// Create directory if it doesn't exist
	if err := os.MkdirAll(configDir, 0755); err != nil {
		return fmt.Errorf("failed to create config directory: %w", err)
	}

	configPath := filepath.Join(configDir, "config.toml")

	// Don't overwrite existing config
	if _, err := os.Stat(configPath); err == nil {
		return nil // File already exists
	}

	defaultConfig := `# Hyde IPC Configuration for Hyprland

[hyprland-ipc]
# Event handlers
# Format: event_name = "script_to_execute"

# Window events
windowtitle = "notify-send \"Window Title Changed\" \"$HYDE_EVENT_DATA\""
activewindow = ""
activewindowv2 = ""
openwindow = ""
closewindow = ""

# Workspace events
workspace = ""
workspacev2 = ""
createworkspace = ""
destroyworkspace = ""
moveworkspace = ""

# Monitor events
focusedmon = ""
monitoradded = ""
monitorremoved = ""

# Special events
fullscreen = ""
screencast = ""
activespecial = ""

# Layout and input events
activelayout = ""
submap = ""

# Layer events
openlayer = ""
closelayer = ""

# Floating mode events
changefloatingmode = ""

# Group events
togglegroup = ""
moveintogroup = ""
moveoutofgroup = ""

# Other events
urgent = ""
configreloaded = ""
`

	return os.WriteFile(configPath, []byte(defaultConfig), 0644)
}
