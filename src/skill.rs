use crate::xp::{LevelProgress, progress_from_total_xp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
    pub total_xp: u32,
    pub unlocked: bool,
}

impl Skill {
    pub fn new(id: u64, name: impl Into<String>, parent_id: Option<u64>) -> Self {
        Self {
            id,
            name: name.into(),
            parent_id,
            total_xp: 0,
            unlocked: parent_id.is_none(),
        }
    }

    pub fn grant_xp(&mut self, amount: u32) -> LevelProgress {
        self.total_xp = self.total_xp.saturating_add(amount);
        progress_from_total_xp(self.total_xp)
    }

    pub fn mastery_percent(&self) -> u8 {
        progress_from_total_xp(self.total_xp)
            .level
            .saturating_mul(5)
            .min(100) as u8
    }
}
