use crate::{RestartPolicy, ServiceConfig};

/// Typed representation of a systemd unit file.
#[derive(Debug, Clone)]
pub struct SystemdUnit {
    pub unit: UnitSection,
    pub service: ServiceSection,
    pub install: InstallSection,
}

#[derive(Debug, Clone, Default)]
pub struct UnitSection {
    pub description: Option<String>,
    pub start_limit_interval_sec: Option<u32>,
    pub start_limit_burst: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct ServiceSection {
    pub exec_start: String,
    pub working_directory: Option<String>,
    pub environment: Vec<(String, String)>,
    pub restart: Option<String>,
    pub restart_sec: Option<u32>,
    pub user: Option<String>,
    pub standard_output: Option<String>,
    pub standard_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct InstallSection {
    pub wanted_by: String,
}

impl SystemdUnit {
    /// Build a unit file from a generic ServiceConfig.
    pub fn from_config(config: &ServiceConfig, user_mode: bool) -> Self {
        let exec_start = std::iter::once(config.program.to_string_lossy().into_owned())
            .chain(config.args.iter().map(|a| a.to_string_lossy().into_owned()))
            .collect::<Vec<_>>()
            .join(" ");

        let (restart, restart_sec, start_limit_burst, reset_after) =
            match &config.restart_policy {
                RestartPolicy::Never => (None, None, None, None),
                RestartPolicy::Always { delay_secs } => {
                    (Some("always".to_string()), *delay_secs, None, None)
                }
                RestartPolicy::OnFailure {
                    delay_secs,
                    max_retries,
                    reset_after_secs,
                } => (
                    Some("on-failure".to_string()),
                    *delay_secs,
                    *max_retries,
                    *reset_after_secs,
                ),
                RestartPolicy::OnSuccess { delay_secs } => {
                    (Some("on-success".to_string()), *delay_secs, None, None)
                }
            };

        let user = if user_mode {
            None
        } else {
            config.username.clone()
        };

        let wanted_by = if user_mode {
            "default.target"
        } else {
            "multi-user.target"
        };

        let stdout_path = config.stdout_file.as_ref()
            .map(|p| format!("file:{}", p.to_string_lossy()));
        let stderr_path = config.stderr_file.as_ref()
            .map(|p| format!("file:{}", p.to_string_lossy()));

        Self {
            unit: UnitSection {
                description: config.description.clone(),
                start_limit_interval_sec: reset_after,
                start_limit_burst: start_limit_burst,
            },
            service: ServiceSection {
                exec_start,
                working_directory: config
                    .working_directory
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                environment: config.environment.clone(),
                restart,
                restart_sec: restart_sec,
                user,
                standard_output: stdout_path.clone(),
                standard_error: stderr_path.or(stdout_path),
            },
            install: InstallSection {
                wanted_by: wanted_by.to_string(),
            },
        }
    }

    /// Render to systemd unit file format (INI-like).
    pub fn render(&self) -> String {
        let mut out = String::new();

        // [Unit]
        out.push_str("[Unit]\n");
        if let Some(desc) = &self.unit.description {
            out.push_str(&format!("Description={}\n", desc));
        }
        if let Some(v) = self.unit.start_limit_interval_sec {
            out.push_str(&format!("StartLimitIntervalSec={}\n", v));
        }
        if let Some(v) = self.unit.start_limit_burst {
            out.push_str(&format!("StartLimitBurst={}\n", v));
        }

        // [Service]
        out.push('\n');
        out.push_str("[Service]\n");
        if let Some(wd) = &self.service.working_directory {
            out.push_str(&format!("WorkingDirectory={}\n", wd));
        }
        for (k, v) in &self.service.environment {
            out.push_str(&format!("Environment=\"{}={}\"\n", k, v));
        }
        out.push_str(&format!("ExecStart={}\n", self.service.exec_start));
        if let Some(restart) = &self.service.restart {
            out.push_str(&format!("Restart={}\n", restart));
        }
        if let Some(sec) = self.service.restart_sec {
            out.push_str(&format!("RestartSec={}\n", sec));
        }
        if let Some(user) = &self.service.user {
            out.push_str(&format!("User={}\n", user));
        }
        if let Some(stdout) = &self.service.standard_output {
            out.push_str(&format!("StandardOutput={}\n", stdout));
        }
        if let Some(stderr) = &self.service.standard_error {
            out.push_str(&format!("StandardError={}\n", stderr));
        }

        // [Install]
        out.push('\n');
        out.push_str("[Install]\n");
        out.push_str(&format!("WantedBy={}\n", self.install.wanted_by));

        out
    }
}
