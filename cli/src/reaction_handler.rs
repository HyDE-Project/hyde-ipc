use crate::dispatch::handle_dispatch;
use crate::flags::{Dispatch, ResizeCmd, WindowId};
use crate::parsers::ParsedWindowIdentifier;
use hyprland::dispatch::WindowIdentifier;
use hyprland::event_listener::EventListener;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// --- Event Type Enums ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WindowEventType {
    #[serde(alias = "Opened")]
    Opened,
    #[serde(alias = "Closed")]
    Closed,
    #[serde(alias = "Moved")]
    Moved,
    #[serde(alias = "Active")]
    Active,
}

impl fmt::Display for WindowEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WindowEventType::Opened => write!(f, "opened"),
            WindowEventType::Closed => write!(f, "closed"),
            WindowEventType::Moved => write!(f, "moved"),
            WindowEventType::Active => write!(f, "active"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkspaceEventType {
    #[serde(alias = "Changed")]
    Changed,
    #[serde(alias = "Added")]
    Added,
    #[serde(alias = "Deleted")]
    Deleted,
}

impl fmt::Display for WorkspaceEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkspaceEventType::Changed => write!(f, "changed"),
            WorkspaceEventType::Added => write!(f, "added"),
            WorkspaceEventType::Deleted => write!(f, "deleted"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GroupEventType {
    #[serde(alias = "Toggled")]
    Toggled,
    #[serde(alias = "MovedIn")]
    MovedIn,
    #[serde(alias = "MovedOut")]
    MovedOut,
}

impl fmt::Display for GroupEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupEventType::Toggled => write!(f, "toggled"),
            GroupEventType::MovedIn => write!(f, "moved-in"),
            GroupEventType::MovedOut => write!(f, "moved-out"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    Window(WindowEventType),
    Workspace(WorkspaceEventType),
    Monitor,
    Float,
    Fullscreen,
    Layout,
    Group(GroupEventType),
    Config,
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EventTypeVisitor;

        impl<'de> Visitor<'de> for EventTypeVisitor {
            type Value = EventType;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or a map for event type")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.to_lowercase().as_str() {
                    "monitor" => Ok(EventType::Monitor),
                    "float" => Ok(EventType::Float),
                    "fullscreen" => Ok(EventType::Fullscreen),
                    "layout" => Ok(EventType::Layout),
                    "config" => Ok(EventType::Config),
                    _ => Err(de::Error::unknown_variant(value, &[
                        "monitor",
                        "float",
                        "fullscreen",
                        "layout",
                        "config",
                    ])),
                }
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let key: String = map.next_key()?.ok_or_else(|| {
                    de::Error::invalid_length(0, &"a map with one key-value pair")
                })?;

                match key.to_lowercase().as_str() {
                    "window" => {
                        let subtype: WindowEventType = map.next_value()?;
                        Ok(EventType::Window(subtype))
                    },
                    "workspace" => {
                        let subtype: WorkspaceEventType = map.next_value()?;
                        Ok(EventType::Workspace(subtype))
                    },
                    "group" => {
                        let subtype: GroupEventType = map.next_value()?;
                        Ok(EventType::Group(subtype))
                    },
                    _ => Err(de::Error::unknown_field(&key, &["window", "workspace", "group"])),
                }
            }
        }

        deserializer.deserialize_any(EventTypeVisitor)
    }
}

