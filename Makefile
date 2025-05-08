# Makefile for hyde-ipc

.PHONY: all build clean install uninstall setup verbose help release

# Default target
all: build

# Ensure bin directory exists
bin:
	mkdir -p bin

# Initialize Go module and download dependencies
setup:
	go mod tidy
	go get github.com/adrg/xdg
	go get github.com/fsnotify/fsnotify
	go get github.com/pelletier/go-toml/v2

# Build the application into bin directory
build: setup bin
	go build -o bin/hyde-ipc ./cmd/hyde-ipc

# Build an optimized release binary
release: setup bin
	go build -ldflags="-s -w" -trimpath -o bin/hyde-ipc ./cmd/hyde-ipc
	strip -s bin/hyde-ipc
	upx -9 bin/hyde-ipc || echo "UPX not installed, skipping compression"

# Install to user's local bin and systemd service
install: release
	mkdir -p $(HOME)/.local/bin
	install -m 755 bin/hyde-ipc $(HOME)/.local/bin/hyde-ipc
	mkdir -p $(HOME)/.config/systemd/user/
	cp systemd/hyde-ipc.service $(HOME)/.config/systemd/user/
	@echo "hyde-ipc installed to $(HOME)/.local/bin/"
	@echo "Installed systemd service to $(HOME)/.config/systemd/user/"
	@echo "To enable the service, run: systemctl --user enable hyde-ipc.service"
	@echo "To start the service, run: systemctl --user start hyde-ipc.service"
	@echo "For verbose logging: systemctl --user edit hyde-ipc.service"
	@echo "  and add --verbose to the ExecStart line"

# Dev install with non-optimized binary
dev-install: build
	mkdir -p $(HOME)/.local/bin
	install -m 755 bin/hyde-ipc $(HOME)/.local/bin/hyde-ipc
	mkdir -p $(HOME)/.config/systemd/user/
	cp hyde-ipc.service $(HOME)/.config/systemd/user/

# Uninstall the application
uninstall:
	rm -f $(HOME)/.local/bin/hyde-ipc
	systemctl --user stop hyde-ipc.service 2>/dev/null || true
	systemctl --user disable hyde-ipc.service 2>/dev/null || true
	rm -f $(HOME)/.config/systemd/user/hyde-ipc.service
	@echo "hyde-ipc uninstalled from $(HOME)/.local/bin/"
	@echo "Systemd service removed"

# Clean build artifacts
clean:
	rm -rf bin
	go clean

# Run the application with verbose logging
verbose: build
	./bin/hyde-ipc --verbose

# Display help information
help:
	@echo "Hyde IPC for Hyprland"
	@echo ""
	@echo "Available targets:"
	@echo "  setup       - Initialize Go module and download dependencies"
	@echo "  build       - Build the hyde-ipc binary (development version)"
	@echo "  release     - Build optimized binary for release (smaller, faster)"
	@echo "  install     - Install optimized hyde-ipc binary and systemd service"
	@echo "  dev-install - Install development binary and systemd service"
	@echo "  uninstall   - Remove hyde-ipc from ~/.local/bin/ and systemd service"
	@echo "  verbose     - Run the application with verbose logging"
	@echo "  clean       - Remove build artifacts"
	@echo ""
	@echo "Usage:"
	@echo "  hyde-ipc         - Run normally with minimal logging"
	@echo "  hyde-ipc --verbose - Run with detailed event logging"