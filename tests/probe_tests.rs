use std::cell::RefCell;
use std::collections::HashMap;

use svc_mgr::{
    probe::{CommandProbeOutput, PackageManagerKind, detect_init_system_with, detect_package_manager_with},
    ServiceManagerKind,
};

fn success(stdout: &str) -> svc_mgr::Result<CommandProbeOutput> {
    Ok(CommandProbeOutput {
        success: true,
        stdout: stdout.to_string(),
        stderr: String::new(),
    })
}

fn failure(stderr: &str) -> svc_mgr::Result<CommandProbeOutput> {
    Ok(CommandProbeOutput {
        success: false,
        stdout: String::new(),
        stderr: stderr.to_string(),
    })
}

#[test]
fn detect_init_system_prefers_systemd() {
    let outputs = RefCell::new(HashMap::from([
        ("which systemctl".to_string(), success("/usr/bin/systemctl").unwrap()),
        ("which rc-service".to_string(), success("/sbin/rc-service").unwrap()),
    ]));

    let detected = detect_init_system_with(|cmd| {
        Ok(outputs.borrow().get(cmd).cloned().unwrap_or_default())
    })
    .unwrap();

    assert_eq!(detected, ServiceManagerKind::Systemd);
}

#[test]
fn detect_init_system_falls_back_to_openrc() {
    let outputs = RefCell::new(HashMap::from([
        ("which systemctl".to_string(), failure("missing").unwrap()),
        ("which rc-service".to_string(), success("/sbin/rc-service").unwrap()),
    ]));

    let detected = detect_init_system_with(|cmd| {
        Ok(outputs.borrow().get(cmd).cloned().unwrap_or_default())
    })
    .unwrap();

    assert_eq!(detected, ServiceManagerKind::OpenRc);
}

#[test]
fn detect_package_manager_uses_os_release_first() {
    let outputs = RefCell::new(HashMap::from([(
        "cat /etc/os-release".to_string(),
        success("ID=ubuntu\nNAME=Ubuntu").unwrap(),
    )]));

    let detected = detect_package_manager_with(|cmd| {
        Ok(outputs.borrow().get(cmd).cloned().unwrap_or_default())
    })
    .unwrap();

    assert_eq!(detected, PackageManagerKind::Apt);
}

#[test]
fn detect_package_manager_falls_back_to_command_checks() {
    let outputs = RefCell::new(HashMap::from([
        ("cat /etc/os-release".to_string(), failure("missing").unwrap()),
        ("which apt-get".to_string(), failure("missing").unwrap()),
        ("which yum".to_string(), success("/usr/bin/yum").unwrap()),
    ]));

    let detected = detect_package_manager_with(|cmd| {
        Ok(outputs.borrow().get(cmd).cloned().unwrap_or_default())
    })
    .unwrap();

    assert_eq!(detected, PackageManagerKind::Yum);
}
