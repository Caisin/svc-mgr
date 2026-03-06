# Service API Hardening Design

**背景**

当前项目已经具备可用的跨平台服务管理能力，但存在几类容易在生产或集成时放大的问题：公共 API 暴露 panic、CLI 输入约束偏弱、部分后端的 `list()` 在构建动作时就触发本地 I/O、以及验证范围偏向单平台。

**目标**

1. 将公共 API 中的 panic 改为可返回错误的显式契约。
2. 让 CLI 在参数解析阶段就拒绝无效值，而不是运行时静默回退。
3. 统一 `ServiceAction` 的延迟执行语义，避免构建动作时提前访问文件系统。
4. 减轻纯库消费者的 CLI 依赖负担，并补齐跨平台 CI 验证。

**设计决策**

- `TypedServiceManager::target` 改为返回 `Result<Self>`，不再因平台不支持而 panic。
- `ActionOutput::into_status` / `into_list` 改为返回 `Result<_>`，并新增错误类型表达“期望输出类型与实际不符”。
- `ServiceBuilder::build` 缺少程序路径时，返回配置错误而不是复用 label 错误。
- 为 `ServiceAction` 新增目录读取步骤，将 OpenRC / rc.d / WinSW 的 `list()` 改为延迟执行。
- CLI 的 `--restart` 使用 `clap::ValueEnum` 强约束。
- 将 `clap` 与 `env_logger` 挪到可选 `cli` feature，二进制通过 `required-features` 启用。
- GitHub Actions 改为 matrix，在 macOS / Linux / Windows 上分别构建与测试。

**测试策略**

- 先补库级测试，覆盖 panic→Result、非法后端、命令预览转义、延迟目录读取步骤。
- 在当前环境运行 `cargo test`、`cargo test --features cli`、`cargo clippy --all-targets --features cli -- -D warnings`。
- 通过 CI matrix 覆盖当前机器无法本地运行到的 Linux / Windows 条件分支。

**约束说明**

- 不做 workspace 重构，避免超出本轮“硬化 + 优化”范围。
- 不提交 git commit；本仓规则与当前任务都未要求提交。
