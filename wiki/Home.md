# Home

Welcome to the developer wiki for the `hyde-ipc` Command-Line Interface (CLI).

## Overview

`hyde-ipc` is a powerful and extensible command-line tool designed to interact with the Hyprland compositor's IPC (Inter-Process Communication) socket. It provides a comprehensive set of commands for controlling, querying, and automating window management workflows.

While it can be used for simple commands like `hyprctl`, its primary strength lies in its advanced event-reaction system, which allows for the creation of sophisticated, declarative automation rules.

This wiki provides detailed documentation for developers and advanced users looking to understand and leverage the full capabilities of the `hyde-ipc` CLI.

## Core Features

*   **Direct Command Dispatching**: Execute any Hyprland dispatcher command synchronously or asynchronously.
*   **State Querying**: Get real-time information from the compositor, such as keyword values or cursor position.
*   **Event Listening**: Monitor the Hyprland event stream for debugging and diagnostics.
*   **Declarative Event-Reaction System**: Define complex automation rules in a simple TOML format. React to events like window creation, workspace changes, and more by dispatching a series of commands.
*   **Systemd Service Integration**: Run `hyde-ipc` as a persistent background service to handle global event reactions seamlessly.

## Navigation

To learn more, please explore the pages in this wiki:

1.  **[Installation and Setup](https://github.com/your-repo/hyde-ipc/wiki/1.-Installation-and-Setup)**: Instructions for building, installing, and configuring the CLI and its background service.
2.  **[Command Reference](https://github.com/your-repo/hyde-ipc/wiki/2.-Command-Reference)**: A detailed breakdown of every command and subcommand.
3.  **[Dispatchers](https://github.com/your-repo/hyde-ipc/wiki/3.-Dispatchers)**: A complete list of available dispatchers and their arguments.
4.  **[Event-Reaction System](https://github.com/your-repo/hyde-ipc/wiki/4.-Event-Reaction-System)**: An in-depth guide to the powerful event-reaction engine.
5.  **[Configuration](https://github.com/your-repo/hyde-ipc/wiki/5.-Configuration)**: A detailed reference for the `reactions.toml` configuration file.
