pub mod xml_def;

use std::path::PathBuf;

use crate::action::{ActionOutput, CmdOutput, ServiceAction};
use crate::error::{Error, Result};
use crate::{ServiceConfig, ServiceLabel, ServiceLevel, ServiceManager, ServiceStatus};

pub struct WinSwServiceManager {
    service_def_dir: PathBuf,
}

impl WinSwServiceManager {
    pub fn new() -> Self {
        Self {
            service_def_dir: PathBuf::from(r"C:\ProgramData\service-manager"),
        }
    }

    pub fn with_dir(dir: impl Into<PathBuf>) -> Self {
        Self {
            service_def_dir: dir.into(),
        }
    }

    fn xml_path(&self, label: &ServiceLabel) -> PathBuf {
        self.service_def_dir
            .join(format!("{}.xml", label.to_qualified_name()))
    }
}

impl ServiceManager for WinSwServiceManager {
    fn available(&self) -> Result<bool> {
        Ok(which::which("winsw").is_ok() || which::which("winsw.exe").is_ok())
    }

    #[cfg(target_os = "windows")]
    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction> {
        let path = self.xml_path(&config.label);
        let data = if let Some(contents) = &config.contents {
            contents.clone()
        } else {
            let xml_def = xml_def::WinSwXmlDef::from_config(config);
            xml_def.render()?
        };
        Ok(ServiceAction::new()
            .write_file(&path, data.into_bytes(), 0o644)
            .cmd("winsw", ["install", &path.to_string_lossy()]))
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _config: &ServiceConfig) -> Result<ServiceAction> {
        Err(Error::Unsupported("WinSW is only available on Windows".into()))
    }

    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = self.xml_path(label);
        Ok(ServiceAction::new()
            .cmd_ignore_error("winsw", ["uninstall", &path.to_string_lossy()])
            .remove_file(&path))
    }

    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = self.xml_path(label);
        Ok(ServiceAction::new().cmd("winsw", ["start", &path.to_string_lossy()]))
    }

    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = self.xml_path(label);
        Ok(ServiceAction::new().cmd("winsw", ["stop", &path.to_string_lossy()]))
    }

    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction> {
        let path = self.xml_path(label);
        let path_str = path.to_string_lossy().to_string();
        Ok(ServiceAction::new()
            .cmd_ignore_error("winsw", ["status", &path_str])
            .with_parser(|outputs: &[CmdOutput]| {
                let out = outputs.last();
                match out {
                    None => Ok(ActionOutput::Status(ServiceStatus::NotInstalled)),
                    Some(o) => {
                        let stdout = o.stdout.trim().to_lowercase();
                        if stdout.contains("running") || stdout.contains("started") {
                            Ok(ActionOutput::Status(ServiceStatus::Running))
                        } else if stdout.contains("stopped") {
                            Ok(ActionOutput::Status(ServiceStatus::Stopped(None)))
                        } else if stdout.contains("nonexistent") || o.exit_code != Some(0) {
                            Ok(ActionOutput::Status(ServiceStatus::NotInstalled))
                        } else {
                            Ok(ActionOutput::Status(ServiceStatus::Stopped(
                                Some(stdout),
                            )))
                        }
                    }
                }
            }))
    }

    fn level(&self) -> ServiceLevel {
        ServiceLevel::System
    }

    fn set_level(&mut self, level: ServiceLevel) -> Result<()> {
        if level != ServiceLevel::System {
            return Err(Error::Unsupported(
                "WinSW only supports system-level services".into(),
            ));
        }
        Ok(())
    }

    fn list(&self) -> Result<ServiceAction> {
        let mut services = Vec::new();
        if self.service_def_dir.exists() {
            for entry in
                std::fs::read_dir(&self.service_def_dir).map_err(|e| Error::FileError {
                    path: self.service_def_dir.clone(),
                    source: e,
                })?
            {
                let entry = entry.map_err(|e| Error::Io(e))?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("xml") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        services.push(stem.to_string());
                    }
                }
            }
        }
        services.sort();
        let action = ServiceAction::new().with_parser(move |_: &[CmdOutput]| {
            Ok(ActionOutput::List(services.clone()))
        });
        Ok(action)
    }
}
