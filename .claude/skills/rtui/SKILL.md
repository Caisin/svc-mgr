---
name: rtui
description: |
  Use when working on the `rtui` terminal UI, the `svc_mgr::tui` module, or its keyboard-driven flows for browsing services and user environment variables, including search, service actions, inline env editing, and service-info viewing behavior.
---

# rtui

`rtui` 是基于 ratatui + crossterm 的交互式终端界面，入口在 `src/bin/rtui.rs`，实现位于 `src/tui/`。

## 何时使用

- 需要修改 TUI 布局、按键或状态流转
- 需要维护服务列表 / 环境变量列表 / 弹窗交互
- 需要确认某个键位是否真的已实现，而不是 README 里“想当然”描述

## 关键源码

- 入口：`src/bin/rtui.rs`
- 应用状态：`src/tui/app.rs`
- 事件分发：`src/tui/events.rs`
- 服务操作：`src/tui/service.rs`
- 环境变量操作：`src/tui/env.rs`
- 操作菜单：`src/tui/menu.rs`
- UI 渲染：`src/tui/ui.rs`

## 当前实际能力

### 标签页

- `Tab`：切换 Services / Environment
- Services 页显示 `TypedServiceManager::native().list()` 结果
- Environment 页显示 user scope 环境变量

### 普通模式快捷键

| 按键 | 行为 |
|------|------|
| `Tab` | 切换标签页 |
| `↑/k` `↓/j` | 上下移动 |
| `r` | 刷新当前列表 |
| `/` | 进入搜索模式 |
| `e` | 编辑当前项 |
| `q` | 退出 |
| `Enter` / 空格 | Services 页打开操作菜单 |
| `i` | Services 页查看详情；Environment 页新增变量 |
| `d` | Environment 页删除变量 |
| `s` | Services 页启动服务 |
| `t` | Services 页停止服务 |

### Services 页菜单

菜单项固定为：

- Start
- Stop
- Restart
- Status
- Info
- Edit
- Uninstall

### 模式

- `Normal`
- `Search`
- `Edit`
- `Menu`
- `AddEnv`
- `ViewInfo`

## 注意事项

- `edit_service()` 会暂时退出 TUI，调用系统编辑器，再重新进入 alternate screen
- Services 操作都走 `TypedServiceManager::native()`，不是显式 backend 选择
- Environment 页的新增/编辑/删除当前都只操作 `EnvScope::User`
- 搜索是前端过滤，不会重新向系统发起查询
- `r` 在 Services / Environment 页分别触发重新拉取列表
