use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceLevel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

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
            ServiceError::Manager(e) => write!(f, "Failed to get service manager: {e}"),
            ServiceError::Install(e) => write!(f, "Failed to install service: {e}"),
            ServiceError::Uninstall(e) => write!(f, "Failed to uninstall service: {e}"),
            ServiceError::Start(e) => write!(f, "Failed to start service: {e}"),
            ServiceError::Stop(e) => write!(f, "Failed to stop service: {e}"),
            ServiceError::Status(e) => write!(f, "Failed to get service status: {e}"),
            ServiceError::Io(e) => write!(f, "IO error: {e}"),
            ServiceError::UserLevel(e) => write!(f, "Failed to set user level: {e}"),
            ServiceError::Config(e) => write!(f, "Failed to get config path: {e}"),
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
        .expect("Failed to install service");

    Ok(manager)
}

fn get_label() -> ServiceLabel {
    ServiceLabel { qualifier: None, organization: None, application: String::from("hyde-ipc") }
}

// FIX: redesign ?
//
pub fn get_config_path() -> Result<PathBuf> {
    let data_dir = dirs::data_dir().expect("Could not get user's data directory");
    let mut path = data_dir;
    path.push("hyde-ipc");
    path.push("config.toml");
    Ok(path)
}

pub fn install() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    // FIX: redesign needed for sure
    let which_output = Command::new("which")
        .arg("hyde-ipc")
        .output()
        .expect("Failed to detect hyde-ipc binary.");

    if !which_output.status.success() {
        eprintln!("Could not find hyde-ipc binary in PATH");
        std::process::exit(1);
    }

    let hyde_ipc_path = String::from_utf8_lossy(&which_output.stdout)
        .trim()
        .to_string();

    let config_path: OsString = get_config_path()?.into_os_string();

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

pub fn uninstall() -> Result<()> {
    if let Err(e) = stop() {
        println!("Failed to stop service during uninstall: {e}. Continuing with uninstall...");
    }

    let label = get_label();
    let manager = get_manager()?;

    manager
        .uninstall(ServiceUninstallCtx { label })
        .map_err(|e| ServiceError::Uninstall(e.to_string()))?;
    println!("Service uninstalled successfully.");
    Ok(())
}

pub fn start() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    manager
        .start(ServiceStartCtx { label })
        .map_err(|e| ServiceError::Start(e.to_string()))?;
    println!("Service started successfully.");
    Ok(())
}

pub fn stop() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    manager
        .stop(ServiceStopCtx { label })
        .map_err(|e| ServiceError::Stop(e.to_string()))?;
    println!("Service stopped successfully.");
    Ok(())
}

pub fn restart() -> Result<()> {
    println!("Restarting service...");
    if let Err(e) = stop() {
        eprintln!("Failed to stop service during restart: {e}. Continuing to start...");
    }
    start()
}

pub fn is_active() -> Result<bool> {
    // FIX: before next release:
    // This is a workaround.
    // We assume that if the start command
    // succeeds, the service is running.
    let status = Command::new("systemctl")
        .args(["--user", "is-active", "hyde-ipc.service"])
        .output()?;
    Ok(status.status.success())
}

pub fn status() -> Result<()> {
    if is_active()? {
        println!("Service is running.");
    } else {
        println!("Service is not running.");
    }
    Ok(())
}

pub fn watch_logs() -> Result<()> {
    let mut child = Command::new("journalctl")
        .args(["--user", "-fu", "hyde-ipc.service"])
        .spawn()
        .map_err(ServiceError::Io)?;

    let status = child.wait().map_err(ServiceError::Io)?;
    if !status.success() {
        return Err(ServiceError::Status("journalctl command failed".to_string()));
    }
    Ok(())
}
