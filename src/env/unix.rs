use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::env::{EnvManager, EnvScope};
use crate::error::{Error, Result};

pub struct UnixEnvManager {
    user_profile: PathBuf,
}

impl UnixEnvManager {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        // Detect shell and use appropriate profile
        let user_profile = if let Ok(shell) = std::env::var("SHELL") {
            if shell.contains("zsh") {
                home.join(".zshrc")
            } else if shell.contains("bash") {
                home.join(".bashrc")
            } else {
                home.join(".profile")
            }
        } else {
            home.join(".profile")
        };

        Self { user_profile }
    }

    fn parse_env_file(&self, path: &PathBuf) -> Result<HashMap<String, String>> {
        let mut vars = HashMap::new();

        if !path.exists() {
            return Ok(vars);
        }

        let content = fs::read_to_string(path).map_err(|e| Error::FileError {
            path: path.clone(),
            source: e,
        })?;

        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse export KEY=VALUE or KEY=VALUE
            if let Some(export_line) = line.strip_prefix("export ") {
                if let Some((key, value)) = export_line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    vars.insert(key.to_string(), value.to_string());
                }
            } else if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                if key.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    let value = value.trim().trim_matches('"').trim_matches('\'');
                    vars.insert(key.to_string(), value.to_string());
                }
            }
        }

        Ok(vars)
    }

    fn write_env_var(&self, path: &PathBuf, key: &str, value: &str) -> Result<()> {
        let mut content = if path.exists() {
            fs::read_to_string(path).map_err(|e| Error::FileError {
                path: path.clone(),
                source: e,
            })?
        } else {
            String::new()
        };

        // Remove existing entry
        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let export_pattern = format!("export {}=", key);
        let direct_pattern = format!("{}=", key);

        for line in lines {
            let trimmed = line.trim();
            if !trimmed.starts_with(&export_pattern) && !trimmed.starts_with(&direct_pattern) {
                new_lines.push(line.to_string());
            }
        }

        // Add new entry
        let new_entry = format!("export {}=\"{}\"", key, value);
        new_lines.push(new_entry);
        content = new_lines.join("\n") + "\n";

        fs::write(path, content).map_err(|e| Error::FileError {
            path: path.clone(),
            source: e,
        })?;

        Ok(())
    }

    fn remove_env_var(&self, path: &PathBuf, key: &str) -> Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path).map_err(|e| Error::FileError {
            path: path.clone(),
            source: e,
        })?;

        let lines: Vec<&str> = content.lines().collect();
        let mut new_lines = Vec::new();
        let export_pattern = format!("export {}=", key);
        let direct_pattern = format!("{}=", key);

        for line in lines {
            let trimmed = line.trim();
            if !trimmed.starts_with(&export_pattern) && !trimmed.starts_with(&direct_pattern) {
                new_lines.push(line.to_string());
            }
        }

        let content = new_lines.join("\n") + "\n";
        fs::write(path, content).map_err(|e| Error::FileError {
            path: path.clone(),
            source: e,
        })?;

        Ok(())
    }
}

impl EnvManager for UnixEnvManager {
    fn list(&self, scope: EnvScope) -> Result<HashMap<String, String>> {
        match scope {
            EnvScope::User => self.parse_env_file(&self.user_profile),
            EnvScope::System => {
                let system_env = PathBuf::from("/etc/environment");
                self.parse_env_file(&system_env)
            }
        }
    }

    fn get(&self, scope: EnvScope, key: &str) -> Result<Option<String>> {
        let vars = self.list(scope)?;
        Ok(vars.get(key).cloned())
    }

    fn set(&self, scope: EnvScope, key: &str, value: &str) -> Result<()> {
        let path = match scope {
            EnvScope::User => self.user_profile.clone(),
            EnvScope::System => PathBuf::from("/etc/environment"),
        };

        self.write_env_var(&path, key, value)
    }

    fn unset(&self, scope: EnvScope, key: &str) -> Result<()> {
        let path = match scope {
            EnvScope::User => self.user_profile.clone(),
            EnvScope::System => PathBuf::from("/etc/environment"),
        };

        self.remove_env_var(&path, key)
    }
}
