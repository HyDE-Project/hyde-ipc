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
        #[arg(short = 'g', long = "get", group = "action")]
        get: bool,

        /// Set the value of a keyword
        #[arg(short = 's', long = "set", group = "action")]
        set: bool,

        /// The keyword to get or set (positional)
        keyword: String,

        /// The value to set (required if --set, positional)
        value: Option<String>,
    },

    /// Execute a dispatcher command.
    Dispatch {
        /// Use async mode
        #[arg(short = 'a', long = "async")]
        r#async: bool,

        /// List available dispatchers
        #[arg(short = 'l', long = "list-dispatchers")]
        list_dispatchers: bool,

        /// The dispatcher to execute
        #[arg(required_unless_present = "list_dispatchers")]
        dispatcher: Option<String>,

        /// The arguments for the dispatcher
        args: Vec<String>,
    },

    /// Listen for and log Hyprland events.
    Listen {
        /// Filter events by type (e.g., "window", "workspace")
        #[arg(short = 'f', long = "filter")]
        filter: Option<String>,

        /// Maximum number of events to log (0 for unlimited)
        #[arg(short = 'n', long = "max-events", default_value = "0")]
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
        #[arg(short = 'c', long = "config", group = "mode")]
        config: Option<String>,

        /// Use inline mode (single reaction)
        #[arg(short = 'i', long = "inline", group = "mode")]
        inline: bool,

        /// Event type to react to (e.g., "window", "workspace")
        #[arg(short = 'e', long = "event", required_unless_present = "config")]
        event: Option<String>,

        /// Event subtype for more specific filtering (e.g., "opened" for window events)
        #[arg(short = 's', long = "subtype")]
        subtype: Option<String>,
        
        /// Window filter for window events (e.g., "title:Google Chrome" or "class:firefox")
        #[arg(short = 'f', long = "filter")]
        filter: Option<String>,

        /// Dispatcher command to execute when the event occurs
        #[arg(short = 'd', long = "dispatch", required_unless_present = "config")]
        dispatcher: Option<String>,

        /// Arguments for the dispatcher
        #[arg(short = 'p', long = "params")]
        args: Vec<String>,

        /// Limit number of reactions (0 for unlimited)
        #[arg(short = 'n', long = "max-reactions", default_value = "0")]
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
