use crate::session::StudySession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    pub id: u64,
    pub title: String,
    pub target: QuestTarget,
    pub reward_xp: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestTarget {
    StudyMinutes(u32),
    CompleteSessions(u32),
}

impl Quest {
    pub fn study_minutes(id: u64, minutes: u32, reward_xp: u32) -> Self {
        Self {
            id,
            title: format!("Study {minutes} minutes"),
            target: QuestTarget::StudyMinutes(minutes),
            reward_xp,
            completed: false,
        }
    }

    pub fn complete_sessions(id: u64, sessions: u32, reward_xp: u32) -> Self {
        Self {
            id,
            title: format!("Complete {sessions} study session"),
            target: QuestTarget::CompleteSessions(sessions),
            reward_xp,
            completed: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestProgress {
    pub quest_id: u64,
    pub current: u32,
    pub target: u32,
    pub completed: bool,
}

pub fn evaluate_quests(quests: &mut [Quest], sessions: &[StudySession]) -> Vec<Quest> {
    let mut newly_completed = Vec::new();

    for quest in quests {
        if quest.completed {
            continue;
        }

        let progress = progress_for_quest(quest, sessions);
        if progress.current >= progress.target {
            quest.completed = true;
            newly_completed.push(quest.clone());
        }
    }

    newly_completed
}

pub fn progress_for_quest(quest: &Quest, sessions: &[StudySession]) -> QuestProgress {
    let mut progress = progress_for_target(quest.target, sessions);
    progress.quest_id = quest.id;
    progress.completed = quest.completed || progress.completed;
    progress
}

pub fn progress_for_target(target: QuestTarget, sessions: &[StudySession]) -> QuestProgress {
    match target {
        QuestTarget::StudyMinutes(target_minutes) => QuestProgress {
            quest_id: 0,
            current: sessions.iter().map(|session| session.duration_minutes).sum(),
            target: target_minutes,
            completed: sessions
                .iter()
                .map(|session| session.duration_minutes)
                .sum::<u32>()
                >= target_minutes,
        },
        QuestTarget::CompleteSessions(target_sessions) => QuestProgress {
            quest_id: 0,
            current: sessions.len() as u32,
            target: target_sessions,
            completed: sessions.len() as u32 >= target_sessions,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_progress_keeps_the_quest_identity() {
        let quest = Quest::study_minutes(42, 30, 60);
        let sessions = vec![StudySession {
            id: 1,
            topic: "Rust".to_string(),
            skill_id: None,
            duration_minutes: 10,
            earned_xp: 16,
            started_at_epoch_seconds: None,
            ended_at_epoch_seconds: None,
        }];

        let progress = progress_for_quest(&quest, &sessions);

        assert_eq!(progress.quest_id, 42);
        assert_eq!(progress.current, 10);
        assert_eq!(progress.target, 30);
        assert!(!progress.completed);
    }
}
