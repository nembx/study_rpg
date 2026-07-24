use study_rpg::{CharacterClass, GrowthEventKind, SqliteStore, StudyRpg, StudySessionInput};

#[test]
fn completed_session_records_player_and_skill_growth_history() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let rust = app.add_skill("Rust", None);

    let result = app.complete_study_session(StudySessionInput {
        topic: "Rust ownership".to_string(),
        skill_id: Some(rust),
        duration_minutes: 30,
    });

    let history = app.dashboard().growth_history;

    assert_eq!(
        result.growth_events,
        history.iter().rev().cloned().collect::<Vec<_>>()
    );
    assert_eq!(history.len(), 2);
    assert_eq!(history[0].topic, "Rust ownership");
    assert_eq!(
        history[0].kind,
        GrowthEventKind::PlayerLevelChange {
            gained_xp: 298,
            level_before: 1,
            level_after: 3,
            total_xp_after: 298,
        }
    );
    assert_eq!(
        history[1].kind,
        GrowthEventKind::SkillGrowth {
            skill_id: rust,
            skill_name: "Rust".to_string(),
            gained_xp: 48,
            level_before: 1,
            level_after: 1,
            total_xp_after: 48,
        }
    );
}

#[test]
fn session_without_a_level_change_does_not_record_player_level_change() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let rust = app.add_skill("Rust", None);

    let result = app.complete_study_session(StudySessionInput {
        topic: "Rust syntax".to_string(),
        skill_id: Some(rust),
        duration_minutes: 1,
    });

    assert_eq!(result.player_xp.before.level, 1);
    assert_eq!(result.player_xp.after.level, 1);
    assert!(
        result
            .growth_events
            .iter()
            .all(|event| !matches!(event.kind, GrowthEventKind::PlayerLevelChange { .. }))
    );
}

#[test]
fn skill_growth_records_the_levels_crossed_by_this_session() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let rust = app.add_skill("Rust", None);
    for topic in ["Rust ownership", "Rust borrowing"] {
        app.complete_study_session(StudySessionInput {
            topic: topic.to_string(),
            skill_id: Some(rust),
            duration_minutes: 30,
        });
    }

    let result = app.complete_study_session(StudySessionInput {
        topic: "Rust lifetimes".to_string(),
        skill_id: Some(rust),
        duration_minutes: 10,
    });

    assert!(result.growth_events.iter().any(|event| {
        matches!(
            &event.kind,
            GrowthEventKind::SkillGrowth {
                skill_id,
                skill_name,
                gained_xp: 16,
                level_before: 1,
                level_after: 2,
                total_xp_after: 112,
            } if *skill_id == rust && skill_name == "Rust"
        )
    }));
}

#[test]
fn growth_history_survives_restart_and_continues_event_ordering() {
    let mut store = SqliteStore::in_memory().unwrap();
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let rust = app.add_skill("Rust", None);
    app.complete_study_session_at(
        StudySessionInput {
            topic: "Rust ownership".to_string(),
            skill_id: Some(rust),
            duration_minutes: 30,
        },
        2_000,
    );
    let expected_history = app.dashboard().growth_history;
    store.save(&app).unwrap();

    let mut restored = store.load().unwrap().unwrap();

    assert_eq!(restored.dashboard().growth_history, expected_history);
    restored.complete_study_session(StudySessionInput {
        topic: "Rust lifetimes".to_string(),
        skill_id: Some(rust),
        duration_minutes: 10,
    });
    assert_eq!(restored.dashboard().growth_history[0].id, 3);
}

#[test]
fn dashboard_exposes_current_skill_progress_for_follow_up_sessions() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let rust = app.add_skill("Rust", None);
    app.complete_study_session(StudySessionInput {
        topic: "Rust ownership".to_string(),
        skill_id: Some(rust),
        duration_minutes: 30,
    });

    let skills = app.dashboard().skills;

    assert_eq!(skills.len(), 1);
    assert_eq!(skills[0].name, "Rust");
    assert_eq!(skills[0].level, 1);
    assert_eq!(skills[0].total_xp, 48);
    assert_eq!(skills[0].xp_into_level, 48);
    assert_eq!(skills[0].xp_for_next_level, 100);
    assert_eq!(skills[0].xp_progress_percent, 48);
}
