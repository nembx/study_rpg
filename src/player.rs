use crate::xp::{LevelProgress, progress_from_total_xp};

pub const MAX_ENERGY: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterClass {
    Scholar,
    Engineer,
    Mage,
    Warrior,
    Archer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    pub name: String,
    pub class: CharacterClass,
    pub title: String,
    pub total_xp: u32,
    pub energy: u8,
    pub mood: Mood,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mood {
    Focused,
    Tired,
    Happy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XpGrant {
    pub gained_xp: u32,
    pub before: LevelProgress,
    pub after: LevelProgress,
}

impl Player {
    pub fn new(name: impl Into<String>, class: CharacterClass) -> Self {
        Self {
            name: name.into(),
            class,
            title: "Novice Learner".to_string(),
            total_xp: 0,
            energy: MAX_ENERGY,
            mood: Mood::Focused,
        }
    }

    pub fn level_progress(&self) -> LevelProgress {
        progress_from_total_xp(self.total_xp)
    }

    pub fn grant_xp(&mut self, amount: u32) -> XpGrant {
        let before = self.level_progress();
        self.total_xp = self.total_xp.saturating_add(amount);
        let after = self.level_progress();
        self.title = title_for_level(after.level).to_string();

        XpGrant {
            gained_xp: amount,
            before,
            after,
        }
    }
}

fn title_for_level(level: u32) -> &'static str {
    match level {
        1..=4 => "Novice Learner",
        5..=9 => "Knowledge Hunter",
        10..=19 => "Scholar Adventurer",
        20..=39 => "Master Student",
        _ => "Legendary Learner",
    }
}
