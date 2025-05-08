# Hyde IPC

A lightweight event handler for Hyprland that executes custom scripts based on Hyprland IPC events.

## Features

- Minimal memory footprint
- Fast event processing
- Configurable script execution for Hyprland events
- Hot-reloading of configuration
- Script timeout handling
- Event debouncing to prevent script spam

## Installation

```bash
# Clone the repository
git clone https://github.com/khing/hyde-ipc.git
cd hyde-ipc

# Build and install
make build
make install

# Enable and start the systemd service
systemctl --user enable hyde-ipc.service
systemctl --user start hyde-ipc.service
```

## Configuration

Configuration is stored in `~/.config/hyde/config.toml`:

```toml
[hyde-ipc]
# Maximum number of concurrent script executions
max_concurrent = 2
# Timeout for script execution in seconds
timeout = 60
# Debounce time for frequent events in milliseconds
debounce_time = 100

[hyprland-ipc]
# Map Hyprland events to scripts
windowtitle = "notify-send \"Window Title Changed\" \"$HYDE_EVENT_DATA\""
workspace = "~/.config/hypr/scripts/workspace-change.sh"
activewindow = "~/.config/hypr/scripts/focus-change.sh"
```

## Event Data

Your scripts receive event data through the `$HYDE_EVENT_DATA` environment variable.
You can also use placeholders in your scripts:

- `{0}`: Whole event data string
- `{1}`, `{2}`, etc.: Individual comma-separated values from the event data


## Usage

```bash
# Run with minimal logging
hyde-ipc

# Run with detailed event logging
hyde-ipc --verbose

# Override timeout for all scripts (in seconds)
hyde-ipc --timeout=30

# Disable configuration hot-reloading
hyde-ipc --nowatch
```
