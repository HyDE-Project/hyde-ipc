//! CLI entry point and command dispatch for hyde-ipc.
//!
//! This module parses CLI arguments and delegates to the appropriate subcommand logic.

mod dispatch;
mod flags;
mod keyword;
mod listen;
mod query;
mod react;
mod react_config;
mod setup;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use flags::{Cli, Commands, DispatchCommand, QueryCommand};
use std::path::PathBuf;
use std::{env, fs, io, process};

/// Main entry point for the hyde-ipc CLI.
///
/// Parses command-line arguments and dispatches to the appropriate subcommand handler.
pub fn main() {
    let cli = Cli::parse();

    match cli.command {
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
            } else {
                keyword::sync_keyword(get, set, keyword, value);
            }
        },
        Commands::Dispatch(dispatch_command) => {
            if dispatch_command.list_dispatchers {
                print_dispatchers_list();
                return;
            }

            if let Some(command) = dispatch_command.command {
                if dispatch_command.r#async {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        let (tx, rx) = tokio::sync::oneshot::channel();

                        tokio::spawn(async move {
                            let result = dispatch::async_dispatch(command).await;
                            let _ = tx.send(result);
                        });

                        match rx.await {
                            Ok(_) => (),
                            Err(_) => {
                                eprintln!("Warning: Async task was dropped before completion")
                            },
                        }
                    });
                } else {
                    dispatch::sync_dispatch(command);
                }
            } else {
                DispatchCommand::command()
                    .print_help()
                    .unwrap();
            }
        },
        Commands::Listen { filter, max_events, json } => {
            if let Err(e) = listen::listen(filter, max_events, json) {
                eprintln!("Error: {e}");
                process::exit(1);
            }
        },
        Commands::React { config, inline: _, event, subtype, filter, dispatch, max_reactions } => {
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
        Commands::Global { config_path, setup, kill, restart } => {
            if kill {
                let status = std::process::Command::new("systemctl")
                    .args(["--user", "stop", "hyde-ipc.service"])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        println!("stopped successfully.");
                        std::process::exit(0);
                    },
                    Ok(s) => {
                        eprintln!("Failed to stop global reactions (exit code: {s}).");
                        std::process::exit(1);
                    },
                    Err(e) => {
                        eprintln!("Error stopping hyde-ipc.service: {e}");
                        std::process::exit(1);
                    },
                }
            }
            if restart {
                let status = std::process::Command::new("systemctl")
                    .args(["--user", "restart", "hyde-ipc.service"])
                    .status();
                match status {
                    Ok(s) if s.success() => {
                        println!("restarted successfully.");
                        std::process::exit(0);
                    },
                    Ok(s) => {
                        eprintln!("Failed to restart global reactions (exit code: {s}).");
                        std::process::exit(1);
                    },
                    Err(e) => {
                        eprintln!("Error restarting hyde-ipc.service: {e}");
                        std::process::exit(1);
                    },
                }
            }
            if setup {
                setup::setup_service_file();
                let home = env::var("HOME").expect("Could not get $HOME");
                let dest_dir = PathBuf::from(&home).join(".local/share/hyde-ipc");
                let dest = dest_dir.join("config.toml");
                if let Err(e) = fs::create_dir_all(&dest_dir) {
                    eprintln!("Error creating global config directory: {e}");
                    std::process::exit(1);
                }
                if let Some(path) = config_path {
                    setup::copy_and_reload_config(&path);
                } else if !dest.exists() {
                    if let Err(e) = fs::File::create(&dest) {
                        eprintln!("Error creating empty config file: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                setup::ensure_service_setup();
                if let Some(path) = config_path {
                    setup::copy_and_reload_config(&path);
                } else {
                    eprintln!("Error: you must provide a config file unless using --setup");
                    std::process::exit(1);
                }
            }
        },
        Commands::Setup => {
            setup::setup_service_file();
        },
        Commands::GenerateCompletion { shell } => {
            let mut cmd = Cli::command();
            let bin_name = cmd.get_name().to_string();
            generate(shell, &mut cmd, bin_name, &mut io::stdout());
        },
        Commands::Query(query_command) => {
            query::run_query(query_command.command);
        },
    }
}
fn print_usage_and_exit() {
    Cli::command().print_help().unwrap();
    process::exit(1);
}

fn print_dispatchers_list() {
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
  resize-active <position>                           - Resize the active window
  resize-window-pixel <pos> <win>                     - Resize a specific window

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
