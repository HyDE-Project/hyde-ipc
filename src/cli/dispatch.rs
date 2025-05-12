//! # Hyprland Dispatch Commands
//! 
//! This module provides functionality for sending commands to Hyprland.
//! It includes parsers for command strings and handlers for synchronous
//! and asynchronous dispatch execution.

use hyprland::dispatch::{
    Corner, CycleDirection, Direction, Dispatch, DispatchType, FullscreenType, WindowIdentifier,
    WorkspaceIdentifierWithSpecial,
};
use hyprland::shared::Address;

/// Synchronously dispatch a command to Hyprland
///
/// # Arguments
///
/// * `list_dispatchers` - Whether to just list available dispatchers instead of executing a command
/// * `dispatcher` - The name of the dispatcher to execute
/// * `args` - Arguments for the dispatcher
pub fn sync_dispatch(list_dispatchers: bool, dispatcher: String, args: Vec<String>) {
    if list_dispatchers {
        print_available_dispatchers();
        return;
    }

    match parse_dispatcher(&dispatcher, &args) {
        Ok(dispatch_type) => {
            if let Err(e) = Dispatch::call(dispatch_type) {
                eprintln!("Error: {}", e);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

pub async fn async_dispatch(list_dispatchers: bool, dispatcher: String, args: Vec<String>) {
    if list_dispatchers {
        print_available_dispatchers();
        return;
    }

    match parse_dispatcher(&dispatcher, &args) {
        Ok(dispatch_type) => match Dispatch::call_async(dispatch_type).await {
            Ok(_) => {
                println!("Async dispatch completed successfully");
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        },
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}

fn print_available_dispatchers() {
    println!("Available dispatchers:");
    println!("  Basic commands:");
    println!("  Exec <command>                    - Execute a command");
    println!("  KillActiveWindow                  - Kill the active window");
    println!("  Exit                              - Exit Hyprland");
    println!("  ForceRendererReload               - Force the renderer to reload");
    println!();

    println!("  Window management:");
    println!("  ToggleFloating [window]           - Toggle floating mode for a window");
    println!(
        "  ToggleFullscreen <type>           - Toggle fullscreen mode (Real, Maximize, NoParam)"
    );
    println!("  ToggleFakeFullscreen              - Toggle fake fullscreen for the active window");
    println!("  TogglePseudo                      - Toggle pseudo tiling for the active window");
    println!("  TogglePin                         - Pin the active window to all workspaces");
    println!("  ToggleOpaque                      - Toggle opacity for the active window");
    println!("  CenterWindow                      - Center the active window");
    println!(
        "  BringActiveToTop                  - Bring the active window to the top of the stack"
    );
    println!();

    println!("  Focus control:");
    println!(
        "  MoveFocus <direction>             - Move focus in a direction (Up, Down, Left, Right)"
    );
    println!("  FocusWindow <window>              - Focus a specific window");
    println!("  FocusMonitor <identifier>         - Focus a specific monitor");
    println!("  FocusUrgentOrLast                 - Focus the urgent window or the last one");
    println!("  FocusCurrentOrLast                - Switch focus between current and last window");
    println!();

    println!("  Window movement:");
    println!("  MoveWindow <direction>            - Move window in a direction");
    println!("  MoveActive <position>             - Move the active window to a position");
    println!("  MoveWindowPixel <position> <win>  - Move a specific window to a position");
    println!("  ResizeActive <position>           - Resize the active window");
    println!("  ResizeWindowPixel <pos> <win>     - Resize a specific window");
    println!();

    println!("  Workspace management:");
    println!(
        "  Workspace <workspace>             - Switch to workspace (number, previous, empty, name:NAME)"
    );
    println!("  MoveToWorkspace <workspace>       - Move window to workspace");
    println!(
        "  MoveToWorkspaceSilent <workspace> - Move window to workspace without switching to it"
    );
    println!("  RenameWorkspace <id> <name>       - Rename a workspace");
    println!();

    println!("  Cycling and swapping:");
    println!("  CycleWindow <direction>           - Cycle windows (Next, Previous)");
    println!("  SwapNext <direction>              - Swap with next window (Next, Previous)");
    println!(
        "  SwapWindow <direction>            - Swap windows in a direction (Up, Down, Left, Right)"
    );
    println!();

    println!("  Cursor control:");
    println!(
        "  MoveCursorToCorner <corner>       - Move cursor to a corner (TopLeft, TopRight, BottomLeft, BottomRight)"
    );
    println!("  MoveCursor <x> <y>                - Move cursor to a position");
    println!("  SetCursor <theme> <size>          - Set cursor theme and size");
    println!();

    println!("  Monitor management:");
    println!("  MoveCurrentWorkspaceToMonitor <mon> - Move current workspace to a monitor");
    println!("  MoveWorkspaceToMonitor <ws> <mon>   - Move a workspace to a monitor");
    println!("  SwapActiveWorkspaces <mon1> <mon2>  - Swap active workspaces of two monitors");
    println!("  ToggleDPMS <on/off> [monitor]       - Toggle DPMS status for monitors");
    println!();

    println!("  Layout-specific commands (Dwindle):");
    println!("  ToggleSplit                       - Toggle the split orientation");
    println!();

    println!("  Layout-specific commands (Master):");
    println!("  SwapWithMaster <param>            - Swap with master window (Master, Child, Auto)");
    println!("  FocusMaster <param>               - Focus the master window (Master, Auto)");
    println!("  AddMaster                         - Add a master to the master side");
    println!("  RemoveMaster                      - Remove a master from the master side");
    println!("  OrientationLeft                   - Set orientation to left");
    println!("  OrientationRight                  - Set orientation to right");
    println!("  OrientationTop                    - Set orientation to top");
    println!("  OrientationBottom                 - Set orientation to bottom");
    println!("  OrientationCenter                 - Set orientation to center");
    println!("  OrientationNext                   - Cycle to next orientation");
    println!("  OrientationPrev                   - Cycle to previous orientation");
    println!();

    println!("  Window grouping:");
    println!("  ToggleGroup                       - Toggle the current window into a group");
    println!(
        "  ChangeGroupActive <direction>     - Switch to next window in group (Forward, Back)"
    );
    println!("  LockGroups <action>               - Lock groups (Lock, Unlock, ToggleLock)");
    println!("  MoveIntoGroup <direction>         - Move window into group in direction");
    println!("  MoveOutOfGroup                    - Move window out of group");
    println!();

    println!("Window identifiers (can be used with ToggleFloating, FocusWindow, etc.):");
    println!("  class:REGEX                       - Match window by class regex");
    println!("  title:REGEX                       - Match window by title regex");
    println!("  pid:PID                           - Match window by process ID");
    println!(
        "  address:ADDR                      - Match window by address (hex value, with or without 0x prefix)"
    );
    println!();

    println!("Examples:");
    println!("  hypr-rs dispatch Exec \"kitty\"");
    println!("  hypr-rs dispatch MoveCursorToCorner TopLeft");
    println!("  hypr-rs dispatch Workspace 1");
    println!("  hypr-rs dispatch --async ToggleFullscreen Maximize");
    println!("  hypr-rs dispatch CycleWindow Next");
    println!("  hypr-rs dispatch MoveFocus Right");
    println!("  hypr-rs dispatch ToggleFloating \"class:^(Google-chrome)$\"");
    println!("  hypr-rs dispatch FocusWindow \"title:^(.*Terminal.*)$\"");
    println!("  hypr-rs dispatch ToggleFloating address:5934277460f0");
}

/// Parse a window identifier from a string
fn parse_window_identifier(identifier: &str) -> Result<WindowIdentifier<'static>, String> {
    if identifier.starts_with("class:") {
        let class = &identifier["class:".len()..];
        let class_static = Box::leak(class.to_string().into_boxed_str());
        Ok(WindowIdentifier::ClassRegularExpression(class_static))
    } else if identifier.starts_with("title:") {
        let title = &identifier["title:".len()..];
        let title_static = Box::leak(title.to_string().into_boxed_str());
        Ok(WindowIdentifier::Title(title_static))
    } else if identifier.starts_with("pid:") {
        let pid_str = &identifier["pid:".len()..];
        if let Ok(pid) = pid_str.parse::<u32>() {
            Ok(WindowIdentifier::ProcessId(pid))
        } else {
            Err(format!("Invalid PID: {}", pid_str))
        }
    } else if identifier.starts_with("address:") {
        // Get the address part without the prefix
        let addr_str = &identifier["address:".len()..];
        // Create an Address directly from the string
        // Address::new handles both with and without 0x prefix
        Ok(WindowIdentifier::Address(Address::new(addr_str)))
    } else {
        // Default to class if no prefix is provided
        let class_static = Box::leak(identifier.to_string().into_boxed_str());
        Ok(WindowIdentifier::ClassRegularExpression(class_static))
    }
}

/// Parse a dispatcher command string into a DispatchType
///
/// This function takes a dispatcher name and arguments and converts them
/// into the appropriate DispatchType enum variant that can be executed.
///
/// # Arguments
///
/// * `dispatcher` - The name of the dispatcher (e.g., "MoveFocus", "Workspace")
/// * `args` - Arguments for the dispatcher
///
/// # Returns
///
/// * `Ok(DispatchType)` - The parsed dispatcher ready to be executed
/// * `Err(String)` - Error message if parsing failed
pub fn parse_dispatcher(
    dispatcher: &str,
    args: &[String],
) -> Result<DispatchType<'static>, String> {
    match dispatcher {
        "Exec" => {
            let program = args.join(" ");
            // Use a static string to avoid lifetime issues
            let program_static = Box::leak(program.into_boxed_str());
            Ok(DispatchType::Exec(program_static))
        }
        "KillActiveWindow" => Ok(DispatchType::KillActiveWindow),
        "ToggleFloating" => {
            if args.is_empty() {
                Ok(DispatchType::ToggleFloating(None))
            } else {
                // Parse window identifier
                let identifier = args.join(" ");
                parse_window_identifier(&identifier)
                    .map(|win_id| DispatchType::ToggleFloating(Some(win_id)))
            }
        }
        "ToggleSplit" => Ok(DispatchType::ToggleSplit),
        "ToggleOpaque" => Ok(DispatchType::ToggleOpaque),
        "MoveCursorToCorner" => {
            if args.is_empty() {
                return Err("MoveCursorToCorner requires a corner argument".to_string());
            }
            match args[0].as_str() {
                "TopLeft" => Ok(DispatchType::MoveCursorToCorner(Corner::TopLeft)),
                "TopRight" => Ok(DispatchType::MoveCursorToCorner(Corner::TopRight)),
                "BottomLeft" => Ok(DispatchType::MoveCursorToCorner(Corner::BottomLeft)),
                "BottomRight" => Ok(DispatchType::MoveCursorToCorner(Corner::BottomRight)),
                _ => Err(format!("Unknown corner: {}", args[0])),
            }
        }
        "ToggleFullscreen" => {
            if args.is_empty() {
                return Err("ToggleFullscreen requires a type argument".to_string());
            }
            match args[0].as_str() {
                "Real" => Ok(DispatchType::ToggleFullscreen(FullscreenType::Real)),
                "Maximize" => Ok(DispatchType::ToggleFullscreen(FullscreenType::Maximize)),
                "NoParam" => Ok(DispatchType::ToggleFullscreen(FullscreenType::NoParam)),
                _ => Err(format!("Unknown fullscreen type: {}", args[0])),
            }
        }
        "MoveToWorkspace" => {
            if args.is_empty() {
                return Err("MoveToWorkspace requires a workspace identifier".to_string());
            }

            // Parse the first argument as a relative workspace number
            if let Ok(rel_num) = args[0].parse::<i32>() {
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Relative(rel_num),
                    None,
                ))
            } else if args[0] == "previous" {
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Previous,
                    None,
                ))
            } else if args[0] == "empty" {
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Empty,
                    None,
                ))
            } else if args[0].starts_with("name:") {
                let name = &args[0]["name:".len()..];
                // Use a static string to avoid lifetime issues
                let name_static = Box::leak(name.to_string().into_boxed_str());
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Name(name_static),
                    None,
                ))
            } else {
                Err(format!("Unknown workspace identifier: {}", args[0]))
            }
        }
        "Workspace" => {
            if args.is_empty() {
                return Err("Workspace requires a workspace identifier".to_string());
            }

            // Parse the first argument as a relative workspace number
            if let Ok(rel_num) = args[0].parse::<i32>() {
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Relative(rel_num),
                ))
            } else if args[0] == "previous" {
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Previous,
                ))
            } else if args[0] == "empty" {
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Empty,
                ))
            } else if args[0].starts_with("name:") {
                let name = &args[0]["name:".len()..];
                // Use a static string to avoid lifetime issues
                let name_static = Box::leak(name.to_string().into_boxed_str());
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Name(name_static),
                ))
            } else {
                Err(format!("Unknown workspace identifier: {}", args[0]))
            }
        }
        "CycleWindow" => {
            if args.is_empty() {
                return Err("CycleWindow requires a direction argument".to_string());
            }
            match args[0].as_str() {
                "Next" => Ok(DispatchType::CycleWindow(CycleDirection::Next)),
                "Previous" => Ok(DispatchType::CycleWindow(CycleDirection::Previous)),
                _ => Err(format!("Unknown cycle direction: {}", args[0])),
            }
        }
        "MoveFocus" => {
            if args.is_empty() {
                return Err("MoveFocus requires a direction argument".to_string());
            }
            match args[0].as_str() {
                "Up" => Ok(DispatchType::MoveFocus(Direction::Up)),
                "Down" => Ok(DispatchType::MoveFocus(Direction::Down)),
                "Left" => Ok(DispatchType::MoveFocus(Direction::Left)),
                "Right" => Ok(DispatchType::MoveFocus(Direction::Right)),
                _ => Err(format!("Unknown direction: {}", args[0])),
            }
        }
        "SwapWindow" => {
            if args.is_empty() {
                return Err("SwapWindow requires a direction argument".to_string());
            }
            match args[0].as_str() {
                "Up" => Ok(DispatchType::SwapWindow(Direction::Up)),
                "Down" => Ok(DispatchType::SwapWindow(Direction::Down)),
                "Left" => Ok(DispatchType::SwapWindow(Direction::Left)),
                "Right" => Ok(DispatchType::SwapWindow(Direction::Right)),
                _ => Err(format!("Unknown direction: {}", args[0])),
            }
        }
        "FocusWindow" => {
            if args.is_empty() {
                return Err("FocusWindow requires a window identifier".to_string());
            }
            let identifier = args.join(" ");
            parse_window_identifier(&identifier).map(DispatchType::FocusWindow)
        }
        "ToggleFakeFullscreen" => Ok(DispatchType::ToggleFakeFullscreen),
        "TogglePseudo" => Ok(DispatchType::TogglePseudo),
        "TogglePin" => Ok(DispatchType::TogglePin),
        "CenterWindow" => Ok(DispatchType::CenterWindow),
        "BringActiveToTop" => Ok(DispatchType::BringActiveToTop),
        "FocusUrgentOrLast" => Ok(DispatchType::FocusUrgentOrLast),
        "FocusCurrentOrLast" => Ok(DispatchType::FocusCurrentOrLast),
        "ForceRendererReload" => Ok(DispatchType::ForceRendererReload),
        "Exit" => Ok(DispatchType::Exit),
        _ => Err(format!("Unknown dispatcher: {}", dispatcher)),
    }
}

