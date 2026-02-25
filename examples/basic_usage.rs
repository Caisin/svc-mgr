//! 基本用法示例：使用 Builder API 构建服务配置并查看生成的服务文件内容。
//!
//! 运行: cargo run --example basic_usage

use svc_mgr::{
    ServiceBuilder, ServiceManagerKind, ServiceStatus,
    TypedServiceManager, ServiceManager, ActionOutput,
};

fn main() -> svc_mgr::Result<()> {
    // 1. 使用 Builder 构建服务配置
    let config = ServiceBuilder::new("com.example.demo")?
        .program("/usr/local/bin/demo-server")
        .args(["--host", "0.0.0.0", "--port", "3000"])
        .working_directory("/opt/demo")
        .env("RUST_LOG", "info")
        .env("APP_ENV", "production")
        .description("Demo Server")
        .autostart(true)
        .restart_on_failure(5, 3)
        .build()?;

    println!("=== ServiceConfig ===");
    println!("Label: {}", config.label);
    println!("Program: {}", config.program.display());
    println!("Args: {:?}", config.args);
    println!("Autostart: {}", config.autostart);
    println!("Restart: {:?}", config.restart_policy);
    println!();

    // 2. 检测当前平台
    let kind = ServiceManagerKind::native()?;
    println!("Native service manager: {:?}", kind);
    println!();

    // 3. 展示各平台生成的服务文件内容
    #[cfg(target_os = "linux")]
    {
        use svc_mgr::platform::systemd::unit::SystemdUnit;
        let unit = SystemdUnit::from_config(&config, false);
        println!("=== systemd unit file ===");
        println!("{}", unit.render());
    }

    #[cfg(target_os = "macos")]
    {
        use svc_mgr::platform::launchd::plist::LaunchdPlist;
        let plist = LaunchdPlist::from_config(&config);
        let data = plist.render()?;
        println!("=== launchd plist ===");
        println!("{}", String::from_utf8_lossy(&data));
    }

    // 4. 使用 TypedServiceManager（仅展示 API，不实际安装）
    let manager = TypedServiceManager::native()?;
    println!("Manager available: {:?}", manager.available()?);
    println!("Manager level: {:?}", manager.level());

    // 查询一个不存在的服务状态
    let label = "com.example.nonexistent".parse()?;
    let status = manager.status(&label)?.exec()?.into_status();
    match status {
        ServiceStatus::NotInstalled => {
            println!("Service 'com.example.nonexistent': not installed (expected)")
        }
        _ => println!("Service status: {:?}", status),
    }

    // 5. 预览命令（不实际执行）
    let action = manager.install(&config)?;
    println!("\n=== install commands (preview) ===");
    for cmd in action.commands() {
        println!("  {}", cmd);
    }

    let action = manager.status(&label)?;
    println!("\n=== status commands (preview) ===");
    for cmd in action.commands() {
        println!("  {}", cmd);
    }

    // 6. list 也返回 ServiceAction
    let list_output = manager.list()?.exec()?;
    if let ActionOutput::List(services) = &list_output {
        println!("\n=== installed services (first 5) ===");
        for name in services.iter().take(5) {
            println!("  {}", name);
        }
    }

    // 要实际执行 install/start，调用 action.exec()?

    Ok(())
}
