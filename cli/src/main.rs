//! CLI entry point and command dispatch for hyde-ipc.
//!
//! This module parses CLI arguments and delegates to the appropriate subcommand logic.

mod dispatch;
mod flags;
mod keyword;
mod listen;
mod parsers;
mod query;
mod react;
mod react_config;
mod reaction_handler;

use clap::{CommandFactory, Parser};
use flags::{Cli, Commands, DispatchCommand, SetupAction};
use hyde_ipc_lib::service;
use std::process;

pub fn main() {
    let cli = Cli::parse();
    cli.command.run();
}

impl Commands {
    pub fn run(self) {
        match self {
            Commands::Keyword { r#async, get, set, keyword, value } => {
                if set && value.is_none() {
                    eprintln!("Error: --set requires a value");
                    print_usage_and_exit();
                }
                if r#async {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();
                    rt.block_on(keyword::async_keyword(get, set, keyword, value));
                    return;
                }

                keyword::sync_keyword(get, set, keyword, value);
            },
            Commands::Dispatch(dispatch_command) => {
                if dispatch_command.list_dispatchers {
                    print_dispatchers_list();
                    return;
                }

                match dispatch_command.command {
                    Some(command) => {
                        dispatch::handle_dispatch(command, dispatch_command.r#async);
                    },
                    None => {
                        DispatchCommand::command()
                            .print_help()
                            .unwrap();
                    },
                }
            },
            Commands::Listen { filter, max_events } => {
                if let Err(e) = listen::listen(filter, max_events) {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            },
            Commands::React {
                config,
                inline: _,
                event,
                subtype,
                filter,
                dispatch,
                max_reactions,
            } => {
                if let Some(config_path) = config {
                    if let Err(e) = react_config::run_from_config(&config_path) {
                        eprintln!("Error running from config: {e}");
                        process::exit(1);
                    }
                    return;
                }
                let event = event.unwrap_or_else(|| {
                    eprintln!("Error: event is required");
                    print_usage_and_exit();
                    String::new()
                });
                let dispatch = match dispatch {
                    Some(d) => d,
                    None => {
                        eprintln!("Error: dispatch is required");
                        print_usage_and_exit();
                        unreachable!();
                    },
                };
                if let Err(e) = react::sync_react(event, subtype, filter, dispatch, max_reactions) {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            },
            Commands::Setup(setup_command) => {
                let result = match setup_command.action {
                    SetupAction::Install => service::install(),
                    SetupAction::Uninstall => service::uninstall(),
                    SetupAction::Start => service::start(),
                    SetupAction::Kill => service::stop(),
                    SetupAction::Restart => service::restart(),
                    SetupAction::Check => service::status(),
                    SetupAction::Watch => service::watch_logs(),
                };

                if let Err(e) = result {
                    eprintln!("Error: {e}");
                    process::exit(1);
                } else {
                    println!("Ok!");
                }
            },
            Commands::Reload => {
                let config_dir = match service::get_config_path() {
                    Ok(path) => path,
                    Err(e) => {
                        eprintln!("Error getting config path: {e}");
                        process::exit(1);
                    },
                };

                if let Err(e) = react_config::validate_path(&config_dir) {
                    eprintln!("Config validation failed: {e}");
                    process::exit(1);
                }

                if let Err(e) = service::restart() {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }

                println!("Ok!");
            },
            Commands::Validate { config_path } => {
                match react_config::ReactConfig::validate_file(&config_path) {
                    Ok(()) => {
                        println!("Config validation successful: {config_path}");
                    },
                    Err(e) => {
                        eprintln!("Config validation failed: {e}");
                        process::exit(1);
                    },
                }
            },
            Commands::Query(query_command) => {
                if let Err(e) = query::run_query(query_command.command) {
                    eprintln!("Error: {e}");
                    process::exit(1);
                }
            },
        }
    }
}

fn print_usage_and_exit() {
    Cli::command().print_help().unwrap();
    process::exit(1);
}

fn print_dispatchers_list() {
    // FIX: macro it into the dispatcher, or juse use clap stuff!
    let list = r#"Available dispatchers:
  Basic commands:
  exec <command>                                    - Execute a command
  kill-active-window                                  - Kill the active window
  exit                                              - Exit Hyprland
  force-renderer-reload                               - Force the renderer to reload

  Window management:
  toggle-floating [window]                           - Toggle floating mode for a window
  toggle-fullscreen <type>                           - Toggle fullscreen mode (Real, Maximize, NoParam)
  toggle-fake-fullscreen                              - Toggle fake fullscreen for the active window
  toggle-pseudo                                      - Toggle pseudo tiling for the active window
  toggle-pin                                         - Pin the active window to all workspaces
  toggle-opaque                                      - Toggle opacity for the active window
  center-window                                      - Center the active window
  bring-active-to-top                                  - Bring the active window to the top of the stack

  Focus control:
  move-focus <direction>                             - Move focus in a direction (Up, Down, Left, Right)
  focus-window <window>                              - Focus a specific window
  focus-monitor <identifier>                         - Focus a specific monitor
  focus-urgent-or-last                                 - Focus the urgent window or the last one
  focus-current-or-last                                - Switch focus between current and last window

  Window movement:
  move-window <direction>                            - Move window in a direction
  move-active <position>                             - Move the active window to a position
  move-window-pixel <position> <win>                  - Move a specific window to a position
  resize-active <x> [y]                              - Resize the active window (exact by default; negatives allowed)
  resize-window-pixel <pos> <win>                     - Resize a specific window
  expand-active <dx> [dy]                             - Expand the active window by delta (dy defaults to 0)
  shrink-active <dx> [dy]                             - Shrink the active window by delta (dy defaults to 0)

  Workspace management:
  workspace <workspace>                             - Switch to workspace (number, previous, empty, name:NAME)
  move-to-workspace <workspace>                       - Move window to workspace
  move-to-workspace-silent <workspace>                 - Move window to workspace without switching to it
  rename-workspace <id> <name>                       - Rename a workspace

  Cycling and swapping:
  cycle-window <direction>                           - Cycle windows (Next, Previous)
  swap-next <direction>                              - Swap with next window (Next, Previous)
  swap-window <direction>                            - Swap windows in a direction (Up, Down, Left, Right)

  Cursor control:
  move-cursor-to-corner <corner>                       - Move cursor to a corner (TopLeft, TopRight, BottomLeft, BottomRight)
  move-cursor <x> <y>                                - Move cursor to a position
  set-cursor <theme> <size>                          - Set cursor theme and size

  Monitor management:
  move-current-workspace-to-monitor <mon>               - Move current workspace to a monitor
  move-workspace-to-monitor <ws> <mon>                 - Move a workspace to a monitor
  swap-active-workspaces <mon1> <mon2>                - Swap active workspaces of two monitors
  toggle-dpms <on/off> [monitor]                     - Toggle DPMS status for monitors

  Layout-specific commands (Dwindle):
  toggle-split                                       - Toggle the split orientation

  Layout-specific commands (Master):
  swap-with-master <param>                            - Swap with master window (Master, Child, Auto)
  focus-master <param>                               - Focus the master window (Master, Auto)
  add-master                                         - Add a master to the master side
  remove-master                                      - Remove a master from the master side
  orientation-left                                   - Set orientation to left
  orientation-right                                  - Set orientation to right
  orientation-top                                    - Set orientation to top
  orientation-bottom                                 - Set orientation to bottom
  orientation-center                                 - Set orientation to center
  orientation-next                                   - Cycle to next orientation
  orientation-prev                                   - Cycle to previous orientation

  Window grouping:
  toggle-group                                       - Toggle the current window into a group
  change-group-active <direction>                     - Switch to next window in group (Forward, Back)
  lock-groups <action>                               - Lock groups (Lock, Unlock, ToggleLock)
  move-into-group <direction>                         - Move window into group in direction
  move-out-of-group                                    - Move window out of group

Window identifiers (can be used with toggle-floating, focus-window, etc.):
  class:REGEX                               - Match window by class regex
  title:REGEX                               - Match window by title regex
  pid:PID                                   - Match window by process ID
  address:ADDR                              - Match window by address (hex value, with or without 0x prefix)

Examples:
  hypr-rs dispatch exec "kitty"
  hypr-rs dispatch move-cursor-to-corner TopLeft
  hypr-rs dispatch workspace 1
  hypr-rs dispatch --async toggle-fullscreen Maximize
  hypr-rs dispatch cycle-window Next
  hypr-rs dispatch move-focus Right
  hypr-rs dispatch toggle-floating "class:^(Google-chrome)$"
  hypr-rs dispatch focus-window "title:^(.*Terminal.*)$"
  hypr-rs dispatch toggle-floating address:5934277460f0
"#;
    println!("{list}");
}
