use crate::error::{Error, Result};
use crate::kind::ServiceManagerKind;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandProbeOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PackageManagerKind {
    Apt,
    Yum,
}

pub fn detect_init_system_with<F>(mut run: F) -> Result<ServiceManagerKind>
where
    F: FnMut(&str) -> Result<CommandProbeOutput>,
{
    if run("which systemctl").map(|output| output.success).unwrap_or(false) {
        return Ok(ServiceManagerKind::Systemd);
    }

    if run("which rc-service")
        .map(|output| output.success)
        .unwrap_or(false)
    {
        return Ok(ServiceManagerKind::OpenRc);
    }

    Err(Error::Unsupported(
        "cannot detect init system (neither systemd nor openrc found)".into(),
    ))
}

pub fn detect_package_manager_with<F>(mut run: F) -> Result<PackageManagerKind>
where
    F: FnMut(&str) -> Result<CommandProbeOutput>,
{
    let os_release = run("cat /etc/os-release")
        .map(|output| output.stdout.to_lowercase())
        .unwrap_or_default();

    if os_release.contains("ubuntu") || os_release.contains("debian") {
        return Ok(PackageManagerKind::Apt);
    }

    if os_release.contains("centos")
        || os_release.contains("rhel")
        || os_release.contains("fedora")
        || os_release.contains("almalinux")
        || os_release.contains("rocky")
    {
        return Ok(PackageManagerKind::Yum);
    }

    if run("which apt-get")
        .map(|output| output.success)
        .unwrap_or(false)
    {
        return Ok(PackageManagerKind::Apt);
    }

    if run("which yum").map(|output| output.success).unwrap_or(false) {
        return Ok(PackageManagerKind::Yum);
    }

    Err(Error::Unsupported(
        "cannot detect package manager (neither apt-get nor yum found)".into(),
    ))
}
