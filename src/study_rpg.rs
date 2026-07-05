use crate::player::{CharacterClass, Player, XpGrant};
use crate::quest::{Quest, evaluate_quests};
use crate::session::{StudySession, xp_for_duration};
use crate::skill::Skill;
use crate::statistics::StudyStatistics;
use crate::xp::LevelProgress;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionInput {
    pub topic: String,
    pub skill_id: Option<u64>,
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionResult {
    pub session: StudySession,
    pub player_xp: XpGrant,
    pub completed_quests: Vec<Quest>,
    pub quest_reward_xp: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dashboard {
    pub player_name: String,
    pub title: String,
    pub level: LevelProgress,
    pub today_minutes: u32,
    pub total_sessions: u32,
    pub active_quests: Vec<Quest>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyRpg {
    player: Player,
    skills: Vec<Skill>,
    sessions: Vec<StudySession>,
    daily_quests: Vec<Quest>,
    next_skill_id: u64,
    next_session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyRpgSnapshot {
    pub player: Player,
    pub skills: Vec<Skill>,
    pub sessions: Vec<StudySession>,
    pub daily_quests: Vec<Quest>,
}

impl StudyRpg {
    pub fn new(player_name: impl Into<String>, class: CharacterClass) -> Self {
        Self {
            player: Player::new(player_name, class),
            skills: Vec::new(),
            sessions: Vec::new(),
            daily_quests: default_daily_quests(),
            next_skill_id: 1,
            next_session_id: 1,
        }
    }

    pub fn from_snapshot(snapshot: StudyRpgSnapshot) -> Self {
        let next_skill_id = next_id(snapshot.skills.iter().map(|skill| skill.id));
        let next_session_id = next_id(snapshot.sessions.iter().map(|session| session.id));

        Self {
            player: snapshot.player,
            skills: snapshot.skills,
            sessions: snapshot.sessions,
            daily_quests: snapshot.daily_quests,
            next_skill_id,
            next_session_id,
        }
    }

    pub fn snapshot(&self) -> StudyRpgSnapshot {
        StudyRpgSnapshot {
            player: self.player.clone(),
            skills: self.skills.clone(),
            sessions: self.sessions.clone(),
            daily_quests: self.daily_quests.clone(),
        }
    }

    pub fn player(&self) -> &Player {
        &self.player
    }

    pub fn skills(&self) -> &[Skill] {
        &self.skills
    }

    pub fn sessions(&self) -> &[StudySession] {
        &self.sessions
    }

    pub fn daily_quests(&self) -> &[Quest] {
        &self.daily_quests
    }

    pub fn add_skill(&mut self, name: impl Into<String>, parent_id: Option<u64>) -> u64 {
        let id = self.next_skill_id;
        self.next_skill_id += 1;
        self.skills.push(Skill::new(id, name, parent_id));
        id
    }

    pub fn complete_study_session(&mut self, input: StudySessionInput) -> StudySessionResult {
        let earned_xp = xp_for_duration(input.duration_minutes);
        let session = StudySession {
            id: self.next_session_id,
            topic: input.topic,
            skill_id: input.skill_id,
            duration_minutes: input.duration_minutes,
            earned_xp,
        };
        self.next_session_id += 1;

        self.sessions.push(session.clone());
        if let Some(skill_id) = session.skill_id {
            if let Some(skill) = self.skills.iter_mut().find(|skill| skill.id == skill_id) {
                skill.grant_xp(earned_xp);
            }
        }

        let completed_quests = evaluate_quests(&mut self.daily_quests, &self.sessions);
        let quest_reward_xp = completed_quests
            .iter()
            .map(|quest| quest.reward_xp)
            .sum::<u32>();
        let player_xp = self.player.grant_xp(earned_xp + quest_reward_xp);

        StudySessionResult {
            session,
            player_xp,
            completed_quests,
            quest_reward_xp,
        }
    }

    pub fn dashboard(&self) -> Dashboard {
        let statistics = self.statistics();

        Dashboard {
            player_name: self.player.name.clone(),
            title: self.player.title.clone(),
            level: self.player.level_progress(),
            today_minutes: statistics.total_minutes,
            total_sessions: statistics.total_sessions,
            active_quests: self
                .daily_quests
                .iter()
                .filter(|quest| !quest.completed)
                .cloned()
                .collect(),
        }
    }

    pub fn statistics(&self) -> StudyStatistics {
        StudyStatistics::from_sessions(&self.sessions)
    }
}

fn default_daily_quests() -> Vec<Quest> {
    vec![
        Quest::study_minutes(1, 30, 60),
        Quest::complete_sessions(2, 1, 40),
    ]
}

fn next_id(ids: impl Iterator<Item = u64>) -> u64 {
    ids.max().unwrap_or(0) + 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completing_a_session_updates_the_core_loop() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);

        let result = app.complete_study_session(StudySessionInput {
            topic: "Rust ownership".to_string(),
            skill_id: Some(rust),
            duration_minutes: 25,
        });

        assert_eq!(result.session.earned_xp, 40);
        assert_eq!(result.quest_reward_xp, 40);
        assert_eq!(result.player_xp.gained_xp, 80);
        assert_eq!(app.sessions().len(), 1);
        assert_eq!(app.skills()[0].total_xp, 40);
        assert_eq!(app.dashboard().today_minutes, 25);
    }

    #[test]
    fn thirty_minutes_completes_the_daily_minutes_quest() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

        let result = app.complete_study_session(StudySessionInput {
            topic: "Reading".to_string(),
            skill_id: None,
            duration_minutes: 30,
        });

        assert_eq!(result.session.earned_xp, 48);
        assert_eq!(result.completed_quests.len(), 2);
        assert_eq!(result.quest_reward_xp, 100);
        assert_eq!(app.dashboard().active_quests.len(), 0);
    }

    #[test]
    fn restored_state_continues_allocating_ids_after_existing_records() {
        let snapshot = StudyRpgSnapshot {
            player: Player::new("Nembx", CharacterClass::Scholar),
            skills: vec![Skill::new(7, "Rust", None)],
            sessions: vec![StudySession {
                id: 12,
                topic: "Rust ownership".to_string(),
                skill_id: Some(7),
                duration_minutes: 25,
                earned_xp: 40,
            }],
            daily_quests: default_daily_quests(),
        };
        let mut app = StudyRpg::from_snapshot(snapshot);

        let next_skill = app.add_skill("SQLite", None);
        let result = app.complete_study_session(StudySessionInput {
            topic: "Persistence".to_string(),
            skill_id: Some(next_skill),
            duration_minutes: 10,
        });

        assert_eq!(next_skill, 8);
        assert_eq!(result.session.id, 13);
    }
}
