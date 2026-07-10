use std::path::Path;

use rusqlite::{Connection, OptionalExtension, params};

use crate::player::{CharacterClass, Mood, Player};
use crate::quest::{Quest, QuestTarget};
use crate::session::{ActiveStudySession, StudySession};
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

        tx.execute("DELETE FROM active_study_session", [])?;
        tx.execute("DELETE FROM daily_quest_state", [])?;
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
                 (id, topic, skill_id, duration_minutes, earned_xp,
                  started_at_epoch_seconds, ended_at_epoch_seconds)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    session.id,
                    session.topic,
                    session.skill_id,
                    session.duration_minutes,
                    session.earned_xp,
                    session.started_at_epoch_seconds,
                    session.ended_at_epoch_seconds,
                ],
            )?;
        }

        for quest in snapshot.daily_quests {
            let (target_kind, target_value) = quest_target_to_parts(quest.target);
            tx.execute(
                "INSERT INTO quests
                 (id, epoch_day, title, target_kind, target_value, reward_xp, completed)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    quest.id,
                    quest.epoch_day,
                    quest.title,
                    target_kind,
                    target_value,
                    quest.reward_xp,
                    quest.completed,
                ],
            )?;
        }

        tx.execute(
            "INSERT INTO daily_quest_state (id, completion_bonus_claimed)
             VALUES (1, ?1)",
            [snapshot.daily_completion_bonus_claimed],
        )?;

        if let Some(active_session) = snapshot.active_session {
            tx.execute(
                "INSERT INTO active_study_session
                 (id, topic, skill_id, started_at_epoch_seconds)
                 VALUES (1, ?1, ?2, ?3)",
                params![
                    active_session.topic,
                    active_session.skill_id,
                    active_session.started_at_epoch_seconds,
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
        let daily_completion_bonus_claimed = self.load_daily_completion_bonus_claimed()?;
        let active_session = self.load_active_session()?;

        Ok(Some(StudyRpg::from_snapshot(StudyRpgSnapshot {
            player,
            skills,
            sessions,
            daily_quests,
            daily_completion_bonus_claimed,
            active_session,
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
                earned_xp INTEGER NOT NULL,
                started_at_epoch_seconds INTEGER,
                ended_at_epoch_seconds INTEGER
            );

            CREATE TABLE IF NOT EXISTS quests (
                id INTEGER PRIMARY KEY,
                epoch_day INTEGER NOT NULL DEFAULT 0,
                title TEXT NOT NULL,
                target_kind TEXT NOT NULL,
                target_value INTEGER NOT NULL,
                reward_xp INTEGER NOT NULL,
                completed INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS daily_quest_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                completion_bonus_claimed INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS active_study_session (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                topic TEXT NOT NULL,
                skill_id INTEGER,
                started_at_epoch_seconds INTEGER NOT NULL
            );
            ",
        )?;

        self.add_column_if_missing("study_sessions", "started_at_epoch_seconds", "INTEGER")?;
        self.add_column_if_missing("study_sessions", "ended_at_epoch_seconds", "INTEGER")?;
        self.add_column_if_missing("quests", "epoch_day", "INTEGER NOT NULL DEFAULT 0")
    }

    fn add_column_if_missing(
        &self,
        table: &str,
        column: &str,
        definition: &str,
    ) -> rusqlite::Result<()> {
        let table_info_sql = format!("PRAGMA table_info({table})");
        let mut stmt = self.conn.prepare(&table_info_sql)?;
        let columns = stmt
            .query_map([], |row| row.get::<_, String>(1))?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        if columns.iter().any(|existing| existing == column) {
            return Ok(());
        }

        let alter_sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        self.conn.execute(&alter_sql, [])?;
        Ok(())
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
            "SELECT id, topic, skill_id, duration_minutes, earned_xp,
                    started_at_epoch_seconds, ended_at_epoch_seconds
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
                started_at_epoch_seconds: row.get(5)?,
                ended_at_epoch_seconds: row.get(6)?,
            })
        })?
        .collect()
    }

    fn load_quests(&self) -> rusqlite::Result<Vec<Quest>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, epoch_day, title, target_kind, target_value, reward_xp, completed
             FROM quests
             ORDER BY id",
        )?;

        stmt.query_map([], |row| {
            let target_kind = row.get::<_, String>(3)?;
            let target_value = row.get(4)?;

            Ok(Quest {
                id: row.get(0)?,
                epoch_day: row.get(1)?,
                title: row.get(2)?,
                target: quest_target_from_parts(&target_kind, target_value),
                reward_xp: row.get(5)?,
                completed: row.get(6)?,
            })
        })?
        .collect()
    }

    fn load_daily_completion_bonus_claimed(&self) -> rusqlite::Result<bool> {
        Ok(self
            .conn
            .query_row(
                "SELECT completion_bonus_claimed FROM daily_quest_state WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(false))
    }

    fn load_active_session(&self) -> rusqlite::Result<Option<ActiveStudySession>> {
        self.conn
            .query_row(
                "SELECT topic, skill_id, started_at_epoch_seconds
                 FROM active_study_session
                 WHERE id = 1",
                [],
                |row| {
                    Ok(ActiveStudySession {
                        topic: row.get(0)?,
                        skill_id: row.get(1)?,
                        started_at_epoch_seconds: row.get(2)?,
                    })
                },
            )
            .optional()
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
    use crate::study_rpg::{StudySessionInput, StudySessionStartInput};

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
        assert_eq!(restored.player().total_xp, 298);
        assert_eq!(restored.skills()[0].name, "Rust");
        assert_eq!(restored.skills()[0].total_xp, 48);
        assert_eq!(restored.sessions()[0].topic, "Rust ownership");
        assert_eq!(restored.daily_quests().len(), 2);
        assert_eq!(restored.daily_quests()[0].epoch_day, 0);
        assert!(restored.daily_quests().iter().all(|quest| quest.completed));

        let result = restored.complete_study_session(StudySessionInput {
            topic: "SQLite storage".to_string(),
            skill_id: Some(rust),
            duration_minutes: 10,
        });
        assert_eq!(result.session.id, 2);
    }

    #[test]
    fn persists_and_restores_an_active_study_session() {
        let mut store = SqliteStore::in_memory().unwrap();
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        let rust = app.add_skill("Rust", None);
        app.start_study_session(
            StudySessionStartInput {
                topic: "Rust ownership".to_string(),
                skill_id: Some(rust),
            },
            2_000,
        )
        .unwrap();

        store.save(&app).unwrap();
        let mut restored = store.load().unwrap().unwrap();

        assert_eq!(
            restored.active_session().unwrap(),
            &ActiveStudySession {
                topic: "Rust ownership".to_string(),
                skill_id: Some(rust),
                started_at_epoch_seconds: 2_000,
            }
        );

        let result = restored
            .finish_active_study_session(2_000 + 30 * 60)
            .unwrap();
        assert_eq!(result.session.duration_minutes, 30);
        assert_eq!(result.session.started_at_epoch_seconds, Some(2_000));
    }

    #[test]
    fn persists_and_restores_daily_quest_day() {
        let mut store = SqliteStore::in_memory().unwrap();
        let mut app = StudyRpg::new("Nembx", CharacterClass::Scholar);
        app.refresh_daily_quests_at(86_400 * 3);

        store.save(&app).unwrap();
        let restored = store.load().unwrap().unwrap();

        assert!(
            restored
                .daily_quests()
                .iter()
                .all(|quest| quest.epoch_day == 3)
        );
    }
}
