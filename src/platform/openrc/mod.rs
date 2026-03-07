pub mod script;

use std::path::PathBuf;

use crate::action::{ActionOutput, CmdOutput, ServiceAction, ServiceInfo};
use crate::error::{Error, Result};
use crate::{ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager, ServiceStatus};

use self::script::OpenRcScript;

pub struct OpenRcServiceManager;

impl Default for OpenRcServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenRcServiceManager {
    pub fn new() -> Self {
        Self
    }

    fn script_path(label: &ServiceLabel) -> PathBuf {
        PathBuf::from("/etc/init.d").join(label.to_script_name())
    }
}

impl ServiceManager for OpenRcServiceManager {
    fn available(&self) -> Result<bool> {
        Ok(which::which("rc-service").is_ok())
    }

    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction> {
        let path = Self::script_path(&config.label);
        let script_name = config.label.to_script_name();
        let data = if let Some(contents) = &config.contents {
            contents.clone()
        } else {
            let script = OpenRcScript::from_config(config);
            script.render()
        };
        let mut action = ServiceAction::new().write_file(&path, data.into_bytes(), 0o755);
        if config.autostart {
            action = action.cmd("rc-update", ["add", &script_name, "default"]);
        }
        Ok(action)
    }

    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = Self::script_path(label);
        let script_name = label.to_script_name();
        Ok(ServiceAction::new()
            .cmd_ignore_error("rc-update", ["del", &script_name, "default"])
            .remove_file(&path))
    }

    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        Ok(ServiceAction::new().cmd("rc-service", [&*script_name, "start"]))
    }

    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        Ok(ServiceAction::new().cmd("rc-service", [&*script_name, "stop"]))
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let script_name = label.to_script_name();
        Ok(ServiceAction::new()
            .cmd_ignore_error("rc-service", [&*script_name, "status"])
            .with_parser(|outputs: &[CmdOutput]| {
                let out = outputs.last();
                match out {
                    None => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                    Some(output) => match output.exit_code {
                        Some(0) => Ok(ActionOutput::Status(ServiceStatus::Running)),
                        Some(3) => Ok(ActionOutput::Status(ServiceStatus::Stopped(None))),
                        Some(1) => {
                            if output.stderr.contains("does not exist") {
                                Ok(ActionOutput::Status(ServiceStatus::NotInstalled))
                            } else {
                                Ok(ActionOutput::Status(ServiceStatus::Stopped(Some(
                                    output.stderr.clone(),
                                ))))
                            }
                        }
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
                "OpenRC only supports system-level services".into(),
            ));
        }
        Ok(())
    }

    fn list(&self) -> Result<ServiceAction> {
        Ok(ServiceAction::new()
            .read_dir("/etc/init.d", None::<String>)
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

    fn info(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = Self::script_path(label);
        let path_str = path.to_string_lossy().to_string();
        let label_str = label.to_script_name();
        Ok(ServiceAction::new()
            .read_file(&path)
            .with_parser(move |outputs: &[CmdOutput]| {
                let content = outputs
                    .last()
                    .map(|o| o.stdout.clone())
                    .unwrap_or_default();
                Ok(ActionOutput::Info(ServiceInfo {
                    label: label_str.clone(),
                    config_path: path_str.clone(),
                    config_content: content,
                }))
            }))
    }
}
