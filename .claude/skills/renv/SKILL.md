---
name: renv
description: |
  Use when working on the `renv` CLI or the `svc_mgr::env` module to list, read, write, or remove user/system environment variables across Unix and Windows, or when debugging shell-profile versus Windows-registry environment persistence behavior.
---

# renv

`renv` 是这个仓库里专门管理环境变量的 CLI，对应实现位于 `src/env/` 与 `src/bin/renv.rs`。

## 何时使用

- 需要查看或修改用户级环境变量
- 需要查看或修改系统级环境变量
- 需要维护 `svc_mgr::env::{EnvManager, EnvScope}`
- 需要排查 Unix shell profile / Windows 注册表的环境变量持久化问题

## 关键源码

- CLI 入口：`src/bin/renv.rs`
- 通用抽象：`src/env/mod.rs`
- Unix 实现：`src/env/unix.rs`
- Windows 实现：`src/env/windows.rs`

## CLI 速查

```bash
cargo run --features cli --bin renv -- --help

renv list
renv get PATH
renv set MY_VAR "my value"
renv unset MY_VAR

renv --system list
renv --system set JAVA_HOME /path/to/jdk
```

## 行为约定

- 默认操作 `EnvScope::User`
- `--system` 操作 `EnvScope::System`，通常需要管理员/root 权限
- Unix：
  - user 级写 `~/.zshrc` / `~/.bashrc` / `~/.profile`（按 `$SHELL` 选择）
  - system 级写 `/etc/environment`
- Windows：
  - user 级写 `HKEY_CURRENT_USER\Environment`
  - system 级写 `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`
  - 写入后会广播 `WM_SETTINGCHANGE`

## 注意事项

- Unix 修改的是配置文件，不会让当前 shell 立即生效
- `renv set/unset` 成功后 CLI 会提示需要重启 shell 或重新登录
- 当前 TUI 里的环境变量操作只针对 user scope，不覆盖 `--system`
