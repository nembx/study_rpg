#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrowthEvent {
    pub id: u64,
    pub session_id: u64,
    pub topic: String,
    pub occurred_at_epoch_seconds: Option<u64>,
    pub kind: GrowthEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GrowthEventKind {
    PlayerLevelChange {
        gained_xp: u32,
        level_before: u32,
        level_after: u32,
        total_xp_after: u32,
    },
    SkillGrowth {
        skill_id: u64,
        skill_name: String,
        gained_xp: u32,
        level_before: u32,
        level_after: u32,
        total_xp_after: u32,
    },
}
