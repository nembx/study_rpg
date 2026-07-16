# Study RPG 程序设计 V1

本文档把 `设计.md` 收敛成第一版可实现程序结构。V1 的中心不是 Todo 管理，而是一个稳定的成长循环：

```text
学习 Session -> 获得 XP -> 升级 -> 推进每日任务 -> 统计反馈
```

## V1 范围

必须实现：

- 创建角色
- 学习 Session 结算
- 学习计时器
- XP 和等级
- Dashboard 所需聚合数据
- Daily Quest 进度
- Statistics 基础汇总

暂缓实现：

- 完整多页面视觉表现
- 技能树编辑器
- 世界地图
- 成就、宠物、装备、云同步

## 核心模块

`StudyRpg` 是 V1 的主模块。它的外部接口保持小：

```rust
StudyRpg::new(player_name, class)
StudyRpg::add_skill(name, parent)
StudyRpg::start_study_session(input, started_at)
StudyRpg::finish_active_study_session(ended_at)
StudyRpg::complete_study_session(input)
StudyRpg::complete_study_session_at(input, ended_at)
StudyRpg::refresh_daily_quests_at(now)
StudyRpg::dashboard()
StudyRpg::dashboard_at(now)
StudyRpg::statistics_at(now)
```

调用方可以启动计时器，也可以手动提交一次学习结算。模块内部负责：

- 维护进行中的学习 Session
- 按日期生成和刷新每日任务
- 按学习时长计算 XP
- 更新玩家等级和称号
- 更新技能 XP
- 推进每日任务
- 当天任务全部完成时自动发放一次 150 XP 全清奖励
- 记录 Session
- 生成 Dashboard/Statistics 聚合数据

这样 UI 不需要复制规则，SQLite 也只是保存和恢复状态。

桌面启动由 `DesktopController` 显式区分两种状态：空数据库进入
`needs_character_creation`，用户提交名称和职业后才创建并保存 `StudyRpg`；已有数据库则直接恢复。
桌面适配器不得再用默认“玩家”身份静默初始化。职业在当前版本只作为身份信息展示，不改变 XP、
任务或统计规则。

Dashboard 当前聚合：

- 玩家等级和 XP 进度百分比
- 今日学习分钟数
- 每日任务日期和进度
- 每日任务是否全清以及全清奖励 XP
- 最近学习记录
- 进行中的学习 Session、已计时分钟和预计 XP

Statistics 当前聚合：

- 今日、本周（周一开始）、本月和累计的 Session 数、学习时长与 XP
- 包含零值日期的最近七日趋势，供柱状图和折线图直接使用
- 当前连续学习天数和历史最长连续学习天数

日期分组和连续学习天数计算规则属于核心模块。UI 只消费 `statistics_at(now)` 返回的报告，不自行遍历 Session 或推导日历边界。

## 模块结构

```text
src/
├── companion.rs
├── desktop.rs
├── lib.rs
├── player.rs
├── quest.rs
├── session.rs
├── skill.rs
├── statistics.rs
├── storage.rs
├── study_rpg.rs
└── xp.rs

src-tauri/          # Tauri 桌面适配器、窗口与托盘生命周期
src-ui/             # Svelte Companion 和 Dashboard
```

## 外层适配器

V1 核心模块保持纯 Rust，便于测试。当前外层适配器：

- `src-ui`: 使用 Svelte 提供右侧 Companion 与完整 Dashboard
- `src-tauri`: 使用 Tauri 2 暴露命令，管理无边框置顶 Companion、普通 Dashboard 和系统托盘
- `companion`: 计算不同 DPI 和显示器工作区下的贴边窗口尺寸与位置
- `desktop`: 在 UI 与核心循环之间协调命令，并在状态变化后保存快照
- `storage`: 通过 SQLite 读写完整的 `StudyRpg` 状态

Companion 以正向计时和即时成长反馈为主要职责，提供收起卡片与展开面板两种形态；展开状态和纵向位置作为 UI 偏好保存在 SQLite。Dashboard 直接消费 `StudyRpg::statistics_at(now)`，展示今日、本周、本月、累计汇总、最近七日学习时长和连续学习天数。七日桶的日历日期由核心统计模块提供，UI 不重新计算日期分组或日历边界。

未来可以替换视觉框架或拆分更多页面，但核心接口和 SQLite 快照边界不随 UI 技术变化。

持久化接口应该围绕整个 `StudyRpg` 状态读写，而不是把数据库细节泄漏到每个页面。

当前已经实现 `SqliteStore`：

```rust
SqliteStore::open(path)
SqliteStore::in_memory()
SqliteStore::save(&app)
SqliteStore::load()
```

`StudyRpg` 通过 `snapshot()` 和 `from_snapshot()` 与存储适配器协作。UI 不直接操作 SQLite 表，也不需要知道实体之间的保存顺序。进行中的 `ActiveStudySession` 也会进入 snapshot，应用重启后可以恢复计时状态。

每日任务全清奖励会自动结算，并在 snapshot 中记录当天是否已经发放。刷新到新日期时该状态重置；SQLite 恢复后则继续保持，避免同一天重复领取奖励。
`StudySessionResult` 分别返回单项任务奖励 `quest_reward_xp` 和全清奖励 `daily_completion_bonus_xp`，调用方将两者作为独立反馈展示。

## 数据规则

等级阈值先匹配设计文档中的早期节奏：

```text
Lv.1 -> Lv.2: 100 XP
Lv.2 -> Lv.3: 150 XP
Lv.3 -> Lv.4: 220 XP
Lv.4 -> Lv.5: 300 XP
```

再往后使用递增曲线，保持无限等级。

学习 XP 先按 `1 分钟 = 1.6 XP` 计算，25 分钟得到 40 XP，匹配设计文档示例。
