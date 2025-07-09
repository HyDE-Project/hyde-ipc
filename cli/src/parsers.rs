use hyprland::dispatch::{
    Corner, CycleDirection, Direction, FullscreenType, MonitorIdentifier, WindowIdentifier,
    WindowMove, WorkspaceIdentifierWithSpecial,
};
use hyprland::shared::Address;
use phf::phf_map;
use std::str::FromStr;

// Parsed Types for Clap

#[derive(Debug, Clone)]
pub struct ParsedWindowIdentifier(pub WindowIdentifier<'static>);
impl FromStr for ParsedWindowIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(class) = s.strip_prefix("class:") {
            let class_static = Box::leak(class.to_string().into_boxed_str());
            Ok(Self(WindowIdentifier::ClassRegularExpression(class_static)))
        } else if let Some(title) = s.strip_prefix("title:") {
            let title_static = Box::leak(title.to_string().into_boxed_str());
            Ok(Self(WindowIdentifier::Title(title_static)))
        } else if let Some(pid_str) = s.strip_prefix("pid:") {
            let pid = pid_str
                .parse::<u32>()
                .map_err(|_| "Invalid PID")?;
            Ok(Self(WindowIdentifier::ProcessId(pid)))
        } else if let Some(addr) = s.strip_prefix("address:") {
            Ok(Self(WindowIdentifier::Address(Address::new(addr))))
        } else {
            // Fallback for raw class name for backward compatibility
            let class_static = Box::leak(s.to_string().into_boxed_str());
            Ok(Self(WindowIdentifier::ClassRegularExpression(class_static)))
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedWorkspaceIdentifier(pub WorkspaceIdentifierWithSpecial<'static>);
impl FromStr for ParsedWorkspaceIdentifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            let name_static = Box::leak(name.to_string().into_boxed_str());
            Ok(Self(WorkspaceIdentifierWithSpecial::Name(name_static)))
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

#[derive(Debug, Clone)]
pub struct ParsedWindowMove(pub WindowMove<'static>);
impl FromStr for ParsedWindowMove {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(monitor_name) = s.strip_prefix("mon:") {
            let monitor_name_static = Box::leak(
                monitor_name
                    .to_string()
                    .into_boxed_str(),
            );
            Ok(Self(WindowMove::Monitor(MonitorIdentifier::Name(monitor_name_static))))
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
