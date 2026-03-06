pub mod script;

use std::path::PathBuf;

use crate::action::{ActionOutput, CmdOutput, ServiceAction};
use crate::error::{Error, Result};
use crate::{ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager, ServiceStatus};

use self::script::RcdScript;

pub struct RcdServiceManager;

impl RcdServiceManager {
    pub fn new() -> Self {
        Self
    }

    fn script_path(label: &ServiceLabel) -> PathBuf {
        PathBuf::from("/usr/local/etc/rc.d").join(label.to_script_name())
    }
}

impl ServiceManager for RcdServiceManager {
    fn available(&self) -> Result<bool> {
        Ok(PathBuf::from("/usr/local/etc/rc.d").exists())
    }

    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction> {
        let path = Self::script_path(&config.label);
        let script_name = config.label.to_script_name();
        let data = if let Some(contents) = &config.contents {
            contents.clone()
        } else {
            let script = RcdScript::from_config(config);
            script.render()
        };
        let mut action = ServiceAction::new().write_file(&path, data.into_bytes(), 0o755);
        if config.autostart {
            action = action.cmd("service", [&*script_name, "enable"]);
        }
        Ok(action)
    }

    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = Self::script_path(label);
        let script_name = label.to_script_name();
        Ok(ServiceAction::new()
            .cmd_ignore_error("service", [&*script_name, "disable"])
            .remove_file(&path))
    }

    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        Ok(ServiceAction::new().cmd("service", [&*script_name, "start"]))
    }

    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        Ok(ServiceAction::new().cmd("service", [&*script_name, "stop"]))
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        Ok(ServiceAction::new()
            .cmd_ignore_error("service", [&*script_name, "status"])
            .with_parser(|outputs: &[CmdOutput]| {
                let out = outputs.last();
                match out {
                    None => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                    Some(output) => match output.exit_code {
                        Some(0) => Ok(ActionOutput::Status(ServiceStatus::Running)),
                        Some(3) => Ok(ActionOutput::Status(ServiceStatus::Stopped(None))),
                        Some(1) => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                        _ => Ok(ActionOutput::Status(ServiceStatus::Stopped(None))),
                    },
                }
            }))
    }

    fn level(&self) -> ServiceLevel {
        ServiceLevel::System
    }

    fn set_level(&mut self, level: ServiceLevel) -> Result<()> {
        if level != ServiceLevel::System {
            return Err(Error::Unsupported(
                "rc.d only supports system-level services".into(),
            ));
        }
        Ok(())
    }

    fn list(&self) -> Result<ServiceAction> {
        Ok(ServiceAction::new()
            .read_dir("/usr/local/etc/rc.d", None::<String>)
            .with_parser(|outputs: &[CmdOutput]| {
                let mut services = outputs
                    .last()
                    .map(|output| {
                        output
                            .stdout
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .map(str::to_owned)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                services.sort();
                Ok(ActionOutput::List(services))
            }))
    }
}
