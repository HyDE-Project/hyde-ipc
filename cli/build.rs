use clap::{Command, CommandFactory};
use clap_complete::{Shell, generate_to};
use std::path::Path;
use std::{env, fs};

#[path = "src/flags.rs"]
mod flags;

fn generate_completions(out_dir: &Path) {
    let mut cmd: Command = flags::Cli::command();
    cmd.set_bin_name("hyde-ipc");

    fs::create_dir_all(out_dir).unwrap();

    for shell in [
        Shell::Bash,
        Shell::Elvish,
        Shell::Fish,
        Shell::PowerShell,
        Shell::Zsh,
    ] {
        generate_to(shell, &mut cmd, "hyde-ipc", out_dir).unwrap();
    }
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("completions");

    generate_completions(&dest_path);
}
