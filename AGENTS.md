# AGENTS.md

## Skill 同步规范

当项目公共 API 发生变更时（包括但不限于）：

- `ServiceConfig` 新增/修改/删除字段
- `ServiceBuilder` 新增/修改/删除方法
- `ServiceManager` trait 方法变更
- `RestartPolicy`、`ServiceStatus` 等枚举变更
- CLI (`rsvc`) 子命令或选项变更
- 新增/移除平台后端

必须同步更新 `.claude/skills/svc-mgr/` 下的对应文件：

| 变更类型 | 需更新的 skill 文件 |
|---------|-------------------|
| Builder/Config/Trait API | `SKILL.md` 快速开始 + `references/api.md` |
| CLI 选项或子命令 | `references/cli-and-platforms.md` |
| 平台后端增减 | `references/cli-and-platforms.md` 平台支持表 |
| 日志相关逻辑 | `SKILL.md` 日志配置段 + `references/api.md` |
| crate 名/依赖方式变更 | `SKILL.md` 添加依赖段 + frontmatter description |
