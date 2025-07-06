use hyprland::dispatch::Dispatch;
use hyprland::event_listener::EventListener;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{fmt, fs};

/// Types of window events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WindowEventType {
    /// Window opened event
    Opened,
    /// Window closed event
    Closed,
    /// Window moved event
    Moved,
    /// Active window changed event
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

/// Types of workspace events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkspaceEventType {
    /// Workspace changed event
    Changed,
    /// Workspace added event
    Added,
    /// Workspace deleted event
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

/// Types of group events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroupEventType {
    /// Group toggled event
    Toggled,
    /// Window moved into group event
    MovedIn,
    /// Window moved out of group event
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

/// Types of events that can trigger reactions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Window-related events
    Window(WindowEventType),
    /// Workspace-related events
    Workspace(WorkspaceEventType),
    /// Monitor-related events
    Monitor,
    /// Float state change events
    Float,
    /// Fullscreen state change events
    Fullscreen,
    /// Layout change events
    Layout,
    /// Group-related events
    Group(GroupEventType),
    /// Config reload events
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

/// A reaction to a Hyprland event, which can dispatch one or more commands when triggered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    /// The event type that triggers this reaction
    pub event_type: EventType,
    /// The dispatcher to execute when the event occurs (legacy field)
    #[serde(default)]
    pub dispatcher: Option<String>,
    /// Arguments for the dispatcher (legacy field)
    #[serde(default)]
    pub args: Vec<String>,
    /// Sequence of dispatchers to execute
    #[serde(default)]
    pub dispatchers: Vec<Dispatcher>,
    /// Window filter (e.g., "title:Google Chrome") for window events
    #[serde(default)]
    pub window_filter: Option<String>,
    /// Maximum number of times this reaction should trigger (0 for unlimited)
    #[serde(default)]
    pub max_count: Option<usize>,
    /// Name of the reaction (optional)
    pub name: Option<String>,
    /// Description of what the reaction does (optional)
    pub description: Option<String>,
}

/// A dispatcher to be executed as part of a reaction chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dispatcher {
    /// The dispatcher name
    pub name: String,
    /// Arguments for the dispatcher
    #[serde(default)]
    pub args: Vec<String>,
}

