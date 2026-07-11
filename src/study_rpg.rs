use crate::player::{CharacterClass, Player, XpGrant};
use crate::quest::{Quest, evaluate_quests, progress_for_quest};
use crate::session::{
    ActiveStudySession, StudySession, completed_minutes_between, epoch_day, xp_for_duration,
};
use crate::skill::Skill;
use crate::statistics::{StudyStatistics, StudyStatisticsReport};
use crate::xp::LevelProgress;

const DAILY_COMPLETION_BONUS_XP: u32 = 150;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionInput {
    pub topic: String,
    pub skill_id: Option<u64>,
    pub duration_minutes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionStartInput {
    pub topic: String,
    pub skill_id: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySessionResult {
    pub session: StudySession,
    pub player_xp: XpGrant,
    pub completed_quests: Vec<Quest>,
    pub daily_completion_bonus_xp: u32,
    pub quest_reward_xp: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StudyRpgError {
    StudySessionAlreadyActive,
    NoActiveStudySession,
    StudySessionTooShort,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dashboard {
    pub player_name: String,
    pub title: String,
    pub energy: u8,
    pub level: LevelProgress,
    pub total_xp: u32,
    pub xp_progress_percent: u8,
    pub today_minutes: u32,
    pub total_sessions: u32,
    pub quest_progress: Vec<DashboardQuest>,
    pub daily_quest_completion: DashboardDailyQuestCompletion,
    pub recent_sessions: Vec<DashboardSession>,
    pub active_session: Option<DashboardActiveSession>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardDailyQuestCompletion {
    pub completed: bool,
    pub reward_xp: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardQuest {
    pub id: u64,
    pub epoch_day: u64,
    pub title: String,
    pub current: u32,
    pub target: u32,
    pub reward_xp: u32,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSession {
    pub id: u64,
    pub topic: String,
    pub skill_name: Option<String>,
    pub duration_minutes: u32,
    pub earned_xp: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardActiveSession {
    pub topic: String,
    pub skill_id: Option<u64>,
    pub skill_name: Option<String>,
    pub started_at_epoch_seconds: u64,
    pub elapsed_minutes: u32,
    pub estimated_xp: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyRpg {
    player: Player,
    skills: Vec<Skill>,
    sessions: Vec<StudySession>,
    daily_quests: Vec<Quest>,
    daily_completion_bonus_claimed: bool,
    active_session: Option<ActiveStudySession>,
    next_skill_id: u64,
    next_session_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyRpgSnapshot {
    pub player: Player,
    pub skills: Vec<Skill>,
    pub sessions: Vec<StudySession>,
    pub daily_quests: Vec<Quest>,
    pub daily_completion_bonus_claimed: bool,
    pub active_session: Option<ActiveStudySession>,
}

impl StudyRpg {
    pub fn new(player_name: impl Into<String>, class: CharacterClass) -> Self {
        Self {
            player: Player::new(player_name, class),
            skills: Vec::new(),
            sessions: Vec::new(),
            daily_quests: default_daily_quests(),
            daily_completion_bonus_claimed: false,
            active_session: None,
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
            daily_completion_bonus_claimed: snapshot.daily_completion_bonus_claimed,
            active_session: snapshot.active_session,
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
            daily_completion_bonus_claimed: self.daily_completion_bonus_claimed,
            active_session: self.active_session.clone(),
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

    pub fn active_session(&self) -> Option<&ActiveStudySession> {
        self.active_session.as_ref()
    }

    pub fn refresh_daily_quests_at(&mut self, current_epoch_seconds: u64) -> bool {
        let current_day = epoch_day(current_epoch_seconds);
        if !self.daily_quests.is_empty()
            && self
                .daily_quests
                .iter()
                .all(|quest| quest.epoch_day == current_day)
        {
            return false;
        }

        self.daily_quests = default_daily_quests_for_day(current_day);
        self.daily_completion_bonus_claimed = false;
        true
    }

    pub fn add_skill(&mut self, name: impl Into<String>, parent_id: Option<u64>) -> u64 {
        let id = self.next_skill_id;
        self.next_skill_id += 1;
        self.skills.push(Skill::new(id, name, parent_id));
        id
    }

    pub fn start_study_session(
        &mut self,
        input: StudySessionStartInput,
        started_at_epoch_seconds: u64,
    ) -> Result<ActiveStudySession, StudyRpgError> {
        if self.active_session.is_some() {
            return Err(StudyRpgError::StudySessionAlreadyActive);
        }

        let active_session = ActiveStudySession {
            topic: input.topic,
            skill_id: input.skill_id,
            started_at_epoch_seconds,
        };

        self.active_session = Some(active_session.clone());

        Ok(active_session)
    }

    pub fn finish_active_study_session(
        &mut self,
        ended_at_epoch_seconds: u64,
    ) -> Result<StudySessionResult, StudyRpgError> {
        let active_session = self
            .active_session
            .clone()
            .ok_or(StudyRpgError::NoActiveStudySession)?;
        let duration_minutes = completed_minutes_between(
            active_session.started_at_epoch_seconds,
            ended_at_epoch_seconds,
        )
        .ok_or(StudyRpgError::StudySessionTooShort)?;

        self.active_session = None;
        self.refresh_daily_quests_at(ended_at_epoch_seconds);

        Ok(self.record_completed_session(
            StudySessionInput {
                topic: active_session.topic,
                skill_id: active_session.skill_id,
                duration_minutes,
            },
            Some(active_session.started_at_epoch_seconds),
            Some(ended_at_epoch_seconds),
        ))
    }

    pub fn complete_study_session(&mut self, input: StudySessionInput) -> StudySessionResult {
        self.record_completed_session(input, None, None)
    }

    pub fn complete_study_session_at(
        &mut self,
        input: StudySessionInput,
        ended_at_epoch_seconds: u64,
    ) -> StudySessionResult {
        self.refresh_daily_quests_at(ended_at_epoch_seconds);
        self.record_completed_session(input, None, Some(ended_at_epoch_seconds))
    }

    fn record_completed_session(
        &mut self,
        input: StudySessionInput,
        started_at_epoch_seconds: Option<u64>,
        ended_at_epoch_seconds: Option<u64>,
    ) -> StudySessionResult {
        let earned_xp = xp_for_duration(input.duration_minutes);
        let session = StudySession {
            id: self.next_session_id,
            topic: input.topic,
            skill_id: input.skill_id,
            duration_minutes: input.duration_minutes,
            earned_xp,
            started_at_epoch_seconds,
            ended_at_epoch_seconds,
        };
        self.next_session_id += 1;

        self.sessions.push(session.clone());
        if let Some(skill_id) = session.skill_id
            && let Some(skill) = self.skills.iter_mut().find(|skill| skill.id == skill_id)
        {
            skill.grant_xp(earned_xp);
        }

        let completed_quests = evaluate_quests(&mut self.daily_quests, &self.sessions);
        let quest_reward_xp = completed_quests
            .iter()
            .map(|quest| quest.reward_xp)
            .sum::<u32>();
        let daily_completion_bonus_xp = if !self.daily_completion_bonus_claimed
            && !self.daily_quests.is_empty()
            && self.daily_quests.iter().all(|quest| quest.completed)
        {
            self.daily_completion_bonus_claimed = true;
            DAILY_COMPLETION_BONUS_XP
        } else {
            0
        };
        let player_xp = self.player.grant_xp(
            earned_xp
                .saturating_add(quest_reward_xp)
                .saturating_add(daily_completion_bonus_xp),
        );

        StudySessionResult {
            session,
            player_xp,
            completed_quests,
            daily_completion_bonus_xp,
            quest_reward_xp,
        }
    }

    pub fn dashboard(&self) -> Dashboard {
        self.build_dashboard(None)
    }

    pub fn dashboard_at(&self, current_epoch_seconds: u64) -> Dashboard {
        self.build_dashboard(Some(current_epoch_seconds))
    }

    fn build_dashboard(&self, current_epoch_seconds: Option<u64>) -> Dashboard {
        let statistics = self.statistics();
        let level = self.player.level_progress();

        Dashboard {
            player_name: self.player.name.clone(),
            title: self.player.title.clone(),
            energy: self.player.energy,
            level,
            total_xp: self.player.total_xp,
            xp_progress_percent: xp_progress_percent(level),
            today_minutes: current_epoch_seconds
                .map(|now| self.study_minutes_for_day(now))
                .unwrap_or(statistics.total_minutes),
            total_sessions: statistics.total_sessions,
            quest_progress: self.dashboard_quests(),
            daily_quest_completion: DashboardDailyQuestCompletion {
                completed: !self.daily_quests.is_empty()
                    && self.daily_quests.iter().all(|quest| quest.completed),
                reward_xp: DAILY_COMPLETION_BONUS_XP,
            },
            recent_sessions: self.recent_sessions(5),
            active_session: self.dashboard_active_session(current_epoch_seconds),
        }
    }

    pub fn statistics(&self) -> StudyStatistics {
        StudyStatistics::from_sessions(&self.sessions)
    }

    pub fn statistics_at(&self, current_epoch_seconds: u64) -> StudyStatisticsReport {
        StudyStatisticsReport::from_sessions_at(&self.sessions, current_epoch_seconds)
    }

    fn dashboard_quests(&self) -> Vec<DashboardQuest> {
        self.daily_quests
            .iter()
            .map(|quest| {
                let progress = progress_for_quest(quest, &self.sessions);
                DashboardQuest {
                    id: quest.id,
                    epoch_day: quest.epoch_day,
                    title: quest.title.clone(),
                    current: progress.current.min(progress.target),
                    target: progress.target,
                    reward_xp: quest.reward_xp,
                    completed: progress.completed,
                }
            })
            .collect()
    }

    fn recent_sessions(&self, limit: usize) -> Vec<DashboardSession> {
        self.sessions
            .iter()
            .rev()
            .take(limit)
            .map(|session| DashboardSession {
                id: session.id,
                topic: session.topic.clone(),
                skill_name: self.skill_name(session.skill_id),
                duration_minutes: session.duration_minutes,
                earned_xp: session.earned_xp,
            })
            .collect()
    }

    fn dashboard_active_session(
        &self,
        current_epoch_seconds: Option<u64>,
    ) -> Option<DashboardActiveSession> {
        self.active_session.as_ref().map(|active_session| {
            let elapsed_minutes = current_epoch_seconds
                .map(|now| elapsed_minutes_since(active_session.started_at_epoch_seconds, now))
                .unwrap_or(0);

            DashboardActiveSession {
                topic: active_session.topic.clone(),
                skill_id: active_session.skill_id,
                skill_name: self.skill_name(active_session.skill_id),
                started_at_epoch_seconds: active_session.started_at_epoch_seconds,
                elapsed_minutes,
                estimated_xp: xp_for_duration(elapsed_minutes),
            }
        })
    }

    fn skill_name(&self, skill_id: Option<u64>) -> Option<String> {
        skill_id.and_then(|id| {
            self.skills
                .iter()
                .find(|skill| skill.id == id)
                .map(|skill| skill.name.clone())
        })
    }

    fn study_minutes_for_day(&self, current_epoch_seconds: u64) -> u32 {
        let current_day = epoch_day(current_epoch_seconds);
        self.sessions
            .iter()
            .filter(|session| {
                session
                    .ended_at_epoch_seconds
                    .map(|ended_at| epoch_day(ended_at) == current_day)
                    .unwrap_or(current_day == 0)
            })
            .map(|session| session.duration_minutes)
            .sum()
    }
}

fn default_daily_quests() -> Vec<Quest> {
    default_daily_quests_for_day(0)
}

fn default_daily_quests_for_day(epoch_day: u64) -> Vec<Quest> {
    vec![
        Quest::study_minutes_for_day(1, epoch_day, 30, 60),
        Quest::complete_sessions_for_day(2, epoch_day, 1, 40),
    ]
}

fn next_id(ids: impl Iterator<Item = u64>) -> u64 {
    ids.max().unwrap_or(0) + 1
}

fn xp_progress_percent(level: LevelProgress) -> u8 {
    if level.xp_for_next_level == 0 {
        return 100;
    }

    ((level.xp_into_level.saturating_mul(100)) / level.xp_for_next_level).min(100) as u8
}

fn elapsed_minutes_since(started_at_epoch_seconds: u64, current_epoch_seconds: u64) -> u32 {
    if current_epoch_seconds <= started_at_epoch_seconds {
        return 0;
    }

    ((current_epoch_seconds - started_at_epoch_seconds) / 60).min(u32::MAX as u64) as u32
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
        assert_eq!(result.daily_completion_bonus_xp, 150);
        assert_eq!(result.quest_reward_xp, 100);
        assert!(
            app.dashboard()
                .quest_progress
                .iter()
                .all(|quest| quest.completed)
        );
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
                started_at_epoch_seconds: None,
                ended_at_epoch_seconds: None,
            }],
            daily_quests: default_daily_quests(),
            daily_completion_bonus_claimed: false,
            active_session: None,
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

    #[test]
    fn timed_session_finishes_through_the_core_loop() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);

        let active = app
            .start_study_session(
                StudySessionStartInput {
                    topic: "Rust ownership".to_string(),
                    skill_id: Some(rust),
                },
                1_000,
            )
            .unwrap();
        assert_eq!(active.topic, "Rust ownership");
        assert_eq!(app.active_session().unwrap().skill_id, Some(rust));

        let result = app.finish_active_study_session(1_000 + 25 * 60).unwrap();

        assert!(app.active_session().is_none());
        assert_eq!(result.session.duration_minutes, 25);
        assert_eq!(result.session.earned_xp, 40);
        assert_eq!(result.session.started_at_epoch_seconds, Some(1_000));
        assert_eq!(result.session.ended_at_epoch_seconds, Some(1_000 + 25 * 60));
        assert_eq!(result.player_xp.gained_xp, 80);
        assert_eq!(app.skills()[0].total_xp, 40);
    }

    #[test]
    fn timer_rejects_double_start_and_too_short_sessions() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

        app.start_study_session(
            StudySessionStartInput {
                topic: "Reading".to_string(),
                skill_id: None,
            },
            1_000,
        )
        .unwrap();

        assert_eq!(
            app.start_study_session(
                StudySessionStartInput {
                    topic: "Second session".to_string(),
                    skill_id: None,
                },
                1_010,
            ),
            Err(StudyRpgError::StudySessionAlreadyActive)
        );

        assert_eq!(
            app.finish_active_study_session(1_030),
            Err(StudyRpgError::StudySessionTooShort)
        );
        assert!(app.active_session().is_some());
    }

    #[test]
    fn refresh_daily_quests_replaces_stale_quests_for_the_current_day() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        app.complete_study_session(StudySessionInput {
            topic: "Manual session".to_string(),
            skill_id: None,
            duration_minutes: 30,
        });
        assert!(app.daily_quests().iter().all(|quest| quest.completed));

        assert!(app.refresh_daily_quests_at(86_400));

        assert!(app.daily_quests().iter().all(|quest| quest.epoch_day == 1));
        assert!(app.daily_quests().iter().all(|quest| !quest.completed));
        assert_eq!(app.dashboard().quest_progress[0].current, 0);
    }

    #[test]
    fn finishing_timed_session_refreshes_daily_quests_by_end_time() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

        app.start_study_session(
            StudySessionStartInput {
                topic: "Day one study".to_string(),
                skill_id: None,
            },
            86_400 + 10,
        )
        .unwrap();
        app.finish_active_study_session(86_400 + 10 + 30 * 60)
            .unwrap();

        assert!(app.daily_quests().iter().all(|quest| quest.epoch_day == 1));
        assert!(
            app.dashboard()
                .quest_progress
                .iter()
                .all(|quest| quest.completed)
        );
    }

    #[test]
    fn dashboard_exposes_quest_progress_and_recent_sessions() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);

        app.complete_study_session(StudySessionInput {
            topic: "Ownership".to_string(),
            skill_id: Some(rust),
            duration_minutes: 10,
        });
        app.complete_study_session(StudySessionInput {
            topic: "Borrowing".to_string(),
            skill_id: Some(rust),
            duration_minutes: 5,
        });

        let dashboard = app.dashboard();

        assert_eq!(dashboard.total_xp, 64);
        assert_eq!(dashboard.xp_progress_percent, 64);
        assert_eq!(dashboard.quest_progress[0].current, 15);
        assert_eq!(dashboard.quest_progress[0].epoch_day, 0);
        assert_eq!(dashboard.quest_progress[0].target, 30);
        assert!(!dashboard.quest_progress[0].completed);
        assert!(dashboard.quest_progress[1].completed);
        assert_eq!(dashboard.recent_sessions[0].topic, "Borrowing");
        assert_eq!(
            dashboard.recent_sessions[0].skill_name,
            Some("Rust".to_string())
        );
        assert_eq!(dashboard.recent_sessions[1].topic, "Ownership");
    }

    #[test]
    fn dashboard_exposes_player_energy() {
        let app = StudyRpg::new("Nembx", CharacterClass::Scholar);

        assert_eq!(app.dashboard().energy, 100);
    }

    #[test]
    fn dashboard_at_summarizes_active_session_progress() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);

        app.start_study_session(
            StudySessionStartInput {
                topic: "Rust ownership".to_string(),
                skill_id: Some(rust),
            },
            1_000,
        )
        .unwrap();

        let dashboard = app.dashboard_at(1_000 + 25 * 60);
        let active = dashboard.active_session.unwrap();

        assert_eq!(active.topic, "Rust ownership");
        assert_eq!(active.skill_name, Some("Rust".to_string()));
        assert_eq!(active.elapsed_minutes, 25);
        assert_eq!(active.estimated_xp, 40);
    }

    #[test]
    fn dashboard_at_counts_minutes_for_the_current_epoch_day() {
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

        app.start_study_session(
            StudySessionStartInput {
                topic: "Yesterday".to_string(),
                skill_id: None,
            },
            10,
        )
        .unwrap();
        app.finish_active_study_session(10 + 10 * 60).unwrap();

        app.start_study_session(
            StudySessionStartInput {
                topic: "Today".to_string(),
                skill_id: None,
            },
            86_400 + 10,
        )
        .unwrap();
        app.finish_active_study_session(86_400 + 10 + 15 * 60)
            .unwrap();

        assert_eq!(app.dashboard_at(86_400 + 3_600).today_minutes, 15);
    }
}
