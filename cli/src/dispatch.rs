use crate::flags::{Dispatch as DispatchCmd, ResizeCmd};
use crate::parsers::{
    ParsedCorner, ParsedCycleDirection, ParsedDirection, ParsedFullscreenType,
    ParsedWindowIdentifier, ParsedWindowMove, ParsedWorkspaceIdentifier,
};
use hyprland::dispatch::{Dispatch, DispatchType, Position};
use std::convert::TryFrom;
use std::str::FromStr;

/// Converts dispatch commands to hyprland DispatchType.
///
/// # Memory Leaks
///
/// This implementation uses `Box::leak()` to satisfy the 'static lifetime requirement
/// of the hyprland library's `DispatchType<'static>`. This is an API constraint that
/// cannot be avoided without modifying the upstream hyprland crate.
///
/// In the context of a CLI application with short-lived processes, these leaks are
/// acceptable as the memory is reclaimed when the process exits. Each command execution
/// leaks a small amount of memory for string data.
///
/// Future improvement: Consider contributing to hyprland crate to support owned types
/// or non-'static lifetimes.
impl TryFrom<DispatchCmd> for DispatchType<'static> {
    type Error = String;

    fn try_from(command: DispatchCmd) -> Result<Self, Self::Error> {
        match command {
            DispatchCmd::Exec { command } => {
                let command = command.join(" ");
                let command_static = Box::leak(command.into_boxed_str());
                Ok(DispatchType::Exec(command_static))
            },
            DispatchCmd::KillActiveWindow => Ok(DispatchType::KillActiveWindow),
            DispatchCmd::ToggleFloating { window } => {
                let window_id = if let Some(window_str) = window.to_identifier_string() {
                    Some(ParsedWindowIdentifier::from_str(&window_str)?.0)
                } else {
                    None
                };
                Ok(DispatchType::ToggleFloating(window_id))
            },
            DispatchCmd::ToggleSplit => Ok(DispatchType::ToggleSplit),
            DispatchCmd::ToggleOpaque => Ok(DispatchType::ToggleOpaque),
            DispatchCmd::MoveCursorToCorner { corner } => {
                let corner = ParsedCorner::from_str(&corner)?.0;
                Ok(DispatchType::MoveCursorToCorner(corner))
            },
            DispatchCmd::MoveCursor { x, y } => Ok(DispatchType::MoveCursor(x, y)),
            DispatchCmd::ToggleFullscreen { mode } => {
                let mode = ParsedFullscreenType::from_str(&mode)?.0;
                Ok(DispatchType::ToggleFullscreen(mode))
            },
            DispatchCmd::MoveToWorkspace { workspace } => {
                let workspace_id = ParsedWorkspaceIdentifier::from_str(&workspace)?.0;
                Ok(DispatchType::MoveToWorkspace(workspace_id, None))
            },
            DispatchCmd::MoveToWorkspaceSilent { workspace, window } => {
                let workspace_id = ParsedWorkspaceIdentifier::from_str(&workspace)?.0;
                let window_id = if let Some(window_str) = window.to_identifier_string() {
                    Some(ParsedWindowIdentifier::from_str(&window_str)?.0)
                } else {
                    None
                };
                Ok(DispatchType::MoveToWorkspaceSilent(workspace_id, window_id))
            },
            DispatchCmd::Workspace { workspace } => {
                let workspace_id = ParsedWorkspaceIdentifier::from_str(&workspace)?.0;
                Ok(DispatchType::Workspace(workspace_id))
            },
            DispatchCmd::CycleWindow { direction } => {
                let dir = ParsedCycleDirection::from_str(&direction)?.0;
                Ok(DispatchType::CycleWindow(dir))
            },
            DispatchCmd::MoveFocus { direction } => {
                let dir = ParsedDirection::from_str(&direction)?.0;
                Ok(DispatchType::MoveFocus(dir))
            },
            DispatchCmd::SwapWindow { direction } => {
                let dir = ParsedDirection::from_str(&direction)?.0;
                Ok(DispatchType::SwapWindow(dir))
            },
            DispatchCmd::FocusWindow { window } => {
                let window_id = window
                    .to_identifier_string()
                    .ok_or("Missing window identifier")?;
                let window_id = ParsedWindowIdentifier::from_str(&window_id)?.0;
                Ok(DispatchType::FocusWindow(window_id))
            },
            DispatchCmd::MoveWindow { target } => {
                let window_move = ParsedWindowMove::from_str(&target)?.0;
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
                let position = Position::Exact(x, y);
                Ok(DispatchType::ResizeActive(position))
            },
            DispatchCmd::ResizeActiveLegacy { params } => {
                let position = match params {
                    ResizeCmd::Delta { dx, dy } => Position::Delta(dx, dy),
                    ResizeCmd::Exact { width, height } => Position::Exact(width, height),
                };
                Ok(DispatchType::ResizeActive(position))
            },
            DispatchCmd::ExpandActive { dx, dy } => {
                let position = Position::Delta(dx, dy);
                Ok(DispatchType::ResizeActive(position))
            },
            DispatchCmd::ShrinkActive { dx, dy } => {
                let position = Position::Delta(-dx, -dy);
                Ok(DispatchType::ResizeActive(position))
            },
            DispatchCmd::ResizeWindowPixel { params, window } => {
                let position = match params {
                    ResizeCmd::Delta { dx, dy } => Position::Delta(dx, dy),
                    ResizeCmd::Exact { width, height } => Position::Exact(width, height),
                };
                let win_id_str = window
                    .to_identifier_string()
                    .ok_or("Missing window identifier")?;
                let win_id = ParsedWindowIdentifier::from_str(&win_id_str)?.0;
                Ok(DispatchType::ResizeWindowPixel(position, win_id))
            },
        }
    }
}

/// Runs one dispatch and hands every failure back to the caller.
///
/// Reporting the error here instead of returning it would leave the CLI exiting
/// with a success status on a dispatch that never happened.
pub fn handle_dispatch(command: DispatchCmd, is_async: bool) -> Result<(), String> {
    let dispatch_type = DispatchType::try_from(command)?;

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