impl Reaction {
    /// Execute this reaction and all chained dispatchers.
    ///
    /// # Arguments
    /// * `count` - Shared counter for limiting reactions.
    ///
    /// # Returns
    /// * `Ok(true)` if the reaction should continue.
    /// * `Ok(false)` if the max count was reached.
    /// * `Err(String)` if a dispatcher fails to parse.
    pub fn execute(&self, count: &Arc<AtomicUsize>) -> Result<bool, String> {
        let max_count = self.max_count.unwrap_or(0);
        let current = if max_count > 0 {
            let current = count.fetch_add(1, Ordering::SeqCst) + 1;
            if current > max_count {
                return Ok(false);
            }
            current
        } else {
            0
        };

        let mut all_dispatchers = Vec::new();

        if let Some(dispatcher) = &self.dispatcher {
            all_dispatchers.push(Dispatcher { name: dispatcher.clone(), args: self.args.clone() });
        }

        all_dispatchers.extend(self.dispatchers.clone());

        if all_dispatchers.is_empty() {
            return Err("No dispatchers defined for this reaction".to_string());
        }

        println!(
            "Executing reaction for event {}: {} dispatchers",
            self.event_type,
            all_dispatchers.len()
        );

        for (index, dispatcher) in all_dispatchers.iter().enumerate() {
            println!(
                "Executing dispatcher {}/{}: {} {:?}",
                index + 1,
                all_dispatchers.len(),
                dispatcher.name,
                dispatcher.args
            );

            match super::dispatch::build_dispatch_cmd(&dispatcher.name, &dispatcher.args) {
                Ok(dispatch_type) => {
                    if let Err(e) = Dispatch::call(dispatch_type) {
                        eprintln!("Error executing dispatcher: {e}");
                    }
                },
                Err(e) => {
                    eprintln!("Error parsing dispatcher: {e}");
                },
            }
        }

        if max_count > 0 && current >= max_count {
            println!("Reached maximum reaction count ({max_count})");
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

/// Manager for handling multiple reactions and event listeners.
#[derive(Debug)]
pub struct ReactionManager {
    reactions: Vec<(Reaction, Arc<AtomicUsize>)>,
}

impl ReactionManager {
    /// Create a new reaction manager.
    pub fn new() -> Self {
        Self { reactions: Vec::new() }
    }

    /// Add a reaction to the manager.
    pub fn add_reaction(&mut self, reaction: Reaction) {
        let counter = Arc::new(AtomicUsize::new(0));
        self.reactions.push((reaction, counter));
    }

    /// Start listening for events and executing reactions.
    pub fn start(&self) -> Result<(), String> {
        println!("Starting reaction manager with {} reactions", self.reactions.len());

        let mut event_listener = EventListener::new();

        for (reaction, counter) in &self.reactions {
            self.setup_handler(&mut event_listener, reaction, counter)?;
        }

        event_listener
            .start_listener()
            .map_err(|e| format!("{e}"))
    }

    /// Set up a handler for a specific event type.
    fn setup_handler(
        &self,
        event_listener: &mut EventListener,
        reaction: &Reaction,
        counter: &Arc<AtomicUsize>,
    ) -> Result<(), String> {
        let reaction_clone = reaction.clone();
        let counter_clone = Arc::clone(counter);

        match &reaction.event_type {
            EventType::Window(WindowEventType::Opened) => {
                event_listener.add_window_opened_handler(move |data| {
                    if let Some(filter) = &reaction_clone.window_filter {
                        if let Some(title_pattern) = filter.strip_prefix("title:") {
                            if !data
                                .window_title
                                .contains(title_pattern)
                            {
                                return;
                            }
                        } else if let Some(class_pattern) = filter.strip_prefix("class:") {
                            if !data
                                .window_class
                                .contains(class_pattern)
                            {
                                return;
                            }
                        }
                    }

                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Window(WindowEventType::Closed) => {
                let has_window_filter = reaction.window_filter.is_some();
                event_listener.add_window_closed_handler(move |_address| {
                    if has_window_filter {
                        println!("Note: Window filter ignored for closed events");
                    }

                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Window(WindowEventType::Moved) => {
                let has_window_filter = reaction.window_filter.is_some();
                event_listener.add_window_moved_handler(move |_move_data| {
                    if has_window_filter {
                        println!("Note: Window filter ignored for move events");
                    }

                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Window(WindowEventType::Active) => {
                event_listener.add_active_window_changed_handler(move |data| {
                    if let Some(window_data) = data.as_ref() {
                        if let Some(filter) = &reaction_clone.window_filter {
                            if let Some(title_pattern) = filter.strip_prefix("title:") {
                                if !window_data
                                    .title
                                    .contains(title_pattern)
                                {
                                    return;
                                }
                            } else if let Some(class_pattern) = filter.strip_prefix("class:") {
                                if !window_data
                                    .class
                                    .contains(class_pattern)
                                {
                                    return;
                                }
                            }
                        }
                    } else if reaction_clone.window_filter.is_some() {
                        return;
                    }

                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Workspace(WorkspaceEventType::Changed) => {
                event_listener.add_workspace_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Workspace(WorkspaceEventType::Added) => {
                event_listener.add_workspace_added_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Workspace(WorkspaceEventType::Deleted) => {
                event_listener.add_workspace_deleted_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Monitor => {
                event_listener.add_active_monitor_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Float => {
                event_listener.add_float_state_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Fullscreen => {
                event_listener.add_fullscreen_state_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Layout => {
                event_listener.add_layout_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Group(GroupEventType::Toggled) => {
                event_listener.add_group_toggled_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Group(GroupEventType::MovedIn) => {
                event_listener.add_window_moved_into_group_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Group(GroupEventType::MovedOut) => {
                event_listener.add_window_moved_out_of_group_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
            EventType::Config => {
                event_listener.add_config_reloaded_handler(move || {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {e}");
                    }
                });
            },
        }

        Ok(())
    }
}

/// A configuration file containing multiple reactions.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReactConfig {
    /// List of reactions to run
    pub reactions: Vec<Reaction>,
}

impl ReactConfig {
    /// Load a ReactConfig from a TOML file.
    /// ```bash
    /// hyde-ipc global path/to/config.toml
    /// ````
    /// > [!WARN]
    /// > make sure the `hyde-ipc.service` is running
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {e}"))?;

        toml::from_str(&content).map_err(|e| format!("Failed to parse TOML config file: {e}"))
    }

    /// Run all reactions in this config.
    pub fn run(&self) -> Result<(), String> {
        let mut manager = ReactionManager::new();

        for reaction in &self.reactions {
            manager.add_reaction(reaction.clone());
        }

        manager.start()
    }
}

/// Run reactions from a configuration file.
pub fn run_from_config<P: AsRef<Path>>(path: P) -> Result<(), String> {
    println!("Loading reactions from {}", path.as_ref().display());

    let config = ReactConfig::from_file(path)?;

    println!("Loaded {} reactions", config.reactions.len());

    config.run()
}
