use crate::action::ServiceAction;
use crate::error::Result;
use crate::kind::ServiceManagerKind;
use crate::{ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager};

/// A service manager that dispatches to the appropriate platform backend.
pub enum TypedServiceManager {
    Launchd(crate::platform::launchd::LaunchdServiceManager),

    Systemd(crate::platform::systemd::SystemdServiceManager),

    OpenRc(crate::platform::openrc::OpenRcServiceManager),

    Rcd(crate::platform::rcd::RcdServiceManager),

    Sc(crate::platform::sc::ScServiceManager),

    WinSw(crate::platform::winsw::WinSwServiceManager),
}

impl TypedServiceManager {
    /// Create a manager for the given kind.
    pub fn target(kind: ServiceManagerKind) -> Result<Self> {
        match kind {
            ServiceManagerKind::Launchd => Ok(Self::Launchd(
                crate::platform::launchd::LaunchdServiceManager::system(),
            )),
            ServiceManagerKind::Systemd => Ok(Self::Systemd(
                crate::platform::systemd::SystemdServiceManager::system(),
            )),
            ServiceManagerKind::OpenRc => {
                Ok(Self::OpenRc(crate::platform::openrc::OpenRcServiceManager::new()))
            }
            ServiceManagerKind::Rcd => {
                Ok(Self::Rcd(crate::platform::rcd::RcdServiceManager::new()))
            }
            ServiceManagerKind::Sc => {
                Ok(Self::Sc(crate::platform::sc::ScServiceManager::new()))
            }
            ServiceManagerKind::WinSw => Ok(Self::WinSw(
                crate::platform::winsw::WinSwServiceManager::new(),
            )),
        }
    }

    /// Create a manager for the native platform.
    pub fn native() -> Result<Self> {
        ServiceManagerKind::native().and_then(Self::target)
    }
}

/// Dispatch macro to avoid repetitive match arms.
macro_rules! dispatch {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            TypedServiceManager::Launchd(m) => m.$method($($arg),*),
            TypedServiceManager::Systemd(m) => m.$method($($arg),*),
            TypedServiceManager::OpenRc(m) => m.$method($($arg),*),
            TypedServiceManager::Rcd(m) => m.$method($($arg),*),
            TypedServiceManager::Sc(m) => m.$method($($arg),*),
            TypedServiceManager::WinSw(m) => m.$method($($arg),*),
        }
    };
}

macro_rules! dispatch_mut {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            TypedServiceManager::Launchd(m) => m.$method($($arg),*),
            TypedServiceManager::Systemd(m) => m.$method($($arg),*),
            TypedServiceManager::OpenRc(m) => m.$method($($arg),*),
            TypedServiceManager::Rcd(m) => m.$method($($arg),*),
            TypedServiceManager::Sc(m) => m.$method($($arg),*),
            TypedServiceManager::WinSw(m) => m.$method($($arg),*),
        }
    };
}

impl ServiceManager for TypedServiceManager {
    fn available(&self) -> Result<bool> {
        dispatch!(self, available)
    }

    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction> {
        dispatch!(self, install, config)
    }

    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, uninstall, label)
    }

    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, start, label)
    }

    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, stop, label)
    }

    fn enable(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, enable, label)
    }

    fn disable(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, disable, label)
    }

    fn restart(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, restart, label)
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, status, label)
    }

    fn info(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, info, label)
    }

    fn list(&self) -> Result<ServiceAction> {
        dispatch!(self, list)
    }

    fn level(&self) -> ServiceLevel {
        dispatch!(self, level)
    }

    fn set_level(&mut self, level: ServiceLevel) -> Result<()> {
        dispatch_mut!(self, set_level, level)
    }
}

#[cfg(test)]
mod tests {
    use super::TypedServiceManager;
    use crate::{
        RestartPolicy, ServiceConfig, ServiceLabel, ServiceManager, ServiceManagerKind,
    };
    use std::path::PathBuf;

    fn sample_config(label: &str) -> ServiceConfig {
        ServiceConfig {
            label: ServiceLabel::new(label),
            program: PathBuf::from("/usr/bin/demo"),
            args: Vec::new(),
            working_directory: None,
            environment: Vec::new(),
            username: None,
            description: Some("demo".to_string()),
            autostart: true,
            restart_policy: RestartPolicy::default(),
            stdout_file: None,
            stderr_file: None,
            contents: None,
        }
    }

    #[test]
    fn test_target_systemd_available_for_remote_command_generation() {
        let manager = TypedServiceManager::target(ServiceManagerKind::Systemd).unwrap();
        let action = manager.status(&ServiceLabel::new("demo")).unwrap();
        let commands = action.commands();
        assert_eq!(commands.len(), 1);
        assert!(commands[0].contains("systemctl status demo"));
    }

    #[test]
    fn test_target_openrc_available_for_remote_command_generation() {
        let manager = TypedServiceManager::target(ServiceManagerKind::OpenRc).unwrap();
        let action = manager.restart(&ServiceLabel::new("demo")).unwrap();
        let commands = action.commands();
        assert_eq!(commands.len(), 2);
        assert!(commands[0].contains("rc-service demo stop"));
        assert!(commands[1].contains("rc-service demo start"));
    }

    #[test]
    fn test_target_other_platform_managers_available_for_remote_command_generation() {
        TypedServiceManager::target(ServiceManagerKind::Launchd).unwrap();
        TypedServiceManager::target(ServiceManagerKind::Rcd).unwrap();
        TypedServiceManager::target(ServiceManagerKind::Sc).unwrap();
        TypedServiceManager::target(ServiceManagerKind::WinSw).unwrap();
    }

    #[test]
    fn test_target_launchd_install_generates_launchctl_commands() {
        let manager = TypedServiceManager::target(ServiceManagerKind::Launchd).unwrap();
        let action = manager.install(&sample_config("com.example.demo")).unwrap();
        let commands = action.commands();
        assert!(commands.iter().any(|cmd| cmd.contains("launchctl")));
    }

    #[test]
    fn test_target_winsw_install_generates_winsw_commands() {
        let manager = TypedServiceManager::target(ServiceManagerKind::WinSw).unwrap();
        let action = manager.install(&sample_config("demo")).unwrap();
        let commands = action.commands();
        assert!(commands.iter().any(|cmd| cmd.contains("winsw")));
    }
}
