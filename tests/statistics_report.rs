use study_rpg::{
    CharacterClass, StudyRpg, StudySessionInput, StudySessionStartInput,
};

const DAY: u64 = 86_400;

#[test]
fn statistics_report_groups_sessions_by_calendar_period() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

    record_session_at(&mut app, 30, 10);
    record_session_at(&mut app, 31, 15);
    record_session_at(&mut app, 33, 20);
    record_session_at(&mut app, 34, 30);

    let report = app.statistics_at(34 * DAY + 12 * 60 * 60);

    assert_eq!(report.today.total_sessions, 1);
    assert_eq!(report.today.total_minutes, 30);
    assert_eq!(report.today.total_xp, 48);

    assert_eq!(report.this_week.total_sessions, 2);
    assert_eq!(report.this_week.total_minutes, 50);
    assert_eq!(report.this_week.total_xp, 80);

    assert_eq!(report.this_month.total_sessions, 3);
    assert_eq!(report.this_month.total_minutes, 65);
    assert_eq!(report.this_month.total_xp, 104);

    assert_eq!(report.all_time.total_sessions, 4);
    assert_eq!(report.all_time.total_minutes, 75);
    assert_eq!(report.all_time.total_xp, 120);
}

#[test]
fn statistics_report_includes_a_dense_seven_day_activity_series() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

    record_session_at(&mut app, 30, 10);
    record_session_at(&mut app, 33, 20);
    record_session_at(&mut app, 34, 30);

    let report = app.statistics_at(34 * DAY + 12 * 60 * 60);
    let activity = report
        .last_seven_days
        .iter()
        .map(|day| (day.epoch_day, day.statistics.total_minutes))
        .collect::<Vec<_>>();

    assert_eq!(
        activity,
        vec![(28, 0), (29, 0), (30, 10), (31, 0), (32, 0), (33, 20), (34, 30)]
    );
}

#[test]
fn statistics_report_calculates_current_and_longest_study_streaks() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

    for day in [20, 21, 22, 30, 31, 33, 34] {
        record_session_at(&mut app, day, 10);
    }

    let report = app.statistics_at(34 * DAY + 12 * 60 * 60);

    assert_eq!(report.current_streak_days, 2);
    assert_eq!(report.longest_streak_days, 3);
}

#[test]
fn timestamped_manual_session_keeps_its_original_calendar_day() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

    app.complete_study_session_at(
        StudySessionInput {
            topic: "Manual review".to_string(),
            skill_id: None,
            duration_minutes: 10,
        },
        33 * DAY + 12 * 60 * 60,
    );

    let next_day = app.statistics_at(34 * DAY + 12 * 60 * 60);
    let two_days_later = app.statistics_at(35 * DAY + 12 * 60 * 60);

    assert_eq!(next_day.today.total_minutes, 0);
    assert_eq!(next_day.this_week.total_minutes, 10);
    assert_eq!(next_day.last_seven_days[5].statistics.total_minutes, 10);
    assert_eq!(two_days_later.today.total_minutes, 0);
    assert_eq!(two_days_later.last_seven_days[4].statistics.total_minutes, 10);
}

fn record_session_at(app: &mut StudyRpg, epoch_day: u64, duration_minutes: u32) {
    let started_at = epoch_day * DAY + 60;
    app.start_study_session(
        StudySessionStartInput {
            topic: format!("Day {epoch_day}"),
            skill_id: None,
        },
        started_at,
    )
    .unwrap();
    app.finish_active_study_session(started_at + u64::from(duration_minutes) * 60)
        .unwrap();
}
