use study_rpg::{CharacterClass, StudyRpg, StudySessionInput};

fn main() {
    let mut app = StudyRpg::new("Player", CharacterClass::Scholar);
    let rust_skill = app.add_skill("Rust", None);

    let result = app.complete_study_session(StudySessionInput {
        topic: "Rust ownership".to_string(),
        skill_id: Some(rust_skill),
        duration_minutes: 25,
    });

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
