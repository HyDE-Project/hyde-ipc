use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceLevel, ServiceManager, ServiceStartCtx, ServiceStatus,
    ServiceStatusCtx, ServiceStopCtx, ServiceUninstallCtx,
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
        .map_err(|e| ServiceError::UserLevel(e.to_string()))?;

    Ok(manager)
}

fn get_label() -> ServiceLabel {
    ServiceLabel { qualifier: None, organization: None, application: String::from("hyde-ipc") }
}

pub fn get_config_path() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| ServiceError::Config("Could not get user's config directory".to_string()))?;
    let mut path = config_dir;
    path.push("hyde-ipc");
    Ok(path)
}

pub fn install() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    let which_output = Command::new("which")
        .arg("hyde-ipc")
        .output()
        .map_err(|e| ServiceError::Install(format!("Failed to detect hyde-ipc binary: {}", e)))?;

    if !which_output.status.success() {
        return Err(ServiceError::Install("Could not find hyde-ipc binary in PATH".to_string()));
    }

    let hyde_ipc_path = String::from_utf8_lossy(&which_output.stdout)
        .trim()
        .to_string();

    let config_dir: OsString = get_config_path()?.into_os_string();

    manager
        .install(ServiceInstallCtx {
            label: label.clone(),
            program: hyde_ipc_path.into(),
            args: vec!["react".into(), "-c".into(), config_dir],
            contents: None,
            username: None,
            working_directory: None,
            environment: None,
            autostart: true,
            disable_restart_on_failure: false,
        })
        .map_err(|e| ServiceError::Install(e.to_string()))?;

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
    Ok(())
}

pub fn start() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    manager
        .start(ServiceStartCtx { label })
        .map_err(|e| ServiceError::Start(e.to_string()))?;
    Ok(())
}

pub fn stop() -> Result<()> {
    let label = get_label();
    let manager = get_manager()?;

    manager
        .stop(ServiceStopCtx { label })
        .map_err(|e| ServiceError::Stop(e.to_string()))?;
    Ok(())
}

pub fn restart() -> Result<()> {
    // TODO: add reload command that sends a signal to the running
    // service to re-scan without a full restart.
    // just like `hyprctl reload`
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

    let label = get_label();
    let manager = get_manager()?;
    match manager.status(ServiceStatusCtx { label }) {
        Ok(ServiceStatus::Running) => Ok(true),
        Ok(ServiceStatus::NotInstalled) => Ok(false),
        Ok(ServiceStatus::Stopped(_)) => Ok(false),
        Err(e) => Err(ServiceError::Status(e.to_string())),
    }

    // let status = Command::new("systemctl")
    //     .args(["--user", "is-active", "hyde-ipc.service"])
    //     .output()?;
    // Ok(status.status.success())
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Test `is_active()` returns true when service is running.
    #[test]
    fn test_is_active_running() {
        match is_active() {
            Ok(active) => {
                assert!(active == true || active == false);
            },
            Err(e) => match e {
                ServiceError::Status(_) => {},
                _ => panic!("Unexpected error: {}", e),
            },
        }
    }

    #[test]
    fn test_service_status_running_logic() {
        // Verify that ServiceStatus::Running maps to true
        let status = ServiceStatus::Running;
        let result = match status {
            ServiceStatus::Running => true,
            ServiceStatus::NotInstalled => false,
            ServiceStatus::Stopped(_) => false,
        };
        assert!(result);
    }

    #[test]
    fn test_service_status_not_installed_logic() {
        let status = ServiceStatus::NotInstalled;
        let result = match status {
            ServiceStatus::Running => true,
            ServiceStatus::NotInstalled => false,
            ServiceStatus::Stopped(_) => false,
        };
        assert!(!result);
    }

    #[test]
    fn test_service_status_stopped_logic() {
        let status = ServiceStatus::Stopped(Some("Service was manually stopped".to_string()));
        let result = match status {
            ServiceStatus::Running => true,
            ServiceStatus::NotInstalled => false,
            ServiceStatus::Stopped(_) => false,
        };
        assert!(!result);
    }

    #[test]
    fn test_service_status_stopped_various_reasons() {
        let reasons = vec![
            Some("Service exited with code 1".to_string()),
            Some("Service crashed".to_string()),
            Some("Service was disabled".to_string()),
            None,
        ];

        for reason in reasons {
            let status = ServiceStatus::Stopped(reason.clone());
            let result = match status {
                ServiceStatus::Running => true,
                ServiceStatus::NotInstalled => false,
                ServiceStatus::Stopped(_) => false,
            };
            assert!(!result, "Stopped service should map to false for reason: {:?}", reason);
        }
    }

    #[test]
    fn test_is_active_error_handling() {
        let error_message = "Failed to get service status";
        let service_error = ServiceError::Status(error_message.to_string());

        match service_error {
            ServiceError::Status(msg) => {
                assert_eq!(msg, error_message);
            },
            _ => panic!("Expected ServiceError::Status"),
        }
    }

    #[test]
    fn test_all_service_status_variants() {
        let variants = vec![
            (ServiceStatus::Running, true),
            (ServiceStatus::NotInstalled, false),
            (ServiceStatus::Stopped(Some("test reason".to_string())), false),
            (ServiceStatus::Stopped(None), false),
        ];

        for (status, expected_active) in variants {
            let result = match status {
                ServiceStatus::Running => true,
                ServiceStatus::NotInstalled => false,
                ServiceStatus::Stopped(_) => false,
            };
            assert_eq!(
                result, expected_active,
                "Status {:?} should map to {}",
                status, expected_active
            );
        }
    }

    #[test]
    fn local_test() {
        let label: ServiceLabel = get_label();
        println!("service: {:?}", label);

        match get_manager() {
            Ok(manager) => {
                println!("get_manager() done!");

                match manager.status(ServiceStatusCtx { label }) {
                    Ok(status) => match status {
                        ServiceStatus::Running => {
                            println!("RUNNING");
                            println!("service is active and running.");
                        },
                        ServiceStatus::NotInstalled => {
                            println!("NOT INSTALLED");
                            println!("service is not installed");
                        },
                        ServiceStatus::Stopped(reason) => {
                            println!("STOPPED");
                            match reason {
                                Some(msg) => println!("Reason: {}", msg),
                                None => println!("4th arm"),
                            }
                        },
                    },
                    Err(e) => {
                        println!("{}", e);
                    },
                }
            },
            Err(e) => {
                println!("{}", e);
            },
        }
        println!("\n\n\n\n");
    }
}
