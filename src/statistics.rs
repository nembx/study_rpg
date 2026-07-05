use crate::session::StudySession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyStatistics {
    pub total_sessions: u32,
    pub total_minutes: u32,
    pub total_xp: u32,
}

impl StudyStatistics {
    pub fn from_sessions(sessions: &[StudySession]) -> Self {
        Self {
            total_sessions: sessions.len() as u32,
            total_minutes: sessions.iter().map(|session| session.duration_minutes).sum(),
            total_xp: sessions.iter().map(|session| session.earned_xp).sum(),
        }
    }
}