impl EventType {
    pub fn from_event_and_subtype(event: &str, subtype: Option<&str>) -> Result<Self, String> {
        match event.to_lowercase().as_str() {
            "window" => {
                let subtype = subtype.ok_or("Window event requires a subtype")?;
                let window_event_type = match subtype.to_lowercase().as_str() {
                    "opened" => WindowEventType::Opened,
                    "closed" => WindowEventType::Closed,
                    "moved" => WindowEventType::Moved,
                    "active" => WindowEventType::Active,
                    _ => return Err(format!("Unknown window subtype: {subtype}")),
                };
                Ok(EventType::Window(window_event_type))
            },
            "workspace" => {
                let subtype = subtype.ok_or("Workspace event requires a subtype")?;
                let workspace_event_type = match subtype.to_lowercase().as_str() {
                    "changed" => WorkspaceEventType::Changed,
                    "added" => WorkspaceEventType::Added,
                    "deleted" => WorkspaceEventType::Deleted,
                    _ => return Err(format!("Unknown workspace subtype: {subtype}")),
                };
                Ok(EventType::Workspace(workspace_event_type))
            },
            "monitor" => Ok(EventType::Monitor),
            "float" => Ok(EventType::Float),
            "fullscreen" => Ok(EventType::Fullscreen),
            "layout" => Ok(EventType::Layout),
            "group" => {
                let subtype = subtype.ok_or("Group event requires a subtype")?;
                let group_event_type = match subtype.to_lowercase().as_str() {
                    "toggled" => GroupEventType::Toggled,
                    "moved-in" => GroupEventType::MovedIn,
                    "moved-out" => GroupEventType::MovedOut,
                    _ => return Err(format!("Unknown group subtype: {subtype}")),
                };
                Ok(EventType::Group(group_event_type))
            },
            "config" => Ok(EventType::Config),
            _ => Err(format!("Unknown event type: {event}")),
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::Window(subtype) => write!(f, "window {subtype}"),
            EventType::Workspace(subtype) => write!(f, "workspace {subtype}"),
            EventType::Monitor => write!(f, "monitor"),
            EventType::Float => write!(f, "float"),
            EventType::Fullscreen => write!(f, "fullscreen"),
            EventType::Layout => write!(f, "layout"),
            EventType::Group(subtype) => write!(f, "group {subtype}"),
            EventType::Config => write!(f, "config"),
        }
    }
}

/// A reaction to a Hyprland event, which can dispatch one or more commands when triggered.
#[derive(Debug, Clone, Deserialize)]
pub struct Reaction {
    pub event_type: EventType,
    #[serde(default)]
    pub dispatchers: Vec<Dispatcher>,
    #[serde(
        default,
        deserialize_with = "deserialize_window_identifier"
    )]
    pub window_filter: Option<WindowIdentifier<'static>>,
    #[serde(default)]
    pub max_count: Option<usize>,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(skip)]
    pub counter: Arc<AtomicUsize>,
}

impl Reaction {
    /// Execute this reaction and all chained dispatchers.
    pub fn execute(&self) -> Result<bool, String> {
        let max_count = self.max_count.unwrap_or(0);
        if max_count > 0 {
            let current = self
                .counter
                .fetch_add(1, Ordering::SeqCst)
                + 1;
            if current > max_count {
                println!("Reached maximum reaction count ({max_count})");
                return Ok(false);
            }
        }

        if self.dispatchers.is_empty() {
            return Err("No dispatchers defined for this reaction".to_string());
        }

        let reaction_name = self
            .name
            .as_deref()
            .unwrap_or("unnamed");
        println!(
            "Executing reaction '{reaction_name}' for event '{}': {} dispatchers",
            self.event_type,
            self.dispatchers.len()
        );

        for (index, dispatcher) in self.dispatchers.iter().enumerate() {
            println!("  - Dispatcher {}/{}: {:?}", index + 1, self.dispatchers.len(), dispatcher);
            handle_dispatch(dispatcher.clone().into(), false);
        }
        Ok(true)
    }
}

// --- Deserialization Logic ---
pub fn deserialize_window_identifier<'de, D>(
    deserializer: D,
) -> Result<Option<WindowIdentifier<'static>>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| {
        ParsedWindowIdentifier::from_str(&s)
            .map(|p| p.0)
            .map_err(de::Error::custom)
    })
    .transpose()
}

// --- Reaction Manager ---
#[derive(Default, Debug)]
pub struct ReactionManager {
    reactions: Vec<Arc<Reaction>>,
}

