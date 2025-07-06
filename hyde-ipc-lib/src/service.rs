use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceLevel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use std::error::Error;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fmt};

#[derive(Debug)]
pub enum ServiceError {
    Manager(String),
    Install(String),
    Uninstall(String),
    Start(String),
    Stop(String),
    Status(String),
    Io(std::io::Error),
    UserLevel(String),
    Config(String),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::Manager(e) => write!(f, "Failed to get service manager: {}", e),
            ServiceError::Install(e) => write!(f, "Failed to install service: {}", e),
            ServiceError::Uninstall(e) => write!(f, "Failed to uninstall service: {}", e),
            ServiceError::Start(e) => write!(f, "Failed to start service: {}", e),
            ServiceError::Stop(e) => write!(f, "Failed to stop service: {}", e),
            ServiceError::Status(e) => write!(f, "Failed to get service status: {}", e),
            ServiceError::Io(e) => write!(f, "IO error: {}", e),
            ServiceError::UserLevel(e) => write!(f, "Failed to set user level: {}", e),
            ServiceError::Config(e) => write!(f, "Failed to get config path: {}", e),
        }
    }
}

impl Error for ServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ServiceError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(err: std::io::Error) -> Self {
        ServiceError::Io(err)
    }
}

pub type Result<T> = std::result::Result<T, ServiceError>;

fn get_manager() -> Result<Box<dyn ServiceManager>> {
    let mut manager =
        <dyn ServiceManager>::native().map_err(|e| ServiceError::Manager(e.to_string()))?;
    manager
        .set_level(ServiceLevel::User)
        .map_err(|e| ServiceError::UserLevel(e.to_string()))?;
    Ok(manager)
}

fn get_label() -> ServiceLabel {
    ServiceLabel { qualifier: None, organization: None, application: String::from("hyde-ipc") }
}

/// Returns the path to the hyde-ipc config file.
///
/// The path is `~/.local/share/hyde-ipc/config.toml`.
pub fn get_config_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| ServiceError::Config("Could not get user's data directory".to_string()))?;
    let mut path = data_dir;
    path.push("hyde-ipc");
    path.push("config.toml");
    Ok(path)
}

/// Installs and starts the hyde-ipc service.
pub fn install() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

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

    let config_path: OsString = get_config_path()?
        .into_os_string()
        .into();

    manager
        .install(ServiceInstallCtx {
            label: label.clone(),
            program: hyde_ipc_path.into(),
            args: vec!["react".into(), "-c".into(), config_path],
            contents: None,
            username: None,
            working_directory: None,
            environment: None,
            autostart: true,
            disable_restart_on_failure: false,
        })
        .map_err(|e| ServiceError::Install(e.to_string()))?;

    println!("Service installed successfully.");
    start()
}

/// Uninstalls the hyde-ipc service.
pub fn uninstall() -> Result<()> {
    if let Err(e) = stop() {
        eprintln!("Failed to stop service during uninstall: {e}. Continuing with uninstall...");
    }

    let label = get_label();
    let manager = get_manager()?;

    manager
        .uninstall(ServiceUninstallCtx { label })
        .map_err(|e| ServiceError::Uninstall(e.to_string()))?;
    println!("Service uninstalled successfully.");
    Ok(())
}

/// Starts the hyde-ipc service.
pub fn start() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    manager
        .start(ServiceStartCtx { label })
        .map_err(|e| ServiceError::Start(e.to_string()))?;
    println!("Service started successfully.");
    Ok(())
}

/// Stops the hyde-ipc service.
pub fn stop() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    manager
        .stop(ServiceStopCtx { label })
        .map_err(|e| ServiceError::Stop(e.to_string()))?;
    println!("Service stopped successfully.");
    Ok(())
}

/// Restarts the hyde-ipc service.
pub fn restart() -> Result<()> {
    println!("Restarting service...");
    if let Err(e) = stop() {
        eprintln!("Failed to stop service during restart: {}. Continuing to start...", e);
    }
    start()
}

/// Checks if the hyde-ipc service is active.
pub fn is_active() -> Result<bool> {
    // This is a workaround. The service-manager crate does not provide a cross-platform
    // way to check if a service is running. We assume that if the start command
    // succeeds, the service is running.
    let status = Command::new("systemctl")
        .args(["--user", "is-active", "hyde-ipc.service"])
        .output()?;
    Ok(status.status.success())
}

/// Prints the status of the hyde-ipc service.
pub fn status() -> Result<()> {
    if is_active()? {
        println!("Service is running.");
    } else {
        println!("Service is not running.");
    }
    Ok(())
}

/// Follow the logs of the hyde-ipc service.
pub fn watch_logs() -> Result<()> {
    let mut child = Command::new("journalctl")
        .args(["--user", "-fu", "hyde-ipc.service"])
        .spawn()
        .map_err(|e| ServiceError::Io(e))?;

    let status = child
        .wait()
        .map_err(|e| ServiceError::Io(e))?;
    if !status.success() {
        return Err(ServiceError::Status("journalctl command failed".to_string()));
    }
    Ok(())
}
