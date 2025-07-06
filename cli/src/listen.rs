use hyprland::event_listener::EventListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn listen(filter: Option<String>, max_events: usize) -> hyprland::Result<()> {
    println!("Listening for Hyprland events...");
    println!("Press Ctrl+C to stop");

    let mut event_listener = EventListener::new();
    let count = Arc::new(AtomicUsize::new(0));

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_active_window_changed_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            println!("[WINDOW] Active window changed - {data:?}");
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_opened_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            println!("[WINDOW] Window opened - {data:?}");
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_closed_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            println!("[WINDOW] Window closed - {data:?}");
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_moved_handler(move |data| {
        if should_log_event("window", &filter_clone) {
            println!(
                "[WINDOW] Window moved - workspace: {}, address: {}",
                data.workspace_name, data.window_address
            );
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_float_state_changed_handler(move |data| {
        if should_log_event("float", &filter_clone) {
            println!(
                "[FLOAT] Float state changed - address: {}, floating: {}",
                data.address, data.floating
            );
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_fullscreen_state_changed_handler(move |state| {
        if should_log_event("fullscreen", &filter_clone) {
            let state_str = if state { "enabled" } else { "disabled" };
            println!("[FULLSCREEN] Fullscreen {state_str}");
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_workspace_changed_handler(move |id| {
        if should_log_event("workspace", &filter_clone) {
            println!("[WORKSPACE] Changed workspace - {id:?}");
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_workspace_added_handler(move |data| {
        if should_log_event("workspace", &filter_clone) {
            println!("[WORKSPACE] Workspace added - name: {}, id: {}", data.name, data.id);
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_workspace_deleted_handler(move |data| {
        if should_log_event("workspace", &filter_clone) {
            println!("[WORKSPACE] Workspace deleted - name: {}, id: {}", data.name, data.id);
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_active_monitor_changed_handler(move |data| {
        if should_log_event("monitor", &filter_clone) {
            println!(
                "[MONITOR] Active monitor changed - monitor: {}, workspace: {:?}",
                data.monitor_name, data.workspace_name
            );
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_layout_changed_handler(move |data| {
        if should_log_event("layout", &filter_clone) {
            println!(
                "[LAYOUT] Layout changed - keyboard: {}, layout: {}",
                data.keyboard_name, data.layout_name
            );
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_group_toggled_handler(move |data| {
        if should_log_event("group", &filter_clone) {
            println!(
                "[GROUP] Group toggled - toggled: {}, window count: {}",
                data.toggled,
                data.window_addresses.len()
            );
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_moved_into_group_handler(move |addr| {
        if should_log_event("group", &filter_clone) {
            println!("[GROUP] Window moved into group - address: {addr}");
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_window_moved_out_of_group_handler(move |addr| {
        if should_log_event("group", &filter_clone) {
            println!("[GROUP] Window moved out of group - address: {addr}");
            increment_count(&count_clone, max_events);
        }
    });

    let count_clone = Arc::clone(&count);
    let filter_clone = filter.clone();
    event_listener.add_config_reloaded_handler(move || {
        if should_log_event("config", &filter_clone) {
            println!("[CONFIG] Config reloaded");
            increment_count(&count_clone, max_events);
        }
    });

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
