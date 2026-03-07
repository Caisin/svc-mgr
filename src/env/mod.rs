//! Cross-platform environment variable management.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(not(target_os = "windows"))]
mod unix;

use std::collections::HashMap;

use crate::error::Result;

/// Scope of environment variable (user or system).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvScope {
    User,
    System,
}

/// Environment variable manager trait.
pub trait EnvManager {
    /// List all environment variables in the given scope.
    fn list(&self, scope: EnvScope) -> Result<HashMap<String, String>>;

    /// Get a specific environment variable value.
    fn get(&self, scope: EnvScope, key: &str) -> Result<Option<String>>;

    /// Set an environment variable.
    fn set(&self, scope: EnvScope, key: &str, value: &str) -> Result<()>;

    /// Remove an environment variable.
    fn unset(&self, scope: EnvScope, key: &str) -> Result<()>;
}

/// Get the platform-specific environment manager.
pub fn manager() -> Box<dyn EnvManager> {
    #[cfg(target_os = "windows")]
    {
        Box::new(windows::WindowsEnvManager::new())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Box::new(unix::UnixEnvManager::new())
    }
}
