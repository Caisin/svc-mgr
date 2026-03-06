# API 参考

## ServiceBuilder 完整方法链

```rust
ServiceBuilder::new("com.example.myapp")?   // 解析 label，返回 Result<Self>
    .program("/usr/bin/myapp")               // 必填，可执行文件路径
    .args(["--port", "8080"])                // 程序参数
    .working_directory("/opt/myapp")         // 工作目录
    .env("KEY", "VALUE")                     // 环境变量，可多次调用
    .username("myapp")                       // 运行用户
    .description("My App")                   // 服务描述
    .autostart(true)                         // 开机自启
    .restart_policy(RestartPolicy::Always { delay_secs: Some(5) })
    .restart_on_failure(5, 3)                // 快捷方式: delay=5s, max_retries=3
    .log("/var/log/app.log")                 // stdout+stderr 合并到单文件
    .stdout_file("/var/log/app.out.log")     // 单独指定 stdout
    .stderr_file("/var/log/app.err.log")     // 单独指定 stderr
    .contents("raw service file content")    // 跳过生成，直接使用原始内容
    .build()?                                // -> Result<ServiceConfig>
```

## RestartPolicy

```rust
RestartPolicy::Never
RestartPolicy::Always { delay_secs: Option<u32> }
RestartPolicy::OnFailure { delay_secs: Option<u32>, max_retries: Option<u32>, reset_after_secs: Option<u32> }
RestartPolicy::OnSuccess { delay_secs: Option<u32> }
// Default: OnFailure { delay_secs: None, max_retries: None, reset_after_secs: None }
```

## 服务生命周期

```rust
let manager = TypedServiceManager::native()?;
let label = "com.example.myapp".parse()?;

manager.install(&config)?.exec()?;
manager.start(&label)?.exec()?;
manager.stop(&label)?.exec()?;
manager.restart(&label)?.exec()?;
manager.uninstall(&label)?.exec()?;

// 状态查询
let status = manager.status(&label)?.exec()?.into_status()?;
// ServiceStatus::Running | Stopped(Option<String>) | NotInstalled

// 列出服务
let services = manager.list()?.exec()?.into_list()?; // Vec<String>

// 预览命令（dry-run）
let cmds = manager.install(&config)?.commands(); // Vec<String>
```

## 指定后端 / 用户级别

```rust
// 指定后端
let manager = TypedServiceManager::target(ServiceManagerKind::Systemd)?;

// 用户级别（launchd/systemd 支持）
let mut manager = TypedServiceManager::native()?;
manager.set_level(ServiceLevel::User)?;
```

## ServiceLabel 命名规则

| 输入 | qualified_name | script_name |
|------|---------------|-------------|
| `"myapp"` | `myapp` | `myapp` |
| `"example.myapp"` | `example.myapp` | `example-myapp` |
| `"com.example.myapp"` | `com.example.myapp` | `example-myapp` |

推荐使用反向域名格式（3 段）。
