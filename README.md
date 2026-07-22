# Study RPG

Study RPG 是一个 Local First 的学习成长软件。目标不是做另一个 Todo List，而是把学习行为包装成 RPG 式的成长循环：

```text
学习 Session -> 获得 XP -> 升级 -> 推进每日任务 -> 统计反馈
```

当前仓库处于早期 V1 阶段，重点先放在可测试的核心领域逻辑和本地持久化。

## 当前能力

- 创建玩家角色
- 首次运行时选择角色名称与职业，并在本地持久化身份
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
- 提供贴在屏幕右侧的 Companion，可收起/展开、上下拖动、开始/结束学习计时并查看即时成长反馈
- 学习结算会分别展示专注 XP、新完成的 Daily Quest、任务奖励、全清奖励与等级变化
- 提供独立 Dashboard 窗口，展示任务、最近记录和完整 Statistics
- 关闭窗口后驻留系统托盘；学习计时与窗口偏好均可在重启后恢复
- 提供 Statistics 页面，展示周期汇总、七日学习时长/XP 趋势和连续学习天数

## 运行

安装前端依赖并启动 Tauri 开发应用：

```bash
npm install
npm run tauri dev
```

首次运行会在系统应用数据目录创建 `study_rpg.sqlite3`。如果开发目录中已有
`data/study_rpg.sqlite3`，首次启动新版桌面端时会自动复制过去。之后再次启动会恢复玩家进度、
未结束的学习计时器，以及 Companion 的收起状态和纵向位置。

运行测试：

```bash
cargo test
```

类型检查：

```bash
cargo check
cargo check --manifest-path src-tauri/Cargo.toml
npm run check
```

当前环境如果缺少 `rustfmt`，`cargo fmt --check` 会提示安装 `rustfmt` 组件。

## 目录结构

```text
src/
├── companion.rs   # Companion 贴边尺寸与位置规则
├── desktop.rs     # 桌面控制器，连接 UI、核心循环和本地持久化
├── player.rs      # 玩家、职业、称号和 XP 授予
├── quest.rs       # 每日任务和任务进度
├── session.rs     # 学习 Session 和基础 XP 计算
├── skill.rs       # 技能成长
├── statistics.rs  # 学习统计聚合
├── storage.rs     # SQLite 本地持久化
├── study_rpg.rs   # 核心成长循环
└── xp.rs          # 等级曲线和等级进度

src-tauri/
├── src/           # Tauri 命令、双窗口、托盘和窗口生命周期
├── capabilities/  # Tauri 权限声明
└── tauri.conf.json

src-ui/
├── App.svelte               # Companion 与 Dashboard 视图
├── DailyQuestStatus.svelte  # Daily Quest 总进度与全清奖励状态
├── styles.css               # 桌面视觉样式
└── types.ts                 # Rust IPC 数据类型
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
- 首次运行角色创建流程
- 核心成长循环
- 学习计时器
- 每日任务按日期刷新
- Daily Quest 的单项进度、完成结算与全清奖励视觉反馈
- Dashboard 聚合数据
- SQLite 持久化
- Tauri 2 + Svelte 桌面应用
- 右侧贴边 Companion 的收起/展开、拖动定位和位置恢复
- 托盘驻留与按需打开 Dashboard
- Statistics 汇总与七日学习时长柱状图、XP 折线图
- 基础测试

下一步：

- 记录并展示等级变化与技能成长历史
