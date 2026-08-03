use crate::flags::{Dispatch as DispatchCmd, ResizeCmd};
use crate::parsers::{
    ParsedCorner, ParsedCycleDirection, ParsedDirection, ParsedFullscreenType, ParsedWindowMove,
    ParsedWorkspaceIdentifier, window_identifier,
};
use hyprland::dispatch::{Dispatch, DispatchType, Position};
use std::str::FromStr;

/// The strings a converted [`DispatchType`] borrows but the command does not hold.
///
/// `DispatchType` borrows every string it carries, so anything derived rather than
/// taken straight from the command needs an owner that outlives the dispatch.
/// Keeping it here is what lets the conversion hand out borrows instead of leaking
/// a `Box` on every call — a leak the `react` daemon would otherwise repeat for
/// every event it handles.
struct DispatchStrings {
    exec: String,
}

impl DispatchStrings {
    fn new(command: &DispatchCmd) -> Self {
        let exec = match command {
            DispatchCmd::Exec { command } => command.join(" "),
            _ => String::new(),
        };

        Self { exec }
    }
}

/// Converts a CLI dispatch command into the hyprland dispatch type.
///
/// The result borrows from both arguments, so they have to stay alive until the
/// dispatch has been sent.
fn to_dispatch_type<'a>(
    command: &'a DispatchCmd,
    strings: &'a DispatchStrings,
) -> Result<DispatchType<'a>, String> {
    let missing_window = || "Missing window identifier".to_string();

    match command {
        DispatchCmd::Exec { .. } => Ok(DispatchType::Exec(&strings.exec)),
        DispatchCmd::KillActiveWindow => Ok(DispatchType::KillActiveWindow),
        DispatchCmd::ToggleFloating { window } => {
            Ok(DispatchType::ToggleFloating(window_identifier(window)))
        },
        DispatchCmd::ToggleSplit => Ok(DispatchType::ToggleSplit),
        DispatchCmd::ToggleOpaque => Ok(DispatchType::ToggleOpaque),
        DispatchCmd::MoveCursorToCorner { corner } => {
            let corner = ParsedCorner::from_str(corner)?.0;
            Ok(DispatchType::MoveCursorToCorner(corner))
        },
        DispatchCmd::MoveCursor { x, y } => Ok(DispatchType::MoveCursor(*x, *y)),
        DispatchCmd::ToggleFullscreen { mode } => {
            let mode = ParsedFullscreenType::from_str(mode)?.0;
            Ok(DispatchType::ToggleFullscreen(mode))
        },
        DispatchCmd::MoveToWorkspace { workspace } => {
            let workspace_id = ParsedWorkspaceIdentifier::parse(workspace)?.0;
            Ok(DispatchType::MoveToWorkspace(workspace_id, None))
        },
        DispatchCmd::MoveToWorkspaceSilent { workspace, window } => {
            let workspace_id = ParsedWorkspaceIdentifier::parse(workspace)?.0;
            Ok(DispatchType::MoveToWorkspaceSilent(workspace_id, window_identifier(window)))
        },
        DispatchCmd::Workspace { workspace } => {
            let workspace_id = ParsedWorkspaceIdentifier::parse(workspace)?.0;
            Ok(DispatchType::Workspace(workspace_id))
        },
        DispatchCmd::CycleWindow { direction } => {
            let dir = ParsedCycleDirection::from_str(direction)?.0;
            Ok(DispatchType::CycleWindow(dir))
        },
        DispatchCmd::MoveFocus { direction } => {
            let dir = ParsedDirection::from_str(direction)?.0;
            Ok(DispatchType::MoveFocus(dir))
        },
        DispatchCmd::SwapWindow { direction } => {
            let dir = ParsedDirection::from_str(direction)?.0;
            Ok(DispatchType::SwapWindow(dir))
        },
        DispatchCmd::FocusWindow { window } => {
            let window_id = window_identifier(window).ok_or_else(missing_window)?;
            Ok(DispatchType::FocusWindow(window_id))
        },
        DispatchCmd::MoveWindow { target } => {
            let window_move = ParsedWindowMove::parse(target)?.0;
            Ok(DispatchType::MoveWindow(window_move))
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
        DispatchCmd::ResizeActive { x, y } => {
            Ok(DispatchType::ResizeActive(Position::Exact(*x, *y)))
        },
        DispatchCmd::ResizeActiveLegacy { params } => {
            Ok(DispatchType::ResizeActive(to_position(params)))
        },
        DispatchCmd::ExpandActive { dx, dy } => {
            Ok(DispatchType::ResizeActive(Position::Delta(*dx, *dy)))
        },
        DispatchCmd::ShrinkActive { dx, dy } => {
            Ok(DispatchType::ResizeActive(Position::Delta(-dx, -dy)))
        },
        DispatchCmd::ResizeWindowPixel { params, window } => {
            let win_id = window_identifier(window).ok_or_else(missing_window)?;
            Ok(DispatchType::ResizeWindowPixel(to_position(params), win_id))
        },
    }
}

fn to_position(params: &ResizeCmd) -> Position {
    match params {
        ResizeCmd::Delta { dx, dy } => Position::Delta(*dx, *dy),
        ResizeCmd::Exact { width, height } => Position::Exact(*width, *height),
    }
}

/// Reports whether a dispatch command converts, without sending it.
pub fn validate_dispatch(command: &DispatchCmd) -> Result<(), String> {
    let strings = DispatchStrings::new(command);
    to_dispatch_type(command, &strings).map(|_| ())
}

/// Runs one dispatch and hands every failure back to the caller.
///
/// Reporting the error here instead of returning it would leave the CLI exiting
/// with a success status on a dispatch that never happened.
pub fn handle_dispatch(command: DispatchCmd, is_async: bool) -> Result<(), String> {
    let strings = DispatchStrings::new(&command);
    let dispatch_type = to_dispatch_type(&command, &strings)?;

    if is_async {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("creating async runtime: {e}"))?;

        return runtime.block_on(async {
            Dispatch::call_async(dispatch_type)
                .await
                .map_err(|e| e.to_string())
        });
    }

    Dispatch::call(dispatch_type).map_err(|e| e.to_string())
}
