use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::player::{CharacterClass, Mood, Player};
use crate::quest::{Quest, QuestTarget};
use crate::session::StudySession;
use crate::skill::Skill;
use crate::study_rpg::{StudyRpg, StudyRpgSnapshot};

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let store = Self {
            conn: Connection::open(path)?,
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn in_memory() -> rusqlite::Result<Self> {
        let store = Self {
            conn: Connection::open_in_memory()?,
        };
        store.migrate()?;
        Ok(store)
    }

    pub fn save(&mut self, app: &StudyRpg) -> rusqlite::Result<()> {
        let snapshot = app.snapshot();
        let tx = self.conn.transaction()?;

        tx.execute("DELETE FROM quests", [])?;
        tx.execute("DELETE FROM study_sessions", [])?;
        tx.execute("DELETE FROM skills", [])?;
        tx.execute("DELETE FROM player", [])?;

        tx.execute(
            "INSERT INTO player (id, name, class, title, total_xp, energy, mood)
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                snapshot.player.name,
                character_class_to_str(snapshot.player.class),
                snapshot.player.title,
                snapshot.player.total_xp,
                snapshot.player.energy,
                mood_to_str(snapshot.player.mood),
            ],
        )?;

        for skill in snapshot.skills {
            tx.execute(
                "INSERT INTO skills (id, name, parent_id, total_xp, unlocked)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    skill.id,
                    skill.name,
                    skill.parent_id,
                    skill.total_xp,
                    skill.unlocked,
                ],
            )?;
        }

        for session in snapshot.sessions {
            tx.execute(
                "INSERT INTO study_sessions
                 (id, topic, skill_id, duration_minutes, earned_xp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    session.id,
                    session.topic,
                    session.skill_id,
                    session.duration_minutes,
                    session.earned_xp,
                ],
            )?;
        }

        for quest in snapshot.daily_quests {
            let (target_kind, target_value) = quest_target_to_parts(quest.target);
            tx.execute(
                "INSERT INTO quests
                 (id, title, target_kind, target_value, reward_xp, completed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    quest.id,
                    quest.title,
                    target_kind,
                    target_value,
                    quest.reward_xp,
                    quest.completed,
                ],
            )?;
        }

        tx.commit()
    }

    pub fn load(&self) -> rusqlite::Result<Option<StudyRpg>> {
        let player = self
            .conn
            .query_row(
                "SELECT name, class, title, total_xp, energy, mood FROM player WHERE id = 1",
                [],
                |row| {
                    Ok(Player {
                        name: row.get(0)?,
                        class: character_class_from_str(row.get::<_, String>(1)?.as_str()),
                        title: row.get(2)?,
                        total_xp: row.get(3)?,
                        energy: row.get(4)?,
                        mood: mood_from_str(row.get::<_, String>(5)?.as_str()),
                    })
                },
            )
            .optional()?;

        let Some(player) = player else {
            return Ok(None);
        };

        let skills = self.load_skills()?;
        let sessions = self.load_sessions()?;
        let daily_quests = self.load_quests()?;

        Ok(Some(StudyRpg::from_snapshot(StudyRpgSnapshot {
            player,
            skills,
            sessions,
            daily_quests,
        })))
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS player (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                name TEXT NOT NULL,
                class TEXT NOT NULL,
                title TEXT NOT NULL,
                total_xp INTEGER NOT NULL,
                energy INTEGER NOT NULL,
                mood TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS skills (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                parent_id INTEGER,
                total_xp INTEGER NOT NULL,
                unlocked INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS study_sessions (
                id INTEGER PRIMARY KEY,
                topic TEXT NOT NULL,
                skill_id INTEGER,
                duration_minutes INTEGER NOT NULL,
                earned_xp INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS quests (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                target_kind TEXT NOT NULL,
                target_value INTEGER NOT NULL,
                reward_xp INTEGER NOT NULL,
                completed INTEGER NOT NULL
            );
            ",
        )
    }

    fn load_skills(&self) -> rusqlite::Result<Vec<Skill>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, parent_id, total_xp, unlocked
             FROM skills
             ORDER BY id",
        )?;

        stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                total_xp: row.get(3)?,
                unlocked: row.get(4)?,
            })
        })?
        .collect()
    }

    fn load_sessions(&self) -> rusqlite::Result<Vec<StudySession>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, topic, skill_id, duration_minutes, earned_xp
             FROM study_sessions
             ORDER BY id",
        )?;

        stmt.query_map([], |row| {
            Ok(StudySession {
                id: row.get(0)?,
                topic: row.get(1)?,
                skill_id: row.get(2)?,
                duration_minutes: row.get(3)?,
                earned_xp: row.get(4)?,
            })
        })?
        .collect()
    }

    fn load_quests(&self) -> rusqlite::Result<Vec<Quest>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, target_kind, target_value, reward_xp, completed
             FROM quests
             ORDER BY id",
        )?;

        stmt.query_map([], |row| {
            let target_kind = row.get::<_, String>(2)?;
            let target_value = row.get(3)?;

            Ok(Quest {
                id: row.get(0)?,
                title: row.get(1)?,
                target: quest_target_from_parts(&target_kind, target_value),
                reward_xp: row.get(4)?,
                completed: row.get(5)?,
            })
        })?
        .collect()
    }
}

