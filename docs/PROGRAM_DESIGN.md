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

- GPUI 完整视觉表现
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
StudyRpg::dashboard()
```

调用方可以启动计时器，也可以手动提交一次学习结算。模块内部负责：

- 维护进行中的学习 Session
- 按学习时长计算 XP
- 更新玩家等级和称号
- 更新技能 XP
- 推进每日任务
- 记录 Session
- 生成 Dashboard/Statistics 聚合数据

这样 UI 不需要复制规则，SQLite 也只是保存和恢复状态。

## 模块结构

```text
src/
├── lib.rs
├── main.rs
├── player.rs
├── quest.rs
├── session.rs
├── skill.rs
├── statistics.rs
├── storage.rs
├── study_rpg.rs
└── xp.rs
```

## 未来适配器

V1 核心模块先保持纯 Rust，便于测试。外层适配器：

- `ui/gpui`: 桌面窗口、导航、Dashboard、Session 计时器
- `storage/sqlite`: 通过 SQLite 读写玩家、技能、任务和学习记录
- `assets`: 图标、字体、音效和头像资源

持久化接口应该围绕整个 `StudyRpg` 状态读写，而不是把数据库细节泄漏到每个页面。

当前已经实现 `SqliteStore`：

```rust
SqliteStore::open(path)
SqliteStore::in_memory()
SqliteStore::save(&app)
SqliteStore::load()
```

`StudyRpg` 通过 `snapshot()` 和 `from_snapshot()` 与存储适配器协作。UI 不直接操作 SQLite 表，也不需要知道实体之间的保存顺序。进行中的 `ActiveStudySession` 也会进入 snapshot，应用重启后可以恢复计时状态。

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
