pub mod unit;

use std::path::PathBuf;

use crate::action::{ActionOutput, CmdOutput, ServiceAction};
use crate::error::{Error, Result};
use crate::{ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager, ServiceStatus};

use self::unit::SystemdUnit;

pub struct SystemdServiceManager {
    user: bool,
}

impl SystemdServiceManager {
    pub fn system() -> Self {
        Self { user: false }
    }

    pub fn user() -> Self {
        Self { user: true }
    }

    fn service_dir(&self) -> Result<PathBuf> {
        if self.user {
            let config = dirs::config_dir().ok_or_else(|| {
                Error::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "config directory not found",
                ))
            })?;
            Ok(config.join("systemd/user"))
        } else {
            Ok(PathBuf::from("/etc/systemd/system"))
        }
    }

    fn unit_path(&self, label: &ServiceLabel) -> Result<PathBuf> {
        let dir = self.service_dir()?;
        Ok(dir.join(format!("{}.service", label.to_script_name())))
    }

    fn systemctl_args(&self, args: &[&str]) -> Vec<String> {
        let mut cmd_args: Vec<String> = Vec::new();
        if self.user {
            cmd_args.push("--user".to_string());
        }
        cmd_args.extend(args.iter().map(|s| s.to_string()));
        cmd_args
    }
}

impl ServiceManager for SystemdServiceManager {
    fn available(&self) -> Result<bool> {
        Ok(which::which("systemctl").is_ok())
    }

    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction> {
        let path = self.unit_path(&config.label)?;
        let script_name = config.label.to_script_name();
        let data = if let Some(contents) = &config.contents {
            contents.clone()
        } else {
            let unit = SystemdUnit::from_config(config, self.user);
            unit.render()
        };
        let reload_args = self.systemctl_args(&["daemon-reload"]);
        let mut action = ServiceAction::new()
            .write_file(&path, data.into_bytes(), 0o644)
            .cmd_ignore_error("systemctl", &reload_args);
        if config.autostart {
            let enable_args = self.systemctl_args(&["enable", &script_name]);
            action = action.cmd("systemctl", &enable_args);
        }
        Ok(action)
    }

    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = self.unit_path(label)?;
        let script_name = label.to_script_name();
        let disable_args = self.systemctl_args(&["disable", &script_name]);
        let reload_args = self.systemctl_args(&["daemon-reload"]);
        Ok(ServiceAction::new()
            .cmd_ignore_error("systemctl", &disable_args)
            .remove_file(&path)
            .cmd_ignore_error("systemctl", &reload_args))
    }

    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        let args = self.systemctl_args(&["start", &script_name]);
        Ok(ServiceAction::new().cmd("systemctl", &args))
    }

    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        let args = self.systemctl_args(&["stop", &script_name]);
        Ok(ServiceAction::new().cmd("systemctl", &args))
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        let args = self.systemctl_args(&["status", &script_name]);
        Ok(ServiceAction::new()
            .cmd_ignore_error("systemctl", &args)
            .with_parser(|outputs: &[CmdOutput]| {
                let out = outputs.last();
                match out {
                    None => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                    Some(o) => match o.exit_code {
                        Some(0) => Ok(ActionOutput::Status(ServiceStatus::Running)),
                        Some(3) => Ok(ActionOutput::Status(ServiceStatus::Stopped(None))),
                        Some(4) => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                        _ => Ok(ActionOutput::Status(ServiceStatus::Stopped(
                            Some(o.stderr.clone()),
                        ))),
                    },
                }
            }))
    }

    fn level(&self) -> ServiceLevel {
        if self.user {
            ServiceLevel::User
        } else {
            ServiceLevel::System
        }
    }

    fn set_level(&mut self, level: ServiceLevel) -> Result<()> {
        self.user = level == ServiceLevel::User;
        Ok(())
    }

    fn list(&self) -> Result<ServiceAction> {
        let args = self.systemctl_args(&[
            "list-unit-files",
            "--type=service",
            "--no-legend",
            "--no-pager",
        ]);
        Ok(ServiceAction::new()
            .cmd_ignore_error("systemctl", &args)
            .with_parser(|outputs: &[CmdOutput]| {
                let mut services = Vec::new();
                if let Some(out) = outputs.last() {
                    for line in out.stdout.lines() {
                        if let Some(name) = line.split_whitespace().next() {
                            let name = name.trim_end_matches(".service");
                            if !name.is_empty() {
                                services.push(name.to_string());
                            }
                        }
                    }
                }
                Ok(ActionOutput::List(services))
            }))
    }
}
