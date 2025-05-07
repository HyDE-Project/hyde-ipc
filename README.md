# Hyde IPC for Hyprland

A simple Go application that monitors Hyprland IPC events and executes configured scripts based on those events.

## Installation

1. Clone this repository:
```bash
git clone https://github.com/your-username/hyde-ipc
cd hyde-ipc
```

2. Build the application:
```bash
go build
```

3. Run the application:
```bash
./hyde-ipc
```

## Configuration

The application automatically creates a default configuration file at `~/.config/hyde/config.toml` if one doesn't exist. You can customize this file to run your own scripts based on Hyprland events.

Example configuration:
```toml
[hyprland-ipc]
# Window title event - triggers notification when window title changes
windowtitle = "notify-send \"Window Title Changed\" \"$HYDE_EVENT_DATA\""

# Workspace change event
workspace = "~/scripts/workspace_changed.sh"

# Fullscreen event
fullscreen = "~/scripts/fullscreen_toggle.sh"
```

## Using Event Data Arguments

Events can pass multiple pieces of data which you can access individually:

```toml
# Using positional arguments from event data with {n} placeholders
# For movewindowv2 which sends: WINDOWADDRESS,WORKSPACEID,WORKSPACENAME
movewindowv2 = "notify-send 'Window Moved' 'Window {0} moved to workspace {2} (ID: {1})'"

# Accessing the entire event data 
windowtitle = "notify-send 'Window Title Changed' \"$HYDE_EVENT_DATA\""
```

## Runtime Options

The application supports the following command line options:

```bash
# Run with minimal logging (default)
./hyde-ipc

# Run with verbose logging for debugging
./hyde-ipc --verbose
```

## Available Events

The application listens for all Hyprland events. Here are some examples:

- `workspace` - Emitted on workspace change
- `activewindow` - Emitted on active window change
- `fullscreen` - Emitted when fullscreen status of a window changes
- `screencast` - Emitted when a screencopy state changes
- `monitoradded` / `monitorremoved` - Emitted when monitors are connected/disconnected

For a complete list of events, check the [Hyprland IPC documentation](https://wiki.hyprland.org/IPC/).

## Environment Variables

When executing a script, the application passes the event data as the `HYDE_EVENT_DATA` environment variable. Your script can use this value to react accordingly.

## Running as a Service

To run Hyde IPC automatically when you log in, create a systemd user service:

1. Create the service file:
```bash
mkdir -p ~/.config/systemd/user/
```

2. Add the following content to `~/.config/systemd/user/hyde-ipc.service`:
```ini
[Unit]
Description=Hyde IPC service for Hyprland
PartOf=graphical-session.target
After=graphical-session.target

[Service]
ExecStart=/path/to/hyde-ipc
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=graphical-session.target
```

3. Enable and start the service:
```bash
systemctl --user enable hyde-ipc.service
systemctl --user start hyde-ipc.service
```

4. For verbose logging with the service:
```bash
systemctl --user edit hyde-ipc.service
```

Then add `--verbose` to the ExecStart line:
```ini
[Service]
ExecStart=/path/to/hyde-ipc --verbose
```

## License

MIT