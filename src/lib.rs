pub mod action;
pub mod builder;
pub mod error;
pub mod kind;
pub mod label;
pub mod platform;
pub mod typed;
pub mod utils;

use std::ffi::OsString;
use std::path::PathBuf;

pub use action::{ActionOutput, ActionStep, CmdOutput, ServiceAction};
pub use builder::ServiceBuilder;
pub use error::{Error, Result};
pub use kind::ServiceManagerKind;
pub use label::ServiceLabel;
pub use typed::TypedServiceManager;

/// The level at which a service is managed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServiceLevel {
    System,
    User,
}

/// Current status of a service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceStatus {
    NotInstalled,
    Running,
    Stopped(Option<String>),
}

/// Policy for restarting a service after exit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartPolicy {
    Never,
    Always {
        delay_secs: Option<u32>,
    },
    OnFailure {
        delay_secs: Option<u32>,
        max_retries: Option<u32>,
        reset_after_secs: Option<u32>,
    },
    OnSuccess {
        delay_secs: Option<u32>,
    },
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self::OnFailure {
            delay_secs: None,
            max_retries: None,
            reset_after_secs: None,
        }
    }
}

/// Unified service configuration for install operations.
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub label: ServiceLabel,
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub working_directory: Option<PathBuf>,
    pub environment: Vec<(String, String)>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub autostart: bool,
    pub restart_policy: RestartPolicy,
    /// Stdout log file path.
    pub stdout_file: Option<PathBuf>,
    /// Stderr log file path. If None and stdout_file is set, stderr goes to stdout_file too.
    pub stderr_file: Option<PathBuf>,
    /// If set, use this raw content as the service file instead of generating one.
    pub contents: Option<String>,
}

impl ServiceConfig {
    /// Iterator over program + args as OsStr references.
    pub fn cmd_iter(&self) -> impl Iterator<Item = &std::ffi::OsStr> {
        let prog = std::iter::once(self.program.as_os_str());
        prog.chain(self.args.iter().map(|a| a.as_os_str()))
    }
}

/// Core trait for platform service managers.
///
/// All action methods return a [`ServiceAction`] that can be:
/// - Executed locally with `.exec()`
/// - Previewed with `.commands()`
/// - Parsed from remote outputs with `.parse()`
pub trait ServiceManager {
    /// Check if this service manager is available on the current system.
    fn available(&self) -> Result<bool>;

    /// Install a service with the given configuration.
    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction>;

    /// Uninstall a service by label.
    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction>;

    /// Start a service by label.
    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction>;

    /// Stop a service by label.
    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction>;

    /// Restart a service (default: stop + start).
    fn restart(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let stop_action = self.stop(label)?;
        let start_action = self.start(label)?;
        Ok(stop_action.merge(start_action))
    }

    /// Query the current status of a service.
    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction>;

    /// List installed services.
    fn list(&self) -> Result<ServiceAction>;

    /// Get the current service level.
    fn level(&self) -> ServiceLevel;

    /// Set the service level (system vs user).
    fn set_level(&mut self, level: ServiceLevel) -> Result<()>;
}
