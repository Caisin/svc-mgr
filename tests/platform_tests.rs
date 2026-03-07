use std::ffi::OsString;
use std::path::PathBuf;

use svc_mgr::RestartPolicy;
use svc_mgr::ServiceConfig;

fn test_config() -> ServiceConfig {
    ServiceConfig {
        label: "com.example.myapp".parse().unwrap(),
        program: PathBuf::from("/usr/bin/myapp"),
        args: vec![OsString::from("--port"), OsString::from("8080")],
        working_directory: Some(PathBuf::from("/opt/myapp")),
        environment: vec![("RUST_LOG".into(), "info".into())],
        username: Some("myapp".into()),
        description: Some("My Application".into()),
        autostart: true,
        restart_policy: RestartPolicy::OnFailure {
            delay_secs: Some(5),
            max_retries: Some(3),
            reset_after_secs: Some(60),
        },
        contents: None,
        stdout_file: None,
        stderr_file: None,
    }
}

fn minimal_config() -> ServiceConfig {
    ServiceConfig {
        label: "myapp".parse().unwrap(),
        program: PathBuf::from("/usr/bin/myapp"),
        args: vec![],
        working_directory: None,
        environment: vec![],
        username: None,
        description: None,
        autostart: false,
        restart_policy: RestartPolicy::Never,
        contents: None,
        stdout_file: None,
        stderr_file: None,
    }
}

// ── systemd ──

#[cfg(target_os = "linux")]
mod systemd_tests {
    use super::*;
    use svc_mgr::platform::systemd::unit::SystemdUnit;

    #[test]
    fn render_full_unit() {
        let unit = SystemdUnit::from_config(&test_config(), false);
        let output = unit.render();

        assert!(output.contains("[Unit]"));
        assert!(output.contains("Description=My Application"));
        assert!(output.contains("StartLimitIntervalSec=60"));
        assert!(output.contains("StartLimitBurst=3"));
        assert!(output.contains("[Service]"));
        assert!(output.contains("WorkingDirectory=/opt/myapp"));
        assert!(output.contains("Environment=\"RUST_LOG=info\""));
        assert!(output.contains("ExecStart=/usr/bin/myapp --port 8080"));
        assert!(output.contains("Restart=on-failure"));
        assert!(output.contains("RestartSec=5"));
        assert!(output.contains("User=myapp"));
        assert!(output.contains("[Install]"));
        assert!(output.contains("WantedBy=multi-user.target"));
    }

    #[test]
    fn render_user_mode_no_user_field() {
        let unit = SystemdUnit::from_config(&test_config(), true);
        let output = unit.render();

        assert!(!output.contains("User="));
        assert!(output.contains("WantedBy=default.target"));
    }

    #[test]
    fn render_minimal_unit() {
        let unit = SystemdUnit::from_config(&minimal_config(), false);
        let output = unit.render();

        assert!(output.contains("ExecStart=/usr/bin/myapp"));
        assert!(!output.contains("Restart="));
        assert!(!output.contains("WorkingDirectory"));
        assert!(!output.contains("Environment"));
        assert!(!output.contains("User="));
    }

    #[test]
    fn render_always_restart() {
        let mut config = minimal_config();
        config.restart_policy = RestartPolicy::Always {
            delay_secs: Some(10),
        };
        let unit = SystemdUnit::from_config(&config, false);
        let output = unit.render();

        assert!(output.contains("Restart=always"));
        assert!(output.contains("RestartSec=10"));
    }
}

// ── openrc ──

#[cfg(target_os = "linux")]
mod openrc_tests {
    use super::*;
    use svc_mgr::platform::openrc::script::OpenRcScript;

    #[test]
    fn render_full_script() {
        let script = OpenRcScript::from_config(&test_config());
        let output = script.render();

        assert!(output.starts_with("#!/sbin/openrc-run"));
        assert!(output.contains("description=\"My Application\""));
        assert!(output.contains("command=\"/usr/bin/myapp\""));
        assert!(output.contains("command_args=\"--port 8080\""));
        assert!(output.contains("pidfile=\"/run/example-myapp.pid\""));
        assert!(output.contains("command_background=true"));
        assert!(output.contains("depend()"));
        assert!(output.contains("provide example-myapp"));
    }

    #[test]
    fn render_minimal_no_args() {
        let script = OpenRcScript::from_config(&minimal_config());
        let output = script.render();

        assert!(output.contains("command=\"/usr/bin/myapp\""));
        assert!(!output.contains("command_args"));
    }

    #[test]
    fn default_description_uses_name() {
        let mut config = minimal_config();
        config.label = "example.testapp".parse().unwrap();
        config.description = None;
        let script = OpenRcScript::from_config(&config);

        assert_eq!(script.description, "example-testapp");
    }

