#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StudySession {
    pub id: u64,
    pub topic: String,
    pub skill_id: Option<u64>,
    pub duration_minutes: u32,
    pub earned_xp: u32,
}

pub fn xp_for_duration(duration_minutes: u32) -> u32 {
    if duration_minutes == 0 {
        return 0;
    }

    duration_minutes.saturating_mul(8) / 5
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_design_example_for_a_focus_session() {
        assert_eq!(xp_for_duration(25), 40);
    }
}

