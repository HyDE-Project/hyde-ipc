# HyDE-IPC: Rust Bindings & CLI for Hyprland IPC

[![Made for Hyprland](https://img.shields.io/badge/Made%20for-Hyprland-blue)](https://github.com/hyprwm/Hyprland)
[![License: MIT](https://img.shields.io/badge/License-MIT.svg)](./LICENSE)

**hyde-ipc** is a Rust Implementation for interacting with the [Hyprland](https://github.com/hyprwm/Hyprland) via its IPC interface. `hyde-ipc` enables you to control Hyprland, monitor events, and automate your workflow with a flexible reaction system using `toml` config files.

---

## Features

- **Event Listening:** Listen and react to Hyprland events in real time.
- **Reactive Event System:** Automate reactions to Hyprland events using TOML/JSON configs.
- **Fast Native Dispatchers:** Native and non-blocking Rust implementations for many dispatchers faster than shelling out!
- **Flexible CLI:** Scriptable, async, and supports advanced low-level automation.

## install

### Archlinux

hyde-ipc is pushed to [AUR](https://aur.archlinux.org/packages/hyde-ipc)

```bash
yay -S hyde-ipc-bin         # perbuilt binaries
# or
yay -S hyde-ipc             # Build from source
```

### clone and build

Makefile defaults to a system wide install

```bash
git  clone --depth 1 https://github.com/HyDE-Project/hyde-ipc.git
cd hyde-ipc
make
```

or use `cargo` for local build

```bash
git  clone --depth 1 https://github.com/HyDE-Project/hyde-ipc.git
cd hyde-ipc
cargo build --frozen
```

## usage

hyde-ipc includes 4

```bash
hydeipc <Command> <options>
```

### `hyprpland` configuration Management (Keywords)

```bash

hyde-ipc keyword --help

# for example to get the blur size of your current config

hyde-ipc keyword --get --async decoration:blur:size
#or
hyde-ipc keyword -g -a decoration:blur:size

and to set it to 8

hyde-ipc keyword --set --async decoration:blur:size 8

```

### Event Listening

Listen for and log Hyprland events:

```bash
hyde-ipc listen

# or with more options
hypr-rs listen --filter window --max-events 5 --json
```

> [!NOTE]
> listen is fully async by default

### Dispatch

Execute a Hyprland dispatcher (event)

```bash


#for example to toggle floating mode for the active window
hyde-ipc dispatch ToggleFloating

# or to focus a window an specific window
hyde-ipc dispatch --async FocusWindow "title:^(Terminal)$"

```

> [!NOTE]
> Native dispatchers are faster than shelling out to `hyprctl`. Use them whenever possible!

all hyprland `Dispatchers` are NOT natively supported yet.

You can get the list of available dispatchers and more usage examples by running:

```bash
hyde-ipc dispatch --list-dispatchers
# or
hyde-ipc dispatch -l

```

### Automation and `react` Command

You can use `react` command to listen for an specific event and dispatch an event (execute a command) as a reaction to the event.

In this simple example, we toggle floating for any window when it opens.

```bash
hyde-ipc react -i --event window --subtype opened --dispatch ToggleFloating
```

#### react configuration files

for more control over automation you can use `toml` config files including `reaction` instructions . with `react` command.

a simple example to get notified when a window's state is changed to float :

```toml
[[reactions]]

event_type = "Float"
dispatcher = "Exec"
args = ["notify-send", "Float Toggled"]

```

you can filter events down, and also chain dispatchers that are triggered by the event.
For example, in below toml file, we set alacritty mode to float and resize it to 800x600 and center it when it opens.

```toml
# my-reaction.toml
[[reactions]]
event_type = { Window = "Opened" }
window_filter = "class:Alacritty"
dispatchers = [
  { name = "FocusWindow", args = ["class:Alacritty"] },
  { name = "ToggleFloating" },
  { name = "ResizeActive", args = ["exact", "800", "600"] },
  { name = "CenterWindow" }
]
```

you can source the file by running:

```bash
hyde-ipc react -c ./path/to/my-reaction.toml
```

TODO explain optional fields in toml configs

#### global configuration file.

to have a global config file and have it sourced by default you can run:

```bash
# make sure you run the setup once after hyde-ipc is installed
hyde-ipc global --setup

# then you can source a toml file as global config
hyde-ipc global ./path/to/my-reaction.toml

# and to stop the global automation run
hyde-ipc global --kill

```

#### More examples

```bash

# you can pass --async before dispatcher

hyde-ipc dispatch --async MoveFocus Right

hyde-ipc dispatch Workspace 3

hyde-ipc dispatch Exec "kitty"

```

class and title are not natively supported.

```bash

hyde-ipc dispatch ToggleFloating "class:^(firefox)$"

hyde-ipc dispatch --async FocusWindow "title:^(Terminal)$"

hyde-ipc dispatch ToggleFloating "address:0x12345678"

```

React Examples

```bash
# Show a notification when a window's float state changes
hypr-ipc react --event float --dispatch Exec --params "notify-send 'Float State Changed' "
```

**Limit the number of reactions:** (not recommended)

```bash
# Move to workspace 1 after 3 window open events
hypr-ipc react --event window --subtype opened --dispatch Workspace --params 1 --max-reactions 3
```

More configuration examples,

this exmaple reacts to float state changes:

by the way in hyprland socket workspaces are 0-indexed for some reason meaning if you want to move to workspace 4 you need to pass 3 as the workspace number

`max_count` , `name` and `description` fiels are optional.

```toml
[[reactions]]
event_type = "Float"
dispatcher = "Exec"
args = ["notify-send", "Float Toggled", "A window's float state was changed"]
max_count = 0
name = "Float Toggle Notification"
description = "Sends a notification when a window's float state changes"

[[reactions]]
event_type = "Float"
dispatcher = "Exec"

# example on how to use dispatchers that are not natively supported, (notify send is supported but if you have a dispatcher that is not supported this is what you do)
args = ["sh", "-c", "hyprctl activewindow | grep -i title | cut -d':' -f2 | xargs notify-send 'Window Name'"]
max_count = 0
name = "Window Name Notification"
description = "Shows the name of the window that changed float state"

[[reactions]]
event_type = "Float"
dispatcher = "MoveToWorkspace"
args = ["4"]
```

various window and workspace management reactions:

```toml

[[reactions]]
event_type = { Window = "Opened" }
dispatcher = "Exec"
args = ["notify-send", "New Window", "A new window has been opened"]
max_count = 0
name = "Window Open Notification"
description = "Sends a notification when a new window opens"

# Workspace change notification
[[reactions]]
event_type = { Workspace = "Changed" }
dispatcher = "Exec"
args = ["sh", "-c", "WORKSPACE=$(hyprctl activeworkspace | grep -i id | awk '{print $2}') && notify-send 'Workspace Changed' \"Switched to workspace $WORKSPACE\""]
max_count = 0
# Note: is_async field is deprecated and will be ignored
name = "Workspace Change Notification"
description = "Shows which workspace you switched to"

# Auto-float new windows
[[reactions]]
event_type = { Window = "Opened" }
dispatcher = "ToggleFloating"
args = []
max_count = 0
# Note: is_async field is deprecated and will be ignored
name = "Auto-Float New Windows"
description = "Automatically makes new windows floating"

# Move Firefox to workspace 2
[[reactions]]
event_type = { Window = "Opened" }
dispatcher = "MoveToWorkspaceOnce"
args = ["class:^(firefox)$", "2"]
max_count = 0
# Note: is_async field is deprecated and will be ignored
name = "Firefox to Workspace 2"
description = "Automatically moves Firefox windows to workspace 2"

```

if you get `$ needs a variable name` error when using hyprland syntax id dispatchers, escape the `$` sign with `\`
