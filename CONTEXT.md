# Study RPG

Study RPG 把学习行为建模为角色与技能的持续成长，而不是任务清单。

## Language

**Study Session**:
一次结算 XP 的连续学习活动，可以选择关联一个 Study Skill。
_Avoid_: 打卡、任务记录

**Study Skill**:
玩家希望通过 Study Session 持续培养的学习能力；同一技能可以跨多次 Session 累积 XP。
技能可通过父子关系组织；一次 Session 的技能 XP 只归属所选节点，选择时以技能 ID 区分不同分支的同名能力。
_Avoid_: 标签、主题分类

**Growth Event**:
Study Session 完成时留下的不可变成长事实，只记录 Player Level Change 或 Skill Growth。
_Avoid_: 历史快照、统计记录

**Player Level Change**:
玩家 XP 跨过一个或多个等级阈值的 Growth Event；没有跨级时不产生该事件。
_Avoid_: XP 变化、升级提示

**Skill Growth**:
Session XP 被计入一个 Study Skill 的 Growth Event，即使该技能没有跨级也会产生。
_Avoid_: 技能打卡、技能统计
