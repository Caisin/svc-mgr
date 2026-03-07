---
description: Rust 跨平台交叉编译指南。支持在 M4 Mac (ARM64) 上编译为 x86_64 Linux/Windows 目标，以及其他常见交叉编译场景。使用 cargo-zigbuild 简化交叉编译流程，避免 glibc 版本问题。
globs:
  - "Cargo.toml"
  - "src/**/*.rs"
  - ".github/workflows/*.yml"
---

# Rust 跨平台交叉编译 Skill

## 快速开始

### 在 M4 Mac 上编译为 x86_64 Linux

```bash
# 1. 安装工具（首次）
brew install zig
cargo install cargo-zigbuild
rustup target add x86_64-unknown-linux-gnu

# 2. 编译（指定 glibc 版本）
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.35

# 3. 输出位置
# target/x86_64-unknown-linux-gnu/release/your-binary
```

### 在 M4 Mac 上编译为 x86_64 Windows

```bash
# 1. 添加目标
rustup target add x86_64-pc-windows-gnu

# 2. 安装 mingw-w64
brew install mingw-w64

# 3. 编译
cargo zigbuild --release --target x86_64-pc-windows-gnu
```

## 核心概念

### 交叉编译挑战

1. **架构差异**：ARM64 → x86_64
2. **操作系统差异**：macOS 与 Linux/Windows 的标准库与链接器不兼容
3. **glibc 版本**：目标 Linux 系统的 glibc 版本可能低于编译环境

### cargo-zigbuild 优势

- 使用 Zig 的 C 工具链作为链接器
- 支持指定目标 glibc 版本（如 `.2.35`）
- 无需配置复杂的 Docker 或交叉编译工具链
- 规避常见的 `GLIBC_X.XX not found` 错误

## 常用目标平台

| 目标平台 | Target Triple | glibc 版本示例 |
|---------|--------------|---------------|
| Ubuntu 22.04 (x86_64) | x86_64-unknown-linux-gnu.2.35 | 2.35 |
| Ubuntu 20.04 (x86_64) | x86_64-unknown-linux-gnu.2.31 | 2.31 |
| Ubuntu 18.04 (x86_64) | x86_64-unknown-linux-gnu.2.27 | 2.27 |
| Debian 11 (x86_64) | x86_64-unknown-linux-gnu.2.31 | 2.31 |
| Windows (x86_64) | x86_64-pc-windows-gnu | N/A |
| macOS (x86_64) | x86_64-apple-darwin | N/A |
| macOS (ARM64) | aarch64-apple-darwin | N/A |

### 查看目标系统 glibc 版本

```bash
# 在目标 Linux 系统上运行
ldd --version
```

## 完整工作流

### 1. 环境准备

```bash
# 安装 Zig（macOS）
brew install zig

# 安装 cargo-zigbuild
cargo install cargo-zigbuild

# 添加目标平台
rustup target add x86_64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-unknown-linux-gnu
```

### 2. 编译命令

```bash
# Linux x86_64（指定 glibc 2.35）
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.35

# Linux x86_64（不指定 glibc，使用默认）
cargo zigbuild --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cargo zigbuild --release --target aarch64-unknown-linux-gnu.2.35

# Windows x86_64
cargo zigbuild --release --target x86_64-pc-windows-gnu

# 编译所有 features
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.35 --all-features

# 编译特定 features
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.35 --features cli,tui
```

### 3. 验证编译产物

```bash
# 查看二进制文件信息
file target/x86_64-unknown-linux-gnu/release/your-binary

# 查看依赖的动态库
otool -L target/x86_64-unknown-linux-gnu/release/your-binary  # macOS
ldd target/x86_64-unknown-linux-gnu/release/your-binary       # Linux
```

### 4. 测试（在目标平台）

```bash
# 复制到目标 Linux 系统
scp target/x86_64-unknown-linux-gnu/release/your-binary user@linux-host:/tmp/

# 在目标系统上测试
ssh user@linux-host '/tmp/your-binary --version'
```

## CI/CD 集成

### GitHub Actions 示例

```yaml
name: Cross-Platform Build

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install Zig (for cross-compilation)
        if: matrix.os == 'macos-latest' && matrix.target == 'x86_64-unknown-linux-gnu'
        run: |
          brew install zig
          cargo install cargo-zigbuild

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Test
        run: cargo test --release --target ${{ matrix.target }}
```

## 常见问题

### Q: 编译后在目标系统提示 `GLIBC_X.XX not found`

**A:** 指定更低的 glibc 版本：

```bash
# 如果目标是 Ubuntu 20.04（glibc 2.31）
cargo zigbuild --release --target x86_64-unknown-linux-gnu.2.31
```

### Q: 如何确定应该使用哪个 glibc 版本？

**A:** 在目标系统上运行 `ldd --version` 查看，或参考：
- Ubuntu 22.04: 2.35
- Ubuntu 20.04: 2.31
- Ubuntu 18.04: 2.27
- Debian 11: 2.31
- CentOS 7: 2.17

### Q: cargo-zigbuild 编译失败

**A:** 尝试以下方案：
1. 更新 Zig: `brew upgrade zig`
2. 更新 cargo-zigbuild: `cargo install --force cargo-zigbuild`
3. 清理缓存: `cargo clean`
4. 检查是否有平台特定的依赖问题

### Q: 如何编译静态链接的二进制文件？

**A:** 使用 musl target（适用于 Linux）：

```bash
rustup target add x86_64-unknown-linux-musl
cargo zigbuild --release --target x86_64-unknown-linux-musl
```

静态链接的二进制文件不依赖系统 glibc，可以在任何 Linux 发行版上运行。

## 最佳实践

1. **CI/CD 中使用 cargo-zigbuild**：在 GitHub Actions 中为 Linux 目标使用 zigbuild
2. **指定明确的 glibc 版本**：避免在旧系统上运行时出错
3. **测试多个目标平台**：至少测试 Linux、macOS、Windows
4. **使用 musl 提供通用 Linux 二进制**：适合分发给未知环境的用户
5. **文档化目标平台要求**：在 README 中说明最低系统要求

## 参考资料

- [cargo-zigbuild GitHub](https://github.com/rust-cross/cargo-zigbuild)
- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [Zig 官网](https://ziglang.org/)
