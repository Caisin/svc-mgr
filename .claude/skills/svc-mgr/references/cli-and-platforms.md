# CLI 与平台支持

## 构建命令

```bash
cargo build --features cli
cargo build --features cli,tui
cargo run --features cli --bin rsvc -- --help
cargo run --features cli --bin renv -- --help
cargo run --features tui --bin rtui
```

## `rsvc`

### 全局选项

```text
--user
--backend <launchd|systemd|openrc|rcd|sc|winsw>
--dry-run
```

- 全局选项可放在子命令前后
- `--backend` 会直接选择目标后端生成动作，不依赖当前开发机 OS
- 省略 `--backend` 时才会调用 `TypedServiceManager::native()`
- `--user` 仅对 launchd / systemd 有意义
- `--dry-run` 只打印 `ServiceAction::commands()`
- `rsvc install --env` 遇到非法 `KEY=VALUE` 参数时会忽略该项并打印提示，不会直接失败

### 子命令

```text
rsvc install <LABEL> --program <PATH> [OPTIONS]
rsvc uninstall <LABEL>
rsvc start <LABEL>
rsvc stop <LABEL>
rsvc restart <LABEL>
rsvc status <LABEL>
rsvc info <LABEL>
rsvc edit <LABEL>
rsvc list
```

### `install` 选项

```text
--program <PATH>              可执行文件路径（必填）
--args <A>...                 程序参数
--workdir <DIR>               工作目录
--env <K=V>...                环境变量
--username <U>                运行用户
--description <D>             服务描述
--autostart                   开机自启
--restart <POLICY>            never|always|on-failure|on-success
--restart-delay <SECS>        重启延迟秒数
--max-retries <N>             最大重试次数（仅 on-failure）
--log <PATH>                  stdout+stderr 同文件
--stdout-file <PATH>          单独 stdout 文件
--stderr-file <PATH>          单独 stderr 文件
```

### `info` / `edit`

- `info`：读取并打印服务配置内容
- `edit`：先通过 `info` 拿配置路径，再调用编辑器打开
- `sc.exe` 后端没有配置文件：`info` 打印 `sc qc` 输出，`edit` 会报错并提示改用 `sc.exe config`

编辑器优先级：`$EDITOR` → `$VISUAL` → Windows `notepad` → Unix `vi`

## `renv`

feature：`cli`

```text
renv list
renv get <KEY>
renv set <KEY> <VALUE>
renv unset <KEY>
```

全局选项：

```text
--system
```

- 默认操作 user scope
- `--system` 操作 system scope，通常需要管理员/root 权限
- Windows 下会广播 `WM_SETTINGCHANGE`
- Unix 下会改 shell profile 或 `/etc/environment`

## `rtui`

feature：`tui`

```bash
cargo run --features tui --bin rtui
```

当前入口：`src/bin/rtui.rs`。

真实范围：

- Services 页支持 start / stop / restart / status / info / edit / uninstall
- Environment 页支持 user scope 的 list / add / edit / delete
- 当前不支持在 TUI 内切到 system scope 环境变量

## 平台后端矩阵

| 平台 | 后端 | 服务文件结构体 | 默认配置位置 | User 级别 | 备注 |
|------|------|---------------|-------------|----------|------|
| macOS | launchd | `LaunchdPlist` | `/Library/LaunchDaemons` 或 `~/Library/LaunchAgents` | 支持 | 使用 `qualified_name + .plist` |
| Linux | systemd | `SystemdUnit` | `/etc/systemd/system` 或 `~/.config/systemd/user` | 支持 | 使用 `script_name + .service` |
| Linux | openrc | `OpenRcScript` | `/etc/init.d` | 不支持 | 使用 `script_name` |
| BSD | rc.d | `RcdScript` | `/usr/local/etc/rc.d` | 不支持 | 使用 `script_name` |
| Windows | sc.exe | `ScServiceConfig` | 无单独配置文件 | 不支持 | 安装/查询基于 `sc.exe` |
| Windows | winsw | `WinSwXmlDef` | `C:\ProgramData\service-manager` | 不支持 | 使用 `qualified_name + .xml` |

## 后端补充

### launchd

- `install + autostart` 会写 plist 后尝试 `launchctl bootstrap`
- `status` 用 `launchctl print`
- `stop` 用 `launchctl kill SIGTERM`

### systemd

- `install` 后会 `daemon-reload`
- `autostart` 对应 `systemctl enable`
- user 模式会给所有 `systemctl` 参数补 `--user`

### openrc / rc.d

- 都是脚本型后端
- 不支持 user 级服务
- `list()` 本质是读取脚本目录

### sc.exe

- 无配置文件路径概念
- `info()` 是 `sc.exe qc <name>` 的文本输出
- `enable/disable()` 使用 `sc.exe config start= auto|demand`

### winsw

- 通过 XML 文件安装服务
- `enable/disable()` 当前为空动作；通常通过 install/uninstall 管理
- `list()` 读取 XML 目录并去掉 `.xml` 后缀
