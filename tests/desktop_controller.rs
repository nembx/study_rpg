use study_rpg::{
    CharacterClass, CompanionMode, CompanionPreferences, DesktopController, GrowthEventKind,
    SqliteStore, StudyRpgError,
};

#[cfg(unix)]
use study_rpg::DesktopError;

#[test]
fn desktop_controller_starts_a_study_session_from_the_topic_input() {
    let store = SqliteStore::in_memory().unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 1_000).unwrap();

    desktop.start_session("  Rust ownership  ", 1_000).unwrap();

    let dashboard = desktop.dashboard_at(1_060).unwrap();
    let active = dashboard.active_session.unwrap();
    assert_eq!(active.topic, "Rust ownership");
    assert_eq!(active.elapsed_minutes, 1);
}

#[test]
fn desktop_controller_records_growth_for_the_selected_skill_name() {
    let store = SqliteStore::in_memory().unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 1_000).unwrap();

    desktop
        .start_session_with_skill("Rust ownership", Some("  Rust  "), 1_000)
        .unwrap();

    let active = desktop.dashboard_at(1_060).unwrap().active_session.unwrap();
    assert_eq!(active.skill_name.as_deref(), Some("Rust"));

    desktop.finish_session(1_000 + 30 * 60).unwrap();
    let history = desktop
        .dashboard_at(1_000 + 30 * 60)
        .unwrap()
        .growth_history;
    assert!(history.iter().any(|event| matches!(
        &event.kind,
        GrowthEventKind::SkillGrowth { skill_name, .. } if skill_name == "Rust"
    )));
}

#[test]
fn desktop_controller_restores_a_child_skill_timer_and_its_growth() {
    let database_path = std::env::temp_dir().join(format!(
        "study-rpg-skill-tree-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut desktop =
            DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 1_000)
                .unwrap();
        let parent = desktop.create_skill("编程", None).unwrap();
        let child = desktop.create_skill("Rust", Some(parent)).unwrap();
        assert_eq!(child, 2);
        assert_eq!(
            desktop.dashboard_at(1_000).unwrap().skills[1].parent_id,
            Some(parent)
        );
        desktop
            .start_session_with_skill_id("Ownership", Some(child), 1_000)
            .unwrap();
    }

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut desktop = DesktopController::load(store, 1_000).unwrap();
        let dashboard = desktop.dashboard_at(1_000).unwrap();
        assert_eq!(dashboard.skills.len(), 2);
        assert_eq!(dashboard.skills[1].name, "Rust");
        assert_eq!(dashboard.skills[1].depth, 1);
        assert_eq!(dashboard.active_session.unwrap().skill_id, Some(2));
        let result = desktop.finish_session(1_000 + 25 * 60).unwrap();
        assert_eq!(result.session.skill_id, Some(2));
    }

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut desktop = DesktopController::load(store, 1_000 + 25 * 60).unwrap();
        let dashboard = desktop.dashboard_at(1_000 + 25 * 60).unwrap();
        assert!(dashboard.active_session.is_none());
        assert_eq!(dashboard.skills[0].total_xp, 0);
        assert_eq!(dashboard.skills[1].total_xp, 40);
        assert!(dashboard.growth_history.iter().any(|event| matches!(
            event.kind,
            GrowthEventKind::SkillGrowth {
                skill_id: 2,
                gained_xp: 40,
                ..
            }
        )));
    }

    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn desktop_controller_surfaces_invalid_skill_tree_input() {
    let store = SqliteStore::in_memory().unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 1_000).unwrap();

    assert!(matches!(
        desktop.create_skill(" ", None),
        Err(study_rpg::DesktopError::StudyRpg(
            StudyRpgError::EmptySkillName
        ))
    ));
}

