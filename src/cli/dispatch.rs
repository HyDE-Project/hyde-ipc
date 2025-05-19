use hyprland::dispatch::{
    Corner, CycleDirection, Direction, Dispatch, DispatchType, FullscreenType, Position,
    WindowIdentifier, WorkspaceIdentifierWithSpecial,
};
use hyprland::shared::Address;

/// Synchronously execute a dispatcher or list available dispatchers.
///
/// # Arguments
/// * `list_dispatchers` - If true, prints available dispatchers and exits.
/// * `dispatcher` - The dispatcher to execute.
/// * `args` - Arguments for the dispatcher.
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

/// Asynchronously execute a dispatcher or list available dispatchers.
///
/// # Arguments
/// * `list_dispatchers` - If true, prints available dispatchers and exits.
/// * `dispatcher` - The dispatcher to execute.
/// * `args` - Arguments for the dispatcher.
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

/// Print all available dispatchers and their usage.
struct Dispatcher {
    name: &'static str,
    description: &'static str,
}

struct DispatcherCategory {
    name: &'static str,
    dispatchers: Vec<Dispatcher>,
}

fn print_available_dispatchers() {
    let categories = vec![
        DispatcherCategory {
            name: "Basic commands",
            dispatchers: vec![
                Dispatcher {
                    name: "Exec <command>",
                    description: "Execute a command",
                },
                Dispatcher {
                    name: "KillActiveWindow",
                    description: "Kill the active window",
                },
                Dispatcher {
                    name: "Exit",
                    description: "Exit Hyprland",
                },
                Dispatcher {
                    name: "ForceRendererReload",
                    description: "Force the renderer to reload",
                },
            ],
        },
        DispatcherCategory {
            name: "Window management",
            dispatchers: vec![
                Dispatcher {
                    name: "ToggleFloating [window]",
                    description: "Toggle floating mode for a window",
                },
                Dispatcher {
                    name: "ToggleFullscreen <type>",
                    description: "Toggle fullscreen mode (Real, Maximize, NoParam)",
                },
                Dispatcher {
                    name: "ToggleFakeFullscreen",
                    description: "Toggle fake fullscreen for the active window",
                },
                Dispatcher {
                    name: "TogglePseudo",
                    description: "Toggle pseudo tiling for the active window",
                },
                Dispatcher {
                    name: "TogglePin",
                    description: "Pin the active window to all workspaces",
                },
                Dispatcher {
                    name: "ToggleOpaque",
                    description: "Toggle opacity for the active window",
                },
                Dispatcher {
                    name: "CenterWindow",
                    description: "Center the active window",
                },
                Dispatcher {
                    name: "BringActiveToTop",
                    description: "Bring the active window to the top of the stack",
                },
            ],
        },
        DispatcherCategory {
            name: "Focus control",
            dispatchers: vec![
                Dispatcher {
                    name: "MoveFocus <direction>",
                    description: "Move focus in a direction (Up, Down, Left, Right)",
                },
                Dispatcher {
                    name: "FocusWindow <window>",
                    description: "Focus a specific window",
                },
                Dispatcher {
                    name: "FocusMonitor <identifier>",
                    description: "Focus a specific monitor",
                },
                Dispatcher {
                    name: "FocusUrgentOrLast",
                    description: "Focus the urgent window or the last one",
                },
                Dispatcher {
                    name: "FocusCurrentOrLast",
                    description: "Switch focus between current and last window",
                },
            ],
        },
        DispatcherCategory {
            name: "Window movement",
            dispatchers: vec![
                Dispatcher {
                    name: "MoveWindow <direction>",
                    description: "Move window in a direction",
                },
                Dispatcher {
                    name: "MoveActive <position>",
                    description: "Move the active window to a position",
                },
                Dispatcher {
                    name: "MoveWindowPixel <position> <win>",
                    description: "Move a specific window to a position",
                },
                Dispatcher {
                    name: "ResizeActive <position>",
                    description: "Resize the active window",
                },
                Dispatcher {
                    name: "ResizeWindowPixel <pos> <win>",
                    description: "Resize a specific window",
                },
            ],
        },
        DispatcherCategory {
            name: "Workspace management",
            dispatchers: vec![
                Dispatcher {
                    name: "Workspace <workspace>",
                    description: "Switch to workspace (number, previous, empty, name:NAME)",
                },
                Dispatcher {
                    name: "MoveToWorkspace <workspace>",
                    description: "Move window to workspace",
                },
                Dispatcher {
                    name: "MoveToWorkspaceSilent <workspace>",
                    description: "Move window to workspace without switching to it",
                },
                Dispatcher {
                    name: "RenameWorkspace <id> <name>",
                    description: "Rename a workspace",
                },
            ],
        },
        DispatcherCategory {
            name: "Cycling and swapping",
            dispatchers: vec![
                Dispatcher {
                    name: "CycleWindow <direction>",
                    description: "Cycle windows (Next, Previous)",
                },
                Dispatcher {
                    name: "SwapNext <direction>",
                    description: "Swap with next window (Next, Previous)",
                },
                Dispatcher {
                    name: "SwapWindow <direction>",
                    description: "Swap windows in a direction (Up, Down, Left, Right)",
                },
            ],
        },
        DispatcherCategory {
            name: "Cursor control",
            dispatchers: vec![
                Dispatcher {
                    name: "MoveCursorToCorner <corner>",
                    description: "Move cursor to a corner (TopLeft, TopRight, BottomLeft, BottomRight)",
                },
                Dispatcher {
                    name: "MoveCursor <x> <y>",
                    description: "Move cursor to a position",
                },
                Dispatcher {
                    name: "SetCursor <theme> <size>",
                    description: "Set cursor theme and size",
                },
            ],
        },
        DispatcherCategory {
            name: "Monitor management",
            dispatchers: vec![
                Dispatcher {
                    name: "MoveCurrentWorkspaceToMonitor <mon>",
                    description: "Move current workspace to a monitor",
                },
                Dispatcher {
                    name: "MoveWorkspaceToMonitor <ws> <mon>",
                    description: "Move a workspace to a monitor",
                },
                Dispatcher {
                    name: "SwapActiveWorkspaces <mon1> <mon2>",
                    description: "Swap active workspaces of two monitors",
                },
                Dispatcher {
                    name: "ToggleDPMS <on/off> [monitor]",
                    description: "Toggle DPMS status for monitors",
                },
            ],
        },
        DispatcherCategory {
            name: "Layout-specific commands (Dwindle)",
            dispatchers: vec![Dispatcher {
                name: "ToggleSplit",
                description: "Toggle the split orientation",
            }],
        },
        DispatcherCategory {
            name: "Layout-specific commands (Master)",
            dispatchers: vec![
                Dispatcher {
                    name: "SwapWithMaster <param>",
                    description: "Swap with master window (Master, Child, Auto)",
                },
                Dispatcher {
                    name: "FocusMaster <param>",
                    description: "Focus the master window (Master, Auto)",
                },
                Dispatcher {
                    name: "AddMaster",
                    description: "Add a master to the master side",
                },
                Dispatcher {
                    name: "RemoveMaster",
                    description: "Remove a master from the master side",
                },
                Dispatcher {
                    name: "OrientationLeft",
                    description: "Set orientation to left",
                },
                Dispatcher {
                    name: "OrientationRight",
                    description: "Set orientation to right",
                },
                Dispatcher {
                    name: "OrientationTop",
                    description: "Set orientation to top",
                },
                Dispatcher {
                    name: "OrientationBottom",
                    description: "Set orientation to bottom",
                },
                Dispatcher {
                    name: "OrientationCenter",
                    description: "Set orientation to center",
                },
                Dispatcher {
                    name: "OrientationNext",
                    description: "Cycle to next orientation",
                },
                Dispatcher {
                    name: "OrientationPrev",
                    description: "Cycle to previous orientation",
                },
            ],
        },
        DispatcherCategory {
            name: "Window grouping",
            dispatchers: vec![
                Dispatcher {
                    name: "ToggleGroup",
                    description: "Toggle the current window into a group",
                },
                Dispatcher {
                    name: "ChangeGroupActive <direction>",
                    description: "Switch to next window in group (Forward, Back)",
                },
                Dispatcher {
                    name: "LockGroups <action>",
                    description: "Lock groups (Lock, Unlock, ToggleLock)",
                },
                Dispatcher {
                    name: "MoveIntoGroup <direction>",
                    description: "Move window into group in direction",
                },
                Dispatcher {
                    name: "MoveOutOfGroup",
                    description: "Move window out of group",
                },
            ],
        },
    ];

    let identifiers = vec![
        Dispatcher {
            name: "class:REGEX",
            description: "Match window by class regex",
        },
        Dispatcher {
            name: "title:REGEX",
            description: "Match window by title regex",
        },
        Dispatcher {
            name: "pid:PID",
            description: "Match window by process ID",
        },
        Dispatcher {
            name: "address:ADDR",
            description: "Match window by address (hex value, with or without 0x prefix)",
        },
    ];

    let examples = vec![
        "hypr-rs dispatch Exec \"kitty\"",
        "hypr-rs dispatch MoveCursorToCorner TopLeft",
        "hypr-rs dispatch Workspace 1",
        "hypr-rs dispatch --async ToggleFullscreen Maximize",
        "hypr-rs dispatch CycleWindow Next",
        "hypr-rs dispatch MoveFocus Right",
        "hypr-rs dispatch ToggleFloating \"class:^(Google-chrome)$\"",
        "hypr-rs dispatch FocusWindow \"title:^(.*Terminal.*)$\"",
        "hypr-rs dispatch ToggleFloating address:5934277460f0",
    ];

    println!("Available dispatchers:");

    for category in categories {
        println!("\n  {}:", category.name);
        // Find the maximum length of dispatcher names in this category for alignment
        let max_name_len = category
            .dispatchers
            .iter()
            .map(|d| d.name.len())
            .max()
            .unwrap_or(0);

        for dispatcher in category.dispatchers {
            // Use formatting specifier :<width> to pad with spaces on the right
            println!(
                "    {:<width$} - {}",
                dispatcher.name,
                dispatcher.description,
                width = max_name_len
            );
        }
    }

    println!("\nWindow identifiers (can be used with ToggleFloating, FocusWindow, etc.):");
    let max_id_name_len = identifiers
        .iter()
        .map(|id| id.name.len())
        .max()
        .unwrap_or(0);
    for identifier in identifiers {
        println!(
            "  {:<width$} - {}",
            identifier.name,
            identifier.description,
            width = max_id_name_len
        );
    }

    println!("\nExamples:");
    for example in examples {
        println!("  {}", example);
    }
}

