# Hyde-IPC

**A powerful and extensible command-line tool for interacting with the Hyprland compositor's IPC socket.**

Hyde-IPC provides a comprehensive set of commands for controlling, querying, and automating window management workflows. While it can be used for simple commands like `hyprctl`, its primary strength lies in its advanced event-reaction system, which allows for the creation of sophisticated, declarative automation rules that run persistently in the background.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Core Commands](#core-commands)
- [Advanced Automation with Event-Reactions](#advanced-automation-with-event-reactions)
  - [The `reactions.toml` Config File](#the-reactionstoml-config-file)
  - [Persistent Automation (Systemd Service)](#persistent-automation-systemd-service)
  - [Testing and Inline Reactions](#testing-and-inline-reactions)
- [Contributing](#contributing)
- [License](#license)

## Features

- **Direct Command Dispatching**: Execute any Hyprland dispatcher command synchronously or asynchronously.
- **State Querying**: Get real-time information from the compositor.
- **Event Listening**: Monitor the Hyprland event stream for debugging and diagnostics.
- **Declarative Event-Reaction System**: Define complex automation rules in a simple TOML format to run in the background.
- **Systemd Service Integration**: Run `hyde-ipc` as a persistent background service to handle global event reactions seamlessly.

## Installation

### Arch Linux

Install the `hyde-ipc` package from the [AUR](https://aur.archlinux.org/packages/hyde-ipc).

```bash
paru -S hyde-ipc
```

### From Source

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/HyDE-Project/hyde-ipc.git
    cd hyde-ipc
    ```

2.  **Build and install**:
    ```bash
    cargo build --release
    sudo cp target/release/hyde-ipc /usr/local/bin/
    ```

## Core Commands

`hyde-ipc` provides several direct commands for interacting with Hyprland.

-   **`dispatch`**: Executes a Hyprland dispatcher command. This is your main tool for scripting and direct control.
    ```bash
    # Open a terminal
    hyde-ipc dispatch exec "kitty"
    # Move focus to the window on the right
    hyde-ipc dispatch movefocus right
    ```
    For a complete list of available dispatchers, run `hyde-ipc dispatch --list-dispatchers` or consult the [Dispatcher Reference](https://github.com/HyDE-Project/hyde-ipc/wiki/3.-Dispatchers) in the wiki.

-   **`listen`**: Connects to the Hyprland event socket and prints all incoming events. This is a powerful tool for debugging or discovering the event names and data you need to build your automations.
    ```bash
    # Listen for all events
    hyde-ipc listen
    # Listen only for workspace-related events
    hyde-ipc listen --filter workspace
    ```

-   **`keyword`**: Gets or sets a Hyprland keyword value.
    ```bash
    # Get the current border size
    hyde-ipc keyword --get general:border_size
    # Set the border size to 2
    hyde-ipc keyword --set general:border_size 2
    ```

-   **`query`**: Retrieves specific information from Hyprland, like the active window or cursor position.
    ```bash
    hyde-ipc query cursor-pos
    ```

## Advanced Automation with Event-Reactions

The true power of `hyde-ipc` is its ability to **react to events** and perform actions automatically. This is primarily achieved by creating a configuration file and running it as a background service.

### The `reactions.toml` Config File

You define your automation rules in a `reactions.toml` file. This file contains an array of `[[reactions]]` tables, where each table is a rule that binds an event to a series of commands.

**Default Location**: `~/.config/hyde-ipc/reactions.toml`

**Example `reactions.toml`**:
```toml
# This reaction will automatically move any browser window to workspace 2 when it opens.
[[reactions]]
name = "Browser on Workspace 2"
event_type = { window = "opened" }
window_filter = "class:^(firefox|google-chrome)$"
dispatchers = [
  { name = "movetoworkspacesilent", args = ["2"] }
]

# This reaction plays a sound whenever you take a screenshot with grim.
[[reactions]]
name = "Screenshot Sound"
event_type = { window = "closed" }
window_filter = "class:^(grim)$"
dispatchers = [
    { name = "exec", args = ["paplay", "/usr/share/sounds/freedesktop/stereo/camera-shutter.oga"] }
]
```
For a complete guide on the configuration file syntax and all available options, please see the [Configuration](https://github.com/HyDE-Project/hyde-ipc/wiki/5.-Configuration) and [Event-Reaction System](https://github.com/HyDE-Project/hyde-ipc/wiki/4.-Event-Reaction-System) pages in the wiki.

### Persistent Automation (Systemd Service)

For your `reactions.toml` rules to be active permanently, `hyde-ipc` provides a `systemd` user service that runs in the background and listens for events.

1.  **Install the service**: This creates and enables the service file.
    ```bash
    hyde-ipc setup --install
    ```

2.  **Create your `reactions.toml`**: Create your configuration file at `~/.config/hyde-ipc/reactions.toml`.

3.  **Set the global config and start the service**: The `global` command installs your config file for the service to use and (re)starts it, applying your rules.
    ```bash
    hyde-ipc global --config-path ~/.config/hyde-ipc/reactions.toml
    ```

4.  **Manage the service**: You can start, stop, and monitor the service at any time.
    - `hyde-ipc setup --start` / `hyde-ipc setup --kill`
    - `hyde-ipc setup --restart` (useful after manually editing the config)
    - `hyde-ipc setup --check` (check status)
    - `hyde-ipc setup --watch` (view live logs)

### Testing and Inline Reactions

While the service is the primary way to use reactions, you can test a config file directly or create a temporary, one-off reaction from the command line. This is useful for debugging a new rule before adding it to your main configuration.

-   **Test a config file**:
    ```bash
    hyde-ipc react --config /path/to/your/reactions.toml
    ```
-   **Test a single rule inline**:
    ```bash
    # Make any newly focused Alacritty terminal float
    hyde-ipc react --inline -e window -s active -f "class:^(Alacritty)$" togglefloating
    ```

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](https://github.com/HyDE-Project/hyde-ipc/blob/main/wiki/CONTRIBUTING.md) for details on how to get started.

## License

This project is licensed under the MIT License. See the [LICENSE](https://github.com/HyDE-Project/hyde-ipc/blob/main/LICENSE) file for details.