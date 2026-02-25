---
name: svc-mgr
description: 使用 svc-mgr 库将程序注册为跨平台系统服务
user_invocable: true
---

# svc-mgr 跨平台服务管理库使用指南

当用户需要将程序注册为系统服务、管理服务生命周期时，使用此 skill。

## 添加依赖

```toml
[dependencies]
svc-mgr = { git = "https://github.com/Caisin/svc-mgr.git" }
```

## 核心概念

所有 `ServiceManager` trait 方法返回 `ServiceAction`（延迟执行）：
- `.exec()` — 本地执行
- `.commands()` — 预览命令字符串（dry-run）
- `.parse(outputs)` — 解析远程输出（SSH 场景）

## 使用 Builder API 安装服务（推荐）

```rust
use svc_mgr::{ServiceBuilder, ServiceManager, TypedServiceManager};

fn main() -> svc_mgr::Result<()> {
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

    let manager = TypedServiceManager::native()?;
    manager.install(&config)?.exec()?;
    manager.start(&config.label)?.exec()?;
    Ok(())
}
```

## 日志配置

```rust
// 单文件：stdout 和 stderr 都写入同一个
let config = ServiceBuilder::new("com.example.myapp")?
    .program("/usr/bin/myapp")
    .log("/var/log/myapp.log")
    .build()?;

// 分别指定
let config = ServiceBuilder::new("com.example.myapp")?
    .program("/usr/bin/myapp")
    .stdout_file("/var/log/myapp.out.log")
    .stderr_file("/var/log/myapp.err.log")
    .build()?;

// 默认行为：设置了 working_directory 但没设 log 时
// 自动使用 {working_directory}/logs/{服务名}.log
let config = ServiceBuilder::new("com.example.myapp")?
    .program("/usr/bin/myapp")
    .working_directory("/opt/myapp")
    .build()?;
// stdout_file = /opt/myapp/logs/example-myapp.log
```

## 服务生命周期管理

```rust
use svc_mgr::{ServiceManager, ServiceStatus, TypedServiceManager};

let manager = TypedServiceManager::native()?;
let label = "com.example.myapp".parse()?;

// 查询状态
let status = manager.status(&label)?.exec()?.into_status();
match status {
    ServiceStatus::Running => println!("运行中"),
    ServiceStatus::Stopped(reason) => println!("已停止: {:?}", reason),
    ServiceStatus::NotInstalled => println!("未安装"),
}

// 停止 / 重启 / 卸载
manager.stop(&label)?.exec()?;
manager.restart(&label)?.exec()?;
manager.uninstall(&label)?.exec()?;

// 列出所有服务
let services = manager.list()?.exec()?.into_list();
```

## 指定后端 / 用户级别

```rust
use svc_mgr::{ServiceManagerKind, ServiceLevel, TypedServiceManager, ServiceManager};

// 指定后端
let manager = TypedServiceManager::target(ServiceManagerKind::Systemd);

// 用户级别服务（launchd/systemd 支持）
let mut manager = TypedServiceManager::native()?;
manager.set_level(ServiceLevel::User)?;
```

## 重启策略

```rust
use svc_mgr::RestartPolicy;

// 从不重启
RestartPolicy::Never

// 总是重启
RestartPolicy::Always { delay_secs: Some(5) }

// 失败时重启（默认策略）
RestartPolicy::OnFailure {
    delay_secs: Some(5),
    max_retries: Some(3),
    reset_after_secs: Some(60),
}

// 成功退出时重启
RestartPolicy::OnSuccess { delay_secs: Some(5) }
```

## 预览命令（dry-run）

```rust
let action = manager.install(&config)?;
for cmd in action.commands() {
    println!("{}", cmd);
}
// 不调用 .exec()，不会实际执行
```

## 平台支持

| 平台 | 后端 | User 级别 |
|------|------|----------|
| macOS | launchd | 支持 |
| Linux | systemd | 支持 |
| Linux | OpenRC | 不支持 |
| FreeBSD/BSD | rc.d | 不支持 |
| Windows | sc.exe | 不支持 |
| Windows | WinSW | 不支持 |

## ServiceLabel 命名规则

- 1 段 `"myapp"` → qualified: `myapp`, script: `myapp`
- 2 段 `"example.myapp"` → qualified: `example.myapp`, script: `example-myapp`
- 3 段 `"com.example.myapp"` → qualified: `com.example.myapp`, script: `example-myapp`

推荐使用反向域名格式（3 段），如 `com.example.myapp`。

## CLI 工具 rsvc

项目同时提供 `rsvc` 命令行工具：

```bash
rsvc install com.example.myapp --program /usr/bin/myapp --workdir /opt/myapp --autostart
rsvc start com.example.myapp
rsvc status com.example.myapp
rsvc list
rsvc --dry-run install com.example.myapp --program /usr/bin/myapp
```
