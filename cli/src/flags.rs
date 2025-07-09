//! CLI argument definitions for hyde-ipc.
//!
//! This module defines the command-line interface using clap, including all subcommands and their
//! options.

use clap::{ArgGroup, Parser, Subcommand};

/// Command-line interface for hyde-ipc.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    /// The subcommand to execute.
    pub command: Commands,
}

/// All supported subcommands for hyde-ipc.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Get or set a keyword value.
    #[command(group(
        ArgGroup::new("action")
            .required(true)
            .args(["get", "set"]),
    ))]
    Keyword {
        /// Use async mode
        #[arg(short = 'a', long = "async")]
        r#async: bool,

        /// Get the value of a keyword
        #[arg(
            short = 'g',
            long = "get",
            group = "action"
        )]
        get: bool,

        /// Set the value of a keyword
        #[arg(
            short = 's',
            long = "set",
            group = "action"
        )]
        set: bool,

        /// The keyword to get or set
        keyword: String,

        /// The value to set
        value: Option<String>,
    },

    /// Execute a dispatcher command.
    Dispatch(DispatchCommand),

    /// Listen for and log Hyprland events.
    Listen {
        /// Filter events by type (e.g., "window", "workspace")
        #[arg(short = 'f', long = "filter")]
        filter: Option<String>,

        /// Maximum number of events to log (0 for unlimited)
        #[arg(
            short = 'n',
            long = "max-events",
            default_value = "0"
        )]
        max_events: usize,
    },

    /// React to specific events by dispatching commands.
    #[command(group(
        ArgGroup::new("mode")
            .required(false)
            .args(["config", "inline"]),
    ))]
    React {
        /// Use a config file to define multiple reactions
        #[arg(
            short = 'c',
            long = "config",
            group = "mode"
        )]
        config: Option<String>,

        /// Use inline mode (single reaction)
        #[arg(
            short = 'i',
            long = "inline",
            group = "mode"
        )]
        inline: bool,

        /// Event type to react to (e.g., "window", "workspace")
        #[arg(
            short = 'e',
            long = "event",
            required_unless_present = "config"
        )]
        event: Option<String>,

        /// Event subtype for more specific filtering (e.g., "opened" for window events)
        #[arg(short = 's', long = "subtype")]
        subtype: Option<String>,

        /// Window filter for window events (e.g., "title:Google Chrome" or "class:firefox")
        #[arg(short = 'f', long = "filter")]
        filter: Option<String>,

        /// Dispatcher command to execute when the event occurs
        #[command(subcommand)]
        dispatch: Option<Dispatch>,

        /// Limit number of reactions (0 for unlimited)
        #[arg(
            short = 'n',
            long = "max-reactions",
            default_value = "0"
        )]
        max_reactions: usize,
    },

    /// Manage the hyde-ipc user service.
    Setup(SetupCommand),

    /// Load a config file for global reactions.
    Global {
        /// Path to the config file to load.
        #[arg(short = 'c', long)]
        config_path: String,
    },

    /// Query Hyprland for information.
    Query(QueryCommand),
}

#[derive(Parser, Debug, Clone)]
#[command(group(
    ArgGroup::new("action")
        .required(true)
        .args(["install", "uninstall", "start", "kill", "restart", "check", "watch"]),
))]
pub struct SetupCommand {
    /// Install the user service.
    #[arg(long)]
    pub install: bool,

    /// Uninstall the user service.
    #[arg(long)]
    pub uninstall: bool,

    /// Start the user service.
    #[arg(short = 's', long)]
    pub start: bool,

    /// Stop (kill) the user service.
    #[arg(short = 'k', long)]
    pub kill: bool,

    /// Restart the user service.
    #[arg(long)]
    pub restart: bool,

    /// Check the status of the user service.
    #[arg(short = 'c', long)]
    pub check: bool,

    /// Watch the logs of the user service.
    #[arg(short = 'w', long)]
    pub watch: bool,
}

#[derive(Parser, Debug, Clone)]
pub struct QueryCommand {
    #[command(subcommand)]
    pub command: Query,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Query {
    /// Get the current cursor position.
    CursorPos {
        /// Watch for changes and log the position continuously.
        #[arg(short = 'w', long = "watch")]
        watch: bool,
    },
}

#[derive(Parser, Debug, Clone)]
#[command(
    help_template = "{before-help}{about-with-newline}{usage-heading}{usage}{options-heading}{options}"
)]
pub struct DispatchCommand {
    /// Use async mode
    #[arg(short = 'a', long = "async")]
    pub r#async: bool,

    /// List available dispatchers
    #[arg(short = 'l', long = "list-dispatchers")]
    pub list_dispatchers: bool,

