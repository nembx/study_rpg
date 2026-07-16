use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::{
    CharacterClass, Dashboard, SqliteStore, StudyRpg, StudyRpgError, StudySessionResult,
    StudySessionStartInput, StudyStatisticsReport,
};

pub struct DesktopController {
    app: Option<StudyRpg>,
    store: SqliteStore,
    companion_preferences: CompanionPreferences,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompanionMode {
    Compact,
    Expanded,
}

impl CompanionMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Expanded => "expanded",
        }
    }

    pub(crate) fn from_str(value: &str) -> Self {
        match value {
            "expanded" => Self::Expanded,
            _ => Self::Compact,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CompanionPreferences {
    pub mode: CompanionMode,
    pub y_position: Option<i32>,
}

impl Default for CompanionPreferences {
    fn default() -> Self {
        Self {
            mode: CompanionMode::Compact,
            y_position: None,
        }
    }
}

impl DesktopController {
    pub fn load(mut store: SqliteStore, current_epoch_seconds: u64) -> Result<Self, DesktopError> {
        let mut app = store.load()?;
        if let Some(existing_app) = app.as_mut()
            && existing_app.refresh_daily_quests_at(current_epoch_seconds)
        {
            store.save(existing_app)?;
        }
        let companion_preferences = store.load_companion_preferences()?;

        Ok(Self {
            app,
            store,
            companion_preferences,
        })
    }

    pub fn load_or_create(
        store: SqliteStore,
        player_name: impl Into<String>,
        class: CharacterClass,
        current_epoch_seconds: u64,
    ) -> Result<Self, DesktopError> {
        let mut controller = Self::load(store, current_epoch_seconds)?;
        if controller.needs_character_creation() {
            controller.create_character(player_name, class, current_epoch_seconds)?;
        }
        Ok(controller)
    }

    pub fn needs_character_creation(&self) -> bool {
        self.app.is_none()
    }

    pub fn create_character(
        &mut self,
        player_name: impl Into<String>,
        class: CharacterClass,
        current_epoch_seconds: u64,
    ) -> Result<(), DesktopError> {
        if self.app.is_some() {
            return Err(DesktopError::CharacterAlreadyCreated);
        }
        let player_name = player_name.into();
        let player_name = player_name.trim();
        if player_name.is_empty() {
            return Err(DesktopError::EmptyPlayerName);
        }

        let mut app = StudyRpg::new(player_name, class);
        app.refresh_daily_quests_at(current_epoch_seconds);
        self.store.save(&app)?;
        self.app = Some(app);
        Ok(())
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

        let previous_app = self.app()?.clone();
        self.app_mut()?.start_study_session(
            StudySessionStartInput {
                topic: topic.to_string(),
                skill_id: None,
            },
            started_at_epoch_seconds,
        )?;
        self.save_or_restore(previous_app)
    }

    pub fn dashboard_at(&mut self, current_epoch_seconds: u64) -> Result<Dashboard, DesktopError> {
        let previous_app = self.app()?.clone();
        if self
            .app_mut()?
            .refresh_daily_quests_at(current_epoch_seconds)
        {
            self.save_or_restore(previous_app)?;
        }

        Ok(self.app()?.dashboard_at(current_epoch_seconds))
    }

    pub fn statistics_at(
        &self,
        current_epoch_seconds: u64,
    ) -> Result<StudyStatisticsReport, DesktopError> {
        Ok(self.app()?.statistics_at(current_epoch_seconds))
    }

    pub fn finish_session(
        &mut self,
        ended_at_epoch_seconds: u64,
    ) -> Result<StudySessionResult, DesktopError> {
        let previous_app = self.app()?.clone();
        let result = self
            .app_mut()?
            .finish_active_study_session(ended_at_epoch_seconds)?;
        self.save_or_restore(previous_app)?;

        Ok(result)
    }

    pub fn companion_preferences(&self) -> CompanionPreferences {
        self.companion_preferences
    }

    pub fn set_companion_preferences(
        &mut self,
        preferences: CompanionPreferences,
    ) -> Result<(), DesktopError> {
        self.store.save_companion_preferences(preferences)?;
        self.companion_preferences = preferences;
        Ok(())
    }

    fn save_or_restore(&mut self, previous_app: StudyRpg) -> Result<(), DesktopError> {
        let current_app = self.app()?.clone();
        if let Err(error) = self.store.save(&current_app) {
            self.app = Some(previous_app);
            return Err(error.into());
        }

        Ok(())
    }

    fn app(&self) -> Result<&StudyRpg, DesktopError> {
        self.app.as_ref().ok_or(DesktopError::CharacterNotCreated)
    }

    fn app_mut(&mut self) -> Result<&mut StudyRpg, DesktopError> {
        self.app.as_mut().ok_or(DesktopError::CharacterNotCreated)
    }
}

#[derive(Debug)]
pub enum DesktopError {
    EmptyPlayerName,
    EmptyTopic,
    CharacterAlreadyCreated,
    CharacterNotCreated,
    StudyRpg(StudyRpgError),
    Storage(rusqlite::Error),
}

impl Display for DesktopError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyPlayerName => formatter.write_str("请先输入角色名称"),
            Self::EmptyTopic => formatter.write_str("请先输入学习主题"),
            Self::CharacterAlreadyCreated => formatter.write_str("角色已经创建"),
            Self::CharacterNotCreated => formatter.write_str("请先创建角色"),
            Self::StudyRpg(StudyRpgError::StudySessionAlreadyActive) => {
                formatter.write_str("已有正在进行的学习计时")
            }
            Self::StudyRpg(StudyRpgError::NoActiveStudySession) => {
                formatter.write_str("当前没有正在进行的学习计时")
            }
            Self::StudyRpg(StudyRpgError::StudySessionTooShort) => {
                formatter.write_str("至少学习满一分钟后才能结算")
            }
            Self::Storage(error) => write!(formatter, "无法保存本地数据：{error}"),
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
