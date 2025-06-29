use crate::cli::dispatch::parse_dispatcher;
use crate::cli::flags::Dispatch as DispatchCmd;
use hyprland::dispatch::Dispatch;
use hyprland::event_listener::EventListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn build_dispatch_cmd(dispatcher: &str, args: &[String]) -> Result<DispatchCmd, String> {
    match dispatcher {
        "Exec" => Ok(DispatchCmd::Exec {
            command: args.to_vec(),
        }),
        "KillActiveWindow" => Ok(DispatchCmd::KillActiveWindow),
        "ToggleFloating" => Ok(DispatchCmd::ToggleFloating {
            window: args.first().cloned(),
        }),
        "ToggleSplit" => Ok(DispatchCmd::ToggleSplit),
        "ToggleOpaque" => Ok(DispatchCmd::ToggleOpaque),
        "MoveCursorToCorner" => Ok(DispatchCmd::MoveCursorToCorner {
            corner: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "ToggleFullscreen" => Ok(DispatchCmd::ToggleFullscreen {
            mode: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "MoveToWorkspace" => Ok(DispatchCmd::MoveToWorkspace {
            workspace: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "Workspace" => Ok(DispatchCmd::Workspace {
            workspace: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "CycleWindow" => Ok(DispatchCmd::CycleWindow {
            direction: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "MoveFocus" => Ok(DispatchCmd::MoveFocus {
            direction: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "SwapWindow" => Ok(DispatchCmd::SwapWindow {
            direction: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "FocusWindow" => Ok(DispatchCmd::FocusWindow {
            window: args
                .first()
                .cloned()
                .unwrap_or_default(),
        }),
        "ToggleFakeFullscreen" => Ok(DispatchCmd::ToggleFakeFullscreen),
        "TogglePseudo" => Ok(DispatchCmd::TogglePseudo),
        "TogglePin" => Ok(DispatchCmd::TogglePin),
        "CenterWindow" => Ok(DispatchCmd::CenterWindow),
        "BringActiveToTop" => Ok(DispatchCmd::BringActiveToTop),
        "FocusUrgentOrLast" => Ok(DispatchCmd::FocusUrgentOrLast),
        "FocusCurrentOrLast" => Ok(DispatchCmd::FocusCurrentOrLast),
        "ForceRendererReload" => Ok(DispatchCmd::ForceRendererReload),
        "Exit" => Ok(DispatchCmd::Exit),
        "ResizeActive" => Ok(DispatchCmd::ResizeActive {
            resize_params: args.to_vec(),
        }),
        "ResizeWindowPixel" => Ok(DispatchCmd::ResizeWindowPixel {
            resize_params: args.to_vec(),
        }),
        _ => Err(format!("Unknown dispatcher: {}", dispatcher)),
    }
}

/// React to Hyprland events and dispatch commands based on CLI arguments.
///
/// # Arguments
/// * `event` - The event type to react to (e.g., "window").
/// * `subtype` - Optional event subtype (e.g., "opened").
/// * `filter` - Optional window filter (e.g., "title:foo").
/// * `dispatcher` - The dispatcher to execute.
/// * `args` - Arguments for the dispatcher.
/// * `max_reactions` - Maximum number of reactions (0 for unlimited).
pub fn sync_react(
    event: String,
    subtype: Option<String>,
    filter: Option<String>,
    dispatcher: String,
    args: Vec<String>,
    max_reactions: usize,
) -> hyprland::Result<()> {
    println!(
        "Reacting to {} events with dispatcher: {}",
        event, dispatcher
    );
    if let Some(filter) = &filter {
        println!("Using window filter: {}", filter);
    }
    println!("Press Ctrl+C to stop");

    // Parse the dispatcher once to validate it
    let dispatch_cmd = match build_dispatch_cmd(&dispatcher, &args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error parsing dispatcher: {}", e);
            return Err(hyprland::shared::HyprError::Other(e));
        }
    };

    let _dispatch_type = match parse_dispatcher(dispatch_cmd) {
        Ok(dt) => dt,
        Err(e) => {
            eprintln!("Error parsing dispatcher: {}", e);
            return Err(hyprland::shared::HyprError::Other(e));
        }
    };

    let mut event_listener = EventListener::new();
    let count = Arc::new(AtomicUsize::new(0));

    setup_event_handlers(
        &mut event_listener,
        &event,
        &subtype,
        &filter,
        dispatcher,
        args,
        count,
        max_reactions,
    )?;

    // Start the listener
    event_listener.start_listener()
}

/// Set up event handlers for the specified event and subtype.
///
/// # Arguments
/// * `event_listener` - The event listener to register handlers on.
/// * `event` - The event type (e.g., "window").
/// * `subtype` - Optional event subtype.
/// * `filter` - Optional window filter.
/// * `dispatcher` - The dispatcher to execute.
/// * `args` - Arguments for the dispatcher.
/// * `count` - Shared counter for limiting reactions.
/// * `max_reactions` - Maximum number of reactions.
#[allow(clippy::too_many_arguments)]
fn setup_event_handlers(
    event_listener: &mut EventListener,
    event: &str,
    subtype: &Option<String>,
    filter: &Option<String>,
    dispatcher: String,
    args: Vec<String>,
    count: Arc<AtomicUsize>,
    max_reactions: usize,
) -> hyprland::Result<()> {
    match event.to_lowercase().as_str() {
        "window" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "opened" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        let filter_clone = filter.clone();
                        event_listener.add_window_opened_handler(move |data| {
                            // Check if we have a window filter
                            if let Some(window_filter) = &filter_clone {
                                if let Some(title_pattern) = window_filter.strip_prefix("title:") {
                                    if !data
                                        .window_title
                                        .contains(title_pattern)
                                    {
                                        return; // Skip if window title doesn't match
                                    }
                                } else if let Some(class_pattern) =
                                    window_filter.strip_prefix("class:")
                                {
                                    if !data
                                        .window_class
                                        .contains(class_pattern)
                                    {
                                        return; // Skip if window class doesn't match
                                    }
                                }
                            }

                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    "closed" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_closed_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    "moved" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    "active" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        let filter_clone = filter.clone();
                        event_listener.add_active_window_changed_handler(move |data| {
                            // Check if we have window data and a window filter
                            if let Some(window_data) = data.as_ref() {
                                if let Some(window_filter) = &filter_clone {
                                    if let Some(title_pattern) =
                                        window_filter.strip_prefix("title:")
                                    {
                                        if !window_data
                                            .title
                                            .contains(title_pattern)
                                        {
                                            return; // Skip if window title doesn't match
                                        }
                                    } else if let Some(class_pattern) =
                                        window_filter.strip_prefix("class:")
                                    {
                                        if !window_data
                                            .class
                                            .contains(class_pattern)
                                        {
                                            return; // Skip if window class doesn't match
                                        }
                                    }
                                }
                            } else if filter_clone.is_some() {
                                // If we have a window filter but no window data, skip
                                return;
                            }

                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    _ => {
                        eprintln!("Unknown window subtype: {}", subtype);
                        return Err(hyprland::shared::HyprError::Other(format!(
                            "Unknown window subtype: {}",
                            subtype
                        )));
                    }
                }
            } else {
                // Add handlers for all window events
                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                let filter_clone = filter.clone();
                event_listener.add_window_opened_handler(move |data| {
                    // Check if we have a window filter
                    if let Some(window_filter) = &filter_clone {
                        if let Some(title_pattern) = window_filter.strip_prefix("title:") {
                            if !data
                                .window_title
                                .contains(title_pattern)
                            {
                                return; // Skip if window title doesn't match
                            }
                        } else if let Some(class_pattern) = window_filter.strip_prefix("class:") {
                            if !data
                                .window_class
                                .contains(class_pattern)
                            {
                                return; // Skip if window class doesn't match
                            }
                        }
                    }

                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });

                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_window_closed_handler(move |_| {
                    // For closed windows, we don't have title/class info, so no filtering
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });

                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                event_listener.add_window_moved_handler(move |_| {
                    // For window move events, we could match on window_address, but will skip for simplicity
                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });

                let dispatcher_clone = dispatcher.clone();
                let args_clone = args.clone();
                let count_clone = Arc::clone(&count);
                let filter_clone = filter.clone();
                event_listener.add_active_window_changed_handler(move |data| {
                    // Check if we have window data and a window filter
                    if let Some(window_data) = data.as_ref() {
                        if let Some(window_filter) = &filter_clone {
                            if let Some(title_pattern) = window_filter.strip_prefix("title:") {
                                if !window_data
                                    .title
                                    .contains(title_pattern)
                                {
                                    return; // Skip if window title doesn't match
                                }
                            } else if let Some(class_pattern) = window_filter.strip_prefix("class:")
                            {
                                if !window_data
                                    .class
                                    .contains(class_pattern)
                                {
                                    return; // Skip if window class doesn't match
                                }
                            }
                        }
                    } else if filter_clone.is_some() {
                        // If we have a window filter but no window data, skip
                        return;
                    }

                    handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
                });
            }
        }
        "workspace" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "changed" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_changed_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    "added" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_added_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    "deleted" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_deleted_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    _ => {
                        eprintln!("Unknown workspace subtype: {}", subtype);
                        return Err(hyprland::shared::HyprError::Other(format!(
                            "Unknown workspace subtype: {}",
                            subtype
                        )));
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
        }
        "monitor" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_active_monitor_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        }
        "float" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_float_state_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        }
        "fullscreen" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_fullscreen_state_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        }
        "layout" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_layout_changed_handler(move |_| {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        }
        "group" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "toggled" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_group_toggled_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    "moved_in" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_into_group_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    "moved_out" => {
                        let dispatcher_clone = dispatcher.clone();
                        let args_clone = args.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_out_of_group_handler(move |_| {
                            handle_event(
                                &dispatcher_clone,
                                &args_clone,
                                &count_clone,
                                max_reactions,
                            );
                        });
                    }
                    _ => {
                        eprintln!("Unknown group subtype: {}", subtype);
                        return Err(hyprland::shared::HyprError::Other(format!(
                            "Unknown group subtype: {}",
                            subtype
                        )));
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
        }
        "config" => {
            let dispatcher_clone = dispatcher.clone();
            let args_clone = args.clone();
            let count_clone = Arc::clone(&count);
            event_listener.add_config_reloaded_handler(move || {
                handle_event(&dispatcher_clone, &args_clone, &count_clone, max_reactions);
            });
        }
        _ => {
            eprintln!("Unknown event type: {}", event);
            return Err(hyprland::shared::HyprError::Other(format!(
                "Unknown event type: {}",
                event
            )));
        }
    }

    Ok(())
}

fn handle_event(dispatcher: &str, args: &[String], count: &Arc<AtomicUsize>, max_reactions: usize) {
    let current = if max_reactions > 0 {
        count.fetch_add(1, Ordering::SeqCst) + 1
    } else {
        0
    };

    let dispatch_cmd = match build_dispatch_cmd(dispatcher, args) {
        Ok(cmd) => cmd,
        Err(e) => {
            eprintln!("Error parsing dispatcher: {}", e);
            return;
        }
    };

    match parse_dispatcher(dispatch_cmd) {
        Ok(dispatch_type) => {
            println!("Event triggered! Executing: {} {:?}", dispatcher, args);

            // Execute synchronously only
            if let Err(e) = Dispatch::call(dispatch_type) {
                eprintln!("Error executing dispatcher: {}", e);
            }

            if max_reactions > 0 && current >= max_reactions {
                println!(
                    "Reached maximum number of reactions ({}). Exiting...",
                    max_reactions
                );
                std::process::exit(0);
            }
        }
        Err(e) => {
            eprintln!("Error parsing dispatcher: {}", e);
        }
    }
}
