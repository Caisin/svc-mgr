# API 参考

## 核心公开类型

- `ServiceBuilder`
- `ServiceConfig`
- `ServiceManager` trait
- `TypedServiceManager`
- `ServiceManagerKind`
- `ServiceAction`
- `ActionOutput`
- `ServiceStatus`
- `ServiceLevel`
- `RestartPolicy`
- `ServiceLabel`
- `detect_init_system_with()` / `detect_package_manager_with()`

## `ServiceBuilder`

```rust
ServiceBuilder::new("com.example.myapp")?
    .program("/usr/bin/myapp")
    .args(["--port", "8080"])
    .working_directory("/opt/myapp")
    .env("KEY", "VALUE")
    .username("myapp")
    .description("My App")
    .autostart(true)
    .restart_policy(RestartPolicy::Always { delay_secs: Some(5) })
    .restart_on_failure(5, 3)
    .log("/var/log/app.log")
    .stdout_file("/var/log/app.out.log")
    .stderr_file("/var/log/app.err.log")
    .contents("raw service file content")
    .build()?
```

规则：

- `program(...)` 是必填项，否则 `build()` 报错
- `log(path)` 会同时设置 `stdout_file` 和 `stderr_file`
- 若未显式设置 `stdout_file` 且存在 `working_directory`，`build()` 会默认补 `logs/{label.to_script_name()}.log`
- `contents(...)` 会跳过平台模板生成

## `ServiceConfig`

```rust
pub struct ServiceConfig {
    pub label: ServiceLabel,
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub working_directory: Option<PathBuf>,
    pub environment: Vec<(String, String)>,
    pub username: Option<String>,
    pub description: Option<String>,
    pub autostart: bool,
    pub restart_policy: RestartPolicy,
    pub stdout_file: Option<PathBuf>,
    pub stderr_file: Option<PathBuf>,
    pub contents: Option<String>,
}
```

辅助方法：

- `cmd_iter()`：遍历 `program + args`

## `RestartPolicy`

```rust
RestartPolicy::Never
RestartPolicy::Always { delay_secs: Option<u32> }
RestartPolicy::OnFailure {
    delay_secs: Option<u32>,
    max_retries: Option<u32>,
    reset_after_secs: Option<u32>,
}
RestartPolicy::OnSuccess { delay_secs: Option<u32> }
```

默认值：

```rust
RestartPolicy::OnFailure {
    delay_secs: None,
    max_retries: None,
    reset_after_secs: None,
}
```

## `ServiceManager` trait

```rust
pub trait ServiceManager {
    fn available(&self) -> Result<bool>;
    fn install(&self, config: &ServiceConfig) -> Result<ServiceAction>;
    fn uninstall(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn start(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn stop(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn restart(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn enable(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn disable(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn status(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn info(&self, label: &ServiceLabel) -> Result<ServiceAction>;
    fn list(&self) -> Result<ServiceAction>;
    fn level(&self) -> ServiceLevel;
    fn set_level(&mut self, level: ServiceLevel) -> Result<()>;
}
```

补充：

- `restart()` 默认实现是 `stop + start` 的 action merge
- 当前 `rsvc` CLI 没有单独暴露 `enable/disable` 子命令，但 trait 已支持

## `ServiceAction`

### 三种用法

```rust
let action = manager.status(&label)?;

let output = action.exec()?;
let preview: Vec<String> = action.commands();
let parsed = action.parse(&[CmdOutput {
    exit_code: Some(0),
    stdout: "...".into(),
    stderr: String::new(),
}])?;
```

### 常用 step / helper

- `write_file(path, data, mode)`
- `remove_file(path)`
- `read_dir(path, extension)`
- `read_file(path)`
- `cmd(program, args)`
- `cmd_ignore_error(program, args)`
- `with_parser(...)`
- `merge(other)`
- `steps()`
- `commands()`

### `ActionOutput`

```rust
ActionOutput::None
ActionOutput::Status(ServiceStatus)
ActionOutput::List(Vec<String>)
ActionOutput::Info(ServiceInfo)
```

提取辅助：

```rust
output.into_status()?;
output.into_list()?;
output.into_info()?;
```

## 字段生效差异（重要）

`ServiceConfig` 是统一抽象，不代表每个后端都完整消费每个字段。

- launchd / systemd：能力最完整，比较适合作为统一字段模型的主要承载者
- openrc / rc.d：以脚本为主，字段会向脚本语义降级，不能假设完整承接所有环境/目录/用户配置
- sc.exe：没有单独配置文件模型，核心是 `ScServiceConfig::to_create_args()` 生成命令参数
- winsw：通过 XML 承载大部分字段，但也会按 WinSW 可表达能力做映射/降级

因此，讨论“某字段是否生效”时，必须落到具体后端实现去确认。

## `TypedServiceManager` / `ServiceManagerKind`

```rust
let native = TypedServiceManager::native()?;
let systemd = TypedServiceManager::target(ServiceManagerKind::Systemd)?;
```

规则：

- `target(kind)`：显式目标后端，可跨平台生成动作
- `native()`：只探测当前机器本机后端
- Windows `native()`：优先 `WinSw`，找不到再回退 `Sc`
- Linux `native()`：优先 `Systemd`，再尝试 `OpenRc`

## `ServiceLevel`

```rust
ServiceLevel::System
ServiceLevel::User
```

支持矩阵：

| 后端 | User 级别 |
|------|----------|
| launchd | 支持 |
| systemd | 支持 |
| openrc | 不支持 |
| rc.d | 不支持 |
| sc.exe | 不支持 |
| winsw | 不支持 |

## `ServiceLabel`

| 输入 | `to_qualified_name()` | `to_script_name()` |
|------|-----------------------|--------------------|
| `"myapp"` | `myapp` | `myapp` |
| `"example.myapp"` | `example.myapp` | `example-myapp` |
| `"com.example.myapp"` | `com.example.myapp` | `example-myapp` |
| `"com.example.foo.bar"` | `com.example.foo.bar` | `example-foo.bar` |

经验规则：

- launchd / sc.exe / winsw 更依赖 `qualified_name`
- systemd / openrc / rc.d 更依赖 `script_name`
- 推荐优先使用反向域名格式

## CLI 与库 API 的边界

- 库层支持 `contents`、`enable/disable`、`available()`、`reset_after_secs` 等更细粒度能力
- `rsvc` CLI 当前偏向常见生命周期操作：`install/uninstall/start/stop/restart/status/info/edit/list`
- 如果需求涉及后端精细配置或远端解析，优先直接用库 API，不要强行塞进 CLI 假设

## probe：远端探测辅助

```rust
use svc_mgr::{
    detect_init_system_with,
    detect_package_manager_with,
    CommandProbeOutput,
};
```

- `detect_init_system_with(...)`：返回 `ServiceManagerKind::Systemd` 或 `OpenRc`
- `detect_package_manager_with(...)`：返回 `PackageManagerKind::Apt` 或 `Yum`
- 设计目标是 transport-agnostic：通过回调注入命令执行，不直接绑定 SSH / app runtime

## 环境变量 API

```rust
use svc_mgr::env::{manager, EnvScope};

let env_mgr = manager();
let vars = env_mgr.list(EnvScope::User)?;
```

支持：

- `list(scope)`
- `get(scope, key)`
- `set(scope, key, value)`
- `unset(scope, key)`

Unix 下改 shell profile 或 `/etc/environment`；Windows 下改注册表并广播 `WM_SETTINGCHANGE`。
