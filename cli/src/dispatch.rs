use crate::flags::Dispatch as DispatchCmd;
use hyprland::dispatch::{
    Corner, CycleDirection, Direction, Dispatch, DispatchType, FullscreenType, MonitorIdentifier,
    Position, WindowIdentifier, WindowMove, WorkspaceIdentifierWithSpecial,
};
use hyprland::shared::Address;

pub fn build_dispatch_cmd(dispatcher: &str, args: &[String]) -> Result<DispatchCmd, String> {
    match dispatcher.to_lowercase().as_str() {
        "exec" => Ok(DispatchCmd::Exec { command: args.to_vec() }),
        "killactivewindow" => Ok(DispatchCmd::KillActiveWindow),
        "togglefloating" => Ok(DispatchCmd::ToggleFloating { window: args.first().cloned() }),
        "togglesplit" => Ok(DispatchCmd::ToggleSplit),
        "toggleopaque" => Ok(DispatchCmd::ToggleOpaque),
        "movecursortocorner" => Ok(DispatchCmd::MoveCursorToCorner {
            corner: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "movecursor" => {
            if args.len() != 2 {
                return Err("movecursor requires x and y arguments".to_string());
            }
            let x = args[0]
                .parse::<i64>()
                .map_err(|_| "Invalid x value")?;
            let y = args[1]
                .parse::<i64>()
                .map_err(|_| "Invalid y value")?;
            Ok(DispatchCmd::MoveCursor { x, y })
        },
        "togglefullscreen" => Ok(DispatchCmd::ToggleFullscreen {
            mode: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "movetoworkspace" => Ok(DispatchCmd::MoveToWorkspace {
            workspace: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "workspace" => Ok(DispatchCmd::Workspace {
            workspace: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "cyclewindow" => Ok(DispatchCmd::CycleWindow {
            direction: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "movefocus" => Ok(DispatchCmd::MoveFocus {
            direction: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "swapwindow" => Ok(DispatchCmd::SwapWindow {
            direction: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "focuswindow" => Ok(DispatchCmd::FocusWindow {
            window: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "togglefakefullscreen" => Ok(DispatchCmd::ToggleFakeFullscreen),
        "togglepseudo" => Ok(DispatchCmd::TogglePseudo),
        "togglepin" => Ok(DispatchCmd::TogglePin),
        "centerwindow" => Ok(DispatchCmd::CenterWindow),
        "bringactivetotop" => Ok(DispatchCmd::BringActiveToTop),
        "focusurgentorlast" => Ok(DispatchCmd::FocusUrgentOrLast),
        "focuscurrentorlast" => Ok(DispatchCmd::FocusCurrentOrLast),
        "forcerendererreload" => Ok(DispatchCmd::ForceRendererReload),
        "exit" => Ok(DispatchCmd::Exit),
        "resizeactive" => Ok(DispatchCmd::ResizeActive { resize_params: args.to_vec() }),
        "resizewindowpixel" => Ok(DispatchCmd::ResizeWindowPixel { resize_params: args.to_vec() }),
        _ => Err(format!("Unknown dispatcher: {dispatcher}")),
    }
}

/// Synchronously execute a dispatcher.
///
/// # Arguments
/// * `command` - The dispatcher command to execute.
pub fn sync_dispatch(command: DispatchCmd) {
    match parse_dispatcher(command) {
        Ok(dispatch_type) => {
            if let Err(e) = Dispatch::call(dispatch_type) {
                eprintln!("Error: {e}");
            }
        },
        Err(e) => {
            eprintln!("Error: {e}");
        },
    }
}

/// Asynchronously execute a dispatcher.
///
/// # Arguments
/// * `command` - The dispatcher command to execute.
pub async fn async_dispatch(command: DispatchCmd) {
    match parse_dispatcher(command) {
        Ok(dispatch_type) => match Dispatch::call_async(dispatch_type).await {
            Ok(_) => {
                println!("Async dispatch completed successfully");
            },
            Err(e) => {
                eprintln!("Error: {e}");
            },
        },
        Err(e) => {
            eprintln!("Error: {e}");
        },
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
            Err(format!("Invalid PID: {pid_str}"))
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
pub fn parse_dispatcher(command: DispatchCmd) -> Result<DispatchType<'static>, String> {
    match command {
        DispatchCmd::Exec { command } => {
            let program = command.join(" ");
            // Use a static string to avoid lifetime issues
            let program_static = Box::leak(program.into_boxed_str());
            Ok(DispatchType::Exec(program_static))
        },
        DispatchCmd::KillActiveWindow => Ok(DispatchType::KillActiveWindow),
        DispatchCmd::ToggleFloating { window } => {
            match window.is_none() {
                true => Ok(DispatchType::ToggleFloating(None)),
                false => {
                    // Parse window identifier
                    let identifier = window.unwrap();
                    parse_window_identifier(&identifier)
                        .map(|win_id| DispatchType::ToggleFloating(Some(win_id)))
                },
            }
        },
        DispatchCmd::ToggleSplit => Ok(DispatchType::ToggleSplit),
        DispatchCmd::ToggleOpaque => Ok(DispatchType::ToggleOpaque),
        DispatchCmd::MoveCursorToCorner { corner } => match corner.as_str() {
            "TopLeft" => Ok(DispatchType::MoveCursorToCorner(Corner::TopLeft)),
            "TopRight" => Ok(DispatchType::MoveCursorToCorner(Corner::TopRight)),
            "BottomLeft" => Ok(DispatchType::MoveCursorToCorner(Corner::BottomLeft)),
            "BottomRight" => Ok(DispatchType::MoveCursorToCorner(Corner::BottomRight)),
            _ => Err(format!("Unknown corner: {corner}")),
        },
        DispatchCmd::MoveCursor { x, y } => Ok(DispatchType::MoveCursor(x, y)),
        DispatchCmd::ToggleFullscreen { mode } => match mode.as_str() {
            "Real" => Ok(DispatchType::ToggleFullscreen(FullscreenType::Real)),
            "Maximize" => Ok(DispatchType::ToggleFullscreen(FullscreenType::Maximize)),
            "NoParam" => Ok(DispatchType::ToggleFullscreen(FullscreenType::NoParam)),
            _ => Err(format!("Unknown fullscreen type: {mode}")),
        },
        DispatchCmd::MoveToWorkspace { workspace } => {
            let workspace_id = if let Ok(id) = workspace.parse::<i32>() {
                WorkspaceIdentifierWithSpecial::Id(id)
            } else if let Some(num_str) = workspace.strip_prefix("right:") {
                let num = num_str
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid number for right: {num_str}"))?;
                WorkspaceIdentifierWithSpecial::Relative(num)
            } else if let Some(num_str) = workspace.strip_prefix("left:") {
                let num = num_str
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid number for left: {num_str}"))?;
                WorkspaceIdentifierWithSpecial::Relative(-num)
            } else if workspace == "previous" {
                WorkspaceIdentifierWithSpecial::Previous
            } else if workspace == "empty" {
                WorkspaceIdentifierWithSpecial::Empty
            } else if let Some(name) = workspace.strip_prefix("name:") {
                let name_static = Box::leak(name.to_string().into_boxed_str());
                WorkspaceIdentifierWithSpecial::Name(name_static)
            } else {
                return Err(format!("Unknown workspace identifier: {workspace}"));
            };
            Ok(DispatchType::MoveToWorkspace(workspace_id, None))
        },
        DispatchCmd::MoveToWorkspaceSilent { workspace, window } => {
            let workspace_id = if let Ok(id) = workspace.parse::<i32>() {
                WorkspaceIdentifierWithSpecial::Id(id)
            } else if let Some(num_str) = workspace.strip_prefix("right:") {
                let num = num_str
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid number for right: {num_str}"))?;
                WorkspaceIdentifierWithSpecial::Relative(num)
            } else if let Some(num_str) = workspace.strip_prefix("left:") {
                let num = num_str
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid number for left: {num_str}"))?;
                WorkspaceIdentifierWithSpecial::Relative(-num)
            } else if workspace == "previous" {
                WorkspaceIdentifierWithSpecial::Previous
            } else if workspace == "empty" {
                WorkspaceIdentifierWithSpecial::Empty
            } else if let Some(name) = workspace.strip_prefix("name:") {
                let name_static = Box::leak(name.to_string().into_boxed_str());
                WorkspaceIdentifierWithSpecial::Name(name_static)
            } else {
                return Err(format!("Unknown workspace identifier: {workspace}"));
            };

            let window_id =
                if let Some(win) = window { Some(parse_window_identifier(&win)?) } else { None };

            Ok(DispatchType::MoveToWorkspaceSilent(workspace_id, window_id))
        },
        DispatchCmd::Workspace { workspace } => {
            let workspace_id = if let Ok(id) = workspace.parse::<i32>() {
                WorkspaceIdentifierWithSpecial::Id(id)
            } else if let Some(num_str) = workspace.strip_prefix("right:") {
                let num = num_str
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid number for right: {num_str}"))?;
                WorkspaceIdentifierWithSpecial::Relative(num)
            } else if let Some(num_str) = workspace.strip_prefix("left:") {
                let num = num_str
                    .parse::<i32>()
                    .map_err(|_| format!("Invalid number for left: {num_str}"))?;
                WorkspaceIdentifierWithSpecial::Relative(-num)
            } else if workspace == "previous" {
                WorkspaceIdentifierWithSpecial::Previous
            } else if workspace == "empty" {
                WorkspaceIdentifierWithSpecial::Empty
            } else if let Some(name) = workspace.strip_prefix("name:") {
                let name_static = Box::leak(name.to_string().into_boxed_str());
                WorkspaceIdentifierWithSpecial::Name(name_static)
            } else {
                return Err(format!("Unknown workspace identifier: {workspace}"));
            };
            Ok(DispatchType::Workspace(workspace_id))
        },
        DispatchCmd::CycleWindow { direction } => match direction.to_lowercase().as_str() {
            "next" => Ok(DispatchType::CycleWindow(CycleDirection::Next)),
            "previous" => Ok(DispatchType::CycleWindow(CycleDirection::Previous)),
            _ => Err(format!("Unknown cycle direction: {direction}")),
        },
        DispatchCmd::MoveFocus { direction } => match direction.to_lowercase().as_str() {
            "up" => Ok(DispatchType::MoveFocus(Direction::Up)),
            "down" => Ok(DispatchType::MoveFocus(Direction::Down)),
            "left" => Ok(DispatchType::MoveFocus(Direction::Left)),
            "right" => Ok(DispatchType::MoveFocus(Direction::Right)),
            _ => Err(format!("Unknown direction: {direction}")),
        },
        DispatchCmd::SwapWindow { direction } => match direction.to_lowercase().as_str() {
            "up" => Ok(DispatchType::SwapWindow(Direction::Up)),
            "down" => Ok(DispatchType::SwapWindow(Direction::Down)),
            "left" => Ok(DispatchType::SwapWindow(Direction::Left)),
            "right" => Ok(DispatchType::SwapWindow(Direction::Right)),
            _ => Err(format!("Unknown direction: {direction}")),
        },
        DispatchCmd::MoveWindow { target } => {
            if let Some(monitor_name) = target.strip_prefix("mon:") {
                let monitor_name_static = Box::leak(
                    monitor_name
                        .to_string()
                        .into_boxed_str(),
                );
                Ok(DispatchType::MoveWindow(WindowMove::Monitor(MonitorIdentifier::Name(
                    monitor_name_static,
                ))))
            } else if let Ok(monitor_id) = target.parse::<i128>() {
                Ok(DispatchType::MoveWindow(WindowMove::Monitor(MonitorIdentifier::Id(monitor_id))))
            } else if target.to_lowercase().as_str() == "current" {
                Ok(DispatchType::MoveWindow(WindowMove::Monitor(MonitorIdentifier::Current)))
            } else if let Ok(rel_num) = target.parse::<i32>() {
                Ok(DispatchType::MoveWindow(WindowMove::Monitor(MonitorIdentifier::Relative(
                    rel_num,
                ))))
            } else if let Some(dir_str) = target
                .to_lowercase()
                .strip_prefix("dir:")
            {
                match dir_str {
                    "up" => Ok(DispatchType::MoveWindow(WindowMove::Direction(Direction::Up))),
                    "down" => Ok(DispatchType::MoveWindow(WindowMove::Direction(Direction::Down))),
                    "left" => Ok(DispatchType::MoveWindow(WindowMove::Direction(Direction::Left))),
                    "right" => {
                        Ok(DispatchType::MoveWindow(WindowMove::Direction(Direction::Right)))
                    },
                    _ => Err(format!("Unknown direction for MoveWindow: {dir_str}")),
                }
            } else {
                Err(format!("Unknown target for MoveWindow: {target}"))
            }
        },
        DispatchCmd::FocusWindow { window } => {
            parse_window_identifier(&window).map(DispatchType::FocusWindow)
        },
        DispatchCmd::ToggleFakeFullscreen => Ok(DispatchType::ToggleFakeFullscreen),
        DispatchCmd::TogglePseudo => Ok(DispatchType::TogglePseudo),
        DispatchCmd::TogglePin => Ok(DispatchType::TogglePin),
        DispatchCmd::CenterWindow => Ok(DispatchType::CenterWindow),
        DispatchCmd::BringActiveToTop => Ok(DispatchType::BringActiveToTop),
        DispatchCmd::FocusUrgentOrLast => Ok(DispatchType::FocusUrgentOrLast),
        DispatchCmd::FocusCurrentOrLast => Ok(DispatchType::FocusCurrentOrLast),
        DispatchCmd::ForceRendererReload => Ok(DispatchType::ForceRendererReload),
        DispatchCmd::Exit => Ok(DispatchType::Exit),
        DispatchCmd::ResizeActive { resize_params } => {
            let args = resize_params;
            if args.is_empty() {
                return Err("ResizeActive requires a position argument: either <dx> <dy> or \
                            exact <width> <height>"
                    .to_string());
            }
            if args[0] == "exact" {
                if args.len() != 3 {
                    return Err(
                        "ResizeActive exact requires two arguments: <width> <height>".to_string()
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
                Err("ResizeActive requires either two arguments (<dx> <dy>) or 'exact <width> \
                     <height>'"
                    .to_string())
            }
        },
        DispatchCmd::ResizeWindowPixel { resize_params } => {
            let args = resize_params;
            if args.is_empty() {
                return Err("ResizeWindowPixel requires a position and window argument: either \
                            <dx> <dy> <win> or exact <width> <height> <win>"
                    .to_string());
            }
            if args[0] == "exact" {
                if args.len() != 4 {
                    return Err("ResizeWindowPixel exact requires three arguments: <width> \
                                <height> <win>"
                        .to_string());
                }
                let width = args[1]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid width: {}", args[1]))?;
                let height = args[2]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid height: {}", args[2]))?;
                let win_id = parse_window_identifier(&args[3])?;
                Ok(DispatchType::ResizeWindowPixel(Position::Exact(width, height), win_id))
            } else if args.len() == 3 {
                let dx = args[0]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid dx: {}", args[0]))?;
                let dy = args[1]
                    .parse::<i16>()
                    .map_err(|_| format!("Invalid dy: {}", args[1]))?;
                let win_id = parse_window_identifier(&args[2])?;
                Ok(DispatchType::ResizeWindowPixel(Position::Delta(dx, dy), win_id))
            } else {
                Err("ResizeWindowPixel requires either three arguments (<dx> <dy> <win>) or \
                     'exact <width> <height> <win>'"
                    .to_string())
            }
        },
    }
}