impl ReactionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_reaction(&mut self, reaction: Reaction) {
        self.reactions.push(Arc::new(reaction));
    }

    pub fn start(self) -> Result<(), String> {
        println!("Starting reaction manager with {} reactions", self.reactions.len());
        let mut event_listener = EventListener::new();

        for reaction in &self.reactions {
            self.setup_handler(&mut event_listener, Arc::clone(reaction));
        }

        event_listener
            .start_listener()
            .map_err(|e| e.to_string())
    }

    fn setup_handler(&self, event_listener: &mut EventListener, reaction: Arc<Reaction>) {
        let handler_reaction = Arc::clone(&reaction);
        let handler = move || {
            if let Err(e) = handler_reaction.execute() {
                eprintln!("Error executing reaction: {e}");
            }
        };

        match reaction.event_type {
            EventType::Window(subtype) => {
                self.setup_window_handler(event_listener, subtype, reaction)
            },
            EventType::Workspace(subtype) => {
                self.setup_workspace_handler(event_listener, subtype, handler)
            },
            EventType::Monitor => {
                event_listener.add_active_monitor_changed_handler(move |_| handler())
            },
            EventType::Float => event_listener.add_float_state_changed_handler(move |_| handler()),
            EventType::Fullscreen => {
                event_listener.add_fullscreen_state_changed_handler(move |_| handler())
            },
            EventType::Layout => event_listener.add_layout_changed_handler(move |_| handler()),
            EventType::Group(subtype) => self.setup_group_handler(event_listener, subtype, handler),
            EventType::Config => event_listener.add_config_reloaded_handler(handler),
        }
    }

    fn setup_window_handler(
        &self,
        event_listener: &mut EventListener,
        subtype: WindowEventType,
        reaction: Arc<Reaction>,
    ) {
        let window_handler_reaction = Arc::clone(&reaction);
        let window_handler = move |class: &str, title: &str| {
            if is_window_match(
                window_handler_reaction
                    .window_filter
                    .as_ref(),
                class,
                title,
            ) {
                if let Err(e) = window_handler_reaction.execute() {
                    eprintln!("Error executing reaction: {e}");
                }
            }
        };

        match subtype {
            WindowEventType::Opened => {
                event_listener.add_window_opened_handler(move |data| {
                    window_handler(&data.window_class, &data.window_title);
                });
            },
            WindowEventType::Active => {
                let active_handler_reaction = Arc::clone(&reaction);
                event_listener.add_active_window_changed_handler(move |data| {
                    if let Some(win_data) = data {
                        window_handler(&win_data.class, &win_data.title);
                    } else if active_handler_reaction
                        .window_filter
                        .is_some()
                    {
                        // No window data, but filter exists, so no match
                    } else {
                        // No window data and no filter, so execute
                        if let Err(e) = active_handler_reaction.execute() {
                            eprintln!("Error executing reaction: {e}");
                        }
                    }
                });
            },
            WindowEventType::Closed => {
                let closed_handler_reaction = Arc::clone(&reaction);
                event_listener.add_window_closed_handler(move |_| {
                    if closed_handler_reaction
                        .window_filter
                        .is_some()
                    {
                        println!("Note: Window filter is not applicable to 'closed' events.");
                    }
                    if let Err(e) = closed_handler_reaction.execute() {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            WindowEventType::Moved => {
                let moved_handler_reaction = Arc::clone(&reaction);
                event_listener.add_window_moved_handler(move |_| {
                    if moved_handler_reaction
                        .window_filter
                        .is_some()
                    {
                        println!("Note: Window filter is not applicable to 'moved' events.");
                    }
                    if let Err(e) = moved_handler_reaction.execute() {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
        }
    }

    fn setup_workspace_handler(
        &self,
        event_listener: &mut EventListener,
        subtype: WorkspaceEventType,
        handler: impl Fn() + Send + Sync + 'static,
    ) {
        match subtype {
            WorkspaceEventType::Changed => {
                event_listener.add_workspace_changed_handler(move |_| handler())
            },
            WorkspaceEventType::Added => {
                event_listener.add_workspace_added_handler(move |_| handler())
            },
            WorkspaceEventType::Deleted => {
                event_listener.add_workspace_deleted_handler(move |_| handler())
            },
        }
    }

    fn setup_group_handler(
        &self,
        event_listener: &mut EventListener,
        subtype: GroupEventType,
        handler: impl Fn() + Send + Sync + 'static,
    ) {
        match subtype {
            GroupEventType::Toggled => event_listener.add_group_toggled_handler(move |_| handler()),
            GroupEventType::MovedIn => {
                event_listener.add_window_moved_into_group_handler(move |_| handler())
            },
            GroupEventType::MovedOut => {
                event_listener.add_window_moved_out_of_group_handler(move |_| handler())
            },
        }
    }
}

fn is_window_match(
    filter: Option<&WindowIdentifier>,
    window_class: &str,
    window_title: &str,
) -> bool {
    match filter {
        Some(WindowIdentifier::ClassRegularExpression(pattern)) => window_class.contains(pattern),
        Some(WindowIdentifier::Title(pattern)) => window_title.contains(pattern),
        Some(_) => false, // PID/Address matching not supported by events
        None => true,     // No filter means it's always a match
    }
}

/// A dispatcher to be executed as part of a reaction chain.
#[derive(Debug, Clone)]
pub enum Dispatcher {
    Exec(Vec<String>),
    KillActiveWindow,
    ToggleFloating(Option<WindowId>),
    ToggleSplit,
    ToggleOpaque,
    MoveCursorToCorner(String),
    MoveCursor(i64, i64),
    ToggleFullscreen(Option<String>),
    MoveToWorkspace(String),
    MoveToWorkspaceSilent(String, Option<WindowId>),
    Workspace(String),
    CycleWindow(Option<String>),
    MoveFocus(String),
    SwapWindow(String),
    FocusWindow(WindowId),
    MoveWindow(String),
    ToggleFakeFullscreen,
    TogglePseudo,
    TogglePin,
    CenterWindow,
    BringActiveToTop,
    FocusUrgentOrLast,
    FocusCurrentOrLast,
    ForceRendererReload,
    Exit,
    ResizeActive(ResizeCmd),
    ResizeWindowPixel(ResizeCmd, WindowId),
}

impl<'de> Deserialize<'de> for Dispatcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            name: String,
            #[serde(default)]
            args: Vec<String>,
        }

        let h = Helper::deserialize(deserializer)?;
        let args = &h.args;

        let get_arg = |i: usize| {
            args.get(i)
                .cloned()
                .ok_or_else(|| de::Error::invalid_length(i, &"not enough arguments"))
        };

        let parse_arg = |i: usize| {
            get_arg(i)?
                .parse()
                .map_err(de::Error::custom)
        };

        let parse_window_id = |i: usize| -> Result<WindowId, D::Error> {
            let s = get_arg(i)?;
            let parts: Vec<&str> = s.splitn(2, ':').collect();
            if parts.len() != 2 {
                return Err(de::Error::custom("invalid window identifier format"));
            }
            let mut id = WindowId::default();
            match parts[0] {
                "class" => id.class = Some(parts[1].to_string()),
                "title" => id.title = Some(parts[1].to_string()),
                "pid" => {
                    id.pid = Some(
                        parts[1]
                            .parse()
                            .map_err(de::Error::custom)?,
                    )
                },
                "address" => id.address = Some(parts[1].to_string()),
                _ => {
                    return Err(de::Error::unknown_field(parts[0], &[
                        "class", "title", "pid", "address",
                    ]));
                },
            }
            Ok(id)
        };

        match h
            .name
            .to_lowercase()
            .replace('-', "")
            .as_str()
        {
            "exec" => Ok(Dispatcher::Exec(args.clone())),
            "killactivewindow" => Ok(Dispatcher::KillActiveWindow),
            "togglefloating" => Ok(Dispatcher::ToggleFloating(
                args.first()
                    .map(|_| parse_window_id(0))
                    .transpose()?,
            )),
            "togglesplit" => Ok(Dispatcher::ToggleSplit),
            "toggleopaque" => Ok(Dispatcher::ToggleOpaque),
            "movecursortocorner" => Ok(Dispatcher::MoveCursorToCorner(get_arg(0)?)),
            "movecursor" => Ok(Dispatcher::MoveCursor(parse_arg(0)?, parse_arg(1)?)),
            "togglefullscreen" => Ok(Dispatcher::ToggleFullscreen(args.first().cloned())),
            "movetoworkspace" => Ok(Dispatcher::MoveToWorkspace(get_arg(0)?)),
            "movetoworkspacesilent" => Ok(Dispatcher::MoveToWorkspaceSilent(
                get_arg(0)?,
                args.get(1)
                    .map(|_| parse_window_id(1))
                    .transpose()?,
            )),
            "workspace" => Ok(Dispatcher::Workspace(get_arg(0)?)),
            "cyclewindow" => Ok(Dispatcher::CycleWindow(args.first().cloned())),
            "movefocus" => Ok(Dispatcher::MoveFocus(get_arg(0)?)),
            "swapwindow" => Ok(Dispatcher::SwapWindow(get_arg(0)?)),
            "focuswindow" => Ok(Dispatcher::FocusWindow(parse_window_id(0)?)),
            "movewindow" => Ok(Dispatcher::MoveWindow(get_arg(0)?)),
            "togglefakefullscreen" => Ok(Dispatcher::ToggleFakeFullscreen),
            "togglepseudo" => Ok(Dispatcher::TogglePseudo),
            "togglepin" => Ok(Dispatcher::TogglePin),
            "centerwindow" => Ok(Dispatcher::CenterWindow),
            "bringactivetotop" => Ok(Dispatcher::BringActiveToTop),
            "focusurgentorlast" => Ok(Dispatcher::FocusUrgentOrLast),
            "focuscurrentorlast" => Ok(Dispatcher::FocusCurrentOrLast),
            "forcerendererreload" => Ok(Dispatcher::ForceRendererReload),
            "exit" => Ok(Dispatcher::Exit),
            "resizeactive" => {
                let resize_type = get_arg(0)?;
                let params = match resize_type.as_str() {
                    "exact" => ResizeCmd::Exact {
                        width: parse_arg(1)? as i16,
                        height: parse_arg(2)? as i16,
                    },
                    "delta" => {
                        ResizeCmd::Delta { dx: parse_arg(1)? as i16, dy: parse_arg(2)? as i16 }
                    },
                    _ => return Err(de::Error::unknown_variant(&resize_type, &["exact", "delta"])),
                };
                Ok(Dispatcher::ResizeActive(params))
            },
            "resizewindowpixel" => {
                let resize_type = get_arg(0)?;
                let params = match resize_type.as_str() {
                    "exact" => ResizeCmd::Exact {
                        width: parse_arg(1)? as i16,
                        height: parse_arg(2)? as i16,
                    },
                    "delta" => {
                        ResizeCmd::Delta { dx: parse_arg(1)? as i16, dy: parse_arg(2)? as i16 }
                    },
                    _ => return Err(de::Error::unknown_variant(&resize_type, &["exact", "delta"])),
                };
                let window = parse_window_id(3)?;
                Ok(Dispatcher::ResizeWindowPixel(params, window))
            },
            _ => Err(de::Error::unknown_variant(&h.name, &["exec" /* ... */])),
        }
    }
}

impl From<Dispatcher> for Dispatch {
    fn from(dispatcher: Dispatcher) -> Self {
        match dispatcher {
            Dispatcher::Exec(command) => Dispatch::Exec { command },
            Dispatcher::KillActiveWindow => Dispatch::KillActiveWindow,
            Dispatcher::ToggleFloating(window) => {
                Dispatch::ToggleFloating { window: window.unwrap_or_default() }
            },
            Dispatcher::ToggleSplit => Dispatch::ToggleSplit,
            Dispatcher::ToggleOpaque => Dispatch::ToggleOpaque,
            Dispatcher::MoveCursorToCorner(corner) => Dispatch::MoveCursorToCorner { corner },
            Dispatcher::MoveCursor(x, y) => Dispatch::MoveCursor { x, y },
            Dispatcher::ToggleFullscreen(mode) => {
                Dispatch::ToggleFullscreen { mode: mode.unwrap_or_else(|| "noparam".to_string()) }
            },
            Dispatcher::MoveToWorkspace(workspace) => Dispatch::MoveToWorkspace { workspace },
            Dispatcher::MoveToWorkspaceSilent(workspace, window) => {
                Dispatch::MoveToWorkspaceSilent { workspace, window: window.unwrap_or_default() }
            },
            Dispatcher::Workspace(workspace) => Dispatch::Workspace { workspace },
            Dispatcher::CycleWindow(direction) => {
                Dispatch::CycleWindow { direction: direction.unwrap_or_else(|| "next".to_string()) }
            },
            Dispatcher::MoveFocus(direction) => Dispatch::MoveFocus { direction },
            Dispatcher::SwapWindow(direction) => Dispatch::SwapWindow { direction },
            Dispatcher::FocusWindow(window) => Dispatch::FocusWindow { window },
            Dispatcher::MoveWindow(target) => Dispatch::MoveWindow { target },
            Dispatcher::ToggleFakeFullscreen => Dispatch::ToggleFakeFullscreen,
            Dispatcher::TogglePseudo => Dispatch::TogglePseudo,
            Dispatcher::TogglePin => Dispatch::TogglePin,
            Dispatcher::CenterWindow => Dispatch::CenterWindow,
            Dispatcher::BringActiveToTop => Dispatch::BringActiveToTop,
            Dispatcher::FocusUrgentOrLast => Dispatch::FocusUrgentOrLast,
            Dispatcher::FocusCurrentOrLast => Dispatch::FocusCurrentOrLast,
            Dispatcher::ForceRendererReload => Dispatch::ForceRendererReload,
            Dispatcher::Exit => Dispatch::Exit,
            Dispatcher::ResizeActive(params) => Dispatch::ResizeActive { params },
            Dispatcher::ResizeWindowPixel(params, window) => {
                Dispatch::ResizeWindowPixel { params, window }
            },
        }
    }
}

impl From<Dispatch> for Dispatcher {
    fn from(dispatch: Dispatch) -> Self {
        match dispatch {
            Dispatch::Exec { command } => Dispatcher::Exec(command),
            Dispatch::KillActiveWindow => Dispatcher::KillActiveWindow,
            Dispatch::ToggleFloating { window } => Dispatcher::ToggleFloating(Some(window)),
            Dispatch::ToggleSplit => Dispatcher::ToggleSplit,
            Dispatch::ToggleOpaque => Dispatcher::ToggleOpaque,
            Dispatch::MoveCursorToCorner { corner } => Dispatcher::MoveCursorToCorner(corner),
            Dispatch::MoveCursor { x, y } => Dispatcher::MoveCursor(x, y),
            Dispatch::ToggleFullscreen { mode } => Dispatcher::ToggleFullscreen(Some(mode)),
            Dispatch::MoveToWorkspace { workspace } => Dispatcher::MoveToWorkspace(workspace),
            Dispatch::MoveToWorkspaceSilent { workspace, window } => {
                Dispatcher::MoveToWorkspaceSilent(workspace, Some(window))
            },
            Dispatch::Workspace { workspace } => Dispatcher::Workspace(workspace),
            Dispatch::CycleWindow { direction } => Dispatcher::CycleWindow(Some(direction)),
            Dispatch::MoveFocus { direction } => Dispatcher::MoveFocus(direction),
            Dispatch::SwapWindow { direction } => Dispatcher::SwapWindow(direction),
            Dispatch::FocusWindow { window } => Dispatcher::FocusWindow(window),
            Dispatch::MoveWindow { target } => Dispatcher::MoveWindow(target),
            Dispatch::ToggleFakeFullscreen => Dispatcher::ToggleFakeFullscreen,
            Dispatch::TogglePseudo => Dispatcher::TogglePseudo,
            Dispatch::TogglePin => Dispatcher::TogglePin,
            Dispatch::CenterWindow => Dispatcher::CenterWindow,
            Dispatch::BringActiveToTop => Dispatcher::BringActiveToTop,
            Dispatch::FocusUrgentOrLast => Dispatcher::FocusUrgentOrLast,
            Dispatch::FocusCurrentOrLast => Dispatcher::FocusCurrentOrLast,
            Dispatch::ForceRendererReload => Dispatcher::ForceRendererReload,
            Dispatch::Exit => Dispatcher::Exit,
            Dispatch::ResizeActive { params } => Dispatcher::ResizeActive(params),
            Dispatch::ResizeWindowPixel { params, window } => {
                Dispatcher::ResizeWindowPixel(params, window)
            },
        }
    }
}
