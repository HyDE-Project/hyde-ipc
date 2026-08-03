use crate::flags::WindowId;
use hyprland::dispatch::{
    Corner, CycleDirection, Direction, FullscreenType, MonitorIdentifier, WindowIdentifier,
    WindowMove, WorkspaceIdentifierWithSpecial,
};
use hyprland::shared::Address;
use phf::phf_map;
use std::str::FromStr;

/// A window filter in owned form.
///
/// [`WindowIdentifier`] borrows its patterns, so a value that has to outlive the
/// string it was parsed from — a reaction filter held for as long as the daemon
/// runs — cannot store one directly. Owning the pattern here replaces what used
/// to be a leaked `Box`.
#[derive(Debug, Clone)]
pub enum WindowFilter {
    /// A pattern matched against the window class
    Class(String),
    /// A pattern matched against the window title
    Title(String),
    /// A filter the event data cannot be compared against.
    ///
    /// Window events carry a class and a title, so a `pid:` or `address:` filter
    /// never matches. Such a filter is kept rather than rejected so existing
    /// configurations still load, and the caller is expected to report it.
    Unmatchable(String),
}

/// Borrows the first field set on a window argument as a hyprland identifier.
///
/// Returns `None` when no field was given. The identifier borrows from the
/// argument, so it stays valid for as long as the parsed command does and no
/// pattern has to be leaked to reach a `'static` lifetime.
///
/// This lives here rather than on [`WindowId`] because `flags.rs` is also
/// compiled by the build script, which does not link the hyprland crate.
pub fn window_identifier(window: &WindowId) -> Option<WindowIdentifier<'_>> {
    if let Some(class) = &window.class {
        Some(WindowIdentifier::ClassRegularExpression(class))
    } else if let Some(title) = &window.title {
        Some(WindowIdentifier::Title(title))
    } else if let Some(pid) = window.pid {
        Some(WindowIdentifier::ProcessId(pid))
    } else {
        window
            .address
            .as_ref()
            .map(|address| WindowIdentifier::Address(Address::new(address)))
    }
}

impl FromStr for WindowFilter {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(class) = s.strip_prefix("class:") {
            Ok(Self::Class(class.to_string()))
        } else if let Some(title) = s.strip_prefix("title:") {
            Ok(Self::Title(title.to_string()))
        } else if let Some(pid) = s.strip_prefix("pid:") {
            pid.parse::<u32>()
                .map_err(|_| "Invalid PID".to_string())?;
            Ok(Self::Unmatchable(s.to_string()))
        } else if s.starts_with("address:") {
            Ok(Self::Unmatchable(s.to_string()))
        } else {
            Ok(Self::Class(s.to_string()))
        }
    }
}

/// Parses workspace identifiers from string format.
///
/// The parsed value borrows any workspace name from the input, so the caller
/// keeps that string alive for as long as the identifier is in use.
#[derive(Debug, Clone)]
pub struct ParsedWorkspaceIdentifier<'a>(pub WorkspaceIdentifierWithSpecial<'a>);
impl<'a> ParsedWorkspaceIdentifier<'a> {
    pub fn parse(s: &'a str) -> Result<Self, String> {
        if let Ok(id) = s.parse::<i32>() {
            if id == 0 {
                Ok(Self(WorkspaceIdentifierWithSpecial::Special(None)))
            } else {
                Ok(Self(WorkspaceIdentifierWithSpecial::Id(id)))
            }
        } else if let Some(num_str) = s.strip_prefix("right:") {
            let num = num_str
                .parse::<i32>()
                .map_err(|_| format!("Invalid number for right: {num_str}"))?;
            Ok(Self(WorkspaceIdentifierWithSpecial::Relative(num)))
        } else if let Some(num_str) = s.strip_prefix("left:") {
            let num = num_str
                .parse::<i32>()
                .map_err(|_| format!("Invalid number for left: {num_str}"))?;
            Ok(Self(WorkspaceIdentifierWithSpecial::Relative(-num)))
        } else if s == "previous" {
            Ok(Self(WorkspaceIdentifierWithSpecial::Previous))
        } else if s == "empty" {
            Ok(Self(WorkspaceIdentifierWithSpecial::Empty))
        } else if let Some(name) = s.strip_prefix("name:") {
            Ok(Self(WorkspaceIdentifierWithSpecial::Name(name)))
        } else {
            Err(format!("Unknown workspace identifier: {s}"))
        }
    }
}

