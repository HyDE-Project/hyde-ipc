# hyde-ipc

## install

### clone

```bash
git  clone --depth 1 https://github.com/primetype/hyde-ipc.git
cd hyde-ipc

```

### build

```bash

cargo build

mkdir -p ./bin
cp ./target/debug/hyde-ipc ./bin

```

or using cargo make (if installed)

```bash

cargo make

```

## usage

```bash

hyde-ipc --help

```

### keyword

```bash

hyde-ipc --keyword --help

## to get a hyprland keyword (empy output means not set)

hyde-ipc keyword --get --async decoration:blur:size

## set a keyword

hyde-ipc keyword -s -a decoration:blur:passes 10

```

### Listen

Listen for and log Hyprland events.

Options:

-f, --filter <FILTER>: Filter events by type (e.g., "window", "workspace")
-n, --max-events <MAX_EVENTS>
-j, --json: Use JSON format for output (NOT TESTED YET!)

```bash

hyde-ipc listen

hypr-rs listen --filter window --max-events 5 --json

```

> [!NOTE]
> listen is fully async by default

### Dispatch

Execute a Hyprland dispatcher command.

```bash

hyde-ipc dispatch [OPTIONS] [DISPATCHER] [ARGS]...

```

> [!NOTE]
> dispatching hyde-ipc nativly supported dispatchers are WAYYY faster than default dispatching. Use native ones unless you have a good reason not to!

all hyprland `Dispatchers` are not natively supported yet.

please ignore the ones provided by `hyprland-rs` and rely on list below.

non-native supported doesn't mean they don't work, it means they are at best as fast as hyprlnad!
Nativeones are faster especially when used in `async` mode (Rust btw)

```bash

hyde-ipc dispatch --list-dispatchers

```

#### examples

```bash

# Toggle floating mode for the active window
hyde-itc dispatch ToggleFloating

```

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

## React

React is the big deal about this project.

```bash
hyde-ipc react [OPTIONS]
```

**Options:**

- `-a, --async`: Use async mode
- `-c, --config <FILE>`: Use a config file to define multiple reactions
- `-t, --create-template <FILE>`: Create a new config file template
- `-i, --inline`: Use inline mode (single reaction)
- `-e, --event <EVENT>`: Event type to react to (e.g., "window", "workspace")
- `-s, --subtype <SUBTYPE>`: Event subtype for more specific filtering (e.g., "opened" for window events)
- `-d, --dispatch <DISPATCHER>`: Dispatcher command to execute when the event occurs
- `-p, --params <ARGS>...`: Arguments for the dispatcher
- `-n, --max-reactions <MAX_REACTIONS>`: Limit number of reactions (0 for unlimited)

### React Examples

**Execute a notification command when a window becomes floating:**

```bash
# Show a notification when a window's float state changes
hypr-ipc react --event float --dispatch Exec --params "notify-send 'Float State Changed' "
```

**Limit the number of reactions:** (not recommended)

```bash
# Move to workspace 1 after 3 window open events
hypr-ipc react --event window --subtype opened --dispatch Workspace --params 1 --max-reactions 3
```

## Automation :: Configuration files

Configuration files allow you to define multiple reactions in a single file, which can then be loaded with a single command. This is more efficient than running multiple separate processes

```bash

hyde-ipc react --config ./path/to/config.toml

```

### Examples

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