    #[command(subcommand)]
    pub command: Option<Dispatch>,
}

#[derive(clap::Args, Debug, Clone, Default)]
pub struct WindowId {
    #[arg(long, group = "winid")]
    pub class: Option<String>,
    #[arg(long, group = "winid")]
    pub title: Option<String>,
    #[arg(long, group = "winid")]
    pub pid: Option<u32>,
    #[arg(long, group = "winid")]
    pub address: Option<String>,
}

impl WindowId {
    pub fn to_identifier_string(&self) -> Option<String> {
        if let Some(class) = &self.class {
            Some(format!("class:{class}"))
        } else if let Some(title) = &self.title {
            Some(format!("title:{title}"))
        } else if let Some(pid) = self.pid {
            Some(format!("pid:{pid}"))
        } else if let Some(address) = &self.address {
            Some(format!("address:{address}"))
        } else {
            None
        }
    }
}

#[derive(Subcommand, Debug, Clone)]
pub enum ResizeCmd {
    Delta { dx: i16, dy: i16 },
    Exact { width: i16, height: i16 },
}

#[derive(Subcommand, Debug, Clone)]
pub enum Dispatch {
    /// Execute a command
    Exec { command: Vec<String> },
    /// Kill the active window
    KillActiveWindow,
    /// Toggle floating mode for a window
    #[command(group(ArgGroup::new("winid_toggle_floating").args(&["class", "title", "pid", "address"])))]
    ToggleFloating {
        #[command(flatten)]
        window: WindowId,
    },
    /// Toggle the split orientation
    ToggleSplit,
    /// Toggle opacity for the active window
    ToggleOpaque,
    /// Move cursor to a corner
    MoveCursorToCorner { corner: String },
    /// Move cursor to a specific position
    MoveCursor {
        #[arg()]
        x: i64,
        #[arg()]
        y: i64,
    },
    /// Toggle fullscreen mode
    ToggleFullscreen { mode: String },
    /// Move window to workspace. Accepts a workspace number (e.g., '5'),
    /// relative movement (e.g., 'right:2', 'left:1'), or special keywords
    /// ('previous', 'empty', 'name:<name>').
    MoveToWorkspace { workspace: String },
    /// Move window to workspace silently. Accepts a workspace number (e.g., '5'),
    /// relative movement (e.g., 'right:2', 'left:1'), or special keywords
    /// ('previous', 'empty', 'name:<name>').
    #[command(group(ArgGroup::new("winid_movetoworkspacesilent").args(&["class", "title", "pid", "address"])))]
    MoveToWorkspaceSilent {
        workspace: String,
        #[command(flatten)]
        window: WindowId,
    },
    /// Switch to a workspace. Accepts a workspace number (e.g., '5'),
    /// relative movement (e.g., 'right:2', 'left:1'), or special keywords
    /// ('previous', 'empty', 'name:<name>').
    Workspace { workspace: String },
    /// Cycle through windows
    CycleWindow {
        #[arg()]
        direction: String,
    },
    /// Move focus in a direction (up, down, left, right)
    MoveFocus {
        #[arg()]
        direction: String,
    },
    /// Move the active window to a monitor or in a specified direction (up, down, left, right)
    MoveWindow {
        #[arg()]
        target: String,
    },
    /// Swap windows in a direction (up, down, left, right)
    SwapWindow {
        #[arg()]
        direction: String,
    },
    /// Focus a specific window
    #[command(group(ArgGroup::new("winid_focus").required(true).args(&["class", "title", "pid", "address"])))]
    FocusWindow {
        #[command(flatten)]
        window: WindowId,
    },
    /// Toggle fake fullscreen
    ToggleFakeFullscreen,
    /// Toggle pseudo tiling
    TogglePseudo,
    /// Pin the active window to all workspaces
    TogglePin,
    /// Center the active window
    CenterWindow,
    /// Bring the active window to the top
    BringActiveToTop,
    /// Focus the urgent or last window
    FocusUrgentOrLast,
    /// Switch focus between current and last window
    FocusCurrentOrLast,
    /// Force the renderer to reload
    ForceRendererReload,
    /// Exit Hyprland
    Exit,
    /// Resize the active window
    ResizeActive {
        #[command(subcommand)]
        params: ResizeCmd,
    },
    /// Resize a specific window by pixel
    #[command(group(ArgGroup::new("winid_resize").required(true).args(&["class", "title", "pid", "address"])))]
    ResizeWindowPixel {
        #[command(subcommand)]
        params: ResizeCmd,
        #[command(flatten)]
        window: WindowId,
    },
}
