# renv - 跨平台环境变量管理工具

## 概述

`renv` 是一个跨平台的环境变量管理 CLI 工具，支持 Windows、macOS 和 Linux。

## 构建

```bash
cargo build --features cli --bin renv
cargo run --features cli --bin renv -- --help
```

## 命令

### 列出所有环境变量

```bash
renv list              # 用户级别
renv list --system     # 系统级别
```

### 查询环境变量

```bash
renv get PATH
renv get --system PATH
```

### 设置环境变量

```bash
renv set MY_VAR "my value"
renv set --system MY_VAR "my value"  # 需要管理员权限
```

### 删除环境变量

```bash
renv unset MY_VAR
renv unset --system MY_VAR  # 需要管理员权限
```

## 平台实现

### Unix/Linux/macOS

- **用户级别**: 修改 shell 配置文件
  - zsh: `~/.zshrc`
  - bash: `~/.bashrc`
  - 其他: `~/.profile`
- **系统级别**: 修改 `/etc/environment`（需要 root 权限）

### Windows

- **用户级别**: 修改注册表 `HKEY_CURRENT_USER\Environment`
- **系统级别**: 修改注册表 `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`（需要管理员权限）
- 自动广播 `WM_SETTINGCHANGE` 消息通知其他应用

## 注意事项

1. 环境变量修改后需要重启 shell 或重新登录才能生效
2. 系统级别操作需要管理员/root 权限
3. Windows 上的修改会立即广播，但某些应用可能需要重启
4. Unix 系统上只修改配置文件，当前 shell 不会立即生效

## 示例

```bash
# 设置开发环境变量
renv set NODE_ENV development
renv set API_URL https://api.example.com

# 查看所有环境变量
renv list | grep NODE

# 删除临时变量
renv unset TEMP_VAR

# 系统级别操作（需要权限）
sudo renv set --system JAVA_HOME /usr/lib/jvm/java-11
```
