use crate::flags::{Dispatch as DispatchCmd, ResizeCmd};
use crate::parsers::{
    ParsedCorner, ParsedCycleDirection, ParsedDirection, ParsedFullscreenType,
    ParsedWindowIdentifier, ParsedWindowMove, ParsedWorkspaceIdentifier,
};
use hyprland::dispatch::{Dispatch, DispatchType, Position};
use phf::phf_map;
use std::str::FromStr;
use std::convert::TryFrom;

type DispatcherBuilder = fn(Vec<String>) -> Result<DispatchType<'static>, String>;

macro_rules! dispatcher_no_args {
    ($name:ident, $dispatch_type:ident) => {
        fn $name(_: Vec<String>) -> Result<DispatchType<'static>, String> {
            Ok(DispatchType::$dispatch_type)
        }
    };
}

macro_rules! dispatcher_one_arg {
    ($name:ident, $dispatch_type:ident, $parser:ty, $error_msg:literal) => {
        fn $name(args: Vec<String>) -> Result<DispatchType<'static>, String> {
            let arg_str = args.first().ok_or($error_msg)?;
            let parsed_arg = <$parser>::from_str(arg_str)?.0;
            Ok(DispatchType::$dispatch_type(parsed_arg))
        }
    };
}

macro_rules! dispatcher_one_arg_optional {
    ($name:ident, $dispatch_type:ident, $parser:ty) => {
        fn $name(args: Vec<String>) -> Result<DispatchType<'static>, String> {
            let arg_str = args.first().map(|s| s.as_str()).unwrap_or("");
            let id = if arg_str.is_empty() {
                None
            } else {
                Some(<$parser>::from_str(arg_str)?.0)
            };
            Ok(DispatchType::$dispatch_type(id))
        }
    };
}

macro_rules! dispatcher_one_arg_optional_default {
    ($name:ident, $dispatch_type:ident, $parser:ty, $default:literal) => {
        fn $name(args: Vec<String>) -> Result<DispatchType<'static>, String> {
            let arg_str = args.first().map(|s| s.as_str()).unwrap_or($default);
            let parsed_arg = <$parser>::from_str(arg_str)?.0;
            Ok(DispatchType::$dispatch_type(parsed_arg))
        }
    };
}

fn build_exec(args: Vec<String>) -> Result<DispatchType<'static>, String> {
    let command = args.join(" ");
    let command_static = Box::leak(command.into_boxed_str());
    Ok(DispatchType::Exec(command_static))
}

dispatcher_no_args!(build_kill_active_window, KillActiveWindow);
dispatcher_one_arg_optional!(build_toggle_floating, ToggleFloating, ParsedWindowIdentifier);
dispatcher_no_args!(build_toggle_split, ToggleSplit);
dispatcher_no_args!(build_toggle_opaque, ToggleOpaque);
dispatcher_one_arg!(
    build_move_cursor_to_corner,
    MoveCursorToCorner,
    ParsedCorner,
    "Missing corner argument"
);

fn build_move_cursor(args: Vec<String>) -> Result<DispatchType<'static>, String> {
    if args.len() != 2 {
        return Err("movecursor requires x and y arguments".to_string());
    }
    let x = args[0].parse::<i64>().map_err(|_| "Invalid x value")?;
    let y = args[1].parse::<i64>().map_err(|_| "Invalid y value")?;
    Ok(DispatchType::MoveCursor(x, y))
}

dispatcher_one_arg_optional_default!(
    build_toggle_fullscreen,
    ToggleFullscreen,
    ParsedFullscreenType,
    "noparam"
);

fn build_move_to_workspace(args: Vec<String>) -> Result<DispatchType<'static>, String> {
    let workspace_str = args.first().ok_or("Missing workspace argument")?;
    let workspace_id = ParsedWorkspaceIdentifier::from_str(workspace_str)?.0;
    Ok(DispatchType::MoveToWorkspace(workspace_id, None))
}

fn build_move_to_workspace_silent(args: Vec<String>) -> Result<DispatchType<'static>, String> {
    let workspace_str = args.first().ok_or("Missing workspace argument")?;
    let workspace_id = ParsedWorkspaceIdentifier::from_str(workspace_str)?.0;
    let window_id = if let Some(window_str) = args.get(1) {
        Some(ParsedWindowIdentifier::from_str(window_str)?.0)
    } else {
        None
    };
    Ok(DispatchType::MoveToWorkspaceSilent(workspace_id, window_id))
}

