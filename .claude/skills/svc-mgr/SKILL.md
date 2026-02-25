---
name: svc-mgr
description: |
  跨平台服务管理 Rust 库。将程序注册为系统服务，管理服务生命周期（install/start/stop/restart/status/list）。
  支持 macOS(launchd)、Linux(systemd/openrc)、FreeBSD(rc.d)、Windows(sc.exe/winsw)。
  当用户需要：(1) 将 Rust 程序注册为系统服务 (2) 管理服务的启停和状态 (3) 跨平台服务部署 (4) 生成服务配置文件 时使用此 skill。
  crate 名: svc-mgr，Rust import: svc_mgr，CLI 工具: rsvc。
---

# svc-mgr 使用指南

## 添加依赖

```toml
[dependencies]
svc-mgr = { git = "https://github.com/Caisin/svc-mgr.git" }
```

## 核心模式

所有 `ServiceManager` 方法返回 `ServiceAction`（延迟执行），调用 `.exec()` 执行，`.commands()` 预览。

## 快速开始

```rust
use svc_mgr::{ServiceBuilder, ServiceManager, TypedServiceManager};

fn main() -> svc_mgr::Result<()> {
    let config = ServiceBuilder::new("com.example.myapp")?
        .program("/usr/bin/myapp")
        .args(["--port", "8080"])
        .working_directory("/opt/myapp")
        .env("RUST_LOG", "info")
        .description("My Application Service")
        .autostart(true)
        .restart_on_failure(5, 3)
        .build()?;

    let manager = TypedServiceManager::native()?;
    manager.install(&config)?.exec()?;
    manager.start(&config.label)?.exec()?;
    Ok(())
}
```

## 日志配置

```rust
// 单文件（stdout+stderr 合并）
builder.log("/var/log/myapp.log")

// 分别指定
builder.stdout_file("/var/log/myapp.out.log")
       .stderr_file("/var/log/myapp.err.log")

// 默认：有 working_directory 时自动用 {workdir}/logs/{服务名}.log
```

## 详细参考

- Builder API 完整方法、RestartPolicy 枚举、ServiceLabel 命名规则：见 [references/api.md](references/api.md)
- rsvc CLI 用法和平台支持矩阵：见 [references/cli-and-platforms.md](references/cli-and-platforms.md)
