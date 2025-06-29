use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Sets up the systemd user service file for hyde-ipc, enables and restarts the service, and checks its status.
pub fn setup_service_file() {
    let username = env::var("USER").expect("Could not get $USER");
    let home = env::var("HOME").expect("Could not get $HOME");
    // Get the full path to the hyde-ipc binary using `which hyde-ipc`
    let which_output = Command::new("which")
        .arg("hyde-ipc")
        .output()
        .expect("Failed to run 'which hyde-ipc'");
    if !which_output.status.success() {
        eprintln!("Could not find hyde-ipc binary in PATH");
        std::process::exit(1);
    }
    let hyde_ipc_path = String::from_utf8_lossy(&which_output.stdout)
        .trim()
        .to_string();
    let config_path = format!("/home/{}/.local/share/hyde-ipc/config.toml", username);
    let service_content = format!(
        r#"[Unit]
            Description=hyde-ipc user service
            After=default.target

            [Service]
            ExecStart={} react -c {}
            Restart=always
            StandardOutput=journal
            StandardError=journal

            [Install]
            WantedBy=default.target
        "#,
        hyde_ipc_path, config_path
    );
    let systemd_dir = PathBuf::from(&home).join(".config/systemd/user");
    if let Err(e) = fs::create_dir_all(&systemd_dir) {
        eprintln!("Error creating systemd user dir: {}", e);
        std::process::exit(1);
    }
    let service_path = systemd_dir.join("hyde-ipc.service");
    if let Err(e) = fs::write(&service_path, service_content) {
        eprintln!("Error writing service file: {}", e);
        std::process::exit(1);
    }
    println!("Done!");
    let enable_status = Command::new("systemctl")
        .args(["--user", "enable", "hyde-ipc.service"])
        .status();
    if let Err(e) = enable_status {
        eprintln!("Error enabling hyde-ipc.service: {}", e);
        std::process::exit(1);
    }
    let restart_status = Command::new("systemctl")
        .args(["--user", "restart", "hyde-ipc.service"])
        .status();
    if let Err(e) = restart_status {
        eprintln!("Error restarting hyde-ipc.service: {}", e);
        std::process::exit(1);
    }
    let status_output = Command::new("systemctl")
        .args(["--user", "status", "hyde-ipc.service"])
        .output();
    match status_output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!(
                    "hyde-ipc.service status error:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Error checking hyde-ipc.service status: {}", e);
        }
    }
}

/// Ensures the systemd user service for hyde-ipc is set up and running. If not, runs setup_service_file().
pub fn ensure_service_setup() {
    let home = env::var("HOME").expect("Could not get $HOME");
    let systemd_dir = PathBuf::from(&home).join(".config/systemd/user");
    let service_path = systemd_dir.join("hyde-ipc.service");
    let mut needs_setup = false;
    if !service_path.exists() {
        needs_setup = true;
    }
    let enabled = Command::new("systemctl")
        .args(["--user", "is-enabled", "hyde-ipc.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !enabled {
        needs_setup = true;
    }
    let active = Command::new("systemctl")
        .args(["--user", "is-active", "hyde-ipc.service"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if !active {
        needs_setup = true;
    }
    if needs_setup {
        setup_service_file();
    }
}

/// Copies the provided config file to the user's global config location and restarts the hyde-ipc service.
///
/// # Arguments
///
/// * `config_path` - The path to the config file to copy.
pub fn copy_and_reload_config(config_path: &str) {
    let home = env::var("HOME").expect("Could not get $HOME");
    let dest_dir = PathBuf::from(&home).join(".local/share/hyde-ipc");
    let dest = dest_dir.join("config.toml");
    if let Err(e) = fs::create_dir_all(&dest_dir) {
        eprintln!("Error setting up config directory: {}", e);
        std::process::exit(1);
    }
    if let Err(e) = fs::copy(config_path, &dest) {
        eprintln!("Error reading config file: {}", e);
        std::process::exit(1);
    }
    println!("Done! {} is set as global ", config_path);
    let restart_status = Command::new("systemctl")
        .args(["--user", "restart", "hyde-ipc.service"])
        .status();
    if let Err(e) = restart_status {
        eprintln!("Error restarting hyde-ipc.service: {}", e);
        std::process::exit(1);
    }
    let status_output = Command::new("systemctl")
        .args(["--user", "status", "hyde-ipc.service"])
        .output();
    match status_output {
        Ok(output) => {
            if !output.status.success() {
                eprintln!(
                    "hyde-ipc status error:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            eprintln!("Error checking hyde-ipc status: {}", e);
        }
    }
}
