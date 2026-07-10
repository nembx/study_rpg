use std::collections::BTreeSet;

use crate::session::{StudySession, epoch_day};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StudyStatistics {
    pub total_sessions: u32,
    pub total_minutes: u32,
    pub total_xp: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudyStatisticsReport {
    pub today: StudyStatistics,
    pub this_week: StudyStatistics,
    pub this_month: StudyStatistics,
    pub all_time: StudyStatistics,
    pub last_seven_days: Vec<DailyStudyStatistics>,
    pub current_streak_days: u32,
    pub longest_streak_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyStudyStatistics {
    pub epoch_day: u64,
    pub statistics: StudyStatistics,
}

impl StudyStatistics {
    pub fn from_sessions(sessions: &[StudySession]) -> Self {
        Self {
            total_sessions: sessions.len() as u32,
            total_minutes: sessions
                .iter()
                .map(|session| session.duration_minutes)
                .sum(),
            total_xp: sessions.iter().map(|session| session.earned_xp).sum(),
        }
    }

    fn add_session(&mut self, session: &StudySession) {
        self.total_sessions = self.total_sessions.saturating_add(1);
        self.total_minutes = self.total_minutes.saturating_add(session.duration_minutes);
        self.total_xp = self.total_xp.saturating_add(session.earned_xp);
    }
}

impl StudyStatisticsReport {
    pub(crate) fn from_sessions_at(sessions: &[StudySession], current_epoch_seconds: u64) -> Self {
        let current_day = epoch_day(current_epoch_seconds);
        let current_week_start = week_start(current_day);
        let current_month = calendar_month(current_day);
        let activity_start_day = current_day.saturating_sub(6);
        let mut study_days = BTreeSet::new();
        let mut report = Self {
            today: StudyStatistics::default(),
            this_week: StudyStatistics::default(),
            this_month: StudyStatistics::default(),
            all_time: StudyStatistics::default(),
            last_seven_days: (activity_start_day..=current_day)
                .map(|epoch_day| DailyStudyStatistics {
                    epoch_day,
                    statistics: StudyStatistics::default(),
                })
                .collect(),
            current_streak_days: 0,
            longest_streak_days: 0,
        };

        for session in sessions {
            report.all_time.add_session(session);

            let session_day = session.ended_at_epoch_seconds.map(epoch_day).unwrap_or(0);
            if session_day <= current_day {
                study_days.insert(session_day);
            }
            if session_day == current_day {
                report.today.add_session(session);
            }
            if week_start(session_day) == current_week_start {
                report.this_week.add_session(session);
            }
            if calendar_month(session_day) == current_month {
                report.this_month.add_session(session);
            }
            if let Some(day) = report
                .last_seven_days
                .iter_mut()
                .find(|day| day.epoch_day == session_day)
            {
                day.statistics.add_session(session);
            }
        }

        report.current_streak_days = current_streak(&study_days, current_day);
        report.longest_streak_days = longest_streak(&study_days);

        report
    }
}

fn current_streak(study_days: &BTreeSet<u64>, current_day: u64) -> u32 {
    let Some(mut day) = [current_day, current_day.saturating_sub(1)]
        .into_iter()
        .find(|day| study_days.contains(day))
    else {
        return 0;
    };
    let mut streak = 0_u32;

    loop {
        if !study_days.contains(&day) {
            break;
        }
        streak = streak.saturating_add(1);
        let Some(previous_day) = day.checked_sub(1) else {
            break;
        };
        day = previous_day;
    }

    streak
}

fn longest_streak(study_days: &BTreeSet<u64>) -> u32 {
    let mut longest = 0_u32;
    let mut current = 0_u32;
    let mut previous_day = None;

    for day in study_days.iter().copied() {
        current = if previous_day.and_then(|previous: u64| previous.checked_add(1)) == Some(day) {
            current.saturating_add(1)
        } else {
            1
        };
        longest = longest.max(current);
        previous_day = Some(day);
    }

    longest
}

fn week_start(day: u64) -> u64 {
    day.saturating_sub((day + 3) % 7)
}

fn calendar_month(day: u64) -> (i64, u32) {
    let z = day as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_part = (5 * day_of_year + 2) / 153;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);

    (year, month as u32)
}
