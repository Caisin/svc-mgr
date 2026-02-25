use std::collections::BTreeMap;

use crate::{RestartPolicy, ServiceConfig};

/// Typed representation of a macOS launchd plist file.
#[derive(Debug, Clone)]
pub struct LaunchdPlist {
    pub label: String,
    pub program_arguments: Vec<String>,
    pub keep_alive: Option<KeepAlive>,
    pub run_at_load: bool,
    pub user_name: Option<String>,
    pub working_directory: Option<String>,
    pub environment_variables: BTreeMap<String, String>,
    pub disabled: bool,
}

#[derive(Debug, Clone)]
pub enum KeepAlive {
    Bool(bool),
    SuccessfulExit(bool),
}

impl LaunchdPlist {
    /// Build a plist from a generic ServiceConfig.
    pub fn from_config(config: &ServiceConfig) -> Self {
        let program_arguments: Vec<String> = std::iter::once(
            config.program.to_string_lossy().into_owned(),
        )
        .chain(config.args.iter().map(|a| a.to_string_lossy().into_owned()))
        .collect();

        let (keep_alive, disabled) = match &config.restart_policy {
            RestartPolicy::Never => (None, false),
            RestartPolicy::Always { .. } => (Some(KeepAlive::Bool(true)), !config.autostart),
            RestartPolicy::OnFailure { .. } => {
                (Some(KeepAlive::SuccessfulExit(false)), !config.autostart)
            }
            RestartPolicy::OnSuccess { .. } => {
                (Some(KeepAlive::SuccessfulExit(true)), !config.autostart)
            }
        };

        let mut env = BTreeMap::new();
        for (k, v) in &config.environment {
            env.insert(k.clone(), v.clone());
        }

        Self {
            label: config.label.to_qualified_name(),
            program_arguments,
            keep_alive,
            run_at_load: config.autostart,
            user_name: config.username.clone(),
            working_directory: config
                .working_directory
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned()),
            environment_variables: env,
            disabled,
        }
    }

    /// Render to plist XML using the `plist` crate.
    #[cfg(target_os = "macos")]
    pub fn render(&self) -> crate::Result<Vec<u8>> {
        use plist::{Dictionary, Value};

        let mut dict = Dictionary::new();
        dict.insert("Label".into(), Value::String(self.label.clone()));

        let args: Vec<Value> = self
            .program_arguments
            .iter()
            .map(|s| Value::String(s.clone()))
            .collect();
        dict.insert("ProgramArguments".into(), Value::Array(args));

        if let Some(ka) = &self.keep_alive {
            match ka {
                KeepAlive::Bool(b) => {
                    dict.insert("KeepAlive".into(), Value::Boolean(*b));
                }
                KeepAlive::SuccessfulExit(b) => {
                    let mut ka_dict = Dictionary::new();
                    ka_dict.insert("SuccessfulExit".into(), Value::Boolean(*b));
                    dict.insert("KeepAlive".into(), Value::Dictionary(ka_dict));
                }
            }
        }

        if self.run_at_load {
            dict.insert("RunAtLoad".into(), Value::Boolean(true));
        }

        if let Some(user) = &self.user_name {
            dict.insert("UserName".into(), Value::String(user.clone()));
        }

        if let Some(wd) = &self.working_directory {
            dict.insert("WorkingDirectory".into(), Value::String(wd.clone()));
        }

        if !self.environment_variables.is_empty() {
            let mut env_dict = Dictionary::new();
            for (k, v) in &self.environment_variables {
                env_dict.insert(k.clone(), Value::String(v.clone()));
            }
            dict.insert(
                "EnvironmentVariables".into(),
                Value::Dictionary(env_dict),
            );
        }

        if self.disabled {
            dict.insert("Disabled".into(), Value::Boolean(true));
        }

        let mut buf = Vec::new();
        Value::Dictionary(dict)
            .to_writer_xml(&mut buf)
            .map_err(|e| crate::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(buf)
    }
}
