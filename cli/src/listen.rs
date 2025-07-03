use hyprland::event_listener::EventListener;
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn listen(filter: Option<String>, max_events: usize, json: bool) -> hyprland::Result<()> {
    println!("Listening for Hyprland events...");
    println!("Press Ctrl+C to stop");

    let mut event_listener = EventListener::new();
    let count = Arc::new(AtomicUsize::new(0));

    // Window events
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_active_window_changed_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "WINDOW",
                        "event": "Active window changed",
                        "data": {
                            "class": data.as_ref().map(|d| d.class.clone()),
                            "title": data.as_ref().map(|d| d.title.clone()),
                            "address": data.as_ref().map(|d| d.address.to_string())
                        }
                    })
                );
            } else {
                println!("[WINDOW] Active window changed - {:?}", data);
            }
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_opened_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "WINDOW",
                        "event": "Window opened",
                        "data": {
                            "window_class": data.window_class,
                            "window_title": data.window_title,
                            "window_address": data.window_address.to_string()
                        }
                    })
                );
            } else {
                println!("[WINDOW] Window opened - {:?}", data);
            }
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_closed_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "WINDOW",
                        "event": "Window closed",
                        "data": data.to_string()
                    })
                );
            } else {
                println!("[WINDOW] Window closed - {:?}", data);
            }
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_moved_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "WINDOW",
                        "event": "Window moved",
                        "data": {
                            "window_address": data.window_address.to_string(),
                            "workspace_id": data.workspace_id,
                            "workspace_name": data.workspace_name
                        }
                    })
                );
            } else {
                println!(
                    "[WINDOW] Window moved - workspace: {}, address: {}",
                    data.workspace_name, data.window_address
                );
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Float state
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_float_state_changed_handler(move |data| {
        if should_log_event("float", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "FLOAT",
                        "event": "Float state changed",
                        "data": {
                            "address": data.address.to_string(),
                            "floating": data.floating
                        }
                    })
                );
            } else {
                println!(
                    "[FLOAT] Float state changed - address: {}, floating: {}",
                    data.address, data.floating
                );
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Fullscreen state
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_fullscreen_state_changed_handler(move |state| {
        if should_log_event("fullscreen", &filter_clone) {
            let state_str = if state { "enabled" } else { "disabled" };
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "FULLSCREEN",
                        "event": "Fullscreen state changed",
                        "data": state
                    })
                );
            } else {
                println!("[FULLSCREEN] Fullscreen {}", state_str);
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Workspace events
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_workspace_changed_handler(move |id| {
        if should_log_event("workspace", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "WORKSPACE",
                        "event": "Changed workspace",
                        "data": format!("{:?}", id)
                    })
                );
            } else {
                println!("[WORKSPACE] Changed workspace - {:?}", id);
            }
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_workspace_added_handler(move |data| {
        if should_log_event("workspace", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "WORKSPACE",
                        "event": "Workspace added",
                        "data": {
                            "name": data.name,
                            "id": data.id
                        }
                    })
                );
            } else {
                println!("[WORKSPACE] Workspace added - name: {}, id: {}", data.name, data.id);
            }
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_workspace_deleted_handler(move |data| {
        if should_log_event("workspace", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "WORKSPACE",
                        "event": "Workspace deleted",
                        "data": {
                            "name": data.name,
                            "id": data.id
                        }
                    })
                );
            } else {
                println!("[WORKSPACE] Workspace deleted - name: {}, id: {}", data.name, data.id);
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Monitor events
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_active_monitor_changed_handler(move |data| {
        if should_log_event("monitor", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "MONITOR",
                        "event": "Active monitor changed",
                        "data": {
                            "monitor_name": data.monitor_name,
                            "workspace_name": data.workspace_name
                        }
                    })
                );
            } else {
                println!(
                    "[MONITOR] Active monitor changed - monitor: {}, workspace: {:?}",
                    data.monitor_name, data.workspace_name
                );
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Layout events
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_layout_changed_handler(move |data| {
        if should_log_event("layout", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "LAYOUT",
                        "event": "Layout changed",
                        "data": {
                            "keyboard_name": data.keyboard_name,
                            "layout_name": data.layout_name
                        }
                    })
                );
            } else {
                println!(
                    "[LAYOUT] Layout changed - keyboard: {}, layout: {}",
                    data.keyboard_name, data.layout_name
                );
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Group events
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_group_toggled_handler(move |data| {
        if should_log_event("group", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "GROUP",
                        "event": "Group toggled",
                        "data": {
                            "toggled": data.toggled,
                            "window_addresses": data.window_addresses
                                .iter()
                                .map(|addr| addr.to_string())
                                .collect::<Vec<String>>()
                        }
                    })
                );
            } else {
                println!(
                    "[GROUP] Group toggled - toggled: {}, window count: {}",
                    data.toggled,
                    data.window_addresses.len()
                );
            }
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_moved_into_group_handler(move |addr| {
        if should_log_event("group", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "GROUP",
                        "event": "Window moved into group",
                        "data": addr.to_string()
                    })
                );
            } else {
                println!("[GROUP] Window moved into group - address: {}", addr);
            }
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_moved_out_of_group_handler(move |addr| {
        if should_log_event("group", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "GROUP",
                        "event": "Window moved out of group",
                        "data": addr.to_string()
                    })
                );
            } else {
                println!("[GROUP] Window moved out of group - address: {}", addr);
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Config events
    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_config_reloaded_handler(move || {
        if should_log_event("config", &filter_clone) {
            if json {
                println!(
                    "{}",
                    json!({
                        "type": "CONFIG",
                        "event": "Config reloaded"
                    })
                );
            } else {
                println!("[CONFIG] Config reloaded");
            }
            increment_count(&count_clone, max_events);
        }
    });

    // Start the listener
    event_listener.start_listener()
}

fn should_log_event(event_type: &str, filter: &Option<String>) -> bool {
    match filter {
        Some(f) if !f.is_empty() => event_type
            .to_lowercase()
            .contains(&f.to_lowercase()),
        _ => true,
    }
}

fn increment_count(count: &Arc<AtomicUsize>, max: usize) {
    if max > 0 {
        let current = count.fetch_add(1, Ordering::SeqCst) + 1;
        if current >= max {
            std::process::exit(0);
        }
    }
}
