//! CLI argument definitions for hyde-ipc.
//!
//! This module defines the command-line interface using clap, including all subcommands and their options.

use clap::{ArgGroup, Parser, Subcommand};

/// Command-line interface for hyde-ipc.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    pub command: Commands,
}

/// All supported subcommands for hyde-ipc.
#[derive(Subcommand, Debug)]
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

        /// The keyword to get or set (positional)
        keyword: String,

        /// The value to set (required if --set, positional)
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

        /// Use JSON format for output
        #[arg(short = 'j', long = "json")]
        json: bool,
    },

    /// React to specific events by dispatching commands.
    #[command(group(
        ArgGroup::new("mode")
            .required(false)
            .args(["config", "inline"]),
    ))]
    React {
        /// Deprecated: Async mode is no longer supported (will be ignored)
        #[arg(short = 'a', long = "async")]
        r#async: bool,

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
        #[arg(
            short = 'd',
            long = "dispatch",
            required_unless_present = "config"
        )]
        dispatcher: Option<String>,

        /// Arguments for the dispatcher
        #[arg(short = 'p', long = "params")]
        args: Vec<String>,

        /// Limit number of reactions (0 for unlimited)
        #[arg(
            short = 'n',
            long = "max-reactions",
            default_value = "0"
        )]
        max_reactions: usize,
    },

    /// Install a config globally and manage the user service.
    Global {
        /// Path to the config file to install globally (optional if --setup is used)
        config_path: Option<String>,
        /// Also set up the user service file
        #[arg(short = 's', long = "setup")]
        setup: bool,
        /// Stop the running service
        #[arg(short = 'k', long = "kill")]
        kill: bool,
        /// Restart the running service
        #[arg(short = 'r', long = "restart")]
        restart: bool,
    },

    /// Set up the systemd user service for hyde-ipc.
    Setup,
    // Future: Add more subcommands here!
}

#[derive(Parser, Debug)]
#[command(help_template = "{before-help}{about-with-newline}{usage-heading}{usage}{sections}")]
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

#[derive(Subcommand, Debug)]
pub enum Dispatch {
    /// Execute a command
    Exec { command: Vec<String> },
    /// Kill the active window
    KillActiveWindow,
    /// Toggle floating mode for a window
    ToggleFloating { window: Option<String> },
    /// Toggle the split orientation
    ToggleSplit,
    /// Toggle opacity for the active window
    ToggleOpaque,
    /// Move cursor to a corner
    MoveCursorToCorner { corner: String },
    /// Toggle fullscreen mode
    ToggleFullscreen { mode: String },
    /// Move window to workspace
    MoveToWorkspace { workspace: String },
    /// Switch to a workspace
    Workspace { workspace: String },
    /// Cycle through windows
    CycleWindow { direction: String },
    /// Move focus in a direction
    MoveFocus { direction: String },
    /// Swap windows in a direction
    SwapWindow { direction: String },
    /// Focus a specific window
    FocusWindow { window: String },
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
    ResizeActive { resize_params: Vec<String> },
    /// Resize a specific window by pixel
    ResizeWindowPixel { resize_params: Vec<String> },
}
