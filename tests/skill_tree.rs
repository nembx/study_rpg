use study_rpg::{CharacterClass, SqliteStore, StudyRpg, StudyRpgError, StudySessionStartInput};

#[test]
fn creates_a_child_skill_and_restores_its_parent_relationship() {
    let mut store = SqliteStore::in_memory().unwrap();
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

    let programming = app.create_skill("  编程  ", None).unwrap();
    let rust = app.create_skill("Rust", Some(programming)).unwrap();

    assert_eq!(app.skills()[0].name, "编程");
    assert_eq!(app.skills()[1].name, "Rust");
    assert_eq!(app.skills()[1].parent_id, Some(programming));
    assert!(!app.skills()[1].unlocked);

    store.save(&app).unwrap();
    let restored = store.load().unwrap().unwrap();
    assert_eq!(restored.skills()[1].id, rust);
    assert_eq!(restored.skills()[1].parent_id, Some(programming));
}

#[test]
fn rejects_empty_unknown_parent_and_duplicate_sibling_skill_names() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let programming = app.create_skill("编程", None).unwrap();

    assert_eq!(
        app.create_skill("  ", None),
        Err(StudyRpgError::EmptySkillName)
    );
    assert_eq!(
        app.create_skill("Rust", Some(999)),
        Err(StudyRpgError::UnknownSkillParent)
    );
    app.create_skill("Rust", Some(programming)).unwrap();
    assert_eq!(
        app.create_skill(" rUsT ", Some(programming)),
        Err(StudyRpgError::DuplicateSkillName)
    );
}

#[test]
fn dashboard_exposes_skill_hierarchy_in_parent_first_order() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    let programming = app.create_skill("编程", None).unwrap();
    let languages = app.create_skill("外语", None).unwrap();
    let rust = app.create_skill("Rust", Some(programming)).unwrap();
    let testing = app.create_skill("测试", Some(programming)).unwrap();
    let ownership = app.create_skill("所有权", Some(rust)).unwrap();

    let skills = app.dashboard().skills;

    assert_eq!(
        skills.iter().map(|skill| skill.id).collect::<Vec<_>>(),
        vec![programming, rust, ownership, testing, languages]
    );
    assert_eq!(skills[0].parent_id, None);
    assert_eq!(skills[0].depth, 0);
    assert_eq!(skills[1].parent_id, Some(programming));
    assert_eq!(skills[1].depth, 1);
    assert_eq!(skills[2].parent_id, Some(rust));
    assert_eq!(skills[2].depth, 2);
    assert_eq!(skills[3].depth, 1);
    assert_eq!(skills[4].depth, 0);
}

#[test]
fn rejects_a_session_for_a_missing_skill_without_starting_the_timer() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);

    assert_eq!(
        app.start_study_session(
            StudySessionStartInput {
                topic: "Rust ownership".to_string(),
                skill_id: Some(999),
            },
            1_000,
        ),
        Err(StudyRpgError::UnknownSkill)
    );
    assert!(app.active_session().is_none());
    assert!(app.skills().is_empty());
}

#[test]
fn restored_legacy_skills_with_missing_or_cyclic_parents_remain_visible_once() {
    let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
    app.add_skill("编程", None);
    app.add_skill("旧技能", Some(999));
    app.add_skill("循环技能 A", Some(4));
    app.add_skill("循环技能 B", Some(3));

    let restored = StudyRpg::from_snapshot(app.snapshot());
    let skills = restored.dashboard().skills;

    assert_eq!(
        skills.iter().map(|skill| skill.id).collect::<Vec<_>>(),
        vec![1, 2, 3, 4]
    );
}
