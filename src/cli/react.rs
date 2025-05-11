use hyprland::event_listener::EventListener;
use hyprland::dispatch::Dispatch;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use crate::cli::dispatch::parse_dispatcher;

pub fn sync_react(
    event: String,
    subtype: Option<String>,
    dispatcher: String,
    args: Vec<String>,
    max_reactions: usize
) -> hyprland::Result<()> {
    println!("Reacting to {} events with dispatcher: {}", event, dispatcher);
    println!("Press Ctrl+C to stop");
    
    // Parse the dispatcher once to validate it
    let _dispatch_type = match parse_dispatcher(&dispatcher, &args) {
        Ok(dt) => dt,
        Err(e) => {
            eprintln!("Error parsing dispatcher: {}", e);
            return Err(hyprland::shared::HyprError::Other(e));
        }
    };
    
    let mut event_listener = EventListener::new();
    let count = Arc::new(AtomicUsize::new(0));
    
    setup_event_handlers(&mut event_listener, &event, &subtype, dispatcher, args, count, max_reactions)?;
    
    // Start the listener
    event_listener.start_listener()
}

fn setup_event_handlers(
    event_listener: &mut EventListener,
    event: &str,
    subtype: &Option<String>,
    dispatcher: String,
    args: Vec<String>,
    count: Arc<AtomicUsize>,
    max_reactions: usize
) -> hyprland::Result<()> {
    match event.to_lowercase().as_str() {
        "window" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "opened" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_opened_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    "closed" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_closed_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    "moved" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    "active" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_active_window_changed_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    _ => {
                        eprintln!("Unknown window subtype: {}", subtype);
                        return Err(hyprland::shared::HyprError::Other(format!("Unknown window subtype: {}", subtype)));
                    }
                }
            } else {
                // Add handlers for all window events
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_window_opened_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
                
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_window_closed_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
                
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_window_moved_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
                
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_active_window_changed_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
            }
        },
        "workspace" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "changed" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_changed_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    "added" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_added_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    "deleted" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_deleted_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    _ => {
                        eprintln!("Unknown workspace subtype: {}", subtype);
                        return Err(hyprland::shared::HyprError::Other(format!("Unknown workspace subtype: {}", subtype)));
                    }
                }
            } else {
                // Add handlers for all workspace events
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_workspace_changed_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
                
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_workspace_added_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
                
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_workspace_deleted_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
            }
        },
        "monitor" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_active_monitor_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        },
        "float" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_float_state_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        },
        "fullscreen" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_fullscreen_state_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        },
        "layout" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_layout_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        },
        "group" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "toggled" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_group_toggled_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    "moved_in" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_into_group_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    "moved_out" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_out_of_group_handler(move |_| {
                            handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                        });
                    },
                    _ => {
                        eprintln!("Unknown group subtype: {}", subtype);
                        return Err(hyprland::shared::HyprError::Other(format!("Unknown group subtype: {}", subtype)));
                    }
                }
            } else {
                // Add handlers for all group events
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_group_toggled_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
                
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_window_moved_into_group_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
                
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_window_moved_out_of_group_handler(move |_| {
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
            }
        },
        "config" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_config_reloaded_handler(move || {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        },
        _ => {
            eprintln!("Unknown event type: {}", event);
            return Err(hyprland::shared::HyprError::Other(format!("Unknown event type: {}", event)));
        }
    }
    
    Ok(())
}

fn handle_event(
    dispatcher: &str,
    args: &[String],
    count: &Arc<AtomicUsize>,
    max_reactions: usize
) {
    let current = if max_reactions > 0 {
        count.fetch_add(1, Ordering::SeqCst) + 1
    } else {
        0
    };
    
    match parse_dispatcher(dispatcher, args) {
        Ok(dispatch_type) => {
            println!("Event triggered! Executing: {} {:?}", dispatcher, args);
            
            // Execute synchronously only
            if let Err(e) = Dispatch::call(dispatch_type) {
                eprintln!("Error executing dispatcher: {}", e);
            }
            
            if max_reactions > 0 && current >= max_reactions {
                println!("Reached maximum number of reactions ({}). Exiting...", max_reactions);
                std::process::exit(0);
            }
        },
        Err(e) => {
            eprintln!("Error parsing dispatcher: {}", e);
        }
    }
} 