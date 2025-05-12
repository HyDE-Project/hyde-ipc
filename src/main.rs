//! # Hyde-IPC
//! 
//! Hyde-IPC is a command-line interface for interacting with Hyprland, 
//! a dynamic tiling Wayland compositor. It provides tools for sending 
//! commands, listening to events, and creating event-based reactions.
//! 
//! ## Features
//! 
//! - **Dispatch** commands to Hyprland
//! - **Listen** to Hyprland events
//! - Set and get **Keywords** in Hyprland
//! - Create **Reactions** to Hyprland events
//!
//! ## Usage
//!
//! Run `hyde-ipc --help` to see available commands.

//!  Delegates to cli::main()
//TODO: add --version

mod cli;

fn main() {
    cli::main();
}
