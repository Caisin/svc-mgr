use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use svc_mgr::{ActionOutput, Error, ServiceAction, ServiceBuilder, ServiceManagerKind, TypedServiceManager};

fn unique_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("svc-mgr-{name}-{nanos}"))
}

#[test]
fn action_output_into_status_returns_error_for_wrong_variant() {
    let err = ActionOutput::List(vec!["demo".into()])
        .into_status()
        .unwrap_err();

    assert!(matches!(
        err,
        Error::UnexpectedActionOutput { expected, actual }
            if expected == "Status" && actual == "List"
    ));
}

#[test]
fn action_output_into_list_returns_error_for_wrong_variant() {
    let err = ActionOutput::None.into_list().unwrap_err();

    assert!(matches!(
        err,
        Error::UnexpectedActionOutput { expected, actual }
            if expected == "List" && actual == "None"
    ));
}

#[test]
fn builder_missing_program_returns_invalid_config() {
    let err = ServiceBuilder::new("com.example.myapp")
        .unwrap()
        .build()
        .unwrap_err();

    assert!(matches!(err, Error::InvalidConfig(msg) if msg.contains("program")));
}

#[test]
fn preview_quotes_args() {
    let action = ServiceAction::new().cmd("echo", ["hello world", "plain"]);
    let preview = action.commands();

    #[cfg(unix)]
    assert_eq!(preview, vec!["echo 'hello world' plain"]);

    #[cfg(windows)]
    assert_eq!(preview, vec!["echo \"hello world\" plain"]);
}

#[test]
fn read_dir_exec_defers_listing_until_exec() {
    let dir = unique_temp_dir("read-dir");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("alpha.service"), []).unwrap();
    fs::write(dir.join("beta.service"), []).unwrap();

    let action = ServiceAction::new()
        .read_dir(&dir, Some("service"))
        .with_parser(|outputs| {
            let mut names: Vec<String> = outputs
                .last()
                .unwrap()
                .stdout
                .lines()
                .map(str::to_owned)
                .collect();
            names.sort();
            Ok(ActionOutput::List(names))
        });

    assert_eq!(
        action.commands(),
        vec![format!("# list dir: {} (*.service)", dir.display())]
    );

    let list = action.exec().unwrap().into_list().unwrap();
    assert_eq!(list, vec!["alpha.service", "beta.service"]);

    fs::remove_dir_all(dir).unwrap();
}

#[cfg(target_os = "macos")]
#[test]
fn target_rejects_unsupported_backend() {
    assert!(matches!(
        TypedServiceManager::target(ServiceManagerKind::Systemd),
        Err(Error::Unsupported(msg)) if msg.contains("Systemd")
    ));
}

#[cfg(target_os = "linux")]
#[test]
fn target_rejects_unsupported_backend() {
    assert!(matches!(
        TypedServiceManager::target(ServiceManagerKind::Launchd),
        Err(Error::Unsupported(msg)) if msg.contains("Launchd")
    ));
}

#[cfg(target_os = "windows")]
#[test]
fn target_rejects_unsupported_backend() {
    assert!(matches!(
        TypedServiceManager::target(ServiceManagerKind::Launchd),
        Err(Error::Unsupported(msg)) if msg.contains("Launchd")
    ));
}
