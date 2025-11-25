use crate::flags;
use crate::reaction_handler::{Reaction, ReactionManager};
use hyprland::dispatch::DispatchType;
use serde::Deserialize;
use std::convert::TryFrom;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

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
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {e}"))?;
        toml::from_str(&content).map_err(|e| format!("Failed to parse TOML config file: {e}"))
    }

    /// Validate the loaded configuration by checking each reaction and its dispatchers.
    /// This performs deeper argument validation by converting into CLI dispatch types
    /// and then attempting to translate them into Hyprland dispatch types.
    pub fn validate(&self) -> Result<(), String> {
        for (ri, rc) in self.reactions_config.iter().enumerate() {
            let reaction = &rc.reaction;
            if reaction.dispatchers.is_empty() {
                return Err(format!("Reaction at index {} has no dispatchers defined", ri));
            }

            for (di, dispatcher) in reaction.dispatchers.iter().enumerate() {
                // Convert reaction dispatcher into CLI dispatch enum
                let cli_dispatch: flags::Dispatch = dispatcher.clone().into();

                // Attempt to convert into Hyprland dispatch type (validates arguments)
                if let Err(e) = DispatchType::try_from(cli_dispatch) {
                    return Err(format!(
                        "Invalid dispatcher at reaction {} (#{}): {}",
                        ri,
                        di + 1,
                        e
                    ));
                }
            }
        }
        Ok(())
    }

    /// Load and validate a config from a file path.
    pub fn validate_file<P: AsRef<Path>>(path: P) -> Result<(), String> {
        let cfg = Self::from_file(&path)?;
        cfg.validate()
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

/// Validate a config path, which can be either a single file or a directory of `.toml` files.
///
/// - If `path` is a file, this is equivalent to `ReactConfig::validate_file`.
/// - If `path` is a directory, all `*.toml` files are validated individually. Any invalid file
///   is reported, and the function returns an error if at least one file is invalid or if no
///   `.toml` files are found.
pub fn validate_path<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();

    if path.is_dir() {
        let entries = fs::read_dir(path)
            .map_err(|e| format!("Failed to read config directory {}: {e}", path.display()))?;

        let mut had_errors = false;
        let mut checked_files = 0usize;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Failed to read entry in {}: {e}; skipping", path.display());
                    had_errors = true;
                    continue;
                },
            };

            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            checked_files += 1;

            if let Err(e) = ReactConfig::validate_file(&file_path) {
                had_errors = true;
                eprintln!("  ! {}: {}", file_path.display(), e);
            }
        }

        if checked_files == 0 {
            return Err(format!(
                "No .toml config files found in directory {}",
                path.display()
            ));
        }

        if had_errors {
            Err("One or more config files in the directory are invalid".to_string())
        } else {
            Ok(())
        }
    } else {
        ReactConfig::validate_file(path)
    }
}

pub fn run_from_config<P: AsRef<Path>>(path: P) -> Result<(), String> {
    let path = path.as_ref();

    if path.is_dir() {
        println!("Loading reactions from directory {}", path.display());

        let mut manager = ReactionManager::new();
        let mut total_reactions = 0usize;

        let entries = fs::read_dir(path)
            .map_err(|e| format!("Failed to read config directory {}: {e}", path.display()))?;

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("Failed to read entry in {}: {e}; skipping", path.display());
                    continue;
                },
            };

            let file_path = entry.path();
            if file_path.extension().and_then(|s| s.to_str()) != Some("toml") {
                continue;
            }

            println!("  - Loading config file {}", file_path.display());

            let config = match ReactConfig::from_file(&file_path) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("    ! Skipping {}: {}", file_path.display(), e);
                    continue;
                },
            };

            if let Err(e) = config.validate() {
                eprintln!("    ! Skipping {}: {}", file_path.display(), e);
                continue;
            }

            let reactions = config.reactions_config;
            let count = reactions.len();
            total_reactions += count;
            println!(
                "    + Loaded {} reaction(s) from {}",
                count,
                file_path.display()
            );

            for rc in reactions {
                manager.add_reaction(Reaction {
                    counter: Arc::new(AtomicUsize::new(0)),
                    ..rc.reaction
                });
            }
        }

        if total_reactions == 0 {
            return Err(format!("No valid reactions found in directory {}", path.display()));
        }

        println!("Loaded {} reactions from directory {}", total_reactions, path.display());
        return manager.start();
    }

    println!("Loading reactions from {}", path.display());
    let config = ReactConfig::from_file(path)?;
    // Optional: perform validation before starting the manager to fail fast on bad args
    config.validate()?;
    println!("Loaded {} reactions", config.reactions_config.len());
    let manager = config.into_manager();
    manager.start()
}
