use study_rpg::{CharacterClass, DesktopController, SqliteStore};

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
