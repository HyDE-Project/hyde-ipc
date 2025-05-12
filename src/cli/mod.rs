mod dispatch;
mod flags;
mod keyword;
mod listen;
mod react;
mod react_config;
mod setup;

use clap::{CommandFactory, Parser};
use flags::{Cli, Commands};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn print_usage_and_exit() -> ! {
    Cli::command().print_help().unwrap();
    println!();
    process::exit(1);
}

pub fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keyword {
            r#async,
            get,
            set,
            keyword,
            value,
        } => {
            if set && value.is_none() {
                eprintln!("Error: --set requires a value");
                print_usage_and_exit();
            }
            if r#async {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();
                rt.block_on(keyword::async_keyword(get, set, keyword, value));
            } else {
                keyword::sync_keyword(get, set, keyword, value);
            }
        }
        Commands::Dispatch {
            r#async,
            list_dispatchers,
            dispatcher,
            args,
        } => {
            if list_dispatchers {
                // Just list dispatchers and exit
                dispatch::sync_dispatch(true, String::new(), vec![]);
                return;
            }

            // At this point, dispatcher should be Some because of the required_unless_present
            let dispatcher = dispatcher.unwrap_or_else(|| {
                eprintln!("Error: dispatcher is required");
                print_usage_and_exit();
            });

            if r#async {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                // Execute the dispatch and ensure it completes
                rt.block_on(async {
                    // Create a oneshot channel to signal completion
                    let (tx, rx) = tokio::sync::oneshot::channel();

                    // Spawn the task with a completion signal
                    tokio::spawn(async move {
                        let result = dispatch::async_dispatch(false, dispatcher, args).await;
                        let _ = tx.send(result); // Signal completion
                    });

                    // Wait for the task to complete
                    match rx.await {
                        Ok(_) => (), // Task completed successfully
                        Err(_) => eprintln!("Warning: Async task was dropped before completion"),
                    }
                });
            } else {
                dispatch::sync_dispatch(false, dispatcher, args);
            }
        }
        Commands::Listen {
            filter,
            max_events,
            json,
        } => {
            if let Err(e) = listen::listen(filter, max_events, json) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Commands::React {
            r#async,
            config,
            inline: _,
            event,
            subtype,
            dispatcher,
            args,
            max_reactions,
        } => {
            // Handle config file mode
            if let Some(config_path) = config {
                if let Err(e) = react_config::run_from_config(&config_path) {
                    eprintln!("Error running from config: {}", e);
                    process::exit(1);
                }
                return;
            }
            // Handle inline mode (single reaction)
            let event = event.unwrap_or_else(|| {
                eprintln!("Error: event is required");
                print_usage_and_exit();
            });
            let dispatcher = dispatcher.unwrap_or_else(|| {
                eprintln!("Error: dispatcher is required");
                print_usage_and_exit();
            });
            // Always use synchronous mode, ignore the async flag
            if r#async {
                println!("Note: async flag is deprecated");
            }
            if let Err(e) = react::sync_react(event, subtype, dispatcher, args, max_reactions) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Commands::Global { config_path, setup } => {
            if setup {
                setup::setup_service_file();
                // If no file is given, create an empty config file
                let home = env::var("HOME").expect("Could not get $HOME");
                let dest_dir = PathBuf::from(&home).join(".local/share/hyde-ipc");
                let dest = dest_dir.join("config.toml");
                if let Err(e) = fs::create_dir_all(&dest_dir) {
                    eprintln!("Error creating global config directory: {}", e);
                    std::process::exit(1);
                }
                if let Some(path) = config_path {
                    setup::copy_and_reload_config(&path);
                } else {
                    // Create empty file if not exists
                    if !dest.exists() {
                        if let Err(e) = fs::File::create(&dest) {
                            eprintln!("Error creating empty config file: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            } else {
                setup::ensure_service_setup();
                if let Some(path) = config_path {
                    setup::copy_and_reload_config(&path);
                } else {
                    eprintln!("Error: you must provide a config file unless using --setup");
                    std::process::exit(1);
                }
            }
        }
        Commands::Setup => {
            setup::setup_service_file();
        }
    }
}
