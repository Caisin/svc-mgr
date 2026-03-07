use crate::error::Result;
#[cfg(any(target_os = "linux", not(any(
    target_os = "macos",
    target_os = "windows",
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "linux"
))))]
use crate::error::Error;

/// Enumeration of supported service manager backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceManagerKind {
    Launchd,
    Systemd,
    OpenRc,
    Rcd,
    Sc,
    WinSw,
}

impl ServiceManagerKind {
    /// Detect the native service manager for the current platform.
    pub fn native() -> Result<Self> {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                Ok(Self::Launchd)
            } else if #[cfg(target_os = "windows")] {
                if which::which("winsw").is_ok() {
                    Ok(Self::WinSw)
                } else {
                    Ok(Self::Sc)
                }
            } else if #[cfg(any(
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "openbsd",
                target_os = "netbsd"
            ))] {
                Ok(Self::Rcd)
            } else if #[cfg(target_os = "linux")] {
                if which::which("systemctl").is_ok() {
                    Ok(Self::Systemd)
                } else if which::which("rc-service").is_ok() {
                    Ok(Self::OpenRc)
                } else {
                    Err(Error::NoNativeManager)
                }
            } else {
                Err(Error::NoNativeManager)
            }
        }
    }
}