dispatcher_one_arg!(
    build_workspace,
    Workspace,
    ParsedWorkspaceIdentifier,
    "Missing workspace argument"
);
dispatcher_one_arg_optional_default!(
    build_cycle_window,
    CycleWindow,
    ParsedCycleDirection,
    "next"
);
dispatcher_one_arg!(
    build_move_focus,
    MoveFocus,
    ParsedDirection,
    "Missing direction argument"
);
dispatcher_one_arg!(
    build_swap_window,
    SwapWindow,
    ParsedDirection,
    "Missing direction argument"
);
dispatcher_one_arg!(
    build_focus_window,
    FocusWindow,
    ParsedWindowIdentifier,
    "Missing window identifier"
);
dispatcher_one_arg!(
    build_move_window,
    MoveWindow,
    ParsedWindowMove,
    "Missing target argument"
);

dispatcher_no_args!(build_toggle_fake_fullscreen, ToggleFakeFullscreen);
dispatcher_no_args!(build_toggle_pseudo, TogglePseudo);
dispatcher_no_args!(build_toggle_pin, TogglePin);
dispatcher_no_args!(build_center_window, CenterWindow);
dispatcher_no_args!(build_bring_active_to_top, BringActiveToTop);
dispatcher_no_args!(build_focus_urgent_or_last, FocusUrgentOrLast);
dispatcher_no_args!(build_focus_current_or_last, FocusCurrentOrLast);
dispatcher_no_args!(build_force_renderer_reload, ForceRendererReload);
dispatcher_no_args!(build_exit, Exit);

fn build_resize_active(args: Vec<String>) -> Result<DispatchType<'static>, String> {
    if args.is_empty() {
        return Err("resizeactive requires arguments".to_string());
    }
    let params = if args[0] == "exact" {
        ResizeCmd::Exact {
            width: args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            height: args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
        }
    } else {
        ResizeCmd::Delta {
            dx: args.first().and_then(|s| s.parse().ok()).unwrap_or(0),
            dy: args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
        }
    };
    let position = match params {
        ResizeCmd::Delta { dx, dy } => Position::Delta(dx, dy),
        ResizeCmd::Exact { width, height } => Position::Exact(width, height),
    };
    Ok(DispatchType::ResizeActive(position))
}

fn build_resize_window_pixel(args: Vec<String>) -> Result<DispatchType<'static>, String> {
    if args.is_empty() {
        return Err("resizewindowpixel requires arguments".to_string());
    }
    let (params, window_str) = if args[0] == "exact" {
        (
            ResizeCmd::Exact {
                width: args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
                height: args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
            },
            args.get(3).ok_or("Missing window identifier")?,
        )
    } else {
        (
            ResizeCmd::Delta {
                dx: args.first().and_then(|s| s.parse().ok()).unwrap_or(0),
                dy: args.get(1).and_then(|s| s.parse().ok()).unwrap_or(0),
            },
            args.get(2).ok_or("Missing window identifier")?,
        )
    };
    let position = match params {
        ResizeCmd::Delta { dx, dy } => Position::Delta(dx, dy),
        ResizeCmd::Exact { width, height } => Position::Exact(width, height),
    };
    let win_id = ParsedWindowIdentifier::from_str(window_str)?.0;
    Ok(DispatchType::ResizeWindowPixel(position, win_id))
}

static DISPATCHERS: phf::Map<&'static str, DispatcherBuilder> = phf_map! {
    "exec" => build_exec,
    "kill-active-window" => build_kill_active_window,
    "toggle-floating" => build_toggle_floating,
    "toggle-split" => build_toggle_split,
    "toggle-opaque" => build_toggle_opaque,
    "move-cursor-to-corner" => build_move_cursor_to_corner,
    "move-cursor" => build_move_cursor,
    "toggle-fullscreen" => build_toggle_fullscreen,
    "move-to-workspace" => build_move_to_workspace,
    "move-to-workspace-silent" => build_move_to_workspace_silent,
    "workspace" => build_workspace,
    "cycle-window" => build_cycle_window,
    "move-focus" => build_move_focus,
    "swap-window" => build_swap_window,
    "focus-window" => build_focus_window,
    "move-window" => build_move_window,
    "toggle-fake-fullscreen" => build_toggle_fake_fullscreen,
    "toggle-pseudo" => build_toggle_pseudo,
    "toggle-pin" => build_toggle_pin,
    "center-window" => build_center_window,
    "bring-active-to-top" => build_bring_active_to_top,
    "focus-urgent-or-last" => build_focus_urgent_or_last,
    "focus-current-or-last" => build_focus_current_or_last,
    "force-renderer-reload" => build_force_renderer_reload,
    "exit" => build_exit,
    "resize-active" => build_resize_active,
    "resize-window-pixel" => build_resize_window_pixel,
};


pub fn build_dispatch_cmd(
    dispatcher: &str,
    args: &[String],
) -> Result<DispatchType<'static>, String> {
    let args_owned = args.iter().map(|s| s.to_string()).collect();
    DISPATCHERS
        .get(dispatcher)
        .ok_or_else(|| format!("Unknown dispatcher: {dispatcher}"))
        .and_then(|builder| builder(args_owned))
}

