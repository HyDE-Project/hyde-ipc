use hyprland::dispatch::Dispatch;
use hyprland::event_listener::EventListener;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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
            EventType::Window(subtype) => write!(f, "window {}", subtype),
            EventType::Workspace(subtype) => write!(f, "workspace {}", subtype),
            EventType::Monitor => write!(f, "monitor"),
            EventType::Float => write!(f, "float"),
            EventType::Fullscreen => write!(f, "fullscreen"),
            EventType::Layout => write!(f, "layout"),
            EventType::Group(subtype) => write!(f, "group {}", subtype),
            EventType::Config => write!(f, "config"),
        }
    }
}

/// A reaction to a Hyprland event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    /// The event type that triggers this reaction
    pub event_type: EventType,
    /// The dispatcher to execute when the event occurs
    pub dispatcher: String,
    /// Arguments for the dispatcher
    pub args: Vec<String>,
    /// Maximum number of times this reaction should trigger (0 for unlimited)
    #[serde(default)]
    pub max_count: Option<usize>,
    /// Name of the reaction (optional)
    pub name: Option<String>,
    /// Description of what the reaction does (optional)
    pub description: Option<String>,
}

impl Reaction {
    /// Execute this reaction
    pub fn execute(&self, count: &Arc<AtomicUsize>) -> Result<bool, String> {
        // Check if we've reached the maximum count
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

        // Parse the dispatcher
        // Convert Vec<&str> to Vec<String> for compatibility with the CLI parser
        let args_as_strings: Vec<String> = self.args.to_vec();
        let dispatch_type = super::dispatch::parse_dispatcher(&self.dispatcher, &args_as_strings)?;

        println!(
            "Executing reaction for event {}: {} {:?}",
            self.event_type, self.dispatcher, self.args
        );

        // Always execute synchronously
        if let Err(e) = Dispatch::call(dispatch_type) {
            eprintln!("Error executing dispatcher: {}", e);
        }

        // Return whether we should continue (i.e., haven't reached max_count)
        if max_count > 0 && current >= max_count {
            println!("Reached maximum reaction count ({})", max_count);
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

/// Manager for handling multiple reactions
#[derive(Debug)]
pub struct ReactionManager {
    reactions: Vec<(Reaction, Arc<AtomicUsize>)>,
}

impl ReactionManager {
    /// Create a new reaction manager
    pub fn new() -> Self {
        Self {
            reactions: Vec::new(),
        }
    }

    /// Add a reaction to the manager
    pub fn add_reaction(&mut self, reaction: Reaction) {
        let counter = Arc::new(AtomicUsize::new(0));
        self.reactions.push((reaction, counter));
    }

    /// Start listening for events and executing reactions
    pub fn start(&self) -> Result<(), String> {
        println!(
            "Starting reaction manager with {} reactions",
            self.reactions.len()
        );

        let mut event_listener = EventListener::new();

        // Set up handlers for all event types
        for (reaction, counter) in &self.reactions {
            self.setup_handler(&mut event_listener, reaction, counter)?;
        }

        // Start the listener
        event_listener
            .start_listener()
            .map_err(|e| format!("{}", e))
    }

    /// Set up a handler for a specific event type
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
                event_listener.add_window_opened_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Window(WindowEventType::Closed) => {
                event_listener.add_window_closed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Window(WindowEventType::Moved) => {
                event_listener.add_window_moved_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Window(WindowEventType::Active) => {
                event_listener.add_active_window_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Workspace(WorkspaceEventType::Changed) => {
                event_listener.add_workspace_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Workspace(WorkspaceEventType::Added) => {
                event_listener.add_workspace_added_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Workspace(WorkspaceEventType::Deleted) => {
                event_listener.add_workspace_deleted_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Monitor => {
                event_listener.add_active_monitor_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Float => {
                event_listener.add_float_state_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Fullscreen => {
                event_listener.add_fullscreen_state_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Layout => {
                event_listener.add_layout_changed_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Group(GroupEventType::Toggled) => {
                event_listener.add_group_toggled_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Group(GroupEventType::MovedIn) => {
                event_listener.add_window_moved_into_group_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Group(GroupEventType::MovedOut) => {
                event_listener.add_window_moved_out_of_group_handler(move |_| {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
            EventType::Config => {
                event_listener.add_config_reloaded_handler(move || {
                    if let Err(e) = reaction_clone.execute(&counter_clone) {
                        eprintln!("Error executing reaction: {}", e);
                    }
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReactConfig {
    pub reactions: Vec<Reaction>,
}

impl ReactConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        // Determine file format based on extension
        let extension = path
            .as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        match extension.to_lowercase().as_str() {
            "json" => serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse JSON config file: {}", e)),
            "toml" => toml::from_str(&content)
                .map_err(|e| format!("Failed to parse TOML config file: {}", e)),
            _ => {
                // Try JSON first, then TOML if JSON fails
                serde_json::from_str(&content)
                    .or_else(|_| toml::from_str(&content))
                    .map_err(|e| format!("Failed to parse config file: {}", e))
            }
        }
    }

    pub fn run(&self) -> Result<(), String> {
        let mut manager = ReactionManager::new();

        for reaction in &self.reactions {
            manager.add_reaction(reaction.clone());
        }

        manager.start()
    }
}

/// Run reactions from a configuration file
pub fn run_from_config<P: AsRef<Path>>(path: P) -> Result<(), String> {
    println!("Loading reactions from {}", path.as_ref().display());

    // Load the config from the file
    let config = ReactConfig::from_file(path)?;

    println!("Loaded {} reactions", config.reactions.len());

    // Run the reactions
    config.run()
}