#[test]
fn desktop_controller_grants_xp_to_the_selected_child_instead_of_a_same_named_root() {
    let store = SqliteStore::in_memory().unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 1_000).unwrap();
    let root = desktop.create_skill("Rust", None).unwrap();
    let programming = desktop.create_skill("编程", None).unwrap();
    let child = desktop.create_skill("Rust", Some(programming)).unwrap();

    desktop
        .start_session_with_skill_id("  Rust ownership  ", Some(child), 1_000)
        .unwrap();
    let result = desktop.finish_session(1_000 + 25 * 60).unwrap();
    let dashboard = desktop.dashboard_at(1_000 + 25 * 60).unwrap();

    assert_eq!(result.session.topic, "Rust ownership");
    assert_eq!(result.session.skill_id, Some(child));
    assert_eq!(dashboard.skills.len(), 3);
    assert_eq!(
        dashboard
            .skills
            .iter()
            .find(|skill| skill.id == root)
            .unwrap()
            .total_xp,
        0
    );
    assert_eq!(
        dashboard
            .skills
            .iter()
            .find(|skill| skill.id == programming)
            .unwrap()
            .total_xp,
        0
    );
    assert_eq!(
        dashboard
            .skills
            .iter()
            .find(|skill| skill.id == child)
            .unwrap()
            .total_xp,
        40
    );
    assert!(result.growth_events.iter().any(|event| matches!(
        event.kind,
        GrowthEventKind::SkillGrowth { skill_id, gained_xp: 40, .. } if skill_id == child
    )));
}

#[test]
fn desktop_controller_reuses_an_existing_skill_name_ignoring_ascii_case() {
    let store = SqliteStore::in_memory().unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 1_000).unwrap();

    desktop
        .start_session_with_skill("Rust syntax", Some("Rust"), 1_000)
        .unwrap();
    desktop.finish_session(1_060).unwrap();
    desktop
        .start_session_with_skill("Rust ownership", Some("  rUsT  "), 2_000)
        .unwrap();

    let dashboard = desktop.dashboard_at(2_000).unwrap();
    assert_eq!(dashboard.skills.len(), 1);
    assert_eq!(dashboard.skills[0].name, "Rust");
    assert_eq!(
        dashboard.active_session.unwrap().skill_name.as_deref(),
        Some("Rust")
    );
}

#[test]
fn desktop_controller_finishes_the_timer_through_the_core_loop() {
    let store = SqliteStore::in_memory().unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 1_000).unwrap();
    desktop.start_session("Rust ownership", 1_000).unwrap();

    let result = desktop.finish_session(1_000 + 25 * 60).unwrap();

    assert_eq!(result.session.duration_minutes, 25);
    assert_eq!(result.session.earned_xp, 40);
    assert_eq!(result.player_xp.gained_xp, 80);
    let dashboard = desktop.dashboard_at(1_000 + 25 * 60).unwrap();
    assert!(dashboard.active_session.is_none());
    assert_eq!(dashboard.recent_sessions[0].topic, "Rust ownership");
}

#[test]
fn desktop_controller_exposes_the_core_statistics_report() {
    const DAY: u64 = 86_400;
    let store = SqliteStore::in_memory().unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 10 * DAY)
            .unwrap();
    desktop.start_session("Yesterday", 9 * DAY).unwrap();
    desktop.finish_session(9 * DAY + 20 * 60).unwrap();
    desktop.start_session("Today", 10 * DAY).unwrap();
    desktop.finish_session(10 * DAY + 30 * 60).unwrap();

    let report = desktop.statistics_at(10 * DAY + 30 * 60).unwrap();

    assert_eq!(report.today.total_minutes, 30);
    assert_eq!(report.all_time.total_minutes, 50);
    assert_eq!(report.current_streak_days, 2);
    assert_eq!(report.last_seven_days.len(), 7);
}

