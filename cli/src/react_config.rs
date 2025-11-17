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

pub fn run_from_config<P: AsRef<Path>>(path: P) -> Result<(), String> {
    println!("Loading reactions from {}", path.as_ref().display());
    let config = ReactConfig::from_file(path)?;
    // Optional: perform validation before starting the manager to fail fast on bad args
    config.validate()?;
    println!("Loaded {} reactions", config.reactions_config.len());
    let manager = config.into_manager();
    manager.start()
}
