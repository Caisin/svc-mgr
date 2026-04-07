---
name: rust-cross-compile
description: |
  Use when building or verifying this repository for non-native targets, especially when cross-compiling from macOS ARM64 to Linux or Windows, choosing an appropriate glibc target for `cargo zigbuild`, or updating CI and pre-commit commands that must validate multi-platform Rust builds.
globs:
  - "Cargo.toml"
  - "src/**/*.rs"
  - ".github/workflows/*.yml"
---

# rust-cross-compile

这个 skill 对应本仓库的跨平台编译约定。重点不是“能不能编”，而是**在提交前验证 macOS / Linux / Windows 三类目标都不过期、不漏检。**

## 何时使用

- 需要从 macOS ARM64 交叉编译到 Linux / Windows
- 需要给 Linux 目标选 glibc 版本
- 需要修改 CI、发布脚本或 pre-commit 的跨平台构建步骤
- 本地出现 `GLIBC_X.XX not found`、目标平台链接失败等问题

## 本仓库默认检查

先看 `CLAUDE.md` 中的 pre-commit checklist。当前最低要求：

```bash
cargo build --features cli,tui
cargo zigbuild --target x86_64-unknown-linux-gnu.2.35 --features cli,tui
cargo zigbuild --target x86_64-pc-windows-gnu --features cli,tui
```

若要把 lint 一起覆盖：

```bash
cargo clippy --all-targets --all-features --target x86_64-unknown-linux-gnu -- -D warnings
cargo clippy --all-targets --all-features --target x86_64-pc-windows-gnu -- -D warnings
cargo clippy --all-targets --all-features -- -D warnings
```

## 工具准备

```bash
brew install zig
cargo install cargo-zigbuild
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-unknown-linux-gnu
```

## 常用命令

```bash
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.35 --features cli,tui
cargo zigbuild --release --target x86_64-pc-windows-gnu --features cli,tui
cargo zigbuild --release --target aarch64-unknown-linux-gnu.2.35 --features cli,tui
```

## glibc 选择

- Ubuntu 22.04: `x86_64-unknown-linux-gnu.2.35`
- Ubuntu 20.04 / Debian 11: `x86_64-unknown-linux-gnu.2.31`
- Ubuntu 18.04: `x86_64-unknown-linux-gnu.2.27`
- CentOS 7: 建议更保守版本或改用 musl

目标机器上可用 `ldd --version` 确认。

## 常见问题

- `GLIBC_X.XX not found`：把 target triple 改到目标机器实际 glibc 版本
- `cargo-zigbuild` 失败：优先升级 Zig / cargo-zigbuild，再 `cargo clean`
- 需要更强的 Linux 可移植性：使用 `x86_64-unknown-linux-musl`