#[test]
fn desktop_controller_restores_an_active_session_from_local_storage() {
    let database_path = std::env::temp_dir().join(format!(
        "study-rpg-desktop-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut desktop =
            DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 2_000)
                .unwrap();
        desktop.start_session("Persistent timer", 2_000).unwrap();
    }

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut restored =
            DesktopController::load_or_create(store, "Ignored", CharacterClass::Mage, 2_060)
                .unwrap();
        let dashboard = restored.dashboard_at(2_060).unwrap();

        assert_eq!(dashboard.player_name, "Nembx");
        assert_eq!(dashboard.active_session.unwrap().topic, "Persistent timer");
    }

    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn desktop_controller_restores_companion_preferences_from_local_storage() {
    let database_path = std::env::temp_dir().join(format!(
        "study-rpg-companion-preferences-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut desktop =
            DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 2_000)
                .unwrap();

        desktop
            .set_companion_preferences(CompanionPreferences {
                mode: CompanionMode::Expanded,
                y_position: Some(248),
            })
            .unwrap();
    }

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let desktop =
            DesktopController::load_or_create(store, "Ignored", CharacterClass::Mage, 2_060)
                .unwrap();

        assert_eq!(
            desktop.companion_preferences(),
            CompanionPreferences {
                mode: CompanionMode::Expanded,
                y_position: Some(248),
            }
        );
    }

    std::fs::remove_file(database_path).unwrap();
}

#[test]
fn first_run_character_creation_becomes_the_persisted_player_identity() {
    let database_path = std::env::temp_dir().join(format!(
        "study-rpg-character-creation-{}-{}.sqlite3",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut desktop = DesktopController::load(store, 2_000).unwrap();

        assert!(desktop.needs_character_creation());
        desktop
            .create_character("  Nembx  ", CharacterClass::Engineer, 2_000)
            .unwrap();
    }

    {
        let store = SqliteStore::open(&database_path).unwrap();
        let mut desktop = DesktopController::load(store, 2_060).unwrap();
        let dashboard = desktop.dashboard_at(2_060).unwrap();

        assert!(!desktop.needs_character_creation());
        assert_eq!(dashboard.player_name, "Nembx");
        assert_eq!(dashboard.player_class, CharacterClass::Engineer);
    }

    std::fs::remove_file(database_path).unwrap();
}

#[cfg(unix)]
#[test]
fn desktop_controller_rolls_back_a_session_when_local_storage_fails() {
    let database_directory = std::env::temp_dir().join(format!(
        "study-rpg-unavailable-storage-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&database_directory).unwrap();
    let database_path = database_directory.join("study-rpg.sqlite3");
    let store = SqliteStore::open(&database_path).unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 3_000).unwrap();

    std::fs::remove_file(database_path).unwrap();
    std::fs::remove_dir(database_directory).unwrap();

    let error = desktop.start_session("Must persist", 3_000).unwrap_err();

    assert!(matches!(error, DesktopError::Storage(_)));
    assert!(
        desktop
            .dashboard_at(3_000)
            .unwrap()
            .active_session
            .is_none()
    );
}

#[cfg(unix)]
#[test]
fn desktop_controller_rolls_back_a_child_skill_when_local_storage_fails() {
    let database_directory = std::env::temp_dir().join(format!(
        "study-rpg-unsaved-skill-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&database_directory).unwrap();
    let database_path = database_directory.join("study-rpg.sqlite3");
    let store = SqliteStore::open(&database_path).unwrap();
    let mut desktop =
        DesktopController::load_or_create(store, "Nembx", CharacterClass::Scholar, 3_000).unwrap();
    let parent = desktop.create_skill("编程", None).unwrap();
    let before = desktop.dashboard_at(3_000).unwrap();

    std::fs::remove_file(database_path).unwrap();
    std::fs::remove_dir(database_directory).unwrap();

    assert!(matches!(
        desktop.create_skill("Rust", Some(parent)),
        Err(DesktopError::Storage(_))
    ));
    assert_eq!(desktop.dashboard_at(3_000).unwrap(), before);
}
