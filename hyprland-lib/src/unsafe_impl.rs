/// This module provides unsafe Send/Sync implementations for event listener types.
///
/// # Safety
///
/// These implementations are provided for types that interact with the Hyprland IPC socket.
/// The safety guarantees are based on the following assumptions:
///
/// 1. Event listener types use internal synchronization (mutexes, atomics) where needed
/// 2. The Hyprland IPC socket API is thread-safe for concurrent reads/writes
/// 3. Event data structures contain only owned data or safe references
///
/// # Thread Safety Guarantees
///
/// - Event listeners can be safely sent between threads (Send)
/// - Event listeners can be safely shared between threads with synchronization (Sync)
/// - Event data structures contain no raw pointers or thread-local state
///
/// # Warnings
///
/// Do not use these implementations unless you understand the threading model
/// of the Hyprland IPC protocol and event system.

/// Unsafe Send/Sync implementations for event listener structs
#[cfg(feature = "listener")]
pub mod listeners {
    use crate::event_listener::*;

    /// SAFETY: AsyncEventListener contains Arc/Mutex for internal state synchronization
    /// and uses tokio's async runtime which handles thread safety.
    unsafe impl Send for AsyncEventListener {
    }

    /// SAFETY: AsyncEventListener's internal Arc/Mutex make it safe to share across threads.
    unsafe impl Sync for AsyncEventListener {
    }

    /// SAFETY: EventListener uses internal synchronization for socket access
    /// and does not contain thread-local state.
    unsafe impl Send for EventListener {
    }

    /// SAFETY: EventListener's socket operations are synchronized internally.
    unsafe impl Sync for EventListener {
    }

    /// SAFETY: WindowMoveEvent contains only owned String and primitive types.
    unsafe impl Send for WindowMoveEvent {
    }

    /// SAFETY: WindowMoveEvent has no interior mutability.
    unsafe impl Sync for WindowMoveEvent {
    }

    /// SAFETY: WindowOpenEvent contains only owned String and primitive types.
    unsafe impl Send for WindowOpenEvent {
    }

    /// SAFETY: WindowOpenEvent has no interior mutability.
    unsafe impl Sync for WindowOpenEvent {
    }

    /// SAFETY: LayoutEvent contains only owned String types.
    unsafe impl Send for LayoutEvent {
    }

    /// SAFETY: LayoutEvent has no interior mutability.
    unsafe impl Sync for LayoutEvent {
    }

    /// SAFETY: State is a simple enum with no references or unsafe fields.
    unsafe impl Send for State {
    }

    /// SAFETY: State has no interior mutability.
    unsafe impl Sync for State {
    }

    /// SAFETY: WindowEventData contains only owned types (String, Address).
    unsafe impl Send for WindowEventData {
    }

    /// SAFETY: WindowEventData has no interior mutability.
    unsafe impl Sync for WindowEventData {
    }

    /// SAFETY: MonitorEventData contains only owned String types.
    unsafe impl Send for MonitorEventData {
    }

    /// SAFETY: MonitorEventData has no interior mutability.
    unsafe impl Sync for MonitorEventData {
    }

    /// SAFETY: WindowFloatEventData contains only owned types (Address, bool).
    unsafe impl Send for WindowFloatEventData {
    }

    /// SAFETY: WindowFloatEventData has no interior mutability.
    unsafe impl Sync for WindowFloatEventData {
    }
}
