use std::ffi::OsString;
use std::path::PathBuf;

use crate::error::Result;
use crate::{RestartPolicy, ServiceConfig, ServiceLabel};

/// Fluent builder for constructing a [`ServiceConfig`].
pub struct ServiceBuilder {
    label: ServiceLabel,
    program: Option<PathBuf>,
    args: Vec<OsString>,
    working_directory: Option<PathBuf>,
    environment: Vec<(String, String)>,
    username: Option<String>,
    description: Option<String>,
    autostart: bool,
    restart_policy: RestartPolicy,
    stdout_file: Option<PathBuf>,
    stderr_file: Option<PathBuf>,
    contents: Option<String>,
}

impl ServiceBuilder {
    pub fn new(label: impl AsRef<str>) -> Result<Self> {
        let label: ServiceLabel = label.as_ref().parse()?;
        Ok(Self {
            label,
            program: None,
            args: Vec::new(),
            working_directory: None,
            environment: Vec::new(),
            username: None,
            description: None,
            autostart: false,
            restart_policy: RestartPolicy::default(),
            stdout_file: None,
            stderr_file: None,
            contents: None,
        })
    }

    pub fn program(mut self, path: impl Into<PathBuf>) -> Self {
        self.program = Some(path.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn working_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(path.into());
        self
    }

    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.push((key.into(), value.into()));
        self
    }

    pub fn username(mut self, name: impl Into<String>) -> Self {
        self.username = Some(name.into());
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn autostart(mut self, enabled: bool) -> Self {
        self.autostart = enabled;
        self
    }

    pub fn restart_policy(mut self, policy: RestartPolicy) -> Self {
        self.restart_policy = policy;
        self
    }

    pub fn restart_on_failure(mut self, delay_secs: u32, max_retries: u32) -> Self {
        self.restart_policy = RestartPolicy::OnFailure {
            delay_secs: Some(delay_secs),
            max_retries: Some(max_retries),
            reset_after_secs: None,
        };
        self
    }

    /// Set a single log file for both stdout and stderr.
    pub fn log(mut self, path: impl Into<PathBuf>) -> Self {
        let p = path.into();
        self.stdout_file = Some(p);
        self.stderr_file = None;
        self
    }

    /// Set stdout log file path.
    pub fn stdout_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.stdout_file = Some(path.into());
        self
    }

    /// Set stderr log file path.
    pub fn stderr_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.stderr_file = Some(path.into());
        self
    }

    pub fn contents(mut self, raw: impl Into<String>) -> Self {
        self.contents = Some(raw.into());
        self
    }

    pub fn build(self) -> Result<ServiceConfig> {
        let program = self.program.ok_or_else(|| {
            crate::Error::InvalidLabel("program path is required".into())
        })?;

        // Default log: {working_directory}/logs/{label}.log
        let stdout_file = self.stdout_file.or_else(|| {
            self.working_directory.as_ref().map(|wd| {
                wd.join("logs").join(format!("{}.log", self.label.to_script_name()))
            })
        });

        Ok(ServiceConfig {
            label: self.label,
            program,
            args: self.args,
            working_directory: self.working_directory,
            environment: self.environment,
            username: self.username,
            description: self.description,
            autostart: self.autostart,
            restart_policy: self.restart_policy,
            stdout_file,
            stderr_file: self.stderr_file,
            contents: self.contents,
        })
    }
}