use study_rpg::{CharacterClass, StudyRpg, StudySessionStartInput};

fn main() {
    let mut app = StudyRpg::new("Player", CharacterClass::Scholar);
    let rust_skill = app.add_skill("Rust", None);

    app.start_study_session(
        StudySessionStartInput {
            topic: "Rust ownership".to_string(),
            skill_id: Some(rust_skill),
        },
        1_000,
    )
    .expect("study session should start");

    let result = app
        .finish_active_study_session(1_000 + 25 * 60)
        .expect("study session should finish");

    let dashboard = app.dashboard();

    println!(
        "{} Lv.{} - {} XP gained",
        dashboard.player_name, dashboard.level.level, result.player_xp.gained_xp
    );
    println!(
        "Progress: {}/{} XP, today: {} minutes",
        dashboard.level.xp_into_level, dashboard.level.xp_for_next_level, dashboard.today_minutes
    );
}
