use crate::parsers::ParsedWindowIdentifier;
use hyprland::dispatch::{Dispatch, WindowIdentifier};
use hyprland::event_listener::EventListener;
use serde::{de, Deserialize, Deserializer};
use std::path::Path;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::{fmt, fs};

// --- Event Type Enums ---
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum WindowEventType {
    Opened,
    Closed,
    Moved,
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
pub enum WorkspaceEventType {
    Changed,
    Added,
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
pub enum GroupEventType {
    Toggled,
    MovedIn,
    MovedOut,
}

impl fmt::Display for GroupEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupEventType::Toggled => write!(f, "toggled"),
            GroupEventType::MovedIn => write!(f, "moved_in"),
            GroupEventType::MovedOut => write!(f, "moved_out"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
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

// --- Main Configuration Structs ---

/// A reaction to a Hyprland event, which can dispatch one or more commands when triggered.
#[derive(Debug, Clone, Deserialize)]
pub struct Reaction {
    pub event_type: EventType,
    #[serde(default)]
    pub dispatcher: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub dispatchers: Vec<Dispatcher>,
    #[serde(default, deserialize_with = "deserialize_window_identifier")]
    pub window_filter: Option<WindowIdentifier<'static>>,
    #[serde(default)]
    pub max_count: Option<usize>,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(skip)]
    counter: Arc<AtomicUsize>,
}

/// A dispatcher to be executed as part of a reaction chain.
#[derive(Debug, Clone, Deserialize)]
pub struct Dispatcher {
    pub name: String,
    #[serde(default)]
    pub args: Vec<String>,
}

// --- Deserialization Logic ---

fn deserialize_window_identifier<'de, D>(
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

// --- Reaction Implementation ---

impl Reaction {
    /// Execute this reaction and all chained dispatchers.
    pub fn execute(&self) -> Result<bool, String> {
        let max_count = self.max_count.unwrap_or(0);
        if max_count > 0 {
            let current = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
            if current > max_count {
                println!("Reached maximum reaction count ({max_count})");
                return Ok(false);
            }
        }

        let mut all_dispatchers = Vec::new();
        if let Some(dispatcher) = &self.dispatcher {
            all_dispatchers.push(Dispatcher {
                name: dispatcher.clone(),
                args: self.args.clone(),
            });
        }
        all_dispatchers.extend(self.dispatchers.clone());

        if all_dispatchers.is_empty() {
            return Err("No dispatchers defined for this reaction".to_string());
        }

        println!(
            "Executing reaction for event '{}': {} dispatchers",
            self.event_type,
            all_dispatchers.len()
        );

        for (index, dispatcher) in all_dispatchers.iter().enumerate() {
            println!(
                "  - Dispatcher {}/{}: {} {:?}",
                index + 1,
                all_dispatchers.len(),
                dispatcher.name,
                dispatcher.args
            );
            match super::dispatch::build_dispatch_cmd(&dispatcher.name, &dispatcher.args) {
                Ok(dispatch_type) => {
                    if let Err(e) = Dispatch::call(dispatch_type) {
                        eprintln!("    Error executing dispatcher: {e}");
                    }
                }
                Err(e) => eprintln!("    Error parsing dispatcher: {e}"),
            }
        }
        Ok(true)
    }
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

        event_listener.start_listener().map_err(|e| e.to_string())
    }

    fn setup_handler(&self, event_listener: &mut EventListener, reaction: Arc<Reaction>) {
        let handler_reaction = Arc::clone(&reaction);
        let handler = move || {
            if let Err(e) = handler_reaction.execute() {
                eprintln!("Error executing reaction: {e}");
            }
        };

        match reaction.event_type {
            EventType::Window(subtype) => self.setup_window_handler(event_listener, subtype, reaction),
            EventType::Workspace(subtype) => self.setup_workspace_handler(event_listener, subtype, handler),
            EventType::Monitor => event_listener.add_active_monitor_changed_handler(move |_| handler()),
            EventType::Float => event_listener.add_float_state_changed_handler(move |_| handler()),
            EventType::Fullscreen => event_listener.add_fullscreen_state_changed_handler(move |_| handler()),
            EventType::Layout => event_listener.add_layout_changed_handler(move |_| handler()),
            EventType::Group(subtype) => self.setup_group_handler(event_listener, subtype, handler),
            EventType::Config => event_listener.add_config_reloaded_handler(handler),
        }
    }

    fn setup_window_handler(&self, event_listener: &mut EventListener, subtype: WindowEventType, reaction: Arc<Reaction>) {
        let window_handler_reaction = Arc::clone(&reaction);
        let window_handler = move |class: &str, title: &str| {
            if is_window_match(window_handler_reaction.window_filter.as_ref(), class, title) {
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
            }
            WindowEventType::Active => {
                let active_handler_reaction = Arc::clone(&reaction);
                event_listener.add_active_window_changed_handler(move |data| {
                    if let Some(win_data) = data {
                        window_handler(&win_data.class, &win_data.title);
                    } else if active_handler_reaction.window_filter.is_some() {
                        // No window data, but filter exists, so no match
                    } else {
                        // No window data and no filter, so execute
                        if let Err(e) = active_handler_reaction.execute() {
                            eprintln!("Error executing reaction: {e}");
                        }
                    }
                });
            }
            WindowEventType::Closed => {
                let closed_handler_reaction = Arc::clone(&reaction);
                event_listener.add_window_closed_handler(move |_| {
                    if closed_handler_reaction.window_filter.is_some() {
                        println!("Note: Window filter is not applicable to 'closed' events.");
                    }
                    if let Err(e) = closed_handler_reaction.execute() {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            }
            WindowEventType::Moved => {
                let moved_handler_reaction = Arc::clone(&reaction);
                event_listener.add_window_moved_handler(move |_| {
                    if moved_handler_reaction.window_filter.is_some() {
                        println!("Note: Window filter is not applicable to 'moved' events.");
                    }
                    if let Err(e) = moved_handler_reaction.execute() {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            }
        }
    }
    
    fn setup_workspace_handler(&self, event_listener: &mut EventListener, subtype: WorkspaceEventType, handler: impl Fn() + Send + Sync + 'static) {
        match subtype {
            WorkspaceEventType::Changed => event_listener.add_workspace_changed_handler(move |_| handler()),
            WorkspaceEventType::Added => event_listener.add_workspace_added_handler(move |_| handler()),
            WorkspaceEventType::Deleted => event_listener.add_workspace_deleted_handler(move |_| handler()),
        }
    }

    fn setup_group_handler(&self, event_listener: &mut EventListener, subtype: GroupEventType, handler: impl Fn() + Send + Sync + 'static) {
        match subtype {
            GroupEventType::Toggled => event_listener.add_group_toggled_handler(move |_| handler()),
            GroupEventType::MovedIn => event_listener.add_window_moved_into_group_handler(move |_| handler()),
            GroupEventType::MovedOut => event_listener.add_window_moved_out_of_group_handler(move |_| handler()),
        }
    }
}

fn is_window_match(filter: Option<&WindowIdentifier>, window_class: &str, window_title: &str) -> bool {
    match filter {
        Some(WindowIdentifier::ClassRegularExpression(pattern)) => window_class.contains(pattern),
        Some(WindowIdentifier::Title(pattern)) => window_title.contains(pattern),
        Some(_) => false, // PID/Address matching not supported by events
        None => true,     // No filter means it's always a match
    }
}

// --- Config File Handling ---

#[derive(Debug, Deserialize)]
pub struct ReactConfig {
    #[serde(rename = "reactions")]
    pub reactions_config: Vec<ReactionConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ReactionConfig {
    #[serde(flatten)]
    reaction: Reaction,
}

impl ReactConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref()).map_err(|e| format!("Failed to read config file: {e}"))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse TOML config file: {e}"))
    }

    pub fn into_manager(self) -> ReactionManager {
        let mut manager = ReactionManager::new();
        for config in self.reactions_config {
            manager.add_reaction(Reaction {
                counter: Arc::new(AtomicUsize::new(0)),
                ..config.reaction
            });
        }
        manager
    }
}

pub fn run_from_config<P: AsRef<Path>>(path: P) -> Result<(), String> {
    println!("Loading reactions from {}", path.as_ref().display());
    let config = ReactConfig::from_file(path)?;
    println!("Loaded {} reactions", config.reactions_config.len());
    let manager = config.into_manager();
    manager.start()
}
