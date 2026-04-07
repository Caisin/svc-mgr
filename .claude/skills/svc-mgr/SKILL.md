---
name: svc-mgr
description: |
  Use when working on the `svc-mgr` crate or its `rsvc` CLI to install programs as system services, generate cross-platform service definitions, preview service-management commands for remote execution, or debug launchd/systemd/openrc/rc.d/sc.exe/winsw integration behavior.
---

# svc-mgr

`svc-mgr` 是这个仓库的核心 crate，用统一抽象屏蔽多平台服务管理差异。
优先把它理解成：**先构造 `ServiceAction`，再决定是在本地执行、预览命令，还是给远端执行后回传解析。**

## 何时使用

- 需要把程序安装成系统服务
- 需要用 Rust API 同时支持 macOS / Linux / BSD / Windows 服务管理
- 需要显式指定目标后端生成命令，而不是依赖当前机器 OS
- 需要修改 `ServiceBuilder` / `ServiceConfig` / `ServiceManager` / `TypedServiceManager`
- 需要维护 `rsvc` CLI 或各平台服务文件模板

## 不适用边界

- 只处理环境变量：看 `renv` skill
- 只处理 TUI 交互：看 `rtui` skill
- 只做 Rust 交叉编译：看 `rust-cross-compile` skill

## 快速导航

- 公共 API：`src/lib.rs`
- builder：`src/builder.rs`
- action 抽象：`src/action.rs`
- 后端分发：`src/typed.rs`、`src/kind.rs`
- label 解析：`src/label.rs`
- 远端探测：`src/probe.rs`
- CLI：`src/bin/rsvc.rs`
- 平台实现：`src/platform/<backend>/`

## 核心心智模型

### 1. 所有服务动作都先返回 `ServiceAction`

- `.exec()`：本地执行
- `.commands()`：只看要执行什么
- `.parse(outputs)`：远端执行后，用 `CmdOutput` 解析结果

这套模型很适合 SSH / agent / 控制面场景：当前进程只负责生成动作，不强耦合执行环境。

### 2. `target(kind)` 与 `native()` 语义不同

- `TypedServiceManager::target(kind)`：为**目标后端**生成动作，可跨 OS
- `TypedServiceManager::native()`：为**当前机器**探测本机后端

远端部署、跨平台生成命令时，优先 `target(kind)`；不要误用 `native()`。

### 3. `contents` 会绕过模板生成

一旦设置 `ServiceConfig.contents` / `ServiceBuilder::contents()`：

- 直接写入原始服务文件内容
- 不再走 `LaunchdPlist::from_config()`、`SystemdUnit::from_config()` 等模板生成流程

适合已经有完整平台模板、只借助本 crate 执行安装/启动时使用。

## 快速开始

```toml
[dependencies]
svc-mgr = { git = "https://github.com/Caisin/svc-mgr.git" }
```

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

## 常见判断规则

- 常规业务优先 `ServiceBuilder`，只有做中间层转换或完全控制字段时才手写 `ServiceConfig`
- 新增平台后端时，同时改 `src/platform/<backend>/`、`src/typed.rs`、`src/kind.rs`
- 只有 launchd / systemd 支持 `ServiceLevel::User`
- `working_directory` 存在且未显式指定 `stdout_file` 时，会自动补 `{workdir}/logs/{script_name}.log`

## 关键边界与降级约定

- trait 能力 > CLI 能力：`ServiceManager` 已支持 `available/enable/disable/info`，但 `rsvc` 当前没有单独暴露 `enable/disable`，也没有直接暴露 `contents`、`reset_after_secs` 等入口
- 字段不是所有后端都会完整消费：
  - `openrc` / `rc.d` 主要面向脚本生成，不会完整承接全部 `working_directory` / `environment` / `username` 语义
  - `sc.exe` 不走配置文件，也不承接日志文件 / 原始模板文件这种模型
  - `winsw` 会生成 XML，并把日志落到目录级配置；某些重启策略会向后端可表达能力降级
- `contents` 适合“我自己提供原始服务文件内容”，不适合和平台模板字段同时期待双向合并

## 进一步阅读

- API / Builder / Action / probe：见 [references/api.md](references/api.md)
- CLI / 平台矩阵 / renv / rtui：见 [references/cli-and-platforms.md](references/cli-and-platforms.md)
