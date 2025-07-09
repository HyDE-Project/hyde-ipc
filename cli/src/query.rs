
use crate::flags::Query;
// use hyprland::data::CursorPosition;
// use hyprland::prelude::*;

use hyprland::dispatch;
use hyprland::dispatch::{
    Dispatch, DispatchType, FullscreenType,
};

pub fn run_query(command: Query) -> hyprland::Result<()> {
    match command {
        Query::CursorPos { watch } => {
            if watch {
                println!("who cares");
                Ok(())
            } else {
                dispatch!(ToggleFullscreen, FullscreenType::Maximize)?;
                Ok(())

                // .expect("failed foe some reason")

                // match CursorPosition::get() {
                //     Ok(pos) => {
                //         println!("x: {}, y: {}", pos.x, pos.y);
                //     },
                //     Err(e) => {
                //         eprintln!("Error: {e}");
                //     },
                // }
            }
        },
    }
}
