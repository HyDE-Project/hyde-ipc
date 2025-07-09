use crate::dispatch;
use crate::flags::Dispatch as DispatchCmd;
use crate::parsers::ParsedWindowIdentifier;
use hyprland::dispatch::WindowIdentifier;
use hyprland::event_listener::EventListener;
use std::str::FromStr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub fn sync_react(
    event: String,
    subtype: Option<String>,
    filter: Option<String>,
    dispatch: DispatchCmd,
    max_reactions: usize,
) -> hyprland::Result<()> {
    println!("Reacting to {event} events with dispatcher: {dispatch:?}");
    if let Some(filter) = &filter {
        println!("Using window filter: {filter}");
    }
    println!("Press Ctrl+C to stop");

    let mut event_listener = EventListener::new();
    let count = Arc::new(AtomicUsize::new(0));

    let parsed_filter = filter
        .as_deref()
        .map(ParsedWindowIdentifier::from_str)
        .transpose()
        .map_err(|e| hyprland::shared::HyprError::Other(e))?
        .map(|p| p.0);

    setup_event_handlers(
        &mut event_listener,
        &event,
        &subtype,
        parsed_filter,
        dispatch,
        count,
        max_reactions,
    )?;

    event_listener.start_listener()
}

#[allow(clippy::too_many_arguments)]
fn setup_event_handlers(
    event_listener: &mut EventListener,
    event: &str,
    subtype: &Option<String>,
    filter: Option<WindowIdentifier<'static>>,
    dispatch: DispatchCmd,
    count: Arc<AtomicUsize>,
    max_reactions: usize,
) -> hyprland::Result<()> {
    match event.to_lowercase().as_str() {
        "window" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "opened" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        let filter_clone = filter.clone();
                        event_listener.add_window_opened_handler(move |data| {
                            if !is_window_match(&filter_clone, &data.window_class, &data.window_title) {
                                return;
                            }
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    }
                    "closed" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_closed_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    }
                    "moved" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    }
                    "active" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        let filter_clone = filter.clone();
                        event_listener.add_active_window_changed_handler(move |data| {
                            if let Some(window_data) = data.as_ref() {
                                if !is_window_match(&filter_clone, &window_data.class, &window_data.title)
                                {
                                    return;
                                }
                            } else if filter_clone.is_some() {
                                return;
                            }
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    }
                    _ => {
                        return Err(hyprland::shared::HyprError::Other(format!(
                            "Unknown window subtype: {subtype}"
                        )));
                    }
                }
            }
        }
        "workspace" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "changed" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_changed_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    }
                    "added" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_added_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    }
                    "deleted" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_deleted_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    }
                    _ => {
                        return Err(hyprland::shared::HyprError::Other(format!(
                            "Unknown workspace subtype: {subtype}"
                        )));
                    }
                }
            }
        }
        _ => {
            return Err(hyprland::shared::HyprError::Other(format!(
                "Unknown event type: {event}"
            )));
        }
    }
    Ok(())
}

fn is_window_match(
    filter: &Option<WindowIdentifier>,
    window_class: &str,
    window_title: &str,
) -> bool {
    if let Some(filter) = filter {
        match filter {
            WindowIdentifier::ClassRegularExpression(pattern) => window_class.contains(pattern),
            WindowIdentifier::Title(pattern) => window_title.contains(pattern),
            // Note: PID and Address matching are not supported by event data
            _ => false,
        }
    } else {
        true
    }
}

fn handle_event(dispatch_cmd: DispatchCmd, count: &Arc<AtomicUsize>, max_reactions: usize) {
    let current = if max_reactions > 0 {
        count.fetch_add(1, Ordering::SeqCst) + 1
    } else {
        0
    };

    println!("Event triggered! Executing: {dispatch_cmd:?}");
    dispatch::handle_dispatch(dispatch_cmd, false);

    if max_reactions > 0 && current >= max_reactions {
        println!("Reached maximum number of reactions ({max_reactions}). Exiting...");
        std::process::exit(0);
    }
}
