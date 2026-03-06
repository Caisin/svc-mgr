# Service API Hardening Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 消除公共 API 的 panic 与弱校验点，并统一延迟执行语义与跨平台验证。

**Architecture:** 保持现有 crate 结构不变，在最小改动下收紧 API 契约、增加 `ServiceAction` 的延迟目录读取能力，并将 CLI 依赖收口到独立 feature。平台后端继续按 `TypedServiceManager` 分发，相关文档与 CI 同步更新。

**Tech Stack:** Rust 2024、clap、GitHub Actions、现有 integration tests。

---

### Task 1: 公共错误模型收敛

**Files:**
- Modify: `src/error.rs`
- Test: `tests/action_tests.rs`

**Step 1: Write the failing test**

- 为 `ActionOutput::into_status()` / `into_list()` 写错误场景测试。
- 为 builder 缺失 `program` 的错误类型写测试。

**Step 2: Run test to verify it fails**

Run: `cargo test action_output -- --nocapture`

**Step 3: Write minimal implementation**

- 新增 `InvalidConfig`、`UnexpectedActionOutput` 错误。
- 将相关调用方改为处理 `Result`。

**Step 4: Run test to verify it passes**

Run: `cargo test action_output builder_missing_program -- --nocapture`

### Task 2: 平台目标选择与 CLI 校验

**Files:**
- Modify: `src/typed.rs`
- Modify: `src/main.rs`
- Test: `tests/platform_tests.rs`

**Step 1: Write the failing test**

- 为 `TypedServiceManager::target()` 的跨平台非法后端场景补测试。
- 为 CLI 的 restart 参数改为枚举做编译级约束。

**Step 2: Run test to verify it fails**

Run: `cargo test target_rejects_unsupported_backend -- --nocapture`

**Step 3: Write minimal implementation**

- `target()` 返回 `Result<Self>`。
- `--restart` 使用 `ValueEnum`。

**Step 4: Run test to verify it passes**

Run: `cargo test target_rejects_unsupported_backend -- --nocapture`

### Task 3: 延迟执行目录读取

**Files:**
- Modify: `src/action.rs`
- Modify: `src/platform/openrc/mod.rs`
- Modify: `src/platform/rcd/mod.rs`
- Modify: `src/platform/winsw/mod.rs`
- Test: `tests/action_tests.rs`
- Test: `tests/platform_tests.rs`

**Step 1: Write the failing test**

- 为目录读取步骤的本地执行、预览命令、输出解析补测试。
- 为各平台 `list()` 断言其使用延迟步骤而不是立即枚举。

**Step 2: Run test to verify it fails**

Run: `cargo test read_dir -- --nocapture`

**Step 3: Write minimal implementation**

- 新增 `ActionStep::ReadDir` 与对应 builder / exec 逻辑。
- 平台 `list()` 改为构造 deferred action。

**Step 4: Run test to verify it passes**

Run: `cargo test read_dir -- --nocapture`

### Task 4: 预览命令与依赖收口

**Files:**
- Modify: `src/action.rs`
- Modify: `src/utils.rs`
- Modify: `Cargo.toml`
- Modify: `CLAUDE.md`

**Step 1: Write the failing test**

- 为带空格参数的 `commands()` 预览输出补测试。

**Step 2: Run test to verify it fails**

Run: `cargo test preview_quotes_args -- --nocapture`

**Step 3: Write minimal implementation**

- 补命令预览的参数转义。
- 将 `clap` / `env_logger` 改为 `cli` feature 的可选依赖。

**Step 4: Run test to verify it passes**

Run: `cargo test preview_quotes_args -- --nocapture`

### Task 5: 文档与 CI 同步

**Files:**
- Modify: `README.md`
- Modify: `.claude/skills/svc-mgr/SKILL.md`
- Modify: `.claude/skills/svc-mgr/references/api.md`
- Modify: `.claude/skills/svc-mgr/references/cli-and-platforms.md`
- Modify: `.github/workflows/rust.yml`

**Step 1: Update docs**

- 同步 API 返回值、CLI feature、restart 枚举约束、CI matrix。

**Step 2: Run verification**

Run: `cargo test && cargo test --features cli && cargo clippy --all-targets --features cli -- -D warnings`