impl TryFrom<DispatchCmd> for DispatchType<'static> {
    type Error = String;

    fn try_from(command: DispatchCmd) -> Result<Self, Self::Error> {
        match command {
            DispatchCmd::Exec { command } => {
                let command = command.join(" ");
                let command_static = Box::leak(command.into_boxed_str());
                Ok(DispatchType::Exec(command_static))
            }
            DispatchCmd::KillActiveWindow => Ok(DispatchType::KillActiveWindow),
            DispatchCmd::ToggleFloating { window } => {
                let window_id = if let Some(window_str) = window.to_identifier_string() {
                    Some(ParsedWindowIdentifier::from_str(&window_str)?.0)
                } else {
                    None
                };
                Ok(DispatchType::ToggleFloating(window_id))
            }
            DispatchCmd::ToggleSplit => Ok(DispatchType::ToggleSplit),
            DispatchCmd::ToggleOpaque => Ok(DispatchType::ToggleOpaque),
            DispatchCmd::MoveCursorToCorner { corner } => {
                let corner = ParsedCorner::from_str(&corner)?.0;
                Ok(DispatchType::MoveCursorToCorner(corner))
            }
            DispatchCmd::MoveCursor { x, y } => Ok(DispatchType::MoveCursor(x, y)),
            DispatchCmd::ToggleFullscreen { mode } => {
                let mode = ParsedFullscreenType::from_str(&mode)?.0;
                Ok(DispatchType::ToggleFullscreen(mode))
            }
            DispatchCmd::MoveToWorkspace { workspace } => {
                let workspace_id = ParsedWorkspaceIdentifier::from_str(&workspace)?.0;
                Ok(DispatchType::MoveToWorkspace(workspace_id, None))
            }
            DispatchCmd::MoveToWorkspaceSilent { workspace, window } => {
                let workspace_id = ParsedWorkspaceIdentifier::from_str(&workspace)?.0;
                let window_id = if let Some(window_str) = window.to_identifier_string() {
                    Some(ParsedWindowIdentifier::from_str(&window_str)?.0)
                } else {
                    None
                };
                Ok(DispatchType::MoveToWorkspaceSilent(workspace_id, window_id))
            }
            DispatchCmd::Workspace { workspace } => {
                let workspace_id = ParsedWorkspaceIdentifier::from_str(&workspace)?.0;
                Ok(DispatchType::Workspace(workspace_id))
            }
            DispatchCmd::CycleWindow { direction } => {
                let dir = ParsedCycleDirection::from_str(&direction)?.0;
                Ok(DispatchType::CycleWindow(dir))
            }
            DispatchCmd::MoveFocus { direction } => {
                let dir = ParsedDirection::from_str(&direction)?.0;
                Ok(DispatchType::MoveFocus(dir))
            }
            DispatchCmd::SwapWindow { direction } => {
                let dir = ParsedDirection::from_str(&direction)?.0;
                Ok(DispatchType::SwapWindow(dir))
            }
            DispatchCmd::FocusWindow { window } => {
                let window_id = window.to_identifier_string().ok_or("Missing window identifier")?;
                let window_id = ParsedWindowIdentifier::from_str(&window_id)?.0;
                Ok(DispatchType::FocusWindow(window_id))
            }
            DispatchCmd::MoveWindow { target } => {
                let window_move = ParsedWindowMove::from_str(&target)?.0;
                Ok(DispatchType::MoveWindow(window_move))
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
            DispatchCmd::ResizeActive { params } => {
                let position = match params {
                    ResizeCmd::Delta { dx, dy } => Position::Delta(dx, dy),
                    ResizeCmd::Exact { width, height } => Position::Exact(width, height),
                };
                Ok(DispatchType::ResizeActive(position))
            }
            DispatchCmd::ResizeWindowPixel { params, window } => {
                let position = match params {
                    ResizeCmd::Delta { dx, dy } => Position::Delta(dx, dy),
                    ResizeCmd::Exact { width, height } => Position::Exact(width, height),
                };
                let win_id_str = window.to_identifier_string().ok_or("Missing window identifier")?;
                let win_id = ParsedWindowIdentifier::from_str(&win_id_str)?.0;
                Ok(DispatchType::ResizeWindowPixel(position, win_id))
            }
        }
    }
}

pub fn handle_dispatch(command: DispatchCmd, is_async: bool) {
    match DispatchType::try_from(command) {
        Ok(dispatch_type) => {
            if is_async {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(async {
                    if let Err(e) = Dispatch::call_async(dispatch_type).await {
                        eprintln!("Error: {e}");
                    }
                });
            } else if let Err(e) = Dispatch::call(dispatch_type) {
                eprintln!("Error: {e}");
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
        }
    }
}
