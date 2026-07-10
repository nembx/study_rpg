use study_rpg::{CharacterClass, SqliteStore, StudyRpg, StudySessionInput};

#[test]
fn completing_all_daily_quests_grants_the_daily_completion_bonus() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

    let result = app.complete_study_session(StudySessionInput {
        topic: "Focused reading".to_string(),
        skill_id: None,
        duration_minutes: 30,
    });

    assert_eq!(result.session.earned_xp, 48);
    assert_eq!(result.completed_quests.len(), 2);
    assert_eq!(result.daily_completion_bonus_xp, 150);
    assert_eq!(result.quest_reward_xp, 100);
    assert_eq!(result.player_xp.gained_xp, 298);
}

#[test]
fn daily_completion_bonus_is_only_granted_once_per_day() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    app.complete_study_session(StudySessionInput {
        topic: "First session".to_string(),
        skill_id: None,
        duration_minutes: 30,
    });

    let result = app.complete_study_session(StudySessionInput {
        topic: "Second session".to_string(),
        skill_id: None,
        duration_minutes: 10,
    });

    assert_eq!(result.daily_completion_bonus_xp, 0);
    assert_eq!(result.quest_reward_xp, 0);
    assert_eq!(result.player_xp.gained_xp, 16);
}

#[test]
fn dashboard_exposes_daily_quest_completion_feedback() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    app.complete_study_session(StudySessionInput {
        topic: "Complete the daily quests".to_string(),
        skill_id: None,
        duration_minutes: 30,
    });

    let dashboard = app.dashboard();

    assert!(dashboard.daily_quest_completion.completed);
    assert_eq!(dashboard.daily_quest_completion.reward_xp, 150);
}

#[test]
fn restored_daily_quest_state_does_not_grant_the_completion_bonus_again() {
    let mut store = SqliteStore::in_memory().unwrap();
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    app.complete_study_session(StudySessionInput {
        topic: "Finish daily quests".to_string(),
        skill_id: None,
        duration_minutes: 30,
    });
    store.save(&app).unwrap();

    let mut restored = store.load().unwrap().unwrap();
    let result = restored.complete_study_session(StudySessionInput {
        topic: "Continue studying".to_string(),
        skill_id: None,
        duration_minutes: 10,
    });

    assert_eq!(result.daily_completion_bonus_xp, 0);
    assert_eq!(result.quest_reward_xp, 0);
    assert_eq!(result.player_xp.gained_xp, 16);
}

#[test]
fn a_new_day_can_grant_a_new_daily_completion_bonus() {
    const DAY: u64 = 86_400;

    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let first_day = app.complete_study_session_at(
        StudySessionInput {
            topic: "Day one".to_string(),
            skill_id: None,
            duration_minutes: 30,
        },
        DAY + 60,
    );

    let second_day = app.complete_study_session_at(
        StudySessionInput {
            topic: "Day two".to_string(),
            skill_id: None,
            duration_minutes: 30,
        },
        2 * DAY + 60,
    );

    assert_eq!(first_day.daily_completion_bonus_xp, 150);
    assert_eq!(second_day.daily_completion_bonus_xp, 150);
}
