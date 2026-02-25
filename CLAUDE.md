# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
cargo build                    # build library + rsvc binary
cargo test                     # run all tests
cargo test label_tests         # run a single test file
cargo test builder_basic       # run a single test by name
cargo run --bin rsvc -- --help # run CLI
cargo run --example basic_usage
```

## Architecture

svc-mgr is a cross-platform service management library (crate: `svc_mgr`) with a CLI binary (`rsvc`).

### Core Pattern: Deferred Execution via ServiceAction

Every `ServiceManager` trait method returns `ServiceAction` — not immediate results. This is the central design:
- `action.exec()` → execute locally, returns `ActionOutput`
- `action.commands()` → preview as strings (dry-run)
- `action.parse(outputs)` → interpret remote command outputs (SSH scenarios)

`ServiceAction` holds a list of `ActionStep`s (WriteFile/RemoveFile/Cmd/CmdIgnoreError) and an optional output parser closure.

### Type Dispatch

`TypedServiceManager` is a compile-time `cfg`-gated enum dispatching to platform backends via `dispatch!`/`dispatch_mut!` macros in `typed.rs`. Each backend lives under `src/platform/<name>/` with a manager struct + typed config builder (e.g., `SystemdUnit`, `LaunchdPlist`).

### Platform Backends

| Backend | Config struct | User-level |
|---------|--------------|------------|
| launchd (macOS) | `LaunchdPlist` | Yes |
| systemd (Linux) | `SystemdUnit` | Yes |
| openrc (Linux) | `OpenRcScript` | No |
| rcd (BSD) | `RcdScript` | No |
| sc (Windows) | `ScServiceConfig` | No |
| winsw (Windows) | `WinSwXmlDef` | No |

Each backend's `from_config()` + `render()` generates the platform service file. The `contents` field on `ServiceConfig` bypasses generation with raw content.

### CLI (`src/main.rs`)

`rsvc` is a thin clap wrapper. Global options (`--user`, `--backend`, `--dry-run`) use `global = true` so they work before or after the subcommand. All subcommands build a `ServiceAction` then call `run_action()` which either executes or previews.

## Conventions

- Use Chinese for commit messages and documentation when the user communicates in Chinese
- Push `master` to remote `main`: `git push origin master:main`
- Crate name: `svc-mgr`, binary name: `rsvc`, Rust import: `svc_mgr`
- Adding a new platform backend: create `src/platform/<name>/mod.rs` + config struct, implement `ServiceManager`, add variant to `TypedServiceManager` and both dispatch macros with appropriate `#[cfg]` gates
- Integration tests go in `tests/`, grouped by platform with `#[cfg(target_os)]` gates

## Skill 同步规范

当项目公共 API 发生变更时（包括但不限于）：

- `ServiceConfig` 新增/修改/删除字段
- `ServiceBuilder` 新增/修改/删除方法
- `ServiceManager` trait 方法变更
- `RestartPolicy`、`ServiceStatus` 等枚举变更
- CLI (`rsvc`) 子命令或选项变更
- 新增/移除平台后端

必须同步更新 `.claude/skills/svc-mgr/` 下的对应文件：

| 变更类型 | 需更新的 skill 文件 |
|---------|-------------------|
| Builder/Config/Trait API | `SKILL.md` 快速开始 + `references/api.md` |
| CLI 选项或子命令 | `references/cli-and-platforms.md` |
| 平台后端增减 | `references/cli-and-platforms.md` 平台支持表 |
| 日志相关逻辑 | `SKILL.md` 日志配置段 + `references/api.md` |
| crate 名/依赖方式变更 | `SKILL.md` 添加依赖段 + frontmatter description |
