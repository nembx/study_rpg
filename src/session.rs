#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySession {
    pub id: u64,
    pub topic: String,
    pub skill_id: Option<u64>,
    pub duration_minutes: u32,
    pub earned_xp: u32,
    pub started_at_epoch_seconds: Option<u64>,
    pub ended_at_epoch_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveStudySession {
    pub topic: String,
    pub skill_id: Option<u64>,
    pub started_at_epoch_seconds: u64,
}

pub fn xp_for_duration(duration_minutes: u32) -> u32 {
    if duration_minutes == 0 {
        return 0;
    }

    duration_minutes.saturating_mul(8) / 5
}

pub fn completed_minutes_between(
    started_at_epoch_seconds: u64,
    ended_at_epoch_seconds: u64,
) -> Option<u32> {
    if ended_at_epoch_seconds <= started_at_epoch_seconds {
        return None;
    }

    let elapsed_seconds = ended_at_epoch_seconds - started_at_epoch_seconds;
    let minutes = elapsed_seconds / 60;
    if minutes == 0 {
        return None;
    }

    Some(minutes.min(u32::MAX as u64) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_design_example_for_a_focus_session() {
        assert_eq!(xp_for_duration(25), 40);
    }

    #[test]
    fn calculates_completed_minutes_from_timer_seconds() {
        assert_eq!(completed_minutes_between(100, 100 + 25 * 60), Some(25));
        assert_eq!(completed_minutes_between(100, 159), None);
        assert_eq!(completed_minutes_between(100, 99), None);
    }
}