    #[test]
    fn openrc_list_is_deferred_action() {
        use svc_mgr::action::ActionStep;
        use svc_mgr::platform::openrc::OpenRcServiceManager;
        use svc_mgr::ServiceManager;

        let manager = OpenRcServiceManager::new();
        let action = manager.list().unwrap();

        assert!(matches!(
            action.steps().first(),
            Some(ActionStep::ReadDir { path, extension })
                if path == &std::path::PathBuf::from("/etc/init.d")
                    && extension.as_deref().is_none()
        ));
    }
}

// ── rcd ──

#[cfg(any(
    target_os = "freebsd",
    target_os = "dragonfly",
    target_os = "openbsd",
    target_os = "netbsd"
))]
mod rcd_tests {
    use super::*;
    use svc_mgr::platform::rcd::script::RcdScript;

    #[test]
    fn render_full_script() {
        let script = RcdScript::from_config(&test_config());
        let output = script.render();

        assert!(output.starts_with("#!/bin/sh"));
        assert!(output.contains("# PROVIDE: example-myapp"));
        assert!(output.contains("# REQUIRE: LOGIN FILESYSTEMS"));
        assert!(output.contains("# KEYWORD: shutdown"));
        assert!(output.contains(". /etc/rc.subr"));
        assert!(output.contains("name=\"example-myapp\""));
        assert!(output.contains("rcvar=\"example-myapp_enable\""));
        assert!(output.contains("pidfile=\"/var/run/example-myapp.pid\""));
        assert!(output.contains("command=\"/usr/sbin/daemon\""));
        assert!(output.contains("load_rc_config $name"));
        assert!(output.contains("run_rc_command \"$1\""));
    }

    #[test]
    fn render_minimal_no_daemon_args() {
        let script = RcdScript::from_config(&minimal_config());
        let output = script.render();

        assert!(output.contains("command_args=\"-p ${pidfile} /usr/bin/myapp\""));
    }

    #[test]
    fn rcd_list_is_deferred_action() {
        use svc_mgr::action::ActionStep;
        use svc_mgr::platform::rcd::RcdServiceManager;
        use svc_mgr::ServiceManager;

        let manager = RcdServiceManager::new();
        let action = manager.list().unwrap();

        assert!(matches!(
            action.steps().first(),
            Some(ActionStep::ReadDir { path, extension })
                if path == &std::path::PathBuf::from("/usr/local/etc/rc.d")
                    && extension.as_deref().is_none()
        ));
    }
}

// ── sc ──

#[cfg(target_os = "windows")]
mod sc_tests {
    use super::*;
    use svc_mgr::platform::sc::config::ScServiceConfig;
    use svc_mgr::platform::sc::shell_escape;

    #[test]
    fn sc_config_from_config() {
        let sc = ScServiceConfig::from_config(&test_config());

        assert_eq!(sc.service_name, "com.example.myapp");
        assert!(sc.binpath.contains("/usr/bin/myapp"));
        assert!(sc.binpath.contains("--port"));
        assert_eq!(sc.start_type.as_str(), "auto");
        assert_eq!(sc.display_name.as_deref(), Some("My Application"));
    }

    #[test]
    fn sc_config_demand_when_no_autostart() {
        let sc = ScServiceConfig::from_config(&minimal_config());
        assert_eq!(sc.start_type.as_str(), "demand");
    }

    #[test]
    fn to_create_args_contains_required_fields() {
        let sc = ScServiceConfig::from_config(&test_config());
        let args = sc.to_create_args();

        assert_eq!(args[0], "create");
        assert_eq!(args[1], "com.example.myapp");
        assert!(args.iter().any(|a| a.starts_with("type=")));
        assert!(args.iter().any(|a| a.starts_with("start=")));
        assert!(args.iter().any(|a| a.starts_with("error=")));
        assert!(args.iter().any(|a| a.starts_with("binpath=")));
        assert!(args.iter().any(|a| a.starts_with("displayname=")));
    }

    #[test]
    fn escape_empty_string() {
        assert_eq!(shell_escape::escape_for_sc(""), "\"\"");
    }

    #[test]
    fn escape_no_spaces() {
        assert_eq!(shell_escape::escape_for_sc("hello"), "hello");
    }

    #[test]
    fn escape_with_spaces() {
        assert_eq!(
            shell_escape::escape_for_sc("hello world"),
            "\"hello world\""
        );
    }

    #[test]
    fn build_binpath_simple() {
        let result = shell_escape::build_binpath("/usr/bin/app", &[]);
        assert_eq!(result, "/usr/bin/app");
    }

