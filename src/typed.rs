use crate::action::ServiceAction;
use crate::error::Result;
use crate::kind::ServiceManagerKind;
use crate::{Error, ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager};

/// A service manager that dispatches to the appropriate platform backend.
pub enum TypedServiceManager {
    #[cfg(target_os = "macos")]
    Launchd(crate::platform::launchd::LaunchdServiceManager),

    #[cfg(target_os = "linux")]
    Systemd(crate::platform::systemd::SystemdServiceManager),

    #[cfg(target_os = "linux")]
    OpenRc(crate::platform::openrc::OpenRcServiceManager),

    #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    Rcd(crate::platform::rcd::RcdServiceManager),

    #[cfg(target_os = "windows")]
    Sc(crate::platform::sc::ScServiceManager),

    #[cfg(target_os = "windows")]
    WinSw(crate::platform::winsw::WinSwServiceManager),
}

impl TypedServiceManager {
    /// Create a manager for the given kind.
    pub fn target(kind: ServiceManagerKind) -> Result<Self> {
        match kind {
            #[cfg(target_os = "macos")]
            ServiceManagerKind::Launchd => Ok(Self::Launchd(
                crate::platform::launchd::LaunchdServiceManager::system(),
            )),
            #[cfg(target_os = "linux")]
            ServiceManagerKind::Systemd => Ok(Self::Systemd(
                crate::platform::systemd::SystemdServiceManager::system(),
            )),
            #[cfg(target_os = "linux")]
            ServiceManagerKind::OpenRc => {
                Ok(Self::OpenRc(crate::platform::openrc::OpenRcServiceManager::new()))
            }
            #[cfg(any(
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd"
            ))]
            ServiceManagerKind::Rcd => {
                Ok(Self::Rcd(crate::platform::rcd::RcdServiceManager::new()))
            }
            #[cfg(target_os = "windows")]
            ServiceManagerKind::Sc => {
                Ok(Self::Sc(crate::platform::sc::ScServiceManager::new()))
            }
            #[cfg(target_os = "windows")]
            ServiceManagerKind::WinSw => Ok(Self::WinSw(
                crate::platform::winsw::WinSwServiceManager::new(),
            )),
            _ => Err(Error::Unsupported(format!(
                "service manager kind {:?} is not supported on this platform",
                kind
            ))),
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
            #[cfg(target_os = "macos")]
            TypedServiceManager::Launchd(m) => m.$method($($arg),*),
            #[cfg(target_os = "linux")]
            TypedServiceManager::Systemd(m) => m.$method($($arg),*),
            #[cfg(target_os = "linux")]
            TypedServiceManager::OpenRc(m) => m.$method($($arg),*),
            #[cfg(any(
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd"
            ))]
            TypedServiceManager::Rcd(m) => m.$method($($arg),*),
            #[cfg(target_os = "windows")]
            TypedServiceManager::Sc(m) => m.$method($($arg),*),
            #[cfg(target_os = "windows")]
            TypedServiceManager::WinSw(m) => m.$method($($arg),*),
        }
    };
}

macro_rules! dispatch_mut {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match $self {
            #[cfg(target_os = "macos")]
            TypedServiceManager::Launchd(m) => m.$method($($arg),*),
            #[cfg(target_os = "linux")]
            TypedServiceManager::Systemd(m) => m.$method($($arg),*),
            #[cfg(target_os = "linux")]
            TypedServiceManager::OpenRc(m) => m.$method($($arg),*),
            #[cfg(any(
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd"
            ))]
            TypedServiceManager::Rcd(m) => m.$method($($arg),*),
            #[cfg(target_os = "windows")]
            TypedServiceManager::Sc(m) => m.$method($($arg),*),
            #[cfg(target_os = "windows")]
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

    fn restart(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, restart, label)
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        dispatch!(self, status, label)
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