fn character_class_to_str(class: CharacterClass) -> &'static str {
    match class {
        CharacterClass::Scholar => "scholar",
        CharacterClass::Engineer => "engineer",
        CharacterClass::Mage => "mage",
        CharacterClass::Warrior => "warrior",
        CharacterClass::Archer => "archer",
    }
}

fn character_class_from_str(value: &str) -> CharacterClass {
    match value {
        "engineer" => CharacterClass::Engineer,
        "mage" => CharacterClass::Mage,
        "warrior" => CharacterClass::Warrior,
        "archer" => CharacterClass::Archer,
        _ => CharacterClass::Scholar,
    }
}

fn mood_to_str(mood: Mood) -> &'static str {
    match mood {
        Mood::Focused => "focused",
        Mood::Tired => "tired",
        Mood::Happy => "happy",
    }
}

fn mood_from_str(value: &str) -> Mood {
    match value {
        "tired" => Mood::Tired,
        "happy" => Mood::Happy,
        _ => Mood::Focused,
    }
}

fn quest_target_to_parts(target: QuestTarget) -> (&'static str, u32) {
    match target {
        QuestTarget::StudyMinutes(minutes) => ("study_minutes", minutes),
        QuestTarget::CompleteSessions(sessions) => ("complete_sessions", sessions),
    }
}

fn quest_target_from_parts(kind: &str, value: u32) -> QuestTarget {
    match kind {
        "complete_sessions" => QuestTarget::CompleteSessions(value),
        _ => QuestTarget::StudyMinutes(value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::study_rpg::StudySessionInput;

    #[test]
    fn returns_none_when_no_player_has_been_saved() {
        let store = SqliteStore::in_memory().unwrap();

        assert!(store.load().unwrap().is_none());
    }

    #[test]
    fn persists_and_restores_the_core_loop_state() {
        let mut store = SqliteStore::in_memory().unwrap();
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);
        app.complete_study_session(StudySessionInput {
            topic: "Rust ownership".to_string(),
            skill_id: Some(rust),
            duration_minutes: 30,
        });

        store.save(&app).unwrap();
        let mut restored = store.load().unwrap().unwrap();

        assert_eq!(restored.player().name, "Nembx");
        assert_eq!(restored.player().total_xp, 148);
        assert_eq!(restored.skills()[0].name, "Rust");
        assert_eq!(restored.skills()[0].total_xp, 48);
        assert_eq!(restored.sessions()[0].topic, "Rust ownership");
        assert_eq!(restored.daily_quests().len(), 2);
        assert!(restored.daily_quests().iter().all(|quest| quest.completed));

        let result = restored.complete_study_session(StudySessionInput {
            topic: "SQLite storage".to_string(),
            skill_id: Some(rust),
            duration_minutes: 10,
        });
        assert_eq!(result.session.id, 2);
    }
}
