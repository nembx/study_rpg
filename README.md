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
- 按日期生成、刷新并推进每日任务
- 完成当天全部任务时额外获得 150 XP，且重启后不会重复发放
- 统计今日、本周、本月和累计学习次数、时长与 XP
- 生成最近七日学习趋势，并计算当前及历史最长连续学习天数
- 生成 Dashboard 聚合数据，包括任务进度、最近学习记录和进行中的学习估算
- 使用 SQLite 保存和恢复本地状态，包括进行中的学习 Session
- 提供本地桌面 Dashboard，可开始/结束学习计时、查看等级、任务和最近记录
- 提供 Statistics 页面，展示周期汇总、七日学习时长/XP 趋势和连续学习天数

## 运行

```bash
cargo run
```

首次运行会创建 `data/study_rpg.sqlite3`。之后再次启动会恢复玩家进度和未结束的学习计时器。

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
├── desktop.rs     # 桌面控制器，连接 UI、核心循环和本地持久化
├── desktop_ui.rs  # eframe/egui 桌面 Dashboard
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
- 每日任务按日期刷新
- Dashboard 聚合数据
- SQLite 持久化
- 桌面 Dashboard 与学习计时交互
- Statistics 汇总与七日学习时长柱状图、XP 折线图
- 基础测试

下一步：

- 增加首次运行的角色创建流程
- 继续完善 Daily Quest 的视觉反馈
- 记录并展示等级变化与技能成长历史