    #[test]
    fn build_binpath_with_args() {
        let args = vec!["--port".to_string(), "8080".to_string()];
        let result = shell_escape::build_binpath("/usr/bin/app", &args);
        assert_eq!(result, "/usr/bin/app --port 8080");
    }

    #[test]
    fn build_binpath_with_spaces_in_path() {
        let result =
            shell_escape::build_binpath("C:\\Program Files\\app.exe", &[]);
        assert_eq!(result, "\"C:\\Program Files\\app.exe\"");
    }

    #[test]
    fn winsw_list_is_deferred_action() {
        use svc_mgr::action::ActionStep;
        use svc_mgr::platform::winsw::WinSwServiceManager;
        use svc_mgr::ServiceManager;

        let manager = WinSwServiceManager::with_dir(r"C:\ProgramData\service-manager");
        let action = manager.list().unwrap();

        assert!(matches!(
            action.steps().first(),
            Some(ActionStep::ReadDir { path, extension })
                if path == &std::path::PathBuf::from(r"C:\ProgramData\service-manager")
                    && extension.as_deref() == Some("xml")
        ));
    }
}

// ── launchd plist (struct only, render needs macOS) ──

#[cfg(target_os = "macos")]
mod launchd_tests {
    use super::*;
    use svc_mgr::platform::launchd::plist::{KeepAlive, LaunchdPlist};

    #[test]
    fn plist_from_config_on_failure() {
        let plist = LaunchdPlist::from_config(&test_config());

        assert_eq!(plist.label, "com.example.myapp");
        assert_eq!(
            plist.program_arguments,
            vec!["/usr/bin/myapp", "--port", "8080"]
        );
        assert!(matches!(plist.keep_alive, Some(KeepAlive::SuccessfulExit(false))));
        assert!(plist.run_at_load);
        assert_eq!(plist.user_name.as_deref(), Some("myapp"));
        assert_eq!(plist.working_directory.as_deref(), Some("/opt/myapp"));
        assert_eq!(
            plist.environment_variables.get("RUST_LOG").map(|s| s.as_str()),
            Some("info")
        );
        assert!(!plist.disabled); // autostart=true → disabled=false
    }

    #[test]
    fn plist_never_restart() {
        let plist = LaunchdPlist::from_config(&minimal_config());

        assert!(plist.keep_alive.is_none());
        assert!(!plist.disabled);
        assert!(!plist.run_at_load);
    }

    #[test]
    fn plist_always_restart_with_autostart() {
        let mut config = minimal_config();
        config.restart_policy = RestartPolicy::Always { delay_secs: None };
        config.autostart = true;

        let plist = LaunchdPlist::from_config(&config);
        assert!(matches!(plist.keep_alive, Some(KeepAlive::Bool(true))));
        assert!(!plist.disabled); // autostart=true
    }

    #[test]
    fn plist_always_restart_without_autostart() {
        let mut config = minimal_config();
        config.restart_policy = RestartPolicy::Always { delay_secs: None };
        config.autostart = false;

        let plist = LaunchdPlist::from_config(&config);
        assert!(matches!(plist.keep_alive, Some(KeepAlive::Bool(true))));
        assert!(plist.disabled); // autostart=false → disabled=true
    }

