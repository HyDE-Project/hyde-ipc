# Contributing to `hyde-ipc`

First off, thank you for considering contributing to `hyde-ipc`! Your help is greatly appreciated.

This document provides guidelines for contributing to the project. Please read it carefully to ensure a smooth and effective contribution process.

## How to Contribute

There are many ways to contribute, from writing code and documentation to reporting bugs and suggesting features.

### Reporting Bugs

If you find a bug, please open an issue on the GitHub repository. To help us resolve the issue quickly, please include the following information:

*   **`hyde-ipc` Version**: The output of `hyde-ipc --version`.
*   **System Information**: Your OS, and Hyprland version.
*   **Steps to Reproduce**: A clear and concise description of how to reproduce the bug.
*   **Expected Behavior**: What you expected to happen.
*   **Actual Behavior**: What actually happened, including any error messages and logs. Use `hyde-ipc setup --watch` for service logs.

### Suggesting Enhancements

If you have an idea for a new feature or an improvement to an existing one, please open an issue on GitHub. Describe your idea in detail, including:

*   **Problem Description**: What problem does this enhancement solve?
*   **Proposed Solution**: A clear description of the enhancement you're proposing.
*   **Alternatives**: Any alternative solutions or features you've considered.

## Development Setup

To get started with development, you'll need to set up your environment.

1.  **Fork and Clone**: Fork the repository on GitHub and clone your fork locally.
    ```bash
    git clone https://github.com/your-username/hyde-ipc.git
    cd hyde-ipc
    ```

2.  **Install Rust**: Ensure you have the Rust toolchain installed. The required version is specified in `rust-toolchain.toml`. `rustup` will handle this automatically if it's installed.

3.  **Build**: Build the project in debug mode.
    ```bash
    cargo build
    ```

4.  **Run**: You can run the development version of the CLI directly with `cargo run`.
    ```bash
    cargo run -- -h
    cargo run -- dispatch killactivewindow
    ```

## Coding Style and Conventions

This project follows standard Rust conventions and formatting.

*   **Formatting**: All code must be formatted with `rustfmt`. A `rustfmt.toml` file is included in the repository to define project-specific settings. Please run `cargo fmt` before committing your changes.
*   **Clippy**: The code should be free of warnings from `clippy`. Run `cargo clippy --all-targets --all-features` to check your code.
*   **Comments**: Add comments to explain complex or non-obvious parts of the code. Focus on the *why*, not the *what*.
*   **Commit Messages**: Follow the conventional commit format. This helps in generating changelogs and understanding the history of the project.

## Pull Request Process

1.  **Create a Branch**: Create a new branch for your feature or bugfix.
    ```bash
    git checkout -b feature/my-new-feature
    ```

2.  **Make Your Changes**: Implement your changes, ensuring you follow the coding style guidelines.

3.  **Test Your Changes**: Add or update tests for your changes. Ensure all existing tests pass by running `cargo test`.

4.  **Format and Lint**: Run `cargo fmt` and `cargo clippy` to ensure your code is clean.

5.  **Commit Your Changes**: Commit your changes with a descriptive commit message.

6.  **Push to Your Fork**: Push your branch to your fork on GitHub.
    ```bash
    git push origin feature/my-new-feature
    ```

7.  **Open a Pull Request**: Open a pull request from your fork to the main `hyde-ipc` repository. Provide a clear title and description for your pull request, referencing any related issues.

Thank you again for your contribution!