static DIRECTIONS: phf::Map<&'static str, Direction> = phf_map! {
    "up" => Direction::Up,
    "down" => Direction::Down,
    "left" => Direction::Left,
    "right" => Direction::Right,
};

#[derive(Debug, Clone)]
pub struct ParsedDirection(pub Direction);
impl FromStr for ParsedDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        DIRECTIONS
            .get(s.to_lowercase().as_str())
            .cloned()
            .map(Self)
            .ok_or_else(|| format!("Unknown direction: {s}"))
    }
}

/// Parses window move targets from string format.
///
/// The parsed value borrows any monitor name from the input, so the caller keeps
/// that string alive for as long as the target is in use.
#[derive(Debug, Clone)]
pub struct ParsedWindowMove<'a>(pub WindowMove<'a>);
impl<'a> ParsedWindowMove<'a> {
    pub fn parse(s: &'a str) -> Result<Self, String> {
        if let Some(monitor_name) = s.strip_prefix("mon:") {
            Ok(Self(WindowMove::Monitor(MonitorIdentifier::Name(monitor_name))))
        } else if let Ok(monitor_id) = s.parse::<i128>() {
            Ok(Self(WindowMove::Monitor(MonitorIdentifier::Id(monitor_id))))
        } else if s.to_lowercase() == "current" {
            Ok(Self(WindowMove::Monitor(MonitorIdentifier::Current)))
        } else if let Ok(rel_num) = s.parse::<i32>() {
            Ok(Self(WindowMove::Monitor(MonitorIdentifier::Relative(rel_num))))
        } else if let Some(dir_str) = s.to_lowercase().strip_prefix("dir:") {
            let dir = dir_str.parse::<ParsedDirection>()?.0;
            Ok(Self(WindowMove::Direction(dir)))
        } else {
            Err(format!("Unknown target for MoveWindow: {s}"))
        }
    }
}

static CORNERS: phf::Map<&'static str, Corner> = phf_map! {
    "topleft" => Corner::TopLeft,
    "topright" => Corner::TopRight,
    "bottomleft" => Corner::BottomLeft,
    "bottomright" => Corner::BottomRight,
};

#[derive(Debug, Clone)]
pub struct ParsedCorner(pub Corner);
impl FromStr for ParsedCorner {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CORNERS
            .get(s.to_lowercase().as_str())
            .cloned()
            .map(Self)
            .ok_or_else(|| format!("Unknown corner: {s}"))
    }
}

static FULLSCREEN_TYPES: phf::Map<&'static str, FullscreenType> = phf_map! {
    "real" => FullscreenType::Real,
    "maximize" => FullscreenType::Maximize,
    "noparam" => FullscreenType::NoParam,
};

#[derive(Debug, Clone)]
pub struct ParsedFullscreenType(pub FullscreenType);
impl FromStr for ParsedFullscreenType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FULLSCREEN_TYPES
            .get(s.to_lowercase().as_str())
            .cloned()
            .map(Self)
            .ok_or_else(|| format!("Unknown fullscreen type: {s}"))
    }
}

static CYCLE_DIRECTIONS: phf::Map<&'static str, CycleDirection> = phf_map! {
    "next" => CycleDirection::Next,
    "previous" => CycleDirection::Previous,
};

#[derive(Debug, Clone)]
pub struct ParsedCycleDirection(pub CycleDirection);
impl FromStr for ParsedCycleDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        CYCLE_DIRECTIONS
            .get(s.to_lowercase().as_str())
            .cloned()
            .map(Self)
            .ok_or_else(|| format!("Unknown cycle direction: {s}"))
    }
}
