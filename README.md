# kx-service

跨平台服务管理库，支持 macOS (launchd)、Linux (systemd/openrc)、FreeBSD (rc.d)、Windows (sc.exe/winsw)。

## 特性

- 统一的 `ServiceManager` trait，屏蔽平台差异
- 操作方法返回 `ServiceAction`，支持 `.exec()` 执行或 `.commands()` 预览命令
- 每个平台都有类型化的服务文件结构体（`LaunchdPlist`、`SystemdUnit` 等），支持 `from_config()` + `render()`
- 链式 `ServiceBuilder` API，快速构建服务配置
- `TypedServiceManager` 自动分发到当前平台的后端
- `ServiceManagerKind::native()` 自动检测当前系统的服务管理器
- 支持 System / User 两种服务级别（launchd、systemd）
- 灵活的重启策略：Never / Always / OnFailure / OnSuccess

## 快速开始

```toml
[dependencies]
kx-service = { path = "." }
```

### 使用 Builder API 安装服务

```rust
use kx_service::{ServiceBuilder, ServiceManager, TypedServiceManager};

fn main() -> kx_service::Result<()> {
    // 构建服务配置
    let config = ServiceBuilder::new("com.example.myapp")?
        .program("/usr/bin/myapp")
        .args(["--port", "8080"])
        .working_directory("/opt/myapp")
        .env("RUST_LOG", "info")
        .description("My Application Service")
        .username("myapp")
        .autostart(true)
        .restart_on_failure(5, 3) // delay_secs=5, max_retries=3
        .build()?;

    // 自动检测当前平台的服务管理器
    let manager = TypedServiceManager::native()?;

    // 安装并启动
    manager.install(&config)?.exec()?;
    manager.start(&config.label)?.exec()?;

    // 查询状态
    let status = manager.status(&config.label)?.exec()?.into_status();
    println!("Service status: {:?}", status);

    // 预览命令（不实际执行）
    let action = manager.stop(&config.label)?;
    for cmd in action.commands() {
        println!("{}", cmd);
    }

    // 列出服务
    let services = manager.list()?.exec()?.into_list();
    println!("Services: {:?}", services);

    Ok(())
}
```

### 手动构建 ServiceConfig

```rust
use std::ffi::OsString;
use std::path::PathBuf;
use kx_service::{ServiceConfig, ServiceLabel, RestartPolicy};

let config = ServiceConfig {
    label: "com.example.myapp".parse().unwrap(),
    program: PathBuf::from("/usr/bin/myapp"),
    args: vec![OsString::from("--port"), OsString::from("8080")],
    working_directory: Some(PathBuf::from("/opt/myapp")),
    environment: vec![("RUST_LOG".into(), "info".into())],
    username: None,
    description: Some("My App".into()),
    autostart: true,
    restart_policy: RestartPolicy::Always { delay_secs: Some(5) },
    contents: None,
};
```

### 指定平台后端

```rust
use kx_service::{ServiceManagerKind, TypedServiceManager};

// 指定使用 systemd
let manager = TypedServiceManager::target(ServiceManagerKind::Systemd);

// 或自动检测
let manager = TypedServiceManager::native()?;
```

### ServiceLabel 解析规则

```rust
use kx_service::ServiceLabel;

// 1 段: application only
let label: ServiceLabel = "myapp".parse().unwrap();
assert_eq!(label.to_qualified_name(), "myapp");
assert_eq!(label.to_script_name(), "myapp");

// 2 段: organization.application
let label: ServiceLabel = "example.myapp".parse().unwrap();
assert_eq!(label.to_qualified_name(), "example.myapp");
assert_eq!(label.to_script_name(), "example-myapp");

// 3 段: qualifier.organization.application
let label: ServiceLabel = "com.example.myapp".parse().unwrap();
assert_eq!(label.to_qualified_name(), "com.example.myapp");
assert_eq!(label.to_script_name(), "example-myapp");
```

### 直接使用平台服务文件结构体

```rust
// systemd unit 文件生成
use kx_service::platform::systemd::unit::SystemdUnit;

let unit = SystemdUnit::from_config(&config, false);
let content = unit.render();
println!("{}", content);
// 输出:
// [Unit]
// Description=My App
//
// [Service]
// ExecStart=/usr/bin/myapp --port 8080
// Restart=always
// RestartSec=5
//
// [Install]
// WantedBy=multi-user.target
```

## 平台支持

| 平台 | 后端 | 服务文件结构体 | User 级别 |
|------|------|---------------|----------|
| macOS | launchd | `LaunchdPlist` | ✅ |
| Linux | systemd | `SystemdUnit` | ✅ |
| Linux | OpenRC | `OpenRcScript` | ❌ |
| FreeBSD | rc.d | `RcdScript` | ❌ |
| Windows | sc.exe | `ScServiceConfig` | ❌ |
| Windows | WinSW | `WinSwXmlDef` | ❌ |

## ServiceManager trait

```rust
pub trait ServiceManager {
    fn available(&self) -> Result<bool>;
    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction>;
    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn restart(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn list(&self) -> Result<ServiceAction>;
    fn level(&self) -> ServiceLevel;
    fn set_level(&mut self, level: ServiceLevel) -> Result<()>;
}
```

`ServiceAction` 支持三种使用方式：
- `.exec()` — 本地执行，返回 `ActionOutput`
- `.commands()` — 预览命令字符串
- `.parse(outputs)` — 解析远程执行的输出（用于 SSH 场景）

## License

MIT OR Apache-2.0
