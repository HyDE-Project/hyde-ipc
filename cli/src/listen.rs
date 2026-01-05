use hyprland::event_listener::EventListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub fn listen(filter: Option<String>, max_events: usize) -> hyprland::Result<()> {
    println!("Listening for Hyprland events...");
    println!("Press Ctrl+C to stop");
    let mut event_listener = EventListener::new();
    let context = ListenContext::new(filter, max_events);
    context.register_handlers(&mut event_listener);
    event_listener.start_listener()
}

struct ListenContext {
    filter: Option<String>,
    count: Arc<AtomicUsize>,
    max_events: usize,
}

struct HandlerContext {
    filter: Option<String>,
    count: Arc<AtomicUsize>,
    max_events: usize,
}

impl ListenContext {
    fn new(filter: Option<String>, max_events: usize) -> Self {
        Self { filter, count: Arc::new(AtomicUsize::new(0)), max_events }
    }

    fn handler_ctx(&self) -> HandlerContext {
        HandlerContext {
            filter: self.filter.clone(),
            count: Arc::clone(&self.count),
            max_events: self.max_events,
        }
    }

    fn with_handler<F>(&self, listener: &mut EventListener, f: F)
    where
        F: FnOnce(HandlerContext, &mut EventListener),
    {
        let ctx = self.handler_ctx();
        f(ctx, listener);
    }

    fn register_handlers(&self, listener: &mut EventListener) {
        self.with_handler(listener, |ctx, listener| {
            listener.add_active_window_changed_handler(move |data| {
                log_event("window", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[WINDOW] Active window changed - {data:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_opened_handler(move |data| {
                log_event("window", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[WINDOW] Window opened - {data:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_closed_handler(move |data| {
                log_event("window", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[WINDOW] Window closed - {data:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_moved_handler(move |data| {
                log_event("window", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!(
                        "[WINDOW] Window moved - workspace: {}, address: {}",
                        data.workspace_name, data.window_address
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_float_state_changed_handler(move |data| {
                log_event("float", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!(
                        "[FLOAT] Float state changed - address: {}, floating: {}",
                        data.address, data.floating
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_fullscreen_state_changed_handler(move |state| {
                log_event("fullscreen", &ctx.filter, &ctx.count, ctx.max_events, || {
                    let state_str = if state { "enabled" } else { "disabled" };
                    println!("[FULLSCREEN] Fullscreen {state_str}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_workspace_changed_handler(move |id| {
                log_event("workspace", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[WORKSPACE] Changed workspace - {id:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_workspace_added_handler(move |data| {
                log_event("workspace", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[WORKSPACE] Workspace added - name: {}, id: {}", data.name, data.id);
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_workspace_deleted_handler(move |data| {
                log_event("workspace", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!(
                        "[WORKSPACE] Workspace deleted - name: {}, id: {}",
                        data.name, data.id
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_active_monitor_changed_handler(move |data| {
                log_event("monitor", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!(
                        "[MONITOR] Active monitor changed - monitor: {}, workspace: {:?}",
                        data.monitor_name, data.workspace_name
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_layout_changed_handler(move |data| {
                log_event("layout", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!(
                        "[LAYOUT] Layout changed - keyboard: {}, layout: {}",
                        data.keyboard_name, data.layout_name
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_group_toggled_handler(move |data| {
                log_event("group", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!(
                        "[GROUP] Group toggled - toggled: {}, window count: {}",
                        data.toggled,
                        data.window_addresses.len()
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_moved_into_group_handler(move |addr| {
                log_event("group", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[GROUP] Window moved into group - address: {addr}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_moved_out_of_group_handler(move |addr| {
                log_event("group", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[GROUP] Window moved out of group - address: {addr}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_config_reloaded_handler(move || {
                log_event("config", &ctx.filter, &ctx.count, ctx.max_events, || {
                    println!("[CONFIG] Config reloaded");
                });
            });
        });
    }
}

fn should_log_event(event_type: &str, filter: &Option<String>) -> bool {
    match filter {
        Some(f) if !f.is_empty() => event_type
            .to_lowercase()
            .contains(&f.to_lowercase()),
        _ => true,
    }
}

fn log_event<F>(
    event_type: &str,
    filter: &Option<String>,
    count: &Arc<AtomicUsize>,
    max: usize,
    f: F,
) where
    F: FnOnce(),
{
    if should_log_event(event_type, filter) {
        f();
        increment_count(count, max);
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
