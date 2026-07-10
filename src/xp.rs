#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LevelProgress {
    pub level: u32,
    pub total_xp: u32,
    pub xp_into_level: u32,
    pub xp_for_next_level: u32,
}

pub fn required_xp_for_level(level: u32) -> u32 {
    match level.max(1) {
        1 => 100,
        2 => 150,
        3 => 220,
        4 => 300,
        level => {
            let extra_levels = level - 4;
            300 + extra_levels * 100 + triangular(extra_levels) * 25
        }
    }
}

pub fn progress_from_total_xp(total_xp: u32) -> LevelProgress {
    let mut level = 1;
    let mut remaining = total_xp;

    loop {
        let needed = required_xp_for_level(level);
        if remaining < needed {
            return LevelProgress {
                level,
                total_xp,
                xp_into_level: remaining,
                xp_for_next_level: needed,
            };
        }

        remaining -= needed;
        level += 1;
    }
}

fn triangular(value: u32) -> u32 {
    value.saturating_mul(value + 1) / 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_the_v1_level_curve() {
        assert_eq!(required_xp_for_level(1), 100);
        assert_eq!(required_xp_for_level(2), 150);
        assert_eq!(required_xp_for_level(3), 220);
        assert_eq!(required_xp_for_level(4), 300);
        assert!(required_xp_for_level(10) > required_xp_for_level(9));
    }

    #[test]
    fn converts_total_xp_into_current_level_progress() {
        assert_eq!(progress_from_total_xp(0).level, 1);
        assert_eq!(progress_from_total_xp(99).level, 1);
        assert_eq!(progress_from_total_xp(100).level, 2);
        assert_eq!(progress_from_total_xp(250).level, 3);
    }
}
