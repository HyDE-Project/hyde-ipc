use crate::cli::flags::Dispatch as DispatchCmd;
use hyprland::dispatch::{
    Corner, CycleDirection, Direction, Dispatch, DispatchType, FullscreenType, Position,
    WindowIdentifier, WorkspaceIdentifierWithSpecial,
};
use hyprland::shared::Address;

/// Synchronously execute a dispatcher.
///
/// # Arguments
/// * `command` - The dispatcher command to execute.
pub fn sync_dispatch(command: DispatchCmd) {
    match parse_dispatcher(command) {
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

/// Asynchronously execute a dispatcher.
///
/// # Arguments
/// * `command` - The dispatcher command to execute.
pub async fn async_dispatch(command: DispatchCmd) {
    match parse_dispatcher(command) {
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
pub fn parse_dispatcher(command: DispatchCmd) -> Result<DispatchType<'static>, String> {
    match command {
        DispatchCmd::Exec { command } => {
            let program = command.join(" ");
            // Use a static string to avoid lifetime issues
            let program_static = Box::leak(program.into_boxed_str());
            Ok(DispatchType::Exec(program_static))
        }
        DispatchCmd::KillActiveWindow => Ok(DispatchType::KillActiveWindow),
        DispatchCmd::ToggleFloating { window } => {
            if window.is_none() {
                Ok(DispatchType::ToggleFloating(None))
            } else {
                // Parse window identifier
                let identifier = window.unwrap();
                parse_window_identifier(&identifier)
                    .map(|win_id| DispatchType::ToggleFloating(Some(win_id)))
            }
        }
        DispatchCmd::ToggleSplit => Ok(DispatchType::ToggleSplit),
        DispatchCmd::ToggleOpaque => Ok(DispatchType::ToggleOpaque),
        DispatchCmd::MoveCursorToCorner { corner } => match corner.as_str() {
            "TopLeft" => Ok(DispatchType::MoveCursorToCorner(Corner::TopLeft)),
            "TopRight" => Ok(DispatchType::MoveCursorToCorner(Corner::TopRight)),
            "BottomLeft" => Ok(DispatchType::MoveCursorToCorner(Corner::BottomLeft)),
            "BottomRight" => Ok(DispatchType::MoveCursorToCorner(Corner::BottomRight)),
            _ => Err(format!("Unknown corner: {}", corner)),
        },
        DispatchCmd::ToggleFullscreen { mode } => match mode.as_str() {
            "Real" => Ok(DispatchType::ToggleFullscreen(FullscreenType::Real)),
            "Maximize" => Ok(DispatchType::ToggleFullscreen(FullscreenType::Maximize)),
            "NoParam" => Ok(DispatchType::ToggleFullscreen(FullscreenType::NoParam)),
            _ => Err(format!("Unknown fullscreen type: {}", mode)),
        },
        DispatchCmd::MoveToWorkspace { workspace } => {
            // Parse the first argument as a relative workspace number
            if let Ok(rel_num) = workspace.parse::<i32>() {
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Relative(rel_num),
                    None,
                ))
            } else if workspace == "previous" {
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Previous,
                    None,
                ))
            } else if workspace == "empty" {
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Empty,
                    None,
                ))
            } else if let Some(name) = workspace.strip_prefix("name:") {
                // Use a static string to avoid lifetime issues
                let name_static = Box::leak(name.to_string().into_boxed_str());
                Ok(DispatchType::MoveToWorkspace(
                    WorkspaceIdentifierWithSpecial::Name(name_static),
                    None,
                ))
            } else {
                Err(format!("Unknown workspace identifier: {}", workspace))
            }
        }
        DispatchCmd::Workspace { workspace } => {
            // Parse the first argument as a relative workspace number
            if let Ok(rel_num) = workspace.parse::<i32>() {
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Relative(rel_num),
                ))
            } else if workspace == "previous" {
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Previous,
                ))
            } else if workspace == "empty" {
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Empty,
                ))
            } else if let Some(name) = workspace.strip_prefix("name:") {
                // Use a static string to avoid lifetime issues
                let name_static = Box::leak(name.to_string().into_boxed_str());
                Ok(DispatchType::Workspace(
                    WorkspaceIdentifierWithSpecial::Name(name_static),
                ))
            } else {
                Err(format!("Unknown workspace identifier: {}", workspace))
            }
        }
        DispatchCmd::CycleWindow { direction } => match direction.as_str() {
            "Next" => Ok(DispatchType::CycleWindow(CycleDirection::Next)),
            "Previous" => Ok(DispatchType::CycleWindow(CycleDirection::Previous)),
            _ => Err(format!("Unknown cycle direction: {}", direction)),
        },
        DispatchCmd::MoveFocus { direction } => match direction.as_str() {
            "Up" => Ok(DispatchType::MoveFocus(Direction::Up)),
            "Down" => Ok(DispatchType::MoveFocus(Direction::Down)),
            "Left" => Ok(DispatchType::MoveFocus(Direction::Left)),
            "Right" => Ok(DispatchType::MoveFocus(Direction::Right)),
            _ => Err(format!("Unknown direction: {}", direction)),
        },
        DispatchCmd::SwapWindow { direction } => match direction.as_str() {
            "Up" => Ok(DispatchType::SwapWindow(Direction::Up)),
            "Down" => Ok(DispatchType::SwapWindow(Direction::Down)),
            "Left" => Ok(DispatchType::SwapWindow(Direction::Left)),
            "Right" => Ok(DispatchType::SwapWindow(Direction::Right)),
            _ => Err(format!("Unknown direction: {}", direction)),
        },
        DispatchCmd::FocusWindow { window } => {
            parse_window_identifier(&window).map(DispatchType::FocusWindow)
        }
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
        DispatchCmd::ResizeWindowPixel { resize_params } => {
            let args = resize_params;
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
    }
}
