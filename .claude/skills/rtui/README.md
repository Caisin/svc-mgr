# rtui - 交互式终端界面

## 概述

`rtui` 是一个基于 ratatui 的交互式终端界面工具，提供可视化的服务和环境变量管理。

## 构建

```bash
cargo build --features tui --bin rtui
cargo run --features tui --bin rtui
```

## 功能

### 服务管理标签页

- 列出所有已安装的服务
- 使用方向键或 j/k 导航
- 按 r 刷新服务列表

### 环境变量标签页

- 列出所有用户级别环境变量
- 彩色显示键值对
- 使用方向键或 j/k 导航
- 按 r 刷新环境变量列表

## 快捷键

| 按键 | 功能 |
|------|------|
| Tab | 切换标签页 |
| ↑/k | 向上移动 |
| ↓/j | 向下移动 |
| r | 刷新当前列表 |
| q | 退出程序 |

## 界面布局

```
┌─ rtui - Service & Environment Manager ─────────────┐
│ Services │ Environment                              │
├──────────────────────────────────────────────────────┤
│                                                      │
│  >> service-name-1                                   │
│     service-name-2                                   │
│     service-name-3                                   │
│     ...                                              │
│                                                      │
├──────────────────────────────────────────────────────┤
│ Status: Loaded 10 services                           │
└──────────────────────────────────────────────────────┘
```

## 特性

- 跨平台支持（macOS、Linux、Windows）
- 实时刷新
- 键盘导航
- 彩色高亮
- 状态栏提示

## 依赖

- ratatui: 终端 UI 框架
- crossterm: 跨平台终端操作
