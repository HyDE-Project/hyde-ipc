use crate::dispatch;
use crate::flags::Dispatch as DispatchCmd;

use hyprland::event_listener::EventListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

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

    setup_event_handlers(
        &mut event_listener,
        &event,
        &subtype,
        &filter,
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
    filter: &Option<String>,
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
                            if let Some(window_filter) = &filter_clone {
                                if let Some(title_pattern) = window_filter.strip_prefix("title:") {
                                    if !data
                                        .window_title
                                        .contains(title_pattern)
                                    {
                                        return;
                                    }
                                } else if let Some(class_pattern) =
                                    window_filter.strip_prefix("class:")
                                {
                                    if !data
                                        .window_class
                                        .contains(class_pattern)
                                    {
                                        return;
                                    }
                                }
                            }
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    },
                    "closed" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_closed_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    },
                    "moved" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_window_moved_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    },
                    "active" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        let filter_clone = filter.clone();
                        event_listener.add_active_window_changed_handler(move |data| {
                            if let Some(window_data) = data.as_ref() {
                                if let Some(window_filter) = &filter_clone {
                                    if let Some(title_pattern) =
                                        window_filter.strip_prefix("title:")
                                    {
                                        if !window_data
                                            .title
                                            .contains(title_pattern)
                                        {
                                            return;
                                        }
                                    } else if let Some(class_pattern) =
                                        window_filter.strip_prefix("class:")
                                    {
                                        if !window_data
                                            .class
                                            .contains(class_pattern)
                                        {
                                            return;
                                        }
                                    }
                                }
                            } else if filter_clone.is_some() {
                                return;
                            }
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    },
                    _ => {
                        return Err(hyprland::shared::HyprError::Other(format!(
                            "Unknown window subtype: {subtype}"
                        )));
                    },
                }
            }
        },
        "workspace" => {
            if let Some(subtype) = subtype {
                match subtype.to_lowercase().as_str() {
                    "changed" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_changed_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    },
                    "added" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_added_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    },
                    "deleted" => {
                        let dispatch_clone = dispatch.clone();
                        let count_clone = Arc::clone(&count);
                        event_listener.add_workspace_deleted_handler(move |_| {
                            handle_event(dispatch_clone.clone(), &count_clone, max_reactions);
                        });
                    },
                    _ => {
                        return Err(hyprland::shared::HyprError::Other(format!(
                            "Unknown workspace subtype: {subtype}"
                        )));
                    },
                }
            }
        },
        _ => {
            return Err(hyprland::shared::HyprError::Other(format!("Unknown event type: {event}")));
        },
    }
    Ok(())
}

fn handle_event(dispatch_cmd: DispatchCmd, count: &Arc<AtomicUsize>, max_reactions: usize) {
    let current = if max_reactions > 0 { count.fetch_add(1, Ordering::SeqCst) + 1 } else { 0 };

    let (dispatcher, args) = match dispatch_cmd {
        DispatchCmd::Exec { command } => ("exec", command),
        DispatchCmd::KillActiveWindow => ("killactivewindow", vec![]),
        DispatchCmd::ToggleFloating { window } => (
            "togglefloating",
            window
                .class
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ),
        DispatchCmd::ToggleSplit => ("togglesplit", vec![]),
        DispatchCmd::ToggleOpaque => ("toggleopaque", vec![]),
        DispatchCmd::MoveCursorToCorner { corner } => ("movecursortocorner", vec![corner]),
        DispatchCmd::MoveCursor { x, y } => ("movecursor", vec![x.to_string(), y.to_string()]),
        DispatchCmd::ToggleFullscreen { mode } => ("togglefullscreen", vec![mode]),
        DispatchCmd::MoveToWorkspace { workspace } => ("movetoworkspace", vec![workspace]),
        DispatchCmd::Workspace { workspace } => ("workspace", vec![workspace]),
        DispatchCmd::CycleWindow { direction } => ("cyclewindow", vec![direction]),
        DispatchCmd::MoveFocus { direction } => ("movefocus", vec![direction]),
        DispatchCmd::SwapWindow { direction } => ("swapwindow", vec![direction]),
        DispatchCmd::FocusWindow { window } => (
            "focuswindow",
            window
                .class
                .iter()
                .map(|s| s.to_string())
                .collect(),
        ),
        DispatchCmd::MoveWindow { target } => ("movewindow", vec![target]),
        DispatchCmd::ToggleFakeFullscreen => ("togglefakefullscreen", vec![]),
        DispatchCmd::TogglePseudo => ("togglepseudo", vec![]),
        DispatchCmd::TogglePin => ("togglepin", vec![]),
        DispatchCmd::CenterWindow => ("centerwindow", vec![]),
        DispatchCmd::BringActiveToTop => ("bringactivetotop", vec![]),
        DispatchCmd::FocusUrgentOrLast => ("focusurgentorlast", vec![]),
        DispatchCmd::FocusCurrentOrLast => ("focuscurrentorlast", vec![]),
        DispatchCmd::ForceRendererReload => ("forcerendererreload", vec![]),
        DispatchCmd::Exit => ("exit", vec![]),
        _ => {
            eprintln!("Dispatcher not fully implemented in react.rs handler yet.");
            return;
        },
    };

    println!("Event triggered! Executing: {dispatcher} with args: {args:?}");
    dispatch::sync_dispatch(dispatcher, &args);

    if max_reactions > 0 && current >= max_reactions {
        println!("Reached maximum number of reactions ({max_reactions}). Exiting...");
        std::process::exit(0);
    }
}
