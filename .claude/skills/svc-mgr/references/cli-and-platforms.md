# CLI 工具 rsvc

## 构建 CLI

```bash
cargo build --features cli
cargo run --features cli --bin rsvc -- --help
```


```bash
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

## install 选项

```
--program <PATH>              可执行文件路径（必填）
--args <A>...                 程序参数
--workdir <DIR>               工作目录
--env <K=V>...                环境变量
--username <U>                运行用户
--description <D>             服务描述
--autostart                   开机自启
--restart <POLICY>            never|always|on-failure|on-success (默认 on-failure)
--restart-delay <SECS>        重启延迟秒数
--max-retries <N>             最大重试次数
--log <PATH>                  日志文件（stdout+stderr）
--stdout-file <PATH>          stdout 日志文件
--stderr-file <PATH>          stderr 日志文件
```

## 全局选项

```
--user                        用户级别服务
--backend <KIND>              指定后端: launchd|systemd|openrc|rcd|sc|winsw
--dry-run                     仅预览命令
```

全局选项可放在子命令前后均可。

## 示例

```bash
rsvc install com.example.myapp --program /usr/bin/myapp --workdir /opt/myapp --autostart
rsvc --dry-run install com.example.myapp --program /usr/bin/myapp
rsvc status com.example.myapp
rsvc info com.example.myapp
rsvc edit com.example.myapp  # 使用 $EDITOR 编辑配置文件
rsvc list --dry-run
```

## 编辑器配置

`rsvc edit` 使用以下优先级选择编辑器：
1. `$EDITOR` 环境变量
2. `$VISUAL` 环境变量
3. 平台默认：Windows 使用 `notepad`，Unix 使用 `vi`

注意：sc.exe 后端不支持 edit 命令（无配置文件）。

# 平台支持

| 平台 | 后端 | User 级别 |
|------|------|----------|
| macOS | launchd | 支持 |
| Linux | systemd | 支持 |
| Linux | OpenRC | 不支持 |
| FreeBSD/BSD | rc.d | 不支持 |
| Windows | sc.exe | 不支持 |
| Windows | WinSW | 不支持 |