    #[test]
    fn plist_on_success_restart() {
        let mut config = minimal_config();
        config.restart_policy = RestartPolicy::OnSuccess { delay_secs: Some(3) };
        config.autostart = false;

        let plist = LaunchdPlist::from_config(&config);
        assert!(matches!(plist.keep_alive, Some(KeepAlive::SuccessfulExit(true))));
        assert!(plist.disabled);
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn plist_render_produces_valid_xml() {
        let plist = LaunchdPlist::from_config(&test_config());
        let data = plist.render().unwrap();
        let xml = String::from_utf8(data).unwrap();

        assert!(xml.contains("<key>Label</key>"));
        assert!(xml.contains("<string>com.example.myapp</string>"));
        assert!(xml.contains("<key>ProgramArguments</key>"));
        assert!(xml.contains("<key>KeepAlive</key>"));
        assert!(xml.contains("<key>RunAtLoad</key>"));
        assert!(xml.contains("<key>UserName</key>"));
        assert!(xml.contains("<key>WorkingDirectory</key>"));
        assert!(xml.contains("<key>EnvironmentVariables</key>"));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn plist_render_minimal() {
        let plist = LaunchdPlist::from_config(&minimal_config());
        let data = plist.render().unwrap();
        let xml = String::from_utf8(data).unwrap();

        assert!(xml.contains("<key>Label</key>"));
        assert!(xml.contains("<string>myapp</string>"));
        assert!(!xml.contains("<key>KeepAlive</key>"));
        assert!(!xml.contains("<key>RunAtLoad</key>"));
        assert!(!xml.contains("<key>Disabled</key>"));
    }
}

// ── kind ──

mod kind_tests {
    use svc_mgr::ServiceManagerKind;

    #[test]
    fn native_returns_ok() {
        // Should succeed on any supported platform
        let result = ServiceManagerKind::native();
        assert!(result.is_ok());
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn native_is_launchd_on_macos() {
        assert_eq!(
            ServiceManagerKind::native().unwrap(),
            ServiceManagerKind::Launchd
        );
    }
}

// ── config ──

mod config_tests {
    use super::*;

    #[test]
    fn cmd_iter_includes_program_and_args() {
        let config = test_config();
        let cmd: Vec<_> = config.cmd_iter().collect();

        assert_eq!(cmd.len(), 3);
        assert_eq!(cmd[0], "/usr/bin/myapp");
        assert_eq!(cmd[1], "--port");
        assert_eq!(cmd[2], "8080");
    }

    #[test]
    fn cmd_iter_program_only() {
        let config = minimal_config();
        let cmd: Vec<_> = config.cmd_iter().collect();

        assert_eq!(cmd.len(), 1);
        assert_eq!(cmd[0], "/usr/bin/myapp");
    }
}

// ── ServiceAction commands ──

#[cfg(target_os = "macos")]
mod commands_tests {
    use super::*;
    use svc_mgr::{ServiceManager, TypedServiceManager};

    #[test]
    fn launchd_install_cmd() {
        let manager = TypedServiceManager::native().unwrap();
        let config = test_config();
        let action = manager.install(&config).unwrap();
        let cmds = action.commands();

        assert!(!cmds.is_empty());
        assert!(cmds[0].contains("write file"));
        // autostart=true → should have bootstrap
        assert!(cmds.iter().any(|c| c.contains("launchctl bootstrap")));
    }

    #[test]
    fn launchd_start_cmd() {
        let manager = TypedServiceManager::native().unwrap();
        let label = "com.example.myapp".parse().unwrap();
        let action = manager.start(&label).unwrap();
        let cmds = action.commands();

        assert!(cmds.iter().any(|c| c.contains("launchctl kickstart")));
        assert!(cmds.iter().any(|c| c.contains("com.example.myapp")));
    }

    #[test]
    fn launchd_stop_cmd() {
        let manager = TypedServiceManager::native().unwrap();
        let label = "com.example.myapp".parse().unwrap();
        let action = manager.stop(&label).unwrap();
        let cmds = action.commands();

        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("launchctl kill SIGTERM"));
        assert!(cmds[0].contains("com.example.myapp"));
    }

    #[test]
    fn launchd_uninstall_cmd() {
        let manager = TypedServiceManager::native().unwrap();
        let label = "com.example.myapp".parse().unwrap();
        let action = manager.uninstall(&label).unwrap();
        let cmds = action.commands();

        assert!(cmds.iter().any(|c| c.contains("launchctl bootout")));
        assert!(cmds.iter().any(|c| c.contains("rm ")));
    }

    #[test]
    fn launchd_restart_cmd() {
        let manager = TypedServiceManager::native().unwrap();
        let label = "com.example.myapp".parse().unwrap();
        let action = manager.restart(&label).unwrap();
        let cmds = action.commands();

        // restart = stop + start
        assert!(cmds.iter().any(|c| c.contains("kill SIGTERM")));
        assert!(cmds.iter().any(|c| c.contains("kickstart")));
    }

    #[test]
    fn launchd_status_cmd() {
        let manager = TypedServiceManager::native().unwrap();
        let label = "com.example.myapp".parse().unwrap();
        let action = manager.status(&label).unwrap();
        let cmds = action.commands();

        assert_eq!(cmds.len(), 1);
        assert!(cmds[0].contains("launchctl print"));
    }

    #[test]
    fn launchd_list_returns_services() {
        let manager = TypedServiceManager::native().unwrap();
        let output = manager.list().unwrap().exec().unwrap();
        let services = output.into_list().unwrap();

        // macOS always has some launchd services
        assert!(!services.is_empty());
    }

    #[test]
    fn launchd_status_exec() {
        let manager = TypedServiceManager::native().unwrap();
        let label = "com.example.nonexistent".parse().unwrap();
        let output = manager.status(&label).unwrap().exec().unwrap();
        let status = output.into_status().unwrap();

        assert_eq!(status, svc_mgr::ServiceStatus::NotInstalled);
    }
}
