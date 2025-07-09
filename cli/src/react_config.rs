use crate::reaction_handler::{Reaction, ReactionManager};
use serde::Deserialize;
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
    println!("Loaded {} reactions", config.reactions_config.len());
    let manager = config.into_manager();
    manager.start()
}
