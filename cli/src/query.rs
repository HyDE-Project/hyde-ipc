use crate::flags::Query;
use hyprland::data::CursorPosition;
use hyprland::prelude::*;
use std::io::{Write, stdout};
use std::thread;
use std::time::Duration;

pub fn run_query(command: Query) {
    match command {
        Query::CursorPos { watch } => {
            if watch {
                loop {
                    match CursorPosition::get() {
                        Ok(pos) => {
                            print!("\r\x1b[Kx: {}, y: {}", pos.x, pos.y);
                            stdout().flush().unwrap();
                        },
                        Err(e) => {
                            eprintln!("Error: {e}");
                            break;
                        },
                    }
                    thread::sleep(Duration::from_millis(150));
                }
            } else {
                match CursorPosition::get() {
                    Ok(pos) => {
                        println!("x: {}, y: {}", pos.x, pos.y);
                    },
                    Err(e) => {
                        eprintln!("Error: {e}");
                    },
                }
            }
        },
    }
}
