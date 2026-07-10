use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{
    CharacterClass, Dashboard, SqliteStore, StudyRpg, StudyRpgError, StudySessionResult,
    StudySessionStartInput,
};

pub struct DesktopController {
    app: StudyRpg,
    store: SqliteStore,
}

impl DesktopController {
    pub fn load_or_create(
        mut store: SqliteStore,
        player_name: impl Into<String>,
        class: CharacterClass,
        current_epoch_seconds: u64,
    ) -> Result<Self, DesktopError> {
        let (mut app, created) = match store.load()? {
            Some(app) => (app, false),
            None => (StudyRpg::new(player_name, class), true),
        };
        let quests_refreshed = app.refresh_daily_quests_at(current_epoch_seconds);

        if created || quests_refreshed {
            store.save(&app)?;
        }

        Ok(Self { app, store })
    }

    pub fn start_session(
        &mut self,
        topic: &str,
        started_at_epoch_seconds: u64,
    ) -> Result<(), DesktopError> {
        let topic = topic.trim();
        if topic.is_empty() {
            return Err(DesktopError::EmptyTopic);
        }

        let previous_app = self.app.clone();
        self.app.start_study_session(
            StudySessionStartInput {
                topic: topic.to_string(),
                skill_id: None,
            },
            started_at_epoch_seconds,
        )?;
        self.save_or_restore(previous_app)
    }

    pub fn dashboard_at(&mut self, current_epoch_seconds: u64) -> Result<Dashboard, DesktopError> {
        let previous_app = self.app.clone();
        if self.app.refresh_daily_quests_at(current_epoch_seconds) {
            self.save_or_restore(previous_app)?;
        }

        Ok(self.app.dashboard_at(current_epoch_seconds))
    }

    pub fn finish_session(
        &mut self,
        ended_at_epoch_seconds: u64,
    ) -> Result<StudySessionResult, DesktopError> {
        let previous_app = self.app.clone();
        let result = self
            .app
            .finish_active_study_session(ended_at_epoch_seconds)?;
        self.save_or_restore(previous_app)?;

        Ok(result)
    }

    fn save_or_restore(&mut self, previous_app: StudyRpg) -> Result<(), DesktopError> {
        if let Err(error) = self.store.save(&self.app) {
            self.app = previous_app;
            return Err(error.into());
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum DesktopError {
    EmptyTopic,
    StudyRpg(StudyRpgError),
    Storage(rusqlite::Error),
}

impl Display for DesktopError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTopic => formatter.write_str("Enter a study topic first"),
            Self::StudyRpg(StudyRpgError::StudySessionAlreadyActive) => {
                formatter.write_str("A study session is already active")
            }
            Self::StudyRpg(StudyRpgError::NoActiveStudySession) => {
                formatter.write_str("No study session is active")
            }
            Self::StudyRpg(StudyRpgError::StudySessionTooShort) => {
                formatter.write_str("Study for at least one complete minute")
            }
            Self::Storage(error) => write!(formatter, "Could not save local data: {error}"),
        }
    }
}

impl Error for DesktopError {}

impl From<StudyRpgError> for DesktopError {
    fn from(error: StudyRpgError) -> Self {
        Self::StudyRpg(error)
    }
}

impl From<rusqlite::Error> for DesktopError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Storage(error)
    }
}
