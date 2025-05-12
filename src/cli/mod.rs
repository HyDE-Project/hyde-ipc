mod flags;
mod keyword;
mod dispatch;
mod listen;
mod react;
mod react_config;

use flags::{Cli, Commands};
use std::process;
use clap::{CommandFactory, Parser};

fn print_usage_and_exit() -> ! {
    Cli::command().print_help().unwrap();
    println!();
    process::exit(1);
}

pub fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Keyword { r#async, get, set, keyword, value } => {
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
        Commands::Dispatch { r#async, list_dispatchers, dispatcher, args } => {
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
        Commands::Listen { filter, max_events, json } => {
            if let Err(e) = listen::listen(filter, max_events, json) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
        Commands::React { r#async, config, create_template, inline: _, event, subtype, dispatcher, args, max_reactions } => {
            // Handle creating a template config if requested
            if let Some(path) = create_template {
                if let Err(e) = react_config::create_template_config(&path) {
                    eprintln!("Error creating template: {}", e);
                    process::exit(1);
                }
                println!("Created template config at {}", path);
                return;
            }

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
                println!("Note: Async mode is disabled, using synchronous execution instead");
            }
            
            if let Err(e) = react::sync_react(event, subtype, dispatcher, args, max_reactions) {
                eprintln!("Error: {}", e);
                process::exit(1);
            }
        }
    }
} 
