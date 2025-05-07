.PHONY: build install clean setup

# Setup module and dependencies
setup:
	go mod tidy
	go get github.com/BurntSushi/toml
	go get github.com/adrg/xdg

# Build the application
build: setup
	go build -o hyde-ipc

# Install the application
install: build
	mkdir -p $(HOME)/.local/bin/
	cp hyde-ipc $(HOME)/.local/bin/
	mkdir -p $(HOME)/.config/systemd/user/
	cp hyde-ipc.service $(HOME)/.config/systemd/user/
	@echo "Installed hyde-ipc to $(HOME)/.local/bin/"
	@echo "Installed systemd service to $(HOME)/.config/systemd/user/"
	@echo "To enable the service, run: systemctl --user enable hyde-ipc.service"
	@echo "To start the service, run: systemctl --user start hyde-ipc.service"

# Clean build artifacts
clean:
	rm -f hyde-ipc

# Help target
help:
	@echo "Hyde IPC for Hyprland"
	@echo ""
	@echo "Available targets:"
	@echo "  setup   - Initialize Go module and download dependencies"
	@echo "  build   - Build the hyde-ipc binary"
	@echo "  install - Install hyde-ipc to ~/.local/bin/ and systemd service"
	@echo "  clean   - Remove build artifacts"