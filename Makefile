.PHONY: build install clean setup


setup:
	go mod tidy
	go get github.com/BurntSushi/toml
	go get github.com/adrg/xdg


build: setup
	go build -o hyde-ipc


install: build
	mkdir -p $(HOME)/.local/bin/
	cp hyde-ipc $(HOME)/.local/bin/
	mkdir -p $(HOME)/.config/systemd/user/
	cp hyde-ipc.service $(HOME)/.config/systemd/user/
	@echo "Installed hyde-ipc to $(HOME)/.local/bin/"
	@echo "Installed systemd service to $(HOME)/.config/systemd/user/"
	@echo "To enable the service, run: systemctl --user enable hyde-ipc.service"
	@echo "To start the service, run: systemctl --user start hyde-ipc.service"
	@echo "For verbose logging: systemctl --user edit hyde-ipc.service"
	@echo "  and add --verbose to the ExecStart line"


clean:
	rm -f hyde-ipc


verbose: build
	./hyde-ipc --verbose


help:
	@echo "Hyde IPC for Hyprland"
	@echo ""
	@echo "Available targets:"
	@echo "  setup   - Initialize Go module and download dependencies"
	@echo "  build   - Build the hyde-ipc binary"
	@echo "  install - Install hyde-ipc to ~/.local/bin/ and systemd service"
	@echo "  verbose - Run the application with verbose logging"
	@echo "  clean   - Remove build artifacts"
	@echo ""
	@echo "Usage:"
	@echo "  hyde-ipc         - Run normally with minimal logging"
	@echo "  hyde-ipc --verbose - Run with detailed event logging"