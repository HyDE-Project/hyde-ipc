use hyprland::event_listener::EventListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub fn listen(filter: Option<String>, max_events: usize) -> hyprland::Result<()> {
    println!("Listening for Hyprland events...");
    println!("Press Ctrl+C to stop");
    let mut event_listener = EventListener::new();
    let context = ListenContext::new(filter, max_events);
    let done = Arc::clone(&context.done);
    context.register_handlers(&mut event_listener);
    event_listener.start_listener_while(|| !done.load(Ordering::SeqCst))
}

struct ListenContext {
    filter: Option<String>,
    count: Arc<AtomicUsize>,
    max_events: usize,
    done: Arc<AtomicBool>,
}

struct HandlerContext {
    filter: Option<String>,
    count: Arc<AtomicUsize>,
    max_events: usize,
    done: Arc<AtomicBool>,
}

impl ListenContext {
    fn new(filter: Option<String>, max_events: usize) -> Self {
        Self {
            filter,
            count: Arc::new(AtomicUsize::new(0)),
            max_events,
            done: Arc::new(AtomicBool::new(false)),
        }
    }

    fn handler_ctx(&self) -> HandlerContext {
        HandlerContext {
            filter: self.filter.clone(),
            count: Arc::clone(&self.count),
            max_events: self.max_events,
            done: Arc::clone(&self.done),
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
                log_event(&ctx, "window", || {
                    println!("[WINDOW] Active window changed - {data:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_opened_handler(move |data| {
                log_event(&ctx, "window", || {
                    println!("[WINDOW] Window opened - {data:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_closed_handler(move |data| {
                log_event(&ctx, "window", || {
                    println!("[WINDOW] Window closed - {data:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_moved_handler(move |data| {
                log_event(&ctx, "window", || {
                    println!(
                        "[WINDOW] Window moved - workspace: {}, address: {}",
                        data.workspace_name, data.window_address
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_float_state_changed_handler(move |data| {
                log_event(&ctx, "float", || {
                    println!(
                        "[FLOAT] Float state changed - address: {}, floating: {}",
                        data.address, data.floating
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_fullscreen_state_changed_handler(move |state| {
                log_event(&ctx, "fullscreen", || {
                    let state_str = if state { "enabled" } else { "disabled" };
                    println!("[FULLSCREEN] Fullscreen {state_str}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_workspace_changed_handler(move |id| {
                log_event(&ctx, "workspace", || {
                    println!("[WORKSPACE] Changed workspace - {id:?}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_workspace_added_handler(move |data| {
                log_event(&ctx, "workspace", || {
                    println!("[WORKSPACE] Workspace added - name: {}, id: {}", data.name, data.id);
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_workspace_deleted_handler(move |data| {
                log_event(&ctx, "workspace", || {
                    println!(
                        "[WORKSPACE] Workspace deleted - name: {}, id: {}",
                        data.name, data.id
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_active_monitor_changed_handler(move |data| {
                log_event(&ctx, "monitor", || {
                    println!(
                        "[MONITOR] Active monitor changed - monitor: {}, workspace: {:?}",
                        data.monitor_name, data.workspace_name
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_layout_changed_handler(move |data| {
                log_event(&ctx, "layout", || {
                    println!(
                        "[LAYOUT] Layout changed - keyboard: {}, layout: {}",
                        data.keyboard_name, data.layout_name
                    );
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_group_toggled_handler(move |data| {
                log_event(&ctx, "group", || {
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
                log_event(&ctx, "group", || {
                    println!("[GROUP] Window moved into group - address: {addr}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_window_moved_out_of_group_handler(move |addr| {
                log_event(&ctx, "group", || {
                    println!("[GROUP] Window moved out of group - address: {addr}");
                });
            });
        });

        self.with_handler(listener, |ctx, listener| {
            listener.add_config_reloaded_handler(move || {
                log_event(&ctx, "config", || {
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

/// Prints one event and marks the context as done once the limit is reached.
///
/// Setting `done` rather than exiting the process lets `listen` return through
/// `start_listener_while`, so the socket is closed and every destructor runs.
fn log_event<F>(ctx: &HandlerContext, event_type: &str, f: F)
where
    F: FnOnce(),
{
    if should_log_event(event_type, &ctx.filter) {
        f();
        if increment_count(&ctx.count, ctx.max_events) {
            println!("Maximum event count ({}) reached. Stopping.", ctx.max_events);
            ctx.done.store(true, Ordering::SeqCst);
        }
    }
}

/// Counts one event and reports whether the limit has just been reached.
///
/// Returns `false` when no limit is set.
fn increment_count(count: &Arc<AtomicUsize>, max: usize) -> bool {
    if max == 0 {
        return false;
    }

    count.fetch_add(1, Ordering::SeqCst) + 1 == max
}
