use crate::flags::Dispatch as DispatchCmd;
use crate::parsers::ParsedWindowIdentifier;
use crate::reaction_handler::{EventType, Reaction, ReactionManager};
use hyprland::shared::HyprError;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

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

    let event_type =
        EventType::from_event_and_subtype(&event, subtype.as_deref()).map_err(HyprError::Other)?;

    let window_filter = filter
        .as_deref()
        .map(ParsedWindowIdentifier::from_str)
        .transpose()
        .map_err(HyprError::Other)?
        .map(|p| p.0);

    let reaction = Reaction {
        event_type,
        dispatchers: vec![dispatch.into()],
        window_filter,
        max_count: if max_reactions > 0 { Some(max_reactions) } else { None },
        name: None,
        description: None,
        counter: Arc::new(AtomicUsize::new(0)),
    };

    let mut manager = ReactionManager::new();
    manager.add_reaction(reaction);
    manager
        .start()
        .map_err(HyprError::Other)
}
