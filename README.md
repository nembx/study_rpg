# Study RPG

Study RPG 是一个 Local First 的学习成长软件。目标不是做另一个 Todo List，而是把学习行为包装成 RPG 式的成长循环：

```text
学习 Session -> 获得 XP -> 升级 -> 推进每日任务 -> 统计反馈
```

当前仓库处于早期 V1 阶段，重点先放在可测试的核心领域逻辑和本地持久化。

## 当前能力

- 创建玩家角色
- 开始和结束学习计时器
- 记录学习 Session
- 按学习时长结算 XP
- 根据总 XP 计算等级进度
- 推进每日任务
- 统计累计学习次数、时长和 XP
- 生成 Dashboard 聚合数据，包括任务进度、最近学习记录和进行中的学习估算
- 使用 SQLite 保存和恢复本地状态，包括进行中的学习 Session

## 运行

```bash
cargo run
```

运行测试：

```bash
cargo test
```

类型检查：

```bash
cargo check
```

当前环境如果缺少 `rustfmt`，`cargo fmt --check` 会提示安装 `rustfmt` 组件。

## 目录结构

```text
src/
├── player.rs      # 玩家、职业、称号和 XP 授予
├── quest.rs       # 每日任务和任务进度
├── session.rs     # 学习 Session 和基础 XP 计算
├── skill.rs       # 技能成长
├── statistics.rs  # 学习统计聚合
├── storage.rs     # SQLite 本地持久化
├── study_rpg.rs   # 核心成长循环
└── xp.rs          # 等级曲线和等级进度
```

## 设计原则

- 核心循环优先：每次学习都必须能产生即时成长反馈。
- Local First：用户数据默认保存在本地。
- 核心模块保持小接口：UI 和数据库不复制成长规则。
- SQLite 是外层适配器：通过 `StudyRpg::snapshot()` 和 `StudyRpg::from_snapshot()` 保存/恢复状态。

## 隐私约定

本地设计草稿、数据库、日志和环境变量不进入 Git。当前 `.gitignore` 已忽略：

- `设计.md`
- `data/`
- `*.db`、`*.sqlite`、`*.sqlite3`
- `.env`、`.env.*`
- `*.log`、`logs/`

## 开发状态

已完成：

- Rust 项目骨架
- 核心成长循环
- 学习计时器
- Dashboard 聚合数据
- SQLite 持久化
- 基础测试

下一步：

- 接入桌面 UI
- 继续完善 Daily Quest 和 Statistics