/// Parse a window identifier from a string (e.g., class, title, pid, address).
fn parse_window_identifier(identifier: &str) -> Result<WindowIdentifier<'static>, String> {
    if let Some(class) = identifier.strip_prefix("class:") {
        let class_static = Box::leak(class.to_string().into_boxed_str());
        Ok(WindowIdentifier::ClassRegularExpression(class_static))
    } else if let Some(title) = identifier.strip_prefix("title:") {
        let title_static = Box::leak(title.to_string().into_boxed_str());
        Ok(WindowIdentifier::Title(title_static))
    } else if let Some(pid_str) = identifier.strip_prefix("pid:") {
        if let Ok(pid) = pid_str.parse::<u32>() {
            Ok(WindowIdentifier::ProcessId(pid))
        } else {
            Err(format!("Invalid PID: {}", pid_str))
        }
    } else if let Some(addr_str) = identifier.strip_prefix("address:") {
        Ok(WindowIdentifier::Address(Address::new(addr_str)))
    } else {
        // Default to class if no prefix is provided
        let class_static = Box::leak(identifier.to_string().into_boxed_str());
        Ok(WindowIdentifier::ClassRegularExpression(class_static))
    }
}

/// Parse a dispatcher and its arguments into a DispatchType.
///
/// # Arguments
/// * `dispatcher` - The dispatcher name.
/// * `args` - Arguments for the dispatcher.
///
/// # Returns
/// * `Ok(DispatchType)` if parsing is successful.
/// * `Err(String)` if parsing fails.
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
        "ResizeActive" => {
            if args.is_empty() {
                return Err("ResizeActive requires a position argument: either <dx> <dy> or exact <width> <height>".to_string());
            }
            if args[0] == "exact" {
                if args.len() != 3 {
                    return Err(
                        "ResizeActive exact requires two arguments: <width> <height>".to_string(),
                    );
                }
                let width = args[1]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid width: {}", args[1]))?;
                let height = args[2]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid height: {}", args[2]))?;
                Ok(DispatchType::ResizeActive(Position::Exact(width, height)))
            } else if args.len() == 2 {
                let dx = args[0]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid dx: {}", args[0]))?;
                let dy = args[1]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid dy: {}", args[1]))?;
                Ok(DispatchType::ResizeActive(Position::Delta(dx, dy)))
            } else {
                Err("ResizeActive requires either two arguments (<dx> <dy>) or 'exact <width> <height>'".to_string())
            }
        }
        "ResizeWindowPixel" => {
            if args.is_empty() {
                return Err("ResizeWindowPixel requires a position and window argument: either <dx> <dy> <win> or exact <width> <height> <win>".to_string());
            }
            if args[0] == "exact" {
                if args.len() != 4 {
                    return Err(
                        "ResizeWindowPixel exact requires three arguments: <width> <height> <win>"
                            .to_string(),
                    );
                }
                let width = args[1]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid width: {}", args[1]))?;
                let height = args[2]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid height: {}", args[2]))?;
                let win_id = parse_window_identifier(&args[3])?;
                Ok(DispatchType::ResizeWindowPixel(
                    Position::Exact(width, height),
                    win_id,
                ))
            } else if args.len() == 3 {
                let dx = args[0]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid dx: {}", args[0]))?;
                let dy = args[1]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid dy: {}", args[1]))?;
                let win_id = parse_window_identifier(&args[2])?;
                Ok(DispatchType::ResizeWindowPixel(
                    Position::Delta(dx, dy),
                    win_id,
                ))
            } else {
                Err("ResizeWindowPixel requires either three arguments (<dx> <dy> <win>) or 'exact <width> <height> <win>'".to_string())
            }
        }
        _ => Err(format!("Unknown dispatcher: {}", dispatcher)),
    }
}
