//! CLI entry point and command dispatch for hyde-ipc.
//!
//! This module parses CLI arguments and delegates to the appropriate subcommand logic.

mod dispatch;
mod flags;
mod keyword;
mod listen;
mod react;
mod react_config;
mod setup;

use clap::{CommandFactory, Parser};
use clap_complete::generate;
use flags::{Cli, Commands, DispatchCommand};
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
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        },
        Commands::React {
            r#async,
            config,
            inline: _,
            event,
            subtype,
            filter,
            dispatcher,
            args,
            max_reactions,
        } => {
            if let Some(config_path) = config {
                if let Err(e) = react_config::run_from_config(&config_path) {
                    eprintln!("Error running from config: {}", e);
                    process::exit(1);
                }
                return;
            }
            let event = event.unwrap_or_else(|| {
                eprintln!("Error: event is required");
                print_usage_and_exit();
                String::new()
            });
            let dispatcher = dispatcher.unwrap_or_else(|| {
                eprintln!("Error: dispatcher is required");
                print_usage_and_exit();
                String::new()
            });
            if r#async {
                println!("Note: async flag is deprecated");
            }
            if let Err(e) =
                react::sync_react(event, subtype, filter, dispatcher, args, max_reactions)
            {
                eprintln!("Error: {}", e);
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
                        eprintln!("Failed to stop global reactions (exit code: {}).", s);
                        std::process::exit(1);
                    },
                    Err(e) => {
                        eprintln!("Error stopping hyde-ipc.service: {}", e);
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
                        eprintln!("Failed to restart global reactions (exit code: {}).", s);
                        std::process::exit(1);
                    },
                    Err(e) => {
                        eprintln!("Error restarting hyde-ipc.service: {}", e);
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
                    eprintln!("Error creating global config directory: {}", e);
                    std::process::exit(1);
                }
                if let Some(path) = config_path {
                    setup::copy_and_reload_config(&path);
                } else {
                    if !dest.exists() {
                        if let Err(e) = fs::File::create(&dest) {
                            eprintln!("Error creating empty config file: {}", e);
                            std::process::exit(1);
                        }
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
    }
}
fn print_usage_and_exit() {
    Cli::command().print_help().unwrap();
    process::exit(1);
}

fn print_dispatchers_list() {
    let list = r#"Available dispatchers:
  Basic commands:
  Exec <command>                            - Execute a command
  KillActiveWindow                          - Kill the active window
  Exit                                      - Exit Hyprland
  ForceRendererReload                       - Force the renderer to reload

  Window management:
  ToggleFloating [window]                   - Toggle floating mode for a window
  ToggleFullscreen <type>                   - Toggle fullscreen mode (Real, Maximize, NoParam)
  ToggleFakeFullscreen                      - Toggle fake fullscreen for the active window
  TogglePseudo                              - Toggle pseudo tiling for the active window
  TogglePin                                 - Pin the active window to all workspaces
  ToggleOpaque                              - Toggle opacity for the active window
  CenterWindow                              - Center the active window
  BringActiveToTop                          - Bring the active window to the top of the stack

  Focus control:
  MoveFocus <direction>                     - Move focus in a direction (Up, Down, Left, Right)
  FocusWindow <window>                      - Focus a specific window
  FocusMonitor <identifier>                 - Focus a specific monitor
  FocusUrgentOrLast                         - Focus the urgent window or the last one
  FocusCurrentOrLast                        - Switch focus between current and last window

  Window movement:
  MoveWindow <direction>                    - Move window in a direction
  MoveActive <position>                     - Move the active window to a position
  MoveWindowPixel <position> <win>          - Move a specific window to a position
  ResizeActive <position>                   - Resize the active window
  ResizeWindowPixel <pos> <win>             - Resize a specific window

  Workspace management:
  Workspace <workspace>                     - Switch to workspace (number, previous, empty, name:NAME)
  MoveToWorkspace <workspace>               - Move window to workspace
  MoveToWorkspaceSilent <workspace>         - Move window to workspace without switching to it
  RenameWorkspace <id> <name>               - Rename a workspace

  Cycling and swapping:
  CycleWindow <direction>                   - Cycle windows (Next, Previous)
  SwapNext <direction>                      - Swap with next window (Next, Previous)
  SwapWindow <direction>                    - Swap windows in a direction (Up, Down, Left, Right)

  Cursor control:
  MoveCursorToCorner <corner>               - Move cursor to a corner (TopLeft, TopRight, BottomLeft, BottomRight)
  MoveCursor <x> <y>                        - Move cursor to a position
  SetCursor <theme> <size>                  - Set cursor theme and size

  Monitor management:
  MoveCurrentWorkspaceToMonitor <mon>       - Move current workspace to a monitor
  MoveWorkspaceToMonitor <ws> <mon>         - Move a workspace to a monitor
  SwapActiveWorkspaces <mon1> <mon2>        - Swap active workspaces of two monitors
  ToggleDPMS <on/off> [monitor]             - Toggle DPMS status for monitors

  Layout-specific commands (Dwindle):
  ToggleSplit                               - Toggle the split orientation

  Layout-specific commands (Master):
  SwapWithMaster <param>                    - Swap with master window (Master, Child, Auto)
  FocusMaster <param>                       - Focus the master window (Master, Auto)
  AddMaster                                 - Add a master to the master side
  RemoveMaster                              - Remove a master from the master side
  OrientationLeft                           - Set orientation to left
  OrientationRight                          - Set orientation to right
  OrientationTop                            - Set orientation to top
  OrientationBottom                         - Set orientation to bottom
  OrientationCenter                         - Set orientation to center
  OrientationNext                           - Cycle to next orientation
  OrientationPrev                           - Cycle to previous orientation

  Window grouping:
  ToggleGroup                               - Toggle the current window into a group
  ChangeGroupActive <direction>             - Switch to next window in group (Forward, Back)
  LockGroups <action>                       - Lock groups (Lock, Unlock, ToggleLock)
  MoveIntoGroup <direction>                 - Move window into group in direction
  MoveOutOfGroup                            - Move window out of group

Window identifiers (can be used with ToggleFloating, FocusWindow, etc.):
  class:REGEX                               - Match window by class regex
  title:REGEX                               - Match window by title regex
  pid:PID                                   - Match window by process ID
  address:ADDR                              - Match window by address (hex value, with or without 0x prefix)

Examples:
  hypr-rs dispatch Exec "kitty"
  hypr-rs dispatch MoveCursorToCorner TopLeft
  hypr-rs dispatch Workspace 1
  hypr-rs dispatch --async ToggleFullscreen Maximize
  hypr-rs dispatch CycleWindow Next
  hypr-rs dispatch MoveFocus Right
  hypr-rs dispatch ToggleFloating "class:^(Google-chrome)$"
  hypr-rs dispatch FocusWindow "title:^(.*Terminal.*)$"
  hypr-rs dispatch ToggleFloating address:5934277460f0
"#;
    println!("{}", list);
}
